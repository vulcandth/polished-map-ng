//! master roof-window.cpp and Tileset::print_roof_rgb. LGPL-3.0-only.
use crate::{RgbImage, Tile, TileCommand, Tileset, TilesetError, UNDEFINED_PALETTE};

/// Nine roof tiles in file order. Roof commands use indices 0..8, corresponding
/// to master slots $0A..$12. The GUI reverses rows; serialization does not.
#[derive(Clone, Debug)]
pub struct Roof {
    model: Tileset,
}

impl Roof {
    pub fn new(tiles: [Tile; 9], attributes: [u8; 9]) -> Result<Self, TilesetError> {
        Ok(Self {
            model: Tileset::new(tiles.into(), attributes.into())?,
        })
    }
    pub fn tileset(&self) -> &Tileset {
        &self.model
    }
    pub fn mark_saved(&mut self) {
        self.model.mark_saved();
    }
    /// Base tileset acceptance updates roof palettes without dirtying roof pixels.
    pub fn synchronize_attributes(&mut self, attributes: [u8; 9]) -> Result<(), TilesetError> {
        let modified = self.model.is_modified();
        let commands: Vec<_> = attributes
            .into_iter()
            .enumerate()
            .map(|(tile, palette)| TileCommand::Palette {
                tile: tile as u8,
                palette,
            })
            .collect();
        self.model.apply_batch(&commands)?;
        if !modified {
            self.model.mark_saved();
        }
        Ok(())
    }
    pub fn apply_batch(&mut self, commands: &[TileCommand]) -> Result<(), TilesetError> {
        let mut draft = self.clone();
        for command in commands {
            match command {
                TileCommand::Palette { .. } | TileCommand::Delete { .. } => {
                    return Err(TilesetError::RoofCommand);
                }
                TileCommand::ImportRgb {
                    tile,
                    width,
                    height,
                    pixels,
                    ..
                } => {
                    let mut value = draft.model.snapshot(*tile)?;
                    let (w, h) = (usize::from(*width), usize::from(*height));
                    if !(1..=2048).contains(&w)
                        || !(1..=2048).contains(&h)
                        || pixels.len() != w * h * 3
                    {
                        return Err(TilesetError::Image);
                    }
                    // Upstream's max(width,8)/max(height,8) loops are only safe
                    // for exactly 8×8. Clip larger images and retain uncovered
                    // pixels for smaller ones; never spill into adjacent tiles.
                    for y in 0..h.min(8) {
                        for x in 0..w.min(8) {
                            let p = (y * w + x) * 3;
                            let gray = (u16::from(pixels[p]) * 31
                                + u16::from(pixels[p + 1]) * 61
                                + u16::from(pixels[p + 2]) * 8)
                                / 100;
                            value.pixels[y * 8 + x] = [3, 1, 2, 0][usize::from(gray / 64)];
                        }
                    }
                    if value.palette == UNDEFINED_PALETTE {
                        value.palette = 0;
                    }
                    draft
                        .model
                        .apply(&TileCommand::Set { tile: *tile, value })?;
                }
                _ => draft.model.apply(command)?,
            }
        }
        *self = draft;
        Ok(())
    }
    pub fn export(&self) -> RgbImage {
        let mut pixels = vec![255; 24 * 24 * 3];
        for (i, tile) in self.model.tiles().iter().enumerate() {
            for (p, &hue) in tile.hues().iter().enumerate() {
                let at = ((i / 3 * 8 + p / 8) * 24 + i % 3 * 8 + p % 8) * 3;
                pixels[at..at + 3].fill([255, 85, 170, 0][usize::from(hue)]);
            }
        }
        RgbImage {
            width: 24,
            height: 24,
            pixels,
        }
    }
}
