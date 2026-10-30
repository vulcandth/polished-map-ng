//! Project discovery, codecs and persistence for Polished Map NG.
//! Derived from upstream master config.cpp, tiled-image.cpp, parse-asm.cpp,
//! palette-map.cpp and tileset.cpp; see upstream.lock.toml. LGPL-3.0-only.

mod asset_names;
mod blockset;
pub mod codecs;
mod collisions;
mod discovery;
mod document;
mod events;
mod graphics;
mod legacy_preferences;
mod map_palette_plan;
mod map_sizes;
mod native_fonts;
mod palette_document;
mod palettes;
mod png_export;
mod png_import;
mod print_fonts;
mod roof;
mod tile_palettes;
mod tileset;
mod transaction;

pub use blockset::{BlocksetDocument, prepare_blockset_save_as};
pub use collisions::prepare_collision_save_as;
pub use discovery::{
    MapMetadata, TilesetChoice, TilesetOptions, discover_map, find_project_root, list_tilesets,
    map_constant, tileset_choices, tileset_options,
};
pub use document::{
    MapDocument, MapOpenReport, OpenOptions, ProjectError, SaveTarget, prepare_save_as,
    read_bounded,
};
pub use events::{EventDocument, EventScript, EventScriptError, event_script_path};
pub use graphics::{
    load_master_graphics, load_master_map_graphics, write_new_png, write_png, write_png_export,
};
pub use legacy_preferences::{
    LegacyPreferenceError, LegacyPreferences, LegacyUserLocation, find_legacy_preferences,
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
pub use tileset::{MasterTileMode, MasterTilesetDocument, TilesetOutput};
