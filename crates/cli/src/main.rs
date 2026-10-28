//! Experimental headless frontend. LGPL-3.0-only.
use std::{path::PathBuf, process::ExitCode};

use clap::{Args, Parser, Subcommand};
use polished_map_core::{BlockCommand, Command, PaletteSet, TileCommand};
use polished_map_project::{
    BlocksetDocument, MapDocument, MasterGraphicsOptions, MasterTilesetDocument, OpenOptions,
    ProjectError, RoofDocument, load_master_map_graphics, prepare_blockset_save_as,
    prepare_collision_save_as, prepare_save_as, read_bounded, write_png,
};
use serde_json::{Value, json};

#[derive(Parser)]
#[command(version, about = "Polished Map NG experimental headless editor")]
struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Args)]
struct MapArgs {
    map: PathBuf,
    /// Project directory; otherwise discover the nearest ancestor Makefile.
    #[arg(long)]
    root: Option<PathBuf>,
    #[arg(long)]
    width: Option<u16>,
    #[arg(long)]
    height: Option<u16>,
}

impl MapArgs {
    fn open(&self) -> Result<MapDocument, Failure> {
        MapDocument::open_in_project(
            &self.map,
            &OpenOptions {
                width: self.width,
                height: self.height,
            },
            self.root.as_deref(),
        )
        .map_err(Failure::from)
    }
}

#[derive(Subcommand)]
enum Action {
    /// Inspect nine roof tiles in file order (indices 0..8, master slots $0A..$12).
    InspectRoof {
        #[command(flatten)]
        input: RoofArgs,
    },
    /// Preview roof pixel edits; applying requires the observed file-state hash.
    EditRoof {
        #[command(flatten)]
        input: RoofArgs,
        #[arg(long)]
        commands: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        expected_hash: Option<String>,
    },
    /// Inspect master tile pixels, attributes, input hashes and PNG destinations.
    InspectTileset {
        #[command(flatten)]
        input: TilesetArgs,
    },
    /// Preview a project tileset edit; applying requires the preview's state hash.
    EditTileset {
        #[command(flatten)]
        input: TilesetArgs,
        #[arg(long)]
        commands: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        expected_hash: Option<String>,
    },
    /// Inspect a binary or imported ASM blockset without graphics or a GUI.
    InspectBlockset {
        path: PathBuf,
        #[arg(long)]
        collisions: Option<PathBuf>,
    },
    /// Preview block edits; --apply writes only after all commands validate.
    EditBlockset {
        path: PathBuf,
        #[arg(long)]
        collisions: Option<PathBuf>,
        #[arg(long)]
        expected_collision_hash: Option<String>,
        #[arg(long)]
        commands: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        expected_hash: Option<String>,
        /// Save a binary copy; required for imported CEL assembly.
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        expected_output_hash: Option<String>,
        #[arg(long, requires = "output")]
        output_collisions: Option<PathBuf>,
        #[arg(long)]
        expected_output_collision_hash: Option<String>,
    },
    /// Preview or create a new zero-filled map. Project constants are not written.
    Create {
        #[arg(long)]
        root: PathBuf,
        #[arg(long, default_value_t = 10)]
        width: u16,
        #[arg(long, default_value_t = 9)]
        height: u16,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        expected_output_hash: Option<String>,
    },
    /// Inspect dimensions, block IDs and the original content hash.
    Inspect {
        #[command(flatten)]
        input: MapArgs,
    },
    /// Validate and preview edits. Files are written only with --apply.
    Edit {
        #[command(flatten)]
        input: MapArgs,
        /// File containing an array of paint/fill/replace/swap/resize commands.
        #[arg(long)]
        commands: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        expected_hash: Option<String>,
    },
    /// Preview or save a binary map copy, including conversion from an ASM .map.
    SaveAs {
        #[command(flatten)]
        input: MapArgs,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        apply: bool,
        /// Required when applying over an existing destination; returned by preview.
        #[arg(long)]
        expected_output_hash: Option<String>,
    },
    /// Render a master map as a PNG, without a display or GPU.
    Render {
        #[command(flatten)]
        input: MapArgs,
        /// Override the tileset inferred from filename or project metadata.
        #[arg(long)]
        tileset: Option<String>,
        #[arg(long)]
        allow_256: bool,
        /// Override automatic map palette selection.
        #[arg(long, value_parser = parse_palette_set)]
        palette: Option<PaletteSet>,
        /// Optional nine-tile graphic replacement from gfx/tilesets/roofs.
        #[arg(long)]
        roof: Option<String>,
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Args)]
struct RoofArgs {
    roof: String,
    #[arg(long)]
    root: PathBuf,
}
impl RoofArgs {
    fn open(&self) -> Result<RoofDocument, Failure> {
        // Standalone pixel edits need no map/tileset; palette attributes are
        // contextual in the GUI and never serialized into roof PNGs.
        Ok(RoofDocument::open_in_project(
            &self.root, &self.roof, [9; 9],
        )?)
    }
}
fn roof_summary(doc: &RoofDocument) -> Value {
    json!({"name":doc.name(),"content_hash":doc.content_hash(),"pixels":doc.roof().tileset().tiles().iter().map(|t|t.hues().as_slice()).collect::<Vec<_>>(),"source":doc.source().path(),"output":doc.output().path()})
}

#[derive(Args)]
struct TilesetArgs {
    tileset: String,
    #[arg(long)]
    root: PathBuf,
    #[arg(long)]
    allow_256: bool,
    #[arg(long)]
    monochrome: bool,
}

impl TilesetArgs {
    fn open(&self) -> Result<(MasterTilesetDocument, BlocksetDocument), Failure> {
        Ok((
            MasterTilesetDocument::open_in_project(
                &self.root,
                &self.tileset,
                &MasterGraphicsOptions {
                    allow_256: self.allow_256,
                    monochrome: self.monochrome,
                    ..Default::default()
                },
            )?,
            BlocksetDocument::open_in_project(&self.root, &self.tileset)?,
        ))
    }
}

fn tileset_summary(doc: &MasterTilesetDocument, blocks: &BlocksetDocument) -> Value {
    json!({"name":doc.name(), "content_hash":doc.content_hash(blocks.blockset()), "allow_256":doc.allow_256(), "monochrome":doc.monochrome(), "allow_priority":doc.allow_priority(),
        "pixels":doc.tileset().tiles().iter().map(|t| t.hues().as_slice()).collect::<Vec<_>>(), "attributes":doc.tileset().attributes(),
        "sources":doc.source_files().iter().map(|s| json!({"path":s.path(),"content_hash":s.content_hash()})).collect::<Vec<_>>(),
        "outputs":doc.outputs().iter().map(|s| json!({"path":s.path(),"content_hash":s.content_hash()})).collect::<Vec<_>>()})
}

struct Failure {
    code: String,
    message: String,
}

fn parse_palette_set(value: &str) -> Result<PaletteSet, String> {
    serde_json::from_value(Value::String(value.into()))
        .map_err(|_| "expected morn, day, nite, darkness, indoor, or custom".into())
}
impl Failure {
    fn new(code: &str, message: impl ToString) -> Self {
        Self {
            code: code.into(),
            message: message.to_string(),
        }
    }
}
impl From<ProjectError> for Failure {
    fn from(error: ProjectError) -> Self {
        Self::new(error.code(), error)
    }
}

fn run(cli: Cli) -> Result<Value, Failure> {
    match cli.action {
        Action::InspectRoof { input } => Ok(roof_summary(&input.open()?)),
        Action::EditRoof {
            input,
            commands,
            apply,
            expected_hash,
        } => {
            let mut doc = input.open()?;
            let before_hash = doc.content_hash();
            if expected_hash
                .as_ref()
                .is_some_and(|hash| hash != &before_hash)
            {
                return Err(Failure::new(
                    "conflict",
                    "roof inputs or output differ from the observed state",
                ));
            }
            if apply && expected_hash.is_none() {
                return Err(Failure::new(
                    "expected_hash_required",
                    "preview first and supply --expected-hash before writing the roof PNG",
                ));
            }
            let commands: Vec<TileCommand> =
                serde_json::from_slice(&read_bounded(&commands, 4 * 1024 * 1024)?)
                    .map_err(|e| Failure::new("invalid_commands", e))?;
            doc.roof_mut()
                .apply_batch(&commands)
                .map_err(ProjectError::from)?;
            let planned = doc.preview_save()?;
            if apply {
                doc.save()?;
            }
            let mut response = roof_summary(&doc);
            response["applied"] = json!(apply);
            response["previous_hash"] = json!(before_hash);
            response["planned_output"] = json!(planned);
            Ok(response)
        }
        Action::InspectTileset { input } => {
            let (doc, blocks) = input.open()?;
            Ok(tileset_summary(&doc, &blocks))
        }
        Action::EditTileset {
            input,
            commands,
            apply,
            expected_hash,
        } => {
            let (mut doc, blocks) = input.open()?;
            let before_hash = doc.content_hash(blocks.blockset());
            if expected_hash.as_ref().is_some_and(|h| h != &before_hash) {
                return Err(Failure::new(
                    "conflict",
                    "tileset inputs or outputs differ from the observed state",
                ));
            }
            if apply && expected_hash.is_none() {
                return Err(Failure::new(
                    "expected_hash_required",
                    "preview first and supply --expected-hash before writing the PNG and palette outputs",
                ));
            }
            let commands: Vec<TileCommand> =
                serde_json::from_slice(&read_bounded(&commands, 4 * 1024 * 1024)?)
                    .map_err(|e| Failure::new("invalid_commands", e))?;
            doc.apply_batch(&commands)?;
            let planned = doc.preview_save(blocks.blockset())?;
            blocks.verify_unchanged()?;
            if apply {
                doc.save(blocks.blockset())?;
            }
            let mut response = tileset_summary(&doc, &blocks);
            response["applied"] = json!(apply);
            response["previous_hash"] = json!(before_hash);
            response["planned_outputs"] = json!(planned);
            Ok(response)
        }
        Action::InspectBlockset { path, collisions } => {
            let doc = BlocksetDocument::open_with_collisions(&path, collisions.as_deref())?;
            Ok(
                json!({"path":doc.path(), "content_hash":doc.content_hash(), "blocks":doc.blockset().blocks(),"collision_path":doc.collision_path(),"collision_hash":doc.collision_hash(),"collisions":doc.blockset().collisions()}),
            )
        }
        Action::EditBlockset {
            path,
            collisions,
            expected_collision_hash,
            commands,
            apply,
            expected_hash,
            output,
            expected_output_hash,
            output_collisions,
            expected_output_collision_hash,
        } => {
            let mut doc = BlocksetDocument::open_with_collisions(&path, collisions.as_deref())?;
            if expected_hash.is_some_and(|h| h != doc.content_hash()) {
                return Err(Failure::new("conflict", "input hash does not match"));
            }
            if expected_collision_hash.is_some_and(|h| Some(h) != doc.collision_hash()) {
                return Err(Failure::new("conflict", "collision hash does not match"));
            }
            let before = doc.blockset().bytes();
            let before_collisions = doc.blockset().collisions().map(|rows| rows.to_vec());
            let commands: Vec<BlockCommand> =
                serde_json::from_slice(&read_bounded(&commands, 4 * 1024 * 1024)?)
                    .map_err(|e| Failure::new("invalid_commands", e))?;
            doc.blockset_mut()
                .apply_batch(&commands)
                .map_err(|e| Failure::new("invalid_command", e))?;
            let target = output
                .as_deref()
                .map(prepare_blockset_save_as)
                .transpose()?;
            let previous_output_hash = target.as_ref().and_then(|t| t.content_hash());
            let collision_target = output_collisions
                .as_deref()
                .map(|path| prepare_collision_save_as(path, doc.binary_collisions()))
                .transpose()?;
            if target.is_some() && (doc.collision_path().is_some() != collision_target.is_some()) {
                return Err(Failure::new(
                    "collision_destination_required",
                    "paired Save As requires --output-collisions when collisions are loaded",
                ));
            }
            let previous_output_collision_hash =
                collision_target.as_ref().and_then(|t| t.content_hash());
            if expected_output_collision_hash
                .as_ref()
                .is_some_and(|h| Some(h) != previous_output_collision_hash.as_ref())
            {
                return Err(Failure::new(
                    "conflict",
                    "collision destination hash does not match",
                ));
            }
            if expected_output_hash
                .as_ref()
                .is_some_and(|h| Some(h) != previous_output_hash.as_ref())
            {
                return Err(Failure::new("conflict", "destination hash does not match"));
            }
            if apply {
                if let Some(target) = &target {
                    if target.exists() && expected_output_hash.is_none() {
                        return Err(Failure::new(
                            "overwrite_confirmation_required",
                            "preview and supply --expected-output-hash before replacing a file",
                        ));
                    }
                    if let Some(collision_target) = &collision_target {
                        if collision_target.exists() && expected_output_collision_hash.is_none() {
                            return Err(Failure::new(
                                "overwrite_confirmation_required",
                                "preview and supply --expected-output-collision-hash before replacing collisions",
                            ));
                        }
                        doc.save_as_pair(target, collision_target)?;
                    } else {
                        doc.save_as(target)?;
                    }
                } else {
                    doc.save()?;
                }
            }
            Ok(
                json!({"applied":apply, "changed":doc.blockset().bytes()!=before || doc.blockset().collisions()!=before_collisions.as_deref(), "path":doc.path(), "output":target.as_ref().map(|t|t.path()), "previous_output_hash":previous_output_hash, "content_hash":doc.content_hash(), "blocks":doc.blockset().blocks(),"collisions":doc.blockset().collisions(),"collision_path":doc.collision_path(),"collision_hash":doc.collision_hash(),"output_collisions":collision_target.as_ref().map(|t|t.path()),"previous_output_collision_hash":previous_output_collision_hash}),
            )
        }
        Action::Create {
            root,
            width,
            height,
            output,
            apply,
            expected_output_hash,
        } => save_copy(
            MapDocument::new_map(&root, width, height)?,
            output,
            apply,
            expected_output_hash,
        ),
        Action::Inspect { input } => {
            let doc = input.open()?;
            Ok(json!({"map": doc.summary(), "blocks": doc.map().blocks()}))
        }
        Action::Edit {
            input,
            commands,
            apply,
            expected_hash,
        } => {
            let mut doc = input.open()?;
            if expected_hash.is_some_and(|expected| Some(expected) != doc.content_hash()) {
                return Err(Failure::new(
                    "conflict",
                    "expected hash does not match the input file",
                ));
            }
            let bytes = read_bounded(&commands, 4 * 1024 * 1024)?;
            let commands: Vec<Command> =
                serde_json::from_slice(&bytes).map_err(|e| Failure::new("invalid_commands", e))?;
            // No writes until every command succeeds. An invalid suffix discards all in-memory edits.
            let before = doc.map().blocks().to_vec();
            let dimensions_before = (doc.map().width(), doc.map().height());
            for command in commands {
                doc.map_mut()
                    .apply(&command)
                    .map_err(|e| Failure::new("invalid_command", e))?;
            }
            let dimensions_changed = dimensions_before != (doc.map().width(), doc.map().height());
            let changed = doc.map().blocks() != before || dimensions_changed;
            if apply {
                doc.save()?;
            }
            Ok(
                json!({"applied": apply, "changed": changed, "dimensions_changed": dimensions_changed, "map": doc.summary(), "blocks": doc.map().blocks()}),
            )
        }
        Action::SaveAs {
            input,
            output,
            apply,
            expected_output_hash,
        } => save_copy(input.open()?, output, apply, expected_output_hash),
        Action::Render {
            input,
            tileset,
            allow_256,
            palette,
            roof,
            output,
        } => {
            if output.exists() {
                return Err(Failure::new("output_exists", "choose a new output path"));
            }
            let doc = input.open()?;
            doc.metadata().root.as_deref().ok_or_else(|| {
                Failure::new("root_required", "no ancestor Makefile found; supply --root")
            })?;
            let tileset = tileset
                .as_deref()
                .or(doc.metadata().tileset.as_deref())
                .ok_or_else(|| {
                    Failure::new(
                        "tileset_required",
                        "no matching tileset found; supply --tileset",
                    )
                })?;
            let graphics = load_master_map_graphics(
                &doc,
                tileset,
                &MasterGraphicsOptions {
                    allow_256,
                    palette,
                    roof,
                    ..Default::default()
                },
            )?;
            let pixels = graphics
                .render_map(doc.map())
                .map_err(|e| Failure::new("invalid_graphics", e))?;
            write_png(&output, &pixels)?;
            Ok(json!({"output":output, "width":pixels.width, "height":pixels.height}))
        }
    }
}

fn save_copy(
    mut doc: MapDocument,
    output: PathBuf,
    apply: bool,
    expected_output_hash: Option<String>,
) -> Result<Value, Failure> {
    let target = prepare_save_as(&output)?;
    if let Some(expected) = &expected_output_hash
        && target.content_hash().as_ref() != Some(expected)
    {
        return Err(Failure::new("conflict", "destination hash does not match"));
    }
    if apply && target.exists() && expected_output_hash.is_none() {
        return Err(Failure::new(
            "overwrite_confirmation_required",
            "preview first, then supply --expected-output-hash to overwrite",
        ));
    }
    if apply {
        doc.save_as(&target)?;
    }
    Ok(
        json!({"applied": apply, "output": target.path(), "output_exists": target.exists(), "previous_output_hash": target.content_hash(), "map": doc.summary()}),
    )
}

fn main() -> ExitCode {
    let (output, status) = match run(Cli::parse()) {
        Ok(result) => (
            json!({"schema_version":1, "ok":true, "result":result}),
            ExitCode::SUCCESS,
        ),
        Err(error) => (
            json!({"schema_version":1, "ok":false, "error":{"code":error.code,"message":error.message}}),
            ExitCode::FAILURE,
        ),
    };
    println!("{output}");
    status
}
