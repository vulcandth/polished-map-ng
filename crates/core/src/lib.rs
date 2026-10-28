//! GUI-independent editing and rasterization.
//!
//! Derived from Rangi42/polished-map master map.cpp, main-window.cpp, tile.h,
//! colors.h/colors.cpp, and metatile.h at the revision in upstream.lock.toml.
//! Copyright 2026 Polished Map NG contributors; LGPL-3.0-only.

mod blockset;
mod map;
mod palettes;
mod raster;
mod roof;
mod tileset;
mod tileset_export;

pub use blockset::{BlockBrush, BlockCommand, Blockset, BlocksetError, CollisionRow};
pub use map::{Command, Map, MapError, Point, ResizeAnchor};
pub use palettes::{PaletteBank, PaletteSet};
pub use raster::{Graphics, Palette, RasterError, RgbImage, Tile, day_palettes};
pub use roof::Roof;
pub use tileset::{
    TileCommand, TileSnapshot, Tileset, TilesetError, UNDEFINED_COLORS, UNDEFINED_PALETTE,
};
