//! Project discovery, codecs and persistence for Polished Map NG.
//! Derived from upstream master config.cpp, tiled-image.cpp, parse-asm.cpp,
//! palette-map.cpp and tileset.cpp; see upstream.lock.toml. LGPL-3.0-only.

pub mod codecs;
mod document;
mod graphics;

pub use document::{MapDocument, OpenOptions, ProjectError, read_bounded};
pub use graphics::{load_master_graphics, write_png};
