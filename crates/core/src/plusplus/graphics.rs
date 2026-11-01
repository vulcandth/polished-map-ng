//! Pinned plusplus deep-tile.cpp, tileset-window.cpp, metatileset.cpp and
//! image.cpp. LGPL-3.0-only. Pixel algorithms and map/Print composition are shared
//! with master; occurrence attributes and raw definition state remain separate.
use super::{AttributeBlockset, BankOrder, Error, TileReference};
use crate::{
    Map, Palette, Point, RasterError, RgbImage, Tile, TilesetError, UNDEFINED_COLORS, pixel_edit,
};
use serde::{Deserialize, Serialize};

const MONOCHROME: Palette = [[255; 3], [85; 3], [170; 3], [0; 3]];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTileSnapshot {
    pub pixels: Vec<u8>,
    pub defined: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum RawTileCommand {
    Set {
        tile: u16,
        value: RawTileSnapshot,
    },
    Paint {
        tile: u16,
        points: Vec<Point>,
        hue: u8,
    },
    Fill {
        tile: u16,
        at: Point,
        hue: u8,
    },
    Replace {
        tile: u16,
        from: u8,
        to: u8,
    },
    SwapHues {
        tile: u16,
        a: u8,
        b: u8,
    },
    Copy {
        from: u16,
        to: u16,
    },
    Swap {
        a: u16,
        b: u16,
    },
    Delete {
        tile: u16,
    },
    ImportRgb {
        tile: u16,
        width: u16,
        height: u16,
        pixels: Vec<u8>,
        #[serde(default)]
        allow_512: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawTileset {
    tiles: Vec<Tile>,
    defined: Vec<bool>,
    modified: bool,
}

impl RawTileset {
    pub fn new(tiles: Vec<Tile>, defined: Vec<bool>) -> Result<Self, Error> {
        if tiles.is_empty() || tiles.len() > 512 || tiles.len() != defined.len() {
            return Err(Error::GraphicsLength);
        }
        Ok(Self {
            tiles,
            defined,
            modified: false,
        })
    }
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }
    pub fn defined(&self) -> &[bool] {
        &self.defined
    }
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
    fn index(&self, id: u16) -> Result<usize, Error> {
        if usize::from(id) < self.tiles.len() {
            Ok(usize::from(id))
        } else {
            Err(Error::TileIndex(id))
        }
    }
    pub fn snapshot(&self, tile: u16) -> Result<RawTileSnapshot, Error> {
        let i = self.index(tile)?;
        Ok(RawTileSnapshot {
            pixels: self.tiles[i].hues().to_vec(),
            defined: self.defined[i],
        })
    }
    pub fn apply_batch(&mut self, commands: &[RawTileCommand]) -> Result<(), Error> {
        let mut draft = self.clone();
        for c in commands {
            draft.apply(c)?;
        }
        *self = draft;
        Ok(())
    }
    pub fn apply(&mut self, command: &RawTileCommand) -> Result<(), Error> {
        match command {
            RawTileCommand::Set { tile, value } => {
                let i = self.index(*tile)?;
                self.tiles[i] = pixel_edit::from_pixels(&value.pixels)?;
                self.defined[i] = value.defined;
            }
            RawTileCommand::Paint { tile, points, hue } => {
                let i = self.index(*tile)?;
                self.tiles[i] = pixel_edit::paint(&self.tiles[i], points, *hue)?;
                self.defined[i] = true;
            }
            RawTileCommand::Fill { tile, at, hue } => {
                let i = self.index(*tile)?;
                self.tiles[i] = pixel_edit::fill(&self.tiles[i], *at, *hue)?;
                self.defined[i] = true;
            }
            RawTileCommand::Replace { tile, from, to }
            | RawTileCommand::SwapHues {
                tile,
                a: from,
                b: to,
            } => {
                let i = self.index(*tile)?;
                self.tiles[i] = pixel_edit::replace(
                    &self.tiles[i],
                    *from,
                    *to,
                    matches!(command, RawTileCommand::SwapHues { .. }),
                )?;
                self.defined[i] = true;
            }
            RawTileCommand::Copy { from, to } => {
                let from = self.index(*from)?;
                let to = self.index(*to)?;
                self.tiles[to] = self.tiles[from].clone();
                self.defined[to] = self.defined[from];
            }
            RawTileCommand::Swap { a, b } => {
                let a = self.index(*a)?;
                let b = self.index(*b)?;
                self.tiles.swap(a, b);
                self.defined.swap(a, b);
            }
            RawTileCommand::Delete { tile } => {
                let i = self.index(*tile)?;
                self.tiles[i] = pixel_edit::from_pixels(&[0; 64])?;
                self.defined[i] = false;
            }
            RawTileCommand::ImportRgb {
                tile,
                width,
                height,
                pixels,
                allow_512,
            } => self.import_rgb(*tile, *width, *height, pixels, *allow_512)?,
        }
        self.modified = true;
        Ok(())
    }
    fn import_rgb(
        &mut self,
        tile: u16,
        width: u16,
        height: u16,
        rgb: &[u8],
        allow_512: bool,
    ) -> Result<(), Error> {
        let i = self.index(tile)?;
        let (w, h) = (usize::from(width), usize::from(height));
        if !(1..=2048).contains(&w) || !(1..=2048).contains(&h) || rgb.len() != w * h * 3 {
            return Err(TilesetError::Image.into());
        }
        let (ox, oy) = (i % 16, i / 16);
        let mut pixels: Vec<_> = self.tiles.iter().map(|t| *t.hues()).collect();
        let mut defined = self.defined.clone();
        for py in 0..h.min((32 - oy) * 8) {
            let ty = oy + py / 8;
            if ty >= if allow_512 { 32 } else { 16 } {
                continue;
            }
            for px in 0..w.min((16 - ox) * 8) {
                let index = ty * 16 + ox + px / 8;
                if index >= pixels.len() {
                    return Err(Error::TileIndex(index as u16));
                }
                let p = (py * w + px) * 3;
                pixels[index][(py % 8) * 8 + px % 8] = pixel_edit::rgb_hue(&rgb[p..p + 3]);
                if px % 8 == 0 && py % 8 == 0 {
                    defined[index] = true;
                }
            }
        }
        self.tiles = pixels
            .iter()
            .map(|p| pixel_edit::from_pixels(p))
            .collect::<Result<_, _>>()?;
        self.defined = defined;
        Ok(())
    }
    /// Unlike master, clear every unused blank tile, including tile zero and
    /// interior holes. Keep its pixels and the sticky dirty state unchanged.
    pub fn trim_unused(&mut self, blocks: &AttributeBlockset, order: BankOrder) {
        let mut used = [false; 512];
        for (offsets, attrs) in blocks.offsets().blocks().iter().zip(blocks.attributes()) {
            for (&o, &a) in offsets.iter().zip(attrs) {
                used[usize::from(TileReference::from_bytes(o, a).index(order))] = true;
            }
        }
        for (i, tile) in self.tiles.iter().enumerate() {
            if !used[i] && tile.hues().iter().all(|&h| h == 0) {
                self.defined[i] = false;
            }
        }
    }
    /// Monochrome raw-tile preview; undefined tiles use the source diagnostic palette.
    pub fn render_tile(&self, id: u16) -> Result<RgbImage, Error> {
        let i = self.index(id)?;
        let palette = if self.defined[i] {
            MONOCHROME
        } else {
            UNDEFINED_COLORS
        };
        Ok(RgbImage {
            width: 8,
            height: 8,
            pixels: self.tiles[i]
                .hues()
                .iter()
                .flat_map(|&h| palette[usize::from(h)])
                .collect(),
        })
    }
    /// Export linear sheet order with white row padding. Bound the source tail
    /// before scanning, avoiding upstream image.cpp's nonzero-offset overread.
    pub fn export_region(&self, offset: usize, count: Option<usize>) -> Result<RgbImage, Error> {
        let tail = self
            .defined
            .get(offset..)
            .ok_or(TilesetError::ExportLayout)?;
        let n = count.unwrap_or_else(|| tail.iter().rposition(|&d| d).map_or(0, |i| i + 1));
        if n == 0 || n > tail.len() {
            return Err(TilesetError::ExportLayout.into());
        }
        let height = n.div_ceil(16) * 8;
        let mut pixels = vec![255; 128 * height * 3];
        for (i, tile) in self.tiles[offset..offset + n].iter().enumerate() {
            for (p, &hue) in tile.hues().iter().enumerate() {
                let at = ((i / 16 * 8 + p / 8) * 128 + i % 16 * 8 + p % 8) * 3;
                pixels[at..at + 3].copy_from_slice(&MONOCHROME[usize::from(hue)]);
            }
        }
        Ok(RgbImage {
            width: 128,
            height: height as u32,
            pixels,
        })
    }
}

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
    pub fn with_roof(mut self, roof: [Tile; 9]) -> Self {
        self.roof = Some(roof);
        self
    }
    pub fn block_count(&self) -> usize {
        self.blocks.attributes().len()
    }
    pub fn block_priorities(&self, id: u8) -> Option<[bool; 16]> {
        self.blocks
            .block(id)
            .ok()
            .map(|tiles| tiles.map(|t| t.priority()))
    }
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
        let tile = roof.or_else(|| self.raw.tiles.get(index));
        let defined = roof.is_some() || self.raw.defined.get(index).copied().unwrap_or(false);
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
    pub fn render_map(&self, map: &Map) -> Result<RgbImage, RasterError> {
        crate::raster::render_map(self, map)
    }
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
