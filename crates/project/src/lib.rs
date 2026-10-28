//! Project discovery, codecs and persistence for Polished Map NG.
//! Derived from upstream master config.cpp, tiled-image.cpp, parse-asm.cpp,
//! palette-map.cpp and tileset.cpp; see upstream.lock.toml. LGPL-3.0-only.

mod blockset;
pub mod codecs;
mod collisions;
mod discovery;
mod document;
mod graphics;
mod palettes;
mod roof;
mod tile_palettes;
mod tileset;
mod transaction;

pub use blockset::{BlocksetDocument, prepare_blockset_save_as};
pub use collisions::prepare_collision_save_as;
pub use discovery::{MapMetadata, discover_map, find_project_root, list_tilesets, map_constant};
pub use document::{
    MapDocument, OpenOptions, ProjectError, SaveTarget, prepare_save_as, read_bounded,
};
pub use graphics::{load_master_graphics, load_master_map_graphics, write_png};
pub use palettes::{
    MasterGraphicsOptions, RoofPaletteLayout, parse_rgb_colors, parse_rgb_palettes,
};
pub use roof::RoofDocument;
pub use tileset::{MasterTilesetDocument, TilesetOutput};
