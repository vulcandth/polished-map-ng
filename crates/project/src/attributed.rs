//! Project documents for separate block-occurrence attributes and raw graphics.
//! Adapted from pinned plusplus config.cpp, option-dialogs.cpp, colors.cpp and
//! metatileset.cpp; LGPL-3.0-only. Shares editing/codecs/guarded persistence,
//! with the classic model, without converting either on disk.
//!
//! Use [`crate::detect_map_format`] before choosing a storage adapter. The unified
//! editor uses `open_unified_in_project` constructors here; ordinary
//! `open_in_project` constructors retain the fixed launcher's narrower discovery.
//! [`BlocksetDocument`] guards paired offsets/attributes/collisions,
//! [`TilesetDocument`] owns raw graphics, and [`PaletteDocument`] owns eight
//! game palettes. All mutations use the validated core models.

use crate::ProjectError;
use std::path::{Path, PathBuf};

mod blockset;
mod events;
mod map;
mod palettes;
mod roof;
mod tileset;
pub use blockset::{BlocksetDocument, prepare_binary_save_as};
pub use events::{
    EventDocument, event_script_for_map, event_script_path, parse_event_script, warp_map_path,
};
pub use map::{
    MapDocument, MapOpenReport, discover_map, find_project_root, inspect_map_sizes, list_tilesets,
    prepare_save_as, tileset_choices, tileset_options,
};
pub use palettes::{MapPalettePlan, PaletteDocument, PaletteOptions};
pub use roof::{RoofDocument, list_roofs};
pub use tileset::TilesetDocument;

fn graphics_path(directory: &Path, name: &str) -> PathBuf {
    ["png", "2bpp", "2bpp.lz"]
        .into_iter()
        .map(|ext| directory.join(format!("{name}.{ext}")))
        .find(|p| p.is_file())
        .unwrap_or_else(|| directory.join(format!("{name}.2bpp.lz")))
}

/// Read plusplus PNG/2bpp/LZ graphics without accepting master's CHR path.
/// The same bounded PNG/desaturation and LZ implementations serve both formats.
pub fn read_tiles(
    path: &Path,
    platform: crate::PngPlatform,
) -> Result<Vec<polished_map_core::Tile>, ProjectError> {
    crate::graphics::read_tiles_for_storage(path, platform, crate::ProjectFormat::PlusPlus, false)
}
