//! CLI grammar and value parsers. Keep flag names and defaults stable. LGPL-3.0-only.
use crate::printing;
use clap::{Args, Parser, Subcommand};
use polished_map_core::PaletteSet;
use polished_map_project::PngPlatform;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Polished Map NG experimental headless editor")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) action: Action,
}

#[derive(Args)]
pub(crate) struct MapArgs {
    pub(crate) map: PathBuf,
    /// Project directory; otherwise discover the nearest ancestor Makefile.
    #[arg(long)]
    pub(crate) root: Option<PathBuf>,
    #[arg(long)]
    pub(crate) width: Option<u16>,
    #[arg(long)]
    pub(crate) height: Option<u16>,
}

#[derive(Subcommand)]
pub(crate) enum Action {
    /// Inspect event coordinates, source lines and warp destinations in assembly.
    InspectEvents { input: PathBuf },
    /// Preview event edits, preserving unrelated assembly. Apply requires a hash.
    EditEvents {
        input: PathBuf,
        #[arg(long)]
        commands: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        expected_hash: Option<String>,
    },
    /// Inspect all six palette sets after importing an upstream RGB palette file.
    InspectPalettes { input: PathBuf },
    /// Preview color edits and RGB5 export. Output is always an explicit destination.
    EditPalettes {
        input: PathBuf,
        #[arg(long)]
        commands: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        monochrome: bool,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        expected_hash: Option<String>,
        #[arg(long)]
        expected_output_hash: Option<String>,
    },
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
        #[command(flatten)]
        png: PngArgs,
        /// Override automatic map palette selection.
        #[arg(long, value_parser = parse_palette_set)]
        palette: Option<PaletteSet>,
        /// Optional nine-tile graphic replacement from gfx/tilesets/roofs.
        #[arg(long)]
        roof: Option<String>,
        #[command(flatten)]
        print: printing::PrintArgs,
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Args)]
pub(crate) struct PngArgs {
    /// Original PNG transparency behavior: unix (Linux/macOS) or windows; defaults to this host.
    #[arg(long, value_parser = parse_png_platform)]
    pub(crate) png_platform: Option<PngPlatform>,
}

impl PngArgs {
    pub(crate) fn platform(&self) -> PngPlatform {
        self.png_platform.unwrap_or_default()
    }
}

fn parse_png_platform(value: &str) -> Result<PngPlatform, String> {
    serde_json::from_value(Value::String(value.into()))
        .map_err(|_| "expected unix (Linux/macOS) or windows".into())
}

#[derive(Args)]
pub(crate) struct RoofArgs {
    pub(crate) roof: String,
    #[arg(long)]
    pub(crate) root: PathBuf,
    #[command(flatten)]
    pub(crate) png: PngArgs,
}

#[derive(Args)]
pub(crate) struct TilesetArgs {
    pub(crate) tileset: String,
    #[arg(long)]
    pub(crate) root: PathBuf,
    #[arg(long)]
    pub(crate) allow_256: bool,
    #[arg(long)]
    pub(crate) monochrome: bool,
    #[command(flatten)]
    pub(crate) png: PngArgs,
}

fn parse_palette_set(value: &str) -> Result<PaletteSet, String> {
    serde_json::from_value(Value::String(value.into()))
        .map_err(|_| "expected morn, day, nite, darkness, indoor, or custom".into())
}
