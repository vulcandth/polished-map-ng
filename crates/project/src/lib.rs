//! Project discovery, codecs and persistence for Polished Map NG.
//!
//! # Choosing an entry point
//!
//! Start with [`detect_map_format`] for a project whose storage is not known.
//! Detection chooses a loader; it does not validate or convert asset contents.
//! [`MapEncoding`] is independent of [`AttributeStorage`]: an ASM map can use
//! either storage model. Preserve the returned context through editing and saving.
//!
//! | Responsibility | API |
//! |---|---|
//! | Context and discovery | [`DetectedFormat`], [`discover_unified_map`] |
//! | Classic maps/blocks/graphics | [`MapDocument`], [`BlocksetDocument`], [`MasterTilesetDocument`] |
//! | Per-occurrence attributes | [`attributed`] |
//! | Source-preserving events | [`EventDocument`] |
//! | Shared decoding and bounded reads | [`codecs`], [`read_bounded`] |
//! | Observed destinations and guarded writes | [`SaveTarget`], [`prepare_save_as`] |
//! | CPU Print font adapters | [`PrintFonts`], [`NativeFonts`] |
//!
//! Constructors named `open_unified_in_project` accept the combined supported
//! source layouts. Fixed-format entry points retain their discovery precedence.
//! The old `plusplus` module import remains an alias for existing Rust callers;
//! implementation lives in [`attributed`]. These APIs are not yet stabilized.
//!
//! # Example: inspect context without opening a window or writing files
//!
//! ```no_run
//! use std::path::Path;
//! use polished_map_project::detect_map_format;
//! let context = detect_map_format(Path::new("maps/CherrygroveCity.ablk"), None, None)?;
//! println!("{:?}: {:?}", context.encoding(), context.format().attribute_storage());
//! # Ok::<(), polished_map_project::FormatDetectionError>(())
//! ```
//!
//! Reads/codecs return typed errors. Writes compare observed source hashes and
//! stage replacements; a conflict requires reloading or obtaining new consent,
//! not an unchecked retry. Multi-domain saves are not a whole-project transaction.
//!
//! Derived from upstream master config.cpp, tiled-image.cpp, parse-asm.cpp,
//! palette-map.cpp and tileset.cpp; see upstream.lock.toml. LGPL-3.0-only.

mod asset_names;
mod asset_paths;
pub mod attributed;
mod blockset;
pub mod codecs;
mod collisions;
mod discovery;
mod document;
mod events;
mod format;
mod graphics;
mod legacy_preferences;
mod map_palette_plan;
mod map_sizes;
mod native_fonts;
mod palette_document;
mod palettes;

/// Compatibility import for callers using the former branch-named module.
#[doc(hidden)]
pub use attributed as plusplus;
mod png_export;
mod png_import;
mod print_fonts;
mod roof;
mod roof_layout;
mod tile_palettes;
mod tileset;
mod transaction;

pub use blockset::{BlocksetDocument, prepare_blockset_save_as};
pub use collisions::prepare_collision_save_as;
pub use discovery::{
    MapMetadata, TilesetChoice, TilesetOptions, discover_map, discover_unified_map,
    find_project_root, list_tilesets, map_constant, map_path_with_context, tileset_choices,
    tileset_options,
};
pub use document::{
    MapDocument, MapOpenReport, OpenOptions, ProjectError, SaveTarget, prepare_save_as,
    read_bounded,
};
pub use events::{EventDocument, EventScript, EventScriptError, event_script_path};
pub use format::{
    AttributeStorage, DetectedFormat, FormatDetectionError, FormatEvidence, FormatEvidenceKind,
    MapEncoding, ProjectFormat, detect_map_format, detect_tileset_format,
    find_unified_project_root, unified_tileset_choices,
};
pub use graphics::{
    load_master_graphics, load_master_map_graphics, read_unified_tiles, write_new_png, write_png,
    write_png_export,
};
pub use legacy_preferences::{
    LegacyPreferenceError, LegacyPreferences, LegacyUserLocation, find_legacy_plusplus_preferences,
    find_legacy_preferences,
};
pub use map_palette_plan::MapPalettePlan;
pub use map_sizes::{MapSizeOptions, inspect_map_sizes};
pub use native_fonts::{FontPlatform, NativeFontFace, NativeFonts};
pub use palette_document::{PaletteDocument, prepare_palette_export};
pub use palettes::{
    MasterGraphicsOptions, RoofPaletteLayout, parse_rgb_colors, parse_rgb_palettes,
    roof_palette_path,
};
pub use png_export::{PngExportTarget, prepare_png_export};
pub use png_import::PngPlatform;
pub use print_fonts::{PrintFontFace, PrintFonts};
pub use roof::{RoofDocument, list_roofs};
pub use roof_layout::infer_roof_palette_layout;
pub use tileset::{MasterTileMode, MasterTilesetDocument, TilesetOutput};
