//! Bounded tile/LZ decoding and recognized assembly-data parsers.
//! Adapted from pinned tiled-image.cpp, parse-asm.cpp and palette-map.cpp.
//! Implementation follows upstream source families in private `tiled_image`,
//! `parse_asm`, and `palette_map` modules. Existing codec imports remain unchanged.
//! Decoding preserves file conventions; these functions never access the filesystem.
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
/// Malformed encoded tiles, compressed data, or assembly literals.
pub enum CodecError {
    #[error("2bpp data must contain 0..=256 complete 16-byte tiles")]
    /// Input is not 0..=256 complete 16-byte tiles.
    TileLength,
    #[error("2bpp data must contain 0..=512 complete 16-byte tiles")]
    /// Input is not 0..=512 complete 16-byte tiles.
    ExpandedTileLength,
    #[error("truncated compressed data at byte {0}")]
    /// Compressed input ended at the given zero-based byte offset.
    Truncated(usize),
    #[error("invalid compressed command at byte {0}")]
    /// An unsupported compressed command begins at this byte offset.
    LzCommand(usize),
    #[error("compressed data exceeds the {0}-byte output limit")]
    /// Decompression would exceed the given output byte limit.
    Limit(usize),
    #[error("invalid compressed back-reference at byte {0}")]
    /// A command refers outside initialized output at this input byte offset.
    BackReference(usize),
    #[error("invalid assembly number on line {line}: {value}")]
    /// An assembly operand is not a supported numeric literal.
    Assembly {
        /// One-based assembly source line.
        line: usize,
        /// Rejected operand text.
        value: String,
    },
    #[error("invalid palette name on line {line}: {value}")]
    /// An assembly palette name is unrecognized.
    Palette {
        /// One-based assembly source line.
        line: usize,
        /// Rejected operand text.
        value: String,
    },
}

pub use crate::palette_map::parse_palette_map;
pub use crate::parse_asm::parse_asm;
pub use crate::tiled_image::{
    decode_2bpp, decode_lz_tiles, decode_plusplus_2bpp, decode_plusplus_lz_tiles, decompress_lz,
    encode_2bpp,
};
