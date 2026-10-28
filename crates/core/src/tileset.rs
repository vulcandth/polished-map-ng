//! master tileset-window.cpp editing operations and metatileset.cpp trim rules.
//! Internal hue order is WHITE, DARK, LIGHT, BLACK. LGPL-3.0-only.
use crate::{Blockset, Palette, Point, RgbImage, Tile};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const UNDEFINED_PALETTE: u8 = 9;
pub const UNDEFINED_COLORS: Palette = [
    [0xab, 0xcd, 0xef],
    [0x45, 0x67, 0x89],
    [0x78, 0x9a, 0xbc],
    [0x12, 0x34, 0x56],
];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TilesetError {
    #[error("roof editing does not support palette assignment or deletion")]
    RoofCommand,
    #[error("expected 1..=256 tiles and one palette attribute per tile")]
    Length,
    #[error("tile {0} is outside the tileset")]
    Tile(u8),
    #[error("pixel ({0}, {1}) is outside the 8×8 tile")]
    Coordinate(u16, u16),
    #[error("hue {0} is outside 0..=3")]
    Hue(u8),
    #[error("invalid palette attribute {0}; expected 0..=9 or 128..=135")]
    Palette(u8),
    #[error("tile replacement needs exactly 64 pixels")]
    Pixels,
    #[error("image paste needs 1..=2048 pixels per axis and exactly width × height × 3 RGB bytes")]
    Image,
    #[error("tileset export is empty or its source layout would write outside the image")]
    ExportLayout,
}

/// A snapshot includes palette/priority, as Tile::copy does. Its source ID is
/// deliberately separate: the GUI's swap action uses the live source tile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileSnapshot {
    pub pixels: Vec<u8>,
    pub palette: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum TileCommand {
    Set {
        tile: u8,
        value: TileSnapshot,
    },
    Paint {
        tile: u8,
        points: Vec<Point>,
        hue: u8,
    },
    Fill {
        tile: u8,
        at: Point,
        hue: u8,
    },
    Replace {
        tile: u8,
        from: u8,
        to: u8,
    },
    SwapHues {
        tile: u8,
        a: u8,
        b: u8,
    },
    Palette {
        tile: u8,
        palette: u8,
    },
    Copy {
        from: u8,
        to: u8,
    },
    Swap {
        a: u8,
        b: u8,
    },
    Delete {
        tile: u8,
    },
    ImportRgb {
        tile: u8,
        width: u16,
        height: u16,
        pixels: Vec<u8>,
        #[serde(default)]
        allow_256: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tileset {
    tiles: Vec<Tile>,
    attributes: Vec<u8>,
    modified: bool,
}

fn palette(value: u8) -> Result<(), TilesetError> {
    if value <= 9 || (0x80..=0x87).contains(&value) {
        Ok(())
    } else {
        Err(TilesetError::Palette(value))
    }
}
fn hue(value: u8) -> Result<(), TilesetError> {
    if value <= 3 {
        Ok(())
    } else {
        Err(TilesetError::Hue(value))
    }
}
fn coordinate(at: Point) -> Result<usize, TilesetError> {
    if at.x < 8 && at.y < 8 {
        Ok(usize::from(at.y * 8 + at.x))
    } else {
        Err(TilesetError::Coordinate(at.x, at.y))
    }
}

impl Tileset {
    pub fn new(tiles: Vec<Tile>, attributes: Vec<u8>) -> Result<Self, TilesetError> {
        if tiles.is_empty() || tiles.len() > 256 || tiles.len() != attributes.len() {
            return Err(TilesetError::Length);
        }
        for &a in &attributes {
            palette(a)?;
        }
        Ok(Self {
            tiles,
            attributes,
            modified: false,
        })
    }
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }
    pub fn attributes(&self) -> &[u8] {
        &self.attributes
    }
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
    pub fn snapshot(&self, tile: u8) -> Result<TileSnapshot, TilesetError> {
        let index = self.index(tile)?;
        Ok(TileSnapshot {
            pixels: self.tiles[index].hues().to_vec(),
            palette: self.attributes[index],
        })
    }
    fn index(&self, tile: u8) -> Result<usize, TilesetError> {
        let index = usize::from(tile);
        if index < self.tiles.len() {
            Ok(index)
        } else {
            Err(TilesetError::Tile(tile))
        }
    }
    /// No undo stack in master. Whole batches validate on a draft before commit.
    pub fn apply_batch(&mut self, commands: &[TileCommand]) -> Result<(), TilesetError> {
        let mut draft = self.clone();
        for command in commands {
            draft.apply(command)?;
        }
        *self = draft;
        Ok(())
    }
    pub fn apply(&mut self, command: &TileCommand) -> Result<(), TilesetError> {
        match command {
            TileCommand::Set { tile, value } => {
                let index = self.index(*tile)?;
                palette(value.palette)?;
                let pixels: [u8; 64] = value
                    .pixels
                    .as_slice()
                    .try_into()
                    .map_err(|_| TilesetError::Pixels)?;
                for &p in &pixels {
                    hue(p)?;
                }
                self.tiles[index] = Tile::new(pixels).map_err(|_| TilesetError::Pixels)?;
                self.attributes[index] = value.palette;
            }
            TileCommand::Paint {
                tile,
                points,
                hue: value,
            } => {
                let index = self.index(*tile)?;
                hue(*value)?;
                let positions = points
                    .iter()
                    .map(|&p| coordinate(p))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut pixels = *self.tiles[index].hues();
                for p in positions {
                    pixels[p] = *value;
                }
                self.tiles[index] = Tile::new(pixels).map_err(|_| TilesetError::Pixels)?;
            }
            TileCommand::Fill {
                tile,
                at,
                hue: value,
            } => {
                let index = self.index(*tile)?;
                hue(*value)?;
                let start = coordinate(*at)?;
                let mut pixels = *self.tiles[index].hues();
                let old = pixels[start];
                if old != *value {
                    let mut queue = std::collections::VecDeque::from([start]);
                    while let Some(p) = queue.pop_front() {
                        if pixels[p] != old {
                            continue;
                        }
                        pixels[p] = *value;
                        if p % 8 > 0 {
                            queue.push_back(p - 1);
                        }
                        if p % 8 < 7 {
                            queue.push_back(p + 1);
                        }
                        if p >= 8 {
                            queue.push_back(p - 8);
                        }
                        if p < 56 {
                            queue.push_back(p + 8);
                        }
                    }
                }
                self.tiles[index] = Tile::new(pixels).map_err(|_| TilesetError::Pixels)?;
            }
            TileCommand::Replace { tile, from, to }
            | TileCommand::SwapHues {
                tile,
                a: from,
                b: to,
            } => {
                let index = self.index(*tile)?;
                hue(*from)?;
                hue(*to)?;
                let swap = matches!(command, TileCommand::SwapHues { .. });
                let pixels = self.tiles[index].hues().map(|p| {
                    if p == *from {
                        *to
                    } else if swap && p == *to {
                        *from
                    } else {
                        p
                    }
                });
                self.tiles[index] = Tile::new(pixels).map_err(|_| TilesetError::Pixels)?;
            }
            TileCommand::Palette {
                tile,
                palette: value,
            } => {
                let index = self.index(*tile)?;
                palette(*value)?;
                self.attributes[index] = *value;
            }
            TileCommand::Copy { from, to } => {
                let from = self.index(*from)?;
                let to = self.index(*to)?;
                self.tiles[to] = self.tiles[from].clone();
                self.attributes[to] = self.attributes[from];
            }
            TileCommand::Swap { a, b } => {
                let a = self.index(*a)?;
                let b = self.index(*b)?;
                self.tiles.swap(a, b);
                self.attributes.swap(a, b);
            }
            TileCommand::Delete { tile } => {
                let index = self.index(*tile)?;
                self.tiles[index] = Tile::new([0; 64]).map_err(|_| TilesetError::Pixels)?;
                self.attributes[index] = UNDEFINED_PALETTE;
            }
            TileCommand::ImportRgb {
                tile,
                width,
                height,
                pixels,
                allow_256,
            } => {
                self.import_rgb(*tile, *width, *height, pixels, *allow_256)?;
            }
        }
        self.modified = true;
        Ok(())
    }

    fn import_rgb(
        &mut self,
        tile: u8,
        width: u16,
        height: u16,
        rgb: &[u8],
        allow_256: bool,
    ) -> Result<(), TilesetError> {
        self.index(tile)?;
        let (w, h) = (usize::from(width), usize::from(height));
        if !(1..=2048).contains(&w) || !(1..=2048).contains(&h) || rgb.len() != w * h * 3 {
            return Err(TilesetError::Image);
        }
        let (ox, oy) = (usize::from(tile) % 16, usize::from(tile) / 16);
        let mut pixels: Vec<_> = self.tiles.iter().map(|t| *t.hues()).collect();
        let mut attributes = self.attributes.clone();
        for py in 0..h.min((16 - oy) * 8) {
            let mut ty = oy + py / 8;
            if !allow_256 {
                if oy < 6 && ty >= 6 {
                    ty += 2;
                }
                if oy < 14 && ty >= 14 {
                    ty += 2;
                }
            }
            if ty >= 16 {
                continue;
            }
            for px in 0..w.min((16 - ox) * 8) {
                let id = ty * 16 + ox + px / 8;
                if id >= pixels.len() {
                    return Err(TilesetError::Tile(id as u8));
                }
                let p = (py * w + px) * 3;
                let gray = (u16::from(rgb[p]) * 31
                    + u16::from(rgb[p + 1]) * 61
                    + u16::from(rgb[p + 2]) * 8)
                    / 100;
                pixels[id][(py % 8) * 8 + px % 8] = [3, 1, 2, 0][usize::from(gray / 64)];
                if px % 8 == 0 && py % 8 == 0 && attributes[id] == UNDEFINED_PALETTE {
                    attributes[id] = 0;
                }
            }
        }
        self.tiles = pixels
            .into_iter()
            .map(|p| Tile::new(p).map_err(|_| TilesetError::Pixels))
            .collect::<Result<_, _>>()?;
        self.attributes = attributes;
        Ok(())
    }

    /// The original save preparation clears trailing palette assignments until
    /// reaching a defined tile that is nonblank or used by a loaded metatile.
    /// Tile zero is always retained. Pixel data is never changed by trimming.
    pub fn trim_unused(&mut self, blocks: &Blockset) {
        let mut used = [false; 256];
        for &id in blocks.blocks().iter().flatten() {
            used[usize::from(id)] = true;
        }
        for i in (1..self.tiles.len()).rev() {
            if self.attributes[i] != UNDEFINED_PALETTE
                && (self.tiles[i].hues().iter().any(|&h| h != 0) || used[i])
            {
                break;
            }
            self.attributes[i] = UNDEFINED_PALETTE;
        }
    }

    pub fn render_tile(&self, tile: u8, colors: &[Palette; 9]) -> Result<RgbImage, TilesetError> {
        let index = self.index(tile)?;
        let attribute = self.attributes[index];
        let palette = if attribute == UNDEFINED_PALETTE {
            UNDEFINED_COLORS
        } else {
            colors[usize::from(attribute & 15)]
        };
        Ok(RgbImage {
            width: 8,
            height: 8,
            pixels: self.tiles[index]
                .hues()
                .iter()
                .flat_map(|&h| palette[usize::from(h)])
                .collect(),
        })
    }
}
