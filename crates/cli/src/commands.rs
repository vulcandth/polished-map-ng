//! Command execution over validated core/project operations. LGPL-3.0-only.
use crate::{args::*, output::*};
use polished_map_core::{BlockCommand, Command, EventCommand, PaletteCommand, TileCommand};
use polished_map_project::{
    BlocksetDocument, EventDocument, MapDocument, MasterGraphicsOptions, MasterTilesetDocument,
    OpenOptions, PaletteDocument, ProjectError, RoofDocument, load_master_map_graphics,
    prepare_blockset_save_as, prepare_collision_save_as, prepare_palette_export, prepare_save_as,
    read_bounded, write_new_png,
};
use serde_json::{Value, json};
use std::path::PathBuf;

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

impl RoofArgs {
    fn open(&self) -> Result<RoofDocument, Failure> {
        // Standalone pixel edits need no map/tileset; palette attributes are
        // contextual in the GUI and never serialized into roof PNGs.
        Ok(RoofDocument::open_in_project_with_png_platform(
            &self.root,
            &self.roof,
            [9; 9],
            self.png.platform(),
        )?)
    }
}

impl TilesetArgs {
    fn open(&self) -> Result<(MasterTilesetDocument, BlocksetDocument), Failure> {
        Ok((
            MasterTilesetDocument::open_in_project(
                &self.root,
                &self.tileset,
                &MasterGraphicsOptions {
                    png_platform: self.png.platform(),
                    allow_256: self.allow_256,
                    monochrome: self.monochrome,
                    ..Default::default()
                },
            )?,
            BlocksetDocument::open_in_project(&self.root, &self.tileset)?,
        ))
    }
}

pub(crate) fn run(cli: Cli) -> Result<Value, Failure> {
    match cli.action {
        Action::InspectEvents { input } => Ok(event_summary(&EventDocument::open(&input)?)),
        Action::EditEvents {
            input,
            commands,
            apply,
            expected_hash,
        } => {
            let mut doc = EventDocument::open(&input)?;
            let before = doc.source().content_hash();
            if expected_hash
                .as_ref()
                .is_some_and(|hash| Some(hash) != before.as_ref())
            {
                return Err(Failure::new(
                    "conflict",
                    "event script differs from the observed state",
                ));
            }
            if apply && expected_hash.is_none() {
                return Err(Failure::new(
                    "expected_hash_required",
                    "preview first and supply --expected-hash",
                ));
            }
            let commands: Vec<EventCommand> =
                serde_json::from_slice(&read_bounded(&commands, 4 * 1024 * 1024)?)
                    .map_err(|e| Failure::new("invalid_commands", e))?;
            doc.apply(&commands)?;
            let planned = doc.preview_save()?;
            if apply {
                doc.save()?;
            }
            let mut response = event_summary(&doc);
            response["applied"] = json!(apply);
            response["previous_hash"] = json!(before);
            response["planned_output"] = json!(planned);
            Ok(response)
        }
        Action::InspectPalettes { input } => {
            let mut doc = PaletteDocument::default();
            doc.load(&input)?;
            Ok(palette_summary(&doc))
        }
        Action::EditPalettes {
            input,
            commands,
            output,
            monochrome,
            apply,
            expected_hash,
            expected_output_hash,
        } => {
            let mut doc = PaletteDocument::default();
            doc.load(&input)?;
            let before = doc.content_hash();
            if expected_hash.as_ref().is_some_and(|h| h != &before) {
                return Err(Failure::new(
                    "conflict",
                    "palette input differs from the observed state",
                ));
            }
            if apply && expected_hash.is_none() {
                return Err(Failure::new(
                    "expected_hash_required",
                    "preview first and supply --expected-hash",
                ));
            }
            let commands: Vec<PaletteCommand> =
                serde_json::from_slice(&read_bounded(&commands, 4 * 1024 * 1024)?)
                    .map_err(|e| Failure::new("invalid_commands", e))?;
            doc.apply_batch(&commands)?;
            doc.verify_source()?;
            let target = output.as_deref().map(prepare_palette_export).transpose()?;
            let previous_output_hash = target.as_ref().and_then(|t| t.content_hash());
            if expected_output_hash
                .as_ref()
                .is_some_and(|h| Some(h) != previous_output_hash.as_ref())
            {
                return Err(Failure::new(
                    "conflict",
                    "palette output differs from the observed state",
                ));
            }
            if apply {
                let target = target.as_ref().ok_or_else(|| {
                    Failure::new("destination_required", "palette export requires --output")
                })?;
                if target.exists() && expected_output_hash.is_none() {
                    return Err(Failure::new(
                        "overwrite_confirmation_required",
                        "supply --expected-output-hash from preview to replace the destination",
                    ));
                }
                doc.export(target, monochrome)?;
            }
            let mut response = palette_summary(&doc);
            response["previous_hash"] = json!(before);
            response["applied"] = json!(apply);
            response["output"] = json!(target.as_ref().map(|t| t.path()));
            response["previous_output_hash"] = json!(previous_output_hash);
            response["export_rgb"] = json!(doc.bank().export_rgb(monochrome));
            Ok(response)
        }
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
            // This command-oriented adapter retains its automatic priority mode.
            // The GUI exposes an explicit option that can remain off with old bits.
            if doc.tileset().attributes().iter().any(|&a| a & 0x80 != 0) {
                doc.set_allow_priority(true);
            }
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
            png,
            palette,
            roof,
            print,
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
                    png_platform: png.platform(),
                    allow_256,
                    palette,
                    roof,
                    ..Default::default()
                },
            )?;
            let (pixels, print_details) = print.render(&graphics, &doc)?;
            write_new_png(&output, &pixels)?;
            let mut result = json!({"output":output, "width":pixels.width, "height":pixels.height, "png_platform":png.platform()});
            if !print_details.is_null() {
                result["print"] = print_details;
            }
            Ok(result)
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
