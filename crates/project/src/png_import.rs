//! master Tiled_Image::read_png_graphics and FLTK 1.4.5 PNG/desaturation behavior.
//! LGPL-3.0-only adaptation; original FLTK attribution/terms in NOTICE.md and
//! LICENSES/FLTK.txt. PNG expansion/STRIP_16 precedes platform RGBA handling.
use crate::ProjectError;
use polished_map_core::Tile;
use std::{io::Cursor, path::Path};

/// Select the original PNG import behavior explicitly for reproducible headless
/// work. Only Windows clears RGB in fully transparent expanded RGBA pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum PngPlatform {
    /// Linux and macOS preserve RGB in fully transparent expanded pixels.
    Unix = 0,
    /// FLTK's Windows adapter clears RGB when expanded RGBA alpha is zero.
    Windows = 1,
}

impl Default for PngPlatform {
    fn default() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}

pub(crate) fn read_png_tiles(
    path: &Path,
    bytes: &[u8],
    platform: PngPlatform,
) -> Result<Vec<Tile>, ProjectError> {
    let invalid = |message: String| ProjectError::Format {
        path: path.into(),
        message,
    };
    let mut decoder = png::Decoder::new_with_limits(
        Cursor::new(bytes),
        png::Limits {
            bytes: 8 * 1024 * 1024,
        },
    );
    // FLTK uses png_set_expand and png_set_strip_16. Generic image conversion
    // rounds u16 samples instead, which changes hues around 64/128/192.
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    decoder.set_ignore_text_chunk(true);
    decoder.set_ignore_iccp_chunk(true);
    let mut reader = decoder.read_info().map_err(|e| invalid(e.to_string()))?;
    let (width, height) = (reader.info().width, reader.info().height);
    if width == 0
        || height == 0
        || width > 2048
        || height > 2048
        || width % 8 != 0
        || height % 8 != 0
        || width * height > 256 * 64
    {
        return Err(invalid(
            "PNG must contain 1..=256 complete 8×8 tiles".into(),
        ));
    }
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| invalid("PNG output size exceeds addressable memory".into()))?;
    let mut pixels = vec![0; size];
    let output = reader
        .next_frame(&mut pixels)
        .map_err(|e| invalid(e.to_string()))?;
    reader.finish().map_err(|e| invalid(e.to_string()))?;
    if output.width != width || output.height != height || output.bit_depth != png::BitDepth::Eight
    {
        return Err(invalid(
            "PNG default image has unexpected dimensions or depth".into(),
        ));
    }
    let channels = match output.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        png::ColorType::Indexed => return Err(invalid("PNG palette was not expanded".into())),
    };
    let mut tiles = Vec::new();
    for ty in 0..height / 8 {
        for tx in 0..width / 8 {
            let hues = std::array::from_fn(|i| {
                let x = tx as usize * 8 + i % 8;
                let y = ty as usize * 8 + i / 8;
                let p = &pixels[y * output.line_size + x * channels..];
                let gray = if channels < 3 {
                    u16::from(p[0])
                } else if platform == PngPlatform::Windows && channels == 4 && p[3] == 0 {
                    0
                } else {
                    (u16::from(p[0]) * 31 + u16::from(p[1]) * 61 + u16::from(p[2]) * 8) / 100
                };
                [3, 1, 2, 0][usize::from(gray / 64)]
            });
            tiles.push(Tile::new(hues)?);
        }
    }
    Ok(tiles)
}
