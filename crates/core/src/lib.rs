//! GUI-independent editing and rasterization.
//!
//! # Start here
//!
//! - [`Map`] and [`Command`] edit block IDs with bounded undo/redo history.
//! - [`Blockset`], [`Tileset`] and [`Graphics`] implement classic per-tile palettes.
//! - [`attributed`] stores palette/bank/flip/priority on each block occurrence.
//!   It is used by the unified editor, not a separate application implementation.
//! - [`Events`] and [`PaletteBank`] provide shared event and color edits.
//! - [`PrintOptions`] and [`PrintTextMask`] compose CPU exports; callers supply fonts.
//!
//! No operation here reads files, opens windows or needs a graphics device.
//! Persistence wrappers live in `polished_map_project`. Public APIs are experimental.
//!
//! # Example: one undoable map edit
//!
//! ```
//! use polished_map_core::{Command, Map, Point};
//! let mut map = Map::blank(10, 9)?;
//! let at = Point { x: 2, y: 3 };
//! map.apply(&Command::Paint { points: vec![at], block: 7 })?;
//! assert_eq!(map.block(at)?, 7);
//! map.undo();
//! assert_eq!(map.block(at)?, 0);
//! # Ok::<(), polished_map_core::MapError>(())
//! ```
//!
//! Derived from Rangi42/polished-map master map.cpp, main-window.cpp, tile.h,
//! colors.h/colors.cpp, and metatile.h at the revision in upstream.lock.toml.
//! Copyright 2026 Polished Map NG contributors; LGPL-3.0-only.

#![deny(missing_docs)]

pub mod attributed;
mod blockset;
pub mod event_catalog;
mod events;
mod map;
mod palettes;
mod pixel_edit;

/// Compatibility import for callers using the former branch-named module.
#[doc(hidden)]
pub use attributed as plusplus;
mod printing;
mod priority;
mod raster;
mod roof;
mod tileset;
mod tileset_export;
mod warp_label;

pub use blockset::{BlockBrush, BlockCommand, Blockset, BlocksetError, CollisionRow};
pub use events::{Event, EventCommand, EventDefinition, EventError, EventKind, Events};
pub use map::{Command, Map, MapError, Point, ResizeAnchor};
pub use palettes::{ColorSlot, PaletteBank, PaletteCommand, PaletteError, PaletteSet};
pub use printing::{PrintError, PrintFont, PrintOptions, PrintStyle, PrintTextMask};
pub use priority::PriorityPattern;
pub use raster::{Graphics, Palette, RasterError, RgbImage, Tile, day_palettes};
pub use roof::Roof;
pub use tileset::{
    TileCommand, TileSnapshot, Tileset, TilesetError, UNDEFINED_COLORS, UNDEFINED_PALETTE,
};
pub use warp_label::{WarpDigit, WarpIdLabel};
