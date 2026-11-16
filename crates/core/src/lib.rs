//! Shared headless editing, project I/O and CPU rendering for Polished Map NG.
//!
//! # Start here
//!
//! - [`Map`] and [`Command`] edit block IDs with bounded undo/redo history.
//! - [`Blockset`], [`Tileset`] and [`Graphics`] implement classic per-tile palettes.
//! - [`attributed`] stores palette/bank/flip/priority on each block occurrence.
//!   It is used by the unified editor, not a separate application implementation.
//! - [`Events`] and [`PaletteBank`] provide shared event and color edits.
//! - [`MapDocument`], [`BlocksetDocument`] and [`MasterTilesetDocument`] own explicit file lifecycles.
//! - [`detect_map_format`] selects storage without converting project bytes.
//! - [`codecs`] and [`SaveTarget`] provide bounded decoding and guarded persistence.
//! - [`PrintOptions`], [`PrintFonts`] and [`NativeFonts`] compose CPU exports.
//!
//! Models, codecs, discovery, guarded persistence and CPU font rendering share
//! this headless backend. File operations are explicit; no operation opens windows
//! or needs a graphics device. Public APIs are experimental.
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
//! # Example: inspect project context without opening a window or writing files
//!
//! ```no_run
//! use std::path::Path;
//! use polished_map_core::detect_map_format;
//! let context = detect_map_format(Path::new("maps/CherrygroveCity.ablk"), None, None)?;
//! println!("{:?}: {:?}", context.encoding(), context.format().attribute_storage());
//! # Ok::<(), polished_map_core::FormatDetectionError>(())
//! ```
//!
//! Derived from Rangi42/polished-map master map.cpp, main-window.cpp, tile.h,
//! colors.h/colors.cpp, metatile.h, config.cpp, tiled-image.cpp and parse-asm.cpp
//! at the revisions in upstream.lock.toml.
//! Copyright 2026 Polished Map NG contributors; LGPL-3.0-only.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod asset_names;
mod asset_paths;
pub mod attributed;
pub mod codecs;
mod collisions;
mod colors;
mod config;
mod error;
mod event;
pub mod event_catalog;
mod file_io;
mod format;
mod graphics;
mod image;
mod legacy_preferences;
mod map;
mod map_events;
mod map_palette_plan;
mod map_sizes;
mod metatile;
mod metatileset;
mod native_fonts;
mod palette_map;
mod parse_asm;
mod pixel_edit;
mod png_export;
mod png_import;
mod print_fonts;
mod printing;
mod priority;
mod raster;
mod roof;
mod roof_layout;
mod tile;
mod tiled_image;
mod tileset;
mod tileset_export;
mod transaction;
mod warp_label;

pub use collisions::prepare_collision_save_as;
pub use colors::{
    ColorSlot, MasterGraphicsOptions, PaletteBank, PaletteCommand, PaletteDocument, PaletteError,
    PaletteSet, RoofPaletteLayout, parse_rgb_colors, parse_rgb_palettes, prepare_palette_export,
    roof_palette_path,
};
pub use config::{
    MapMetadata, TilesetChoice, TilesetOptions, discover_map, discover_unified_map,
    find_project_root, list_tilesets, map_constant, map_path_with_context, tileset_choices,
    tileset_options,
};
pub use error::ProjectError;
pub use event::{Event, EventCommand, EventDefinition, EventError, EventKind, Events};
pub use file_io::{SaveTarget, read_bounded};
pub use format::{
    AttributeStorage, DetectedFormat, FormatDetectionError, FormatEvidence, FormatEvidenceKind,
    MapEncoding, ProjectFormat, detect_map_format, detect_tileset_format,
    find_unified_project_root, unified_tileset_choices,
};
pub use graphics::{load_master_graphics, load_master_map_graphics, read_unified_tiles};
pub use image::{write_new_png, write_png, write_png_export};
pub use legacy_preferences::{
    LegacyPreferenceError, LegacyPreferences, LegacyUserLocation, find_legacy_plusplus_preferences,
    find_legacy_preferences,
};
pub use map::{
    Command, Map, MapDocument, MapError, MapOpenReport, MapSummary, OpenOptions, Point,
    ResizeAnchor, prepare_save_as,
};
pub use map_events::{EventDocument, EventScript, EventScriptError, event_script_path};
pub use map_palette_plan::MapPalettePlan;
pub use map_sizes::{MapSizeOptions, inspect_map_sizes};
pub use metatile::BlockBrush;
pub use metatileset::{
    BlockCommand, Blockset, BlocksetDocument, BlocksetError, CollisionRow, prepare_blockset_save_as,
};
pub use native_fonts::{FontPlatform, NativeFontFace, NativeFonts};
pub use png_export::{PngExportTarget, prepare_png_export};
pub use png_import::PngPlatform;
pub use print_fonts::{PrintFontFace, PrintFonts};
pub use printing::{PrintError, PrintFont, PrintOptions, PrintStyle, PrintTextMask};
pub use priority::PriorityPattern;
pub use raster::{Graphics, Palette, RasterError, RgbImage, day_palettes};
pub use roof::{Roof, RoofDocument, list_roofs};
pub use roof_layout::infer_roof_palette_layout;
pub use tile::Tile;
pub use tileset::{
    MasterTileMode, MasterTilesetDocument, TileCommand, TileSnapshot, Tileset, TilesetError,
    TilesetOutput, UNDEFINED_COLORS, UNDEFINED_PALETTE,
};
pub use warp_label::{WarpDigit, WarpIdLabel};

/// Compatibility import for callers using the former branch-named module.
#[doc(hidden)]
pub use attributed as plusplus;
