use polished_map_core::Tile;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CodecError {
    #[error("2bpp data must contain 1..=256 complete 16-byte tiles")]
    TileLength,
    #[error("truncated compressed data at byte {0}")]
    Truncated(usize),
    #[error("invalid compressed command at byte {0}")]
    LzCommand(usize),
    #[error("compressed data exceeds the {0}-byte output limit")]
    Limit(usize),
    #[error("invalid compressed back-reference at byte {0}")]
    BackReference(usize),
    #[error("invalid assembly number on line {line}: {value}")]
    Assembly { line: usize, value: String },
    #[error("invalid palette name on line {line}: {value}")]
    Palette { line: usize, value: String },
}

/// Preserve Polished Map's internal bit-plane weighting (first byte * 2).
pub fn decode_2bpp(bytes: &[u8]) -> Result<Vec<Tile>, CodecError> {
    if bytes.is_empty() || bytes.len() > 4096 || !bytes.len().is_multiple_of(16) {
        return Err(CodecError::TileLength);
    }
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

/// The original data parser recognizes only indented db/dw statements.
pub fn parse_asm(text: &str) -> Result<Vec<u8>, CodecError> {
    let mut bytes = Vec::new();
    for (line, raw) in text.lines().enumerate() {
        let Some((op, arguments)) = indented_macro(raw) else {
            continue;
        };
        let words = op.eq_ignore_ascii_case("dw");
        if !words && !op.eq_ignore_ascii_case("db") {
            continue;
        }
        for value in arguments.split(',').filter(|s| !s.trim().is_empty()) {
            let value = value.trim();
            let (digits, base) = if let Some(s) = value.strip_prefix('$') {
                (s, 16)
            } else if let Some(s) = value.strip_prefix('%') {
                (s, 2)
            } else if let Some(s) = value.strip_suffix(['h', 'H']) {
                (s, 16)
            } else {
                (value, 10)
            };
            let parsed = u16::from_str_radix(digits, base)
                .ok()
                .filter(|&n| words || n <= 255)
                .ok_or_else(|| CodecError::Assembly {
                    line: line + 1,
                    value: value.into(),
                })?;
            bytes.push(parsed as u8);
            if words {
                bytes.push((parsed >> 8) as u8);
            }
        }
    }
    Ok(bytes)
}

fn indented_macro(line: &str) -> Option<(&str, &str)> {
    if !line.starts_with([' ', '\t']) {
        return None;
    }
    let statement = line.split(';').next()?.trim();
    let offset = statement.find(char::is_whitespace)?;
    Some((&statement[..offset], statement[offset..].trim()))
}

/// Palette IDs retain the upstream priority bit. Bank arguments are ignored upstream.
pub fn parse_palette_map(text: &str) -> Result<Vec<u8>, CodecError> {
    let names = [
        "GRAY", "RED", "GREEN", "WATER", "YELLOW", "BROWN", "ROOF", "TEXT",
    ];
    let mut palettes = Vec::new();
    for (line, raw) in text.lines().enumerate() {
        let Some(("tilepal", arguments)) = indented_macro(raw) else {
            continue;
        };
        for token in arguments.split(',').skip(1) {
            let token = token.trim();
            let (priority, name) = token
                .strip_prefix("PRIORITY_")
                .map_or((0, token), |s| (128, s));
            let index = names
                .iter()
                .position(|&n| n == name)
                .filter(|_| palettes.len() < 256)
                .ok_or_else(|| CodecError::Palette {
                    line: line + 1,
                    value: token.into(),
                })?;
            palettes.push(index as u8 | priority);
        }
    }
    Ok(palettes)
}
