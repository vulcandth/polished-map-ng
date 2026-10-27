//! Experimental headless frontend. LGPL-3.0-only.
use std::{path::PathBuf, process::ExitCode};

use clap::{Args, Parser, Subcommand};
use polished_map_core::Command;
use polished_map_project::{
    MapDocument, OpenOptions, ProjectError, load_master_graphics, read_bounded, write_png,
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
    /// Inspect dimensions, block IDs and the original content hash.
    Inspect {
        #[command(flatten)]
        input: MapArgs,
    },
    /// Validate and preview edits. Files are written only with --apply.
    Edit {
        #[command(flatten)]
        input: MapArgs,
        /// File containing an array of paint/fill/replace/swap commands.
        #[arg(long)]
        commands: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        expected_hash: Option<String>,
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
        #[arg(long)]
        output: PathBuf,
    },
}

struct Failure {
    code: String,
    message: String,
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
            if expected_hash.is_some_and(|expected| expected != doc.content_hash()) {
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
            for command in commands {
                doc.map_mut()
                    .apply(&command)
                    .map_err(|e| Failure::new("invalid_command", e))?;
            }
            let changed = doc.map().blocks() != before;
            if apply {
                doc.save()?;
            }
            Ok(
                json!({"applied": apply, "changed": changed, "map": doc.summary(), "blocks": doc.map().blocks()}),
            )
        }
        Action::Render {
            input,
            tileset,
            allow_256,
            output,
        } => {
            if output.exists() {
                return Err(Failure::new("output_exists", "choose a new output path"));
            }
            let doc = input.open()?;
            let root = doc.metadata().root.as_deref().ok_or_else(|| {
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
            let graphics = load_master_graphics(root, tileset, allow_256)?;
            let pixels = graphics
                .render_map(doc.map())
                .map_err(|e| Failure::new("invalid_graphics", e))?;
            write_png(&output, &pixels)?;
            Ok(json!({"output":output, "width":pixels.width, "height":pixels.height}))
        }
    }
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
