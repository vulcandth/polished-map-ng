//! Tile bitplanes and bounded LZ decoding from pinned upstream tiled-image.cpp.
//! LGPL-3.0-only; revisions are recorded in upstream.lock.toml.
use crate::Tile;
use crate::codecs::CodecError;

/// Preserve Polished Map's internal bit-plane weighting (first byte * 2).
/// The source accepts an empty image; the project compositor supplies blank slots.
pub fn decode_2bpp(bytes: &[u8]) -> Result<Vec<Tile>, CodecError> {
    if bytes.len() > 4096 || !bytes.len().is_multiple_of(16) {
        return Err(CodecError::TileLength);
    }
    decode_2bpp_pixels(bytes)
}

/// Plusplus uses the same bitplanes but permits two 256-tile banks.
pub fn decode_plusplus_2bpp(bytes: &[u8]) -> Result<Vec<Tile>, CodecError> {
    if bytes.len() > 8192 || !bytes.len().is_multiple_of(16) {
        return Err(CodecError::ExpandedTileLength);
    }
    decode_2bpp_pixels(bytes)
}

fn decode_2bpp_pixels(bytes: &[u8]) -> Result<Vec<Tile>, CodecError> {
    bytes
        .as_chunks::<16>()
        .0
        .iter()
        .map(|chunk| {
            let hues = std::array::from_fn(|i| {
                let shift = 7 - i % 8;
                ((chunk[(i / 8) * 2] >> shift) & 1) * 2 + ((chunk[(i / 8) * 2 + 1] >> shift) & 1)
            });
            Tile::new(hues).map_err(|_| CodecError::TileLength)
        })
        .collect()
}

/// Encode each tile into 16 bytes, with the first bitplane weighted by two.
///
/// The output contains exactly `tiles.len() * 16` bytes without bank padding.
pub fn encode_2bpp(tiles: &[Tile]) -> Vec<u8> {
    let mut out = Vec::with_capacity(tiles.len() * 16);
    for tile in tiles {
        for row in tile.hues().as_chunks::<8>().0 {
            let mut planes = [0u8; 2];
            for (x, hue) in row.iter().enumerate() {
                planes[0] |= ((hue >> 1) & 1) << (7 - x);
                planes[1] |= (hue & 1) << (7 - x);
            }
            out.extend_from_slice(&planes);
        }
    }
    out
}

/// Crystal LZ decompression. Returns the initialized bytes, without loader padding.
pub fn decompress_lz(input: &[u8], limit: usize) -> Result<Vec<u8>, CodecError> {
    let mut pos = 0;
    let mut out = Vec::new();
    fn take(input: &[u8], pos: &mut usize) -> Result<u8, CodecError> {
        let byte = *input.get(*pos).ok_or(CodecError::Truncated(*pos))?;
        *pos += 1;
        Ok(byte)
    }
    loop {
        let command_pos = pos;
        let byte = take(input, &mut pos)?;
        if byte == 255 {
            return Ok(out);
        }
        let mut command = byte >> 5;
        let length = if command == 7 {
            command = (byte >> 2) & 7;
            usize::from(byte & 3) * 256 + usize::from(take(input, &mut pos)?) + 1
        } else {
            usize::from(byte & 31) + 1
        };
        if length > limit.saturating_sub(out.len()) {
            return Err(CodecError::Limit(limit));
        }
        match command {
            0 => {
                for _ in 0..length {
                    out.push(take(input, &mut pos)?);
                }
            }
            1 => {
                let b = take(input, &mut pos)?;
                out.resize(out.len() + length, b);
            }
            2 => {
                let pair = [take(input, &mut pos)?, take(input, &mut pos)?];
                for i in 0..length {
                    out.push(pair[i % 2]);
                }
            }
            3 => out.resize(out.len() + length, 0),
            4..=6 => {
                let b = take(input, &mut pos)?;
                let start = if b >= 128 {
                    out.len().checked_sub(usize::from(b & 127) + 1)
                } else {
                    Some(usize::from(b) * 256 + usize::from(take(input, &mut pos)?))
                }
                .ok_or(CodecError::BackReference(command_pos))?;
                for i in 0..length {
                    let index = if command == 6 {
                        start.checked_sub(i)
                    } else {
                        start.checked_add(i)
                    }
                    .ok_or(CodecError::BackReference(command_pos))?;
                    let b = *out
                        .get(index)
                        .ok_or(CodecError::BackReference(command_pos))?;
                    out.push(if command == 5 { b.reverse_bits() } else { b });
                }
            }
            _ => return Err(CodecError::LzCommand(command_pos)),
        }
    }
}

/// Upstream read_lz_graphics passes its entire zero-initialized output buffer onward.
pub fn decode_lz_tiles(input: &[u8]) -> Result<Vec<Tile>, CodecError> {
    let mut bytes = decompress_lz(input, 4096)?;
    bytes.resize(4096, 0);
    decode_2bpp(&bytes)
}

/// Plusplus likewise passes its entire zero-initialized 8192-byte buffer on.
pub fn decode_plusplus_lz_tiles(input: &[u8]) -> Result<Vec<Tile>, CodecError> {
    let mut bytes = decompress_lz(input, 8192)?;
    bytes.resize(8192, 0);
    decode_plusplus_2bpp(&bytes)
}
