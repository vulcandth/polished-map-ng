use thiserror::Error;

use crate::Map;

/// Internal hue order is WHITE, DARK, LIGHT, BLACK, as in upstream colors.h.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tile([u8; 64]);

impl Tile {
    pub fn new(hues: [u8; 64]) -> Result<Self, RasterError> {
        if let Some(&hue) = hues.iter().find(|&&h| h > 3) {
            return Err(RasterError::Hue(hue));
        }
        Ok(Self(hues))
    }
    pub fn hues(&self) -> &[u8; 64] {
        &self.0
    }
}

pub type Palette = [[u8; 3]; 4];

/// RGB data in row-major order, independent of an image library or GPU.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RasterError {
    #[error("invalid hue {0}; expected 0..=3")]
    Hue(u8),
    #[error("expected 1..=256 tiles and one palette per tile")]
    TileCount,
    #[error("expected 1..=256 metatiles")]
    MetatileCount,
    #[error("metatile references missing tile {0}")]
    MissingTile(u8),
}

#[derive(Clone, Debug)]
pub struct Graphics {
    tiles: Vec<Tile>,
    palettes: Vec<Palette>,
    metatiles: Vec<[u8; 16]>,
}

impl Graphics {
    pub fn new(
        tiles: Vec<Tile>,
        palettes: Vec<Palette>,
        metatiles: Vec<[u8; 16]>,
    ) -> Result<Self, RasterError> {
        if tiles.is_empty() || tiles.len() > 256 || tiles.len() != palettes.len() {
            return Err(RasterError::TileCount);
        }
        if metatiles.is_empty() || metatiles.len() > 256 {
            return Err(RasterError::MetatileCount);
        }
        for &id in metatiles.iter().flatten() {
            if usize::from(id) >= tiles.len() {
                return Err(RasterError::MissingTile(id));
            }
        }
        Ok(Self {
            tiles,
            palettes,
            metatiles,
        })
    }

    pub fn block_count(&self) -> usize {
        self.metatiles.len()
    }

    /// Render a single metatile, used by both canvas and sidebar texture caches.
    pub fn render_block(&self, block: u8) -> Result<RgbImage, RasterError> {
        let Some(metatile) = self.metatiles.get(usize::from(block)) else {
            // draw_metatile uses EMPTY_RGB for IDs beyond the loaded blockset.
            return Ok(RgbImage {
                width: 32,
                height: 32,
                pixels: [0xab, 0xcd, 0xef].repeat(32 * 32),
            });
        };
        let mut pixels = vec![0; 32 * 32 * 3];
        for (slot, &tile_id) in metatile.iter().enumerate() {
            let tile = &self.tiles[usize::from(tile_id)];
            let palette = &self.palettes[usize::from(tile_id)];
            for (pixel, &hue) in tile.0.iter().enumerate() {
                let x = (slot % 4) * 8 + pixel % 8;
                let y = (slot / 4) * 8 + pixel / 8;
                let offset = (y * 32 + x) * 3;
                pixels[offset..offset + 3].copy_from_slice(&palette[usize::from(hue)]);
            }
        }
        Ok(RgbImage {
            width: 32,
            height: 32,
            pixels,
        })
    }

    /// Render at native resolution; allocation is bounded by the validated map dimensions.
    pub fn render_map(&self, map: &Map) -> Result<RgbImage, RasterError> {
        let blocks: Vec<_> = (0..256)
            .map(|i| self.render_block(i as u8))
            .collect::<Result<_, _>>()?;
        let width = usize::from(map.width()) * 32;
        let height = usize::from(map.height()) * 32;
        let mut pixels = vec![0; width * height * 3];
        for (i, &id) in map.blocks().iter().enumerate() {
            let bx = i % usize::from(map.width());
            let by = i / usize::from(map.width());
            for y in 0..32 {
                let start = ((by * 32 + y) * width + bx * 32) * 3;
                pixels[start..start + 96]
                    .copy_from_slice(&blocks[usize::from(id)].pixels[y * 96..y * 96 + 96]);
            }
        }
        Ok(RgbImage {
            width: width as u32,
            height: height as u32,
            pixels,
        })
    }
}

/// Upstream DAY colors, including its exact five-bit expansion (channel * 33 / 4).
pub fn day_palettes() -> [Palette; 9] {
    let colors: [[[u16; 3]; 4]; 9] = [
        [[27, 31, 27], [13, 13, 13], [21, 21, 21], [7, 7, 7]],
        [[27, 31, 27], [30, 10, 6], [31, 19, 24], [7, 7, 7]],
        [[22, 31, 10], [5, 14, 0], [12, 25, 1], [7, 7, 7]],
        [[27, 31, 27], [13, 12, 31], [18, 19, 31], [7, 7, 7]],
        [[27, 31, 27], [31, 16, 1], [31, 31, 7], [7, 7, 7]],
        [[27, 31, 27], [20, 15, 3], [24, 18, 7], [7, 7, 7]],
        [[27, 31, 27], [5, 17, 31], [15, 31, 31], [7, 7, 7]],
        [[31, 0, 31], [16, 0, 16], [31, 0, 31], [0, 0, 0]],
        [[28, 31, 26], [6, 13, 10], [17, 24, 14], [1, 3, 4]],
    ];
    colors.map(|palette| palette.map(|rgb| rgb.map(|c| (c * 33 / 4) as u8)))
}
