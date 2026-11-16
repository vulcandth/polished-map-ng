//! Per-occurrence block attributes and bank addressing, adapted from
//! `tile.cpp`, `block-window.cpp` and `metatileset.cpp` at the plusplus revision
//! in upstream.lock.toml. Copyright Rangi42 and Polished Map NG contributors.
//! LGPL-3.0-only. Shared by the unified editor and fixed attributed launcher.
//!
//! [`AttributeBlockset`] pairs each block's 16 offsets with 16 attribute bytes.
//! [`TileReference`] interprets those bytes; [`RawTileset`] edits pixels without
//! changing occurrence attributes. [`Graphics`] combines both for CPU rendering.
//! Classic per-tile palettes remain in the crate-root [`crate::Tileset`] model.
//! [`BlocksetDocument`], [`TilesetDocument`] and [`PaletteDocument`] own the
//! corresponding files, without converting classic and attributed storage.
//! Use [`crate::detect_map_format`] before selecting a document. Fixed-format
//! loaders retain their discovery rules; `open_unified_in_project` accepts the
//! combined supported layouts. All writes retain their conflict guards.
//!
//! ```
//! use polished_map_core::attributed::{BankOrder, TileReference};
//! let tile = TileReference::from_index(0x100, BankOrder::Bank1First)?;
//! assert!(tile.bank1());
//! assert_eq!(tile.index(BankOrder::Bank1First), 0x100);
//! # Ok::<(), polished_map_core::attributed::Error>(())
//! ```

#![deny(missing_docs)]

use thiserror::Error;

use crate::{BlocksetError, ProjectError};
use std::path::{Path, PathBuf};

mod colors;
mod deep_tile;
mod graphics;
mod map;
mod map_events;
mod metatileset;
mod roof;
mod tile;
mod tileset;
pub use colors::{MapPalettePlan, PaletteDocument, PaletteOptions};
pub use deep_tile::{RawTileCommand, RawTileSnapshot, RawTileset};
pub use graphics::Graphics;
pub use map::{
    MapDocument, MapOpenReport, discover_map, find_project_root, inspect_map_sizes, list_tilesets,
    prepare_save_as, tileset_choices, tileset_options,
};
pub use map_events::{
    EventDocument, event_script_for_map, event_script_path, parse_event_script, warp_map_path,
};
pub use metatileset::{AttributeBlockset, AttributeCommand};
pub use metatileset::{BlocksetDocument, prepare_binary_save_as};
pub use roof::{RoofDocument, list_roofs};
pub use tile::{BankOrder, TileReference};
pub use tileset::TilesetDocument;

// Preserve existing imports; source syntax is independent of attribute storage.
#[doc(hidden)]
pub use crate::event_catalog::{EVENT_KINDS, event_kind_from_macro};

#[derive(Debug, Error, PartialEq, Eq)]
/// Validation failures for occurrence attributes and raw graphics.
pub enum Error {
    #[error(transparent)]
    /// Pixel dimensions, hue values or image payload are invalid.
    Pixels(#[from] crate::TilesetError),
    #[error("expected 1..=512 raw tiles and one defined flag per tile")]
    /// Tile count is outside 1–512 or the definition flags have a different length.
    GraphicsLength,
    #[error(transparent)]
    /// A shared block/collision invariant was violated.
    Blockset(#[from] BlocksetError),
    #[error("attribute data must contain complete 16-byte rows, at most one per block")]
    /// Attribute bytes contain a partial row or more rows than the offsets.
    AttributeLength,
    #[error("tile {0} is outside the 512-entry graphics sheet")]
    /// The requested raw tile index is outside the supported or loaded sheet.
    TileIndex(u16),
    #[error("palette {0} is outside 0..=7")]
    /// An occurrence palette is outside 0–7.
    Palette(u8),
}

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
) -> Result<Vec<crate::Tile>, ProjectError> {
    crate::graphics::read_tiles_for_storage(path, platform, crate::ProjectFormat::PlusPlus, false)
}
