//! master palette-map.cpp, Image::write_tileset_image and Tileset::print_rgb.
use crate::{RgbImage, Tileset, TilesetError, UNDEFINED_PALETTE};

impl Tileset {
    /// Preserve upstream bank/gap/line conventions. Empty maps return an error
    /// instead of the original backwards-read underflow.
    pub fn palette_map_asm(
        &self,
        allow_256: bool,
        allow_priority: bool,
    ) -> Result<String, TilesetError> {
        let n = self
            .attributes()
            .iter()
            .rposition(|&p| p != UNDEFINED_PALETTE)
            .map(|p| p + 1)
            .ok_or(TilesetError::ExportLayout)?;
        let names = [
            "GRAY", "RED", "GREEN", "WATER", "YELLOW", "BROWN", "ROOF", "TEXT",
        ];
        let mut text = String::new();
        let mut i = 0;
        while i < n {
            if !allow_256 && (i == 0x60 || i == 0xe0) {
                text.push_str(if allow_priority {
                    "\nrept 32\n\tdb $ff\nendr\n\n"
                } else {
                    "\nrept 16\n\tdb $ff\nendr\n\n"
                });
                i += 32;
                continue;
            }
            if i % 8 == 0 {
                text.push_str(if i < 0x80 {
                    "\ttilepal 0"
                } else {
                    "\ttilepal 1"
                });
            }
            text.push_str(", ");
            let a = self.attributes()[i];
            if a & 0x80 != 0 {
                text.push_str("PRIORITY_");
            }
            text.push_str(names[usize::from(a & 15).min(7)]);
            if i % 8 == 7 {
                text.push('\n');
            }
            i += 1;
        }
        if !allow_priority {
            if n % 2 == 1 {
                text.push_str(", TEXT\n");
            } else if n % 8 != 0 {
                text.push('\n');
            }
        }
        Ok(text)
    }

    /// Grayscale pixels before PNG encoding. Offsets/counts are master tile-slot
    /// coordinates, not file tile order. Unsafe compound offset/height pairs
    /// fail rather than reproducing the original out-of-bounds writes.
    pub fn export_region(
        &self,
        offset: usize,
        count: Option<usize>,
        allow_256: bool,
    ) -> Result<RgbImage, TilesetError> {
        let tail = self
            .attributes()
            .get(offset..)
            .ok_or(TilesetError::ExportLayout)?;
        let n = count.unwrap_or_else(|| {
            tail.iter()
                .rposition(|&p| p != UNDEFINED_PALETTE)
                .map_or(0, |i| i + 1)
        });
        if n == 0 || n > tail.len() {
            return Err(TilesetError::ExportLayout);
        }
        let mut height = n.div_ceil(16) * 8;
        if !allow_256 && offset < 0x60 && offset + n > 0x60 {
            height = height.checked_sub(16).ok_or(TilesetError::ExportLayout)?;
        }
        if height == 0 {
            return Err(TilesetError::ExportLayout);
        }
        let mut rgb = vec![255; 128 * height * 3];
        let mut i = 0;
        while i < n {
            let j = i + offset;
            if !allow_256 && j == 0x60 {
                i += 32;
                continue;
            }
            let mut ty = (i / 16) * 8;
            if !allow_256 && j >= 0x80 {
                ty = ty.checked_sub(16).ok_or(TilesetError::ExportLayout)?;
            }
            if ty + 8 > height {
                return Err(TilesetError::ExportLayout);
            }
            let tx = (i % 16) * 8;
            for (p, &hue) in self.tiles()[j].hues().iter().enumerate() {
                let at = ((ty + p / 8) * 128 + tx + p % 8) * 3;
                rgb[at..at + 3].fill([255, 85, 170, 0][usize::from(hue)]);
            }
            i += 1;
        }
        Ok(RgbImage {
            width: 128,
            height: height as u32,
            pixels: rgb,
        })
    }
}
