//! Attributed block/map/Print composition over raw deep tiles. LGPL-3.0-only.
use super::{AttributeBlockset, BankOrder, RawTileset, TileReference};
use crate::{Map, Palette, RasterError, RgbImage, Tile, UNDEFINED_COLORS};

/// CPU canvas/Print renderer. Per-occurrence palettes and flips are resolved
/// here; the shared map/Print compositor never assumes master tile attributes.
#[derive(Clone, Debug)]
pub struct Graphics {
    raw: RawTileset,
    blocks: AttributeBlockset,
    palettes: [Palette; 8],
    order: BankOrder,
    roof: Option<[Tile; 9]>,
}
impl Graphics {
    /// Combine validated raw tiles, block references and eight RGB palettes for rendering.
    pub fn new(
        raw: RawTileset,
        blocks: AttributeBlockset,
        palettes: [Palette; 8],
        order: BankOrder,
    ) -> Self {
        Self {
            raw,
            blocks,
            palettes,
            order,
            roof: None,
        }
    }
    /// Overlay nine roof tiles at raw indices 0x0a–0x12 during rendering.
    pub fn with_roof(mut self, roof: [Tile; 9]) -> Self {
        self.roof = Some(roof);
        self
    }
    /// Number of block definitions available to the renderer.
    pub fn block_count(&self) -> usize {
        self.blocks.attributes().len()
    }
    /// Return 16 occurrence priority flags, or `None` when the block is absent.
    pub fn block_priorities(&self, id: u8) -> Option<[bool; 16]> {
        self.blocks
            .block(id)
            .ok()
            .map(|tiles| tiles.map(|t| t.priority()))
    }
    /// Render a 32×32 block. Absent IDs use the shared empty-block image.
    /// The fallible signature matches the shared rendering interface; this adapter
    /// currently has no error-producing branch.
    pub fn render_block(&self, id: u8) -> Result<RgbImage, RasterError> {
        let Ok(tiles) = self.blocks.block(id) else {
            return Ok(crate::raster::empty_block());
        };
        let mut pixels = vec![0; 32 * 32 * 3];
        for (slot, t) in tiles.into_iter().enumerate() {
            let tile = self.tile_pixels(t);
            for y in 0..8 {
                let at = ((slot / 4 * 8 + y) * 32 + slot % 4 * 8) * 3;
                pixels[at..at + 24].copy_from_slice(&tile[y * 24..y * 24 + 24]);
            }
        }
        Ok(RgbImage {
            width: 32,
            height: 32,
            pixels,
        })
    }
    /// Render one occurrence for the block editor using the same roof, palette,
    /// undefined-color and flip rules as the map compositor.
    pub fn render_tile(&self, tile: TileReference) -> RgbImage {
        RgbImage {
            width: 8,
            height: 8,
            pixels: self.tile_pixels(tile).to_vec(),
        }
    }
    fn tile_pixels(&self, t: TileReference) -> [u8; 192] {
        let mut pixels = [0; 192];
        let index = usize::from(t.index(self.order));
        let roof = self
            .roof
            .as_ref()
            .and_then(|r| index.checked_sub(0x0a).and_then(|i| r.get(i)));
        let tile = roof.or_else(|| self.raw.tiles().get(index));
        let defined = roof.is_some() || self.raw.defined().get(index).copied().unwrap_or(false);
        let palette = if defined {
            self.palettes[usize::from(t.palette())]
        } else {
            UNDEFINED_COLORS
        };
        for y in 0..8 {
            for x in 0..8 {
                let sx = if t.x_flip() { 7 - x } else { x };
                let sy = if t.y_flip() { 7 - y } else { y };
                let hue = tile.map_or(0, |t| t.hues()[sy * 8 + sx]);
                let at = (y * 8 + x) * 3;
                pixels[at..at + 3].copy_from_slice(&palette[usize::from(hue)]);
            }
        }
        pixels
    }
    /// Compose the complete map from block images using the shared CPU renderer.
    ///
    /// # Errors
    /// Propagates composition errors from the shared renderer.
    pub fn render_map(&self, map: &Map) -> Result<RgbImage, RasterError> {
        crate::raster::render_map(self, map)
    }
    /// Compose map pixels and optional overlays using caller-provided text masks.
    ///
    /// # Errors
    /// Propagates rasterization failures, invalid text masks and compositor bounds errors.
    pub fn render_print<F>(
        &self,
        map: &Map,
        events: Option<&crate::Events>,
        options: crate::PrintOptions,
        style: crate::PrintStyle,
        rasterize: F,
    ) -> Result<RgbImage, crate::PrintError>
    where
        F: FnMut(crate::PrintFont, &str) -> Result<crate::PrintTextMask, crate::PrintError>,
    {
        crate::printing::render_print(self, map, events, options, style, rasterize)
    }
}
impl crate::raster::BlockRaster for Graphics {
    fn render_block(&self, id: u8) -> Result<RgbImage, RasterError> {
        self.render_block(id)
    }
    fn block_priorities(&self, id: u8) -> Option<[bool; 16]> {
        self.block_priorities(id)
    }
}
