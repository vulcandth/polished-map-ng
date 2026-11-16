//! Plusplus deep-tile.cpp raw pixels, definition state, edits and export.
//! LGPL-3.0-only; occurrence attributes remain in tile.rs.
use super::{AttributeBlockset, BankOrder, Error, TileReference};
use crate::{Palette, Point, RgbImage, Tile, TilesetError, UNDEFINED_COLORS, pixel_edit};
use serde::{Deserialize, Serialize};

const MONOCHROME: Palette = [[255; 3], [85; 3], [170; 3], [0; 3]];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Serializable copy of one tile and its definition flag; occurrence attributes are separate.
pub struct RawTileSnapshot {
    /// 64 row-major hue indices in 0–3; validated when applied.
    pub pixels: Vec<u8>,
    /// Whether the tile is defined for preview and export trimming.
    pub defined: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
/// Pixel and definition-state edits; never changes palettes/flips on block occurrences.
pub enum RawTileCommand {
    /// Restore one pixel/definition snapshot.
    Set {
        /// Loaded raw tile index in graphics-sheet order.
        tile: u16,
        /// Replacement pixels and definition flag.
        value: RawTileSnapshot,
    },
    /// Paint selected pixel coordinates in one tile.
    Paint {
        /// Loaded raw tile index in graphics-sheet order.
        tile: u16,
        /// Pixel coordinates within the 8×8 tile.
        points: Vec<Point>,
        /// Destination hue in 0–3.
        hue: u8,
    },
    /// Flood-fill a connected hue region within one tile.
    Fill {
        /// Loaded raw tile index in graphics-sheet order.
        tile: u16,
        /// Seed pixel coordinate within the 8×8 tile.
        at: Point,
        /// Destination hue in 0–3.
        hue: u8,
    },
    /// Replace every occurrence of one hue in a tile.
    Replace {
        /// Loaded raw tile index in graphics-sheet order.
        tile: u16,
        /// Hue to replace, in 0–3.
        from: u8,
        /// Replacement hue in 0–3.
        to: u8,
    },
    /// Exchange two hue indices within a tile.
    SwapHues {
        /// Loaded raw tile index in graphics-sheet order.
        tile: u16,
        /// First hue in 0–3.
        a: u8,
        /// Second hue in 0–3.
        b: u8,
    },
    /// Copy pixels and definition state between two loaded tiles.
    Copy {
        /// Source raw tile index.
        from: u16,
        /// Destination raw tile index.
        to: u16,
    },
    /// Exchange pixels and definition state between two loaded tiles.
    Swap {
        /// First raw tile index.
        a: u16,
        /// Second raw tile index.
        b: u16,
    },
    /// Clear all pixels and mark the tile undefined.
    Delete {
        /// Loaded raw tile index in graphics-sheet order.
        tile: u16,
    },
    /// Quantize RGB pixels into the sheet, clipping to its enabled extent.
    ImportRgb {
        /// Loaded raw tile index in graphics-sheet order.
        tile: u16,
        /// Input image width in pixels, in 1–2048.
        width: u16,
        /// Input image height in pixels, in 1–2048.
        height: u16,
        /// Packed row-major RGB bytes, exactly width × height × 3.
        pixels: Vec<u8>,
        #[serde(default)]
        /// Allow import into all 32 sheet rows instead of the first 16.
        allow_512: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Validated 1–512 raw tiles and matching definition flags, without palette ownership.
pub struct RawTileset {
    tiles: Vec<Tile>,
    defined: Vec<bool>,
    modified: bool,
}

impl RawTileset {
    /// Construct a clean raw sheet.
    ///
    /// # Errors
    /// Returns [`Error::GraphicsLength`] for an empty/oversized sheet or mismatched flags.
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
    /// Raw tiles in graphics-sheet order.
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }
    /// Definition flags corresponding one-to-one to [`Self::tiles`].
    pub fn defined(&self) -> &[bool] {
        &self.defined
    }
    /// Whether a successful edit has marked the raw sheet dirty.
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    /// Clear the dirty flag after persistence succeeds.
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
    /// Copy one tile for clipboard/draft operations.
    ///
    /// # Errors
    /// Returns [`Error::TileIndex`] when the tile is not loaded.
    pub fn snapshot(&self, tile: u16) -> Result<RawTileSnapshot, Error> {
        let i = self.index(tile)?;
        Ok(RawTileSnapshot {
            pixels: self.tiles[i].hues().to_vec(),
            defined: self.defined[i],
        })
    }
    /// Apply a command batch on a clone and publish only after every command validates.
    ///
    /// # Errors
    /// Invalid indices, hues or image payloads leave the original sheet unchanged.
    pub fn apply_batch(&mut self, commands: &[RawTileCommand]) -> Result<(), Error> {
        let mut draft = self.clone();
        for c in commands {
            draft.apply(c)?;
        }
        *self = draft;
        Ok(())
    }
    /// Validate and apply one pixel command, marking the sheet dirty even for a no-op.
    ///
    /// # Errors
    /// Invalid indices, hues or image payloads leave the original sheet unchanged.
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
