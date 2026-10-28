//! Master metatileset.cpp and block-window.cpp editing rules. LGPL-3.0-only.
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Point;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BlocksetError {
    #[error("expected 1..=256 complete 16-byte blocks")]
    Length,
    #[error("block {0} is outside the blockset")]
    Block(u8),
    #[error("tile coordinate ({0}, {1}) is outside the 4×4 block")]
    Coordinate(u16, u16),
}

/// Block definitions have their own sticky dirty flag and no upstream undo stack.
#[derive(Clone, Debug)]
pub struct Blockset {
    blocks: Vec<[u8; 16]>,
    modified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum BlockCommand {
    Paint {
        block: u8,
        at: Point,
        tile: u8,
        #[serde(default)]
        brush: BlockBrush,
        #[serde(default)]
        allow_256: bool,
    },
    Set {
        block: u8,
        tiles: [u8; 16],
    },
    Copy {
        from: u8,
        to: u8,
    },
    Swap {
        a: u8,
        b: u8,
    },
    Resize {
        count: u16,
    },
}

impl Blockset {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BlocksetError> {
        if bytes.is_empty() || bytes.len() > 4096 || !bytes.len().is_multiple_of(16) {
            return Err(BlocksetError::Length);
        }
        Ok(Self {
            blocks: bytes.as_chunks::<16>().0.to_vec(),
            modified: false,
        })
    }
    pub fn blocks(&self) -> &[[u8; 16]] {
        &self.blocks
    }
    pub fn bytes(&self) -> Vec<u8> {
        self.blocks.iter().flatten().copied().collect()
    }
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
    pub fn block(&self, id: u8) -> Result<&[u8; 16], BlocksetError> {
        self.blocks
            .get(usize::from(id))
            .ok_or(BlocksetError::Block(id))
    }
    /// Validate before changing anything. Accepting even an unchanged draft marks dirty.
    pub fn apply(&mut self, command: &BlockCommand) -> Result<(), BlocksetError> {
        match command {
            BlockCommand::Paint {
                block,
                at,
                tile,
                brush,
                allow_256,
            } => {
                let mut draft = *self.block(*block)?;
                brush.paint(&mut draft, *at, *tile, *allow_256)?;
                self.blocks[usize::from(*block)] = draft;
            }
            BlockCommand::Set { block, tiles } => {
                self.block(*block)?;
                self.blocks[usize::from(*block)] = *tiles;
            }
            BlockCommand::Copy { from, to } => {
                let source = *self.block(*from)?;
                self.block(*to)?;
                self.blocks[usize::from(*to)] = source;
            }
            BlockCommand::Swap { a, b } => {
                self.block(*a)?;
                self.block(*b)?;
                self.blocks.swap(usize::from(*a), usize::from(*b));
            }
            BlockCommand::Resize { count } => {
                if !(1..=256).contains(count) {
                    return Err(BlocksetError::Length);
                }
                self.blocks.resize(usize::from(*count), [0; 16]);
            }
        }
        self.modified = true;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockBrush {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl BlockBrush {
    pub fn label(self) -> &'static str {
        [
            "",
            "Place 2/2",
            "Place 2x2",
            "Place 4x4",
            "Place 2+2",
            "Place 4+4+4+4",
            "Place 2-2",
            "Place 4-4-4-4",
        ][usize::from(self.shift) | (usize::from(self.ctrl) << 1) | (usize::from(self.alt) << 2)]
    }
    /// Paint a draft with upstream's byte wrapping, clipping and 0x60 tile skip.
    pub fn paint(
        self,
        draft: &mut [u8; 16],
        at: Point,
        tile: u8,
        allow_256: bool,
    ) -> Result<(), BlocksetError> {
        if at.x >= 4 || at.y >= 4 {
            return Err(BlocksetError::Coordinate(at.x, at.y));
        }
        let checkered = self.shift && !self.ctrl && !self.alt;
        let n: i16 = if self.ctrl || self.alt { 2 } else { 1 } * if self.shift { 2 } else { 1 };
        for dy in 0..n {
            for dx in 0..n {
                let x = at.x + dx as u16;
                let y = at.y + dy as u16;
                let offset = if checkered {
                    dy ^ dx
                } else {
                    (if self.ctrl {
                        if self.alt { -n } else { 16 }
                    } else {
                        n
                    }) * dy
                        + dx
                };
                let mut id = tile.wrapping_add(offset as u8);
                if !allow_256 && tile < 0x60 && id >= 0x60 {
                    id = id.wrapping_add(0x20);
                }
                if x < 4 && y < 4 && (allow_256 || id < 0xe0) {
                    draft[usize::from(y * 4 + x)] = id;
                }
            }
        }
        Ok(())
    }
}
