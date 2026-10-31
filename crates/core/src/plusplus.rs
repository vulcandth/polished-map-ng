//! Separate plusplus block attributes and bank addressing, adapted from
//! `tile.cpp`, `block-window.cpp` and `metatileset.cpp` at the plusplus revision
//! in upstream.lock.toml. Copyright Rangi42 and Polished Map NG contributors.
//! LGPL-3.0-only. Master types and their byte-sized tile semantics are unchanged.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{BlockBrush, BlockCommand, Blockset, BlocksetError, CollisionRow, Point};

/// Ordering of the upper halves of the two VRAM banks in the graphics sheet.
/// The first 256 entries are bank 0:00–7f followed by bank 1:00–7f in both modes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BankOrder {
    #[default]
    Bank1First,
    Bank0First,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error(transparent)]
    Blockset(#[from] BlocksetError),
    #[error("attribute data must contain complete 16-byte rows, at most one per block")]
    AttributeLength,
    #[error("tile {0} is outside the 512-entry graphics sheet")]
    TileIndex(u16),
    #[error("palette {0} is outside 0..=7")]
    Palette(u8),
}

/// A tile occurrence in a block, not a raw graphics tile. Bit 4 is unused and
/// canonicalized away by the original attribute reader/writer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TileReference {
    offset: u8,
    attribute: u8,
}

impl TileReference {
    pub fn from_bytes(offset: u8, attribute: u8) -> Self {
        Self {
            offset,
            attribute: attribute & 0xef,
        }
    }
    pub fn from_index(index: u16, order: BankOrder) -> Result<Self, Error> {
        if index >= 512 {
            return Err(Error::TileIndex(index));
        }
        Ok(Self::from_brush_index(i32::from(index), order))
    }
    // The upstream brush can step below 0 or above 511. Preserve Tile::bank_offset
    // for those signed intermediates, rather than inventing modulo-512 wrapping.
    fn from_brush_index(index: i32, order: BankOrder) -> Self {
        let offset = (index & 0x7f) as u8 | if index >= 0x100 { 0x80 } else { 0 };
        let bank1 = match order {
            BankOrder::Bank0First => index & 0xff >= 0x80,
            BankOrder::Bank1First => (0x80..0x180).contains(&index),
        };
        Self::from_bytes(offset, if bank1 { 8 } else { 0 })
    }
    pub fn offset(self) -> u8 {
        self.offset
    }
    pub fn attribute(self) -> u8 {
        self.attribute
    }
    pub fn palette(self) -> u8 {
        self.attribute & 7
    }
    pub fn bank1(self) -> bool {
        self.attribute & 8 != 0
    }
    pub fn x_flip(self) -> bool {
        self.attribute & 0x20 != 0
    }
    pub fn y_flip(self) -> bool {
        self.attribute & 0x40 != 0
    }
    pub fn priority(self) -> bool {
        self.attribute & 0x80 != 0
    }
    pub fn index(self, order: BankOrder) -> u16 {
        let offset = u16::from(self.offset);
        match order {
            BankOrder::Bank0First => {
                offset + if self.bank1() { 0x80 } else { 0 } + if offset >= 0x80 { 0x80 } else { 0 }
            }
            BankOrder::Bank1First => {
                offset
                    + if self.bank1() {
                        0x80
                    } else if offset >= 0x80 {
                        0x100
                    } else {
                        0
                    }
            }
        }
    }
}

/// Byte-sized map/block IDs with a separate 16-byte attribute row per block.
/// Offset and collision operations reuse the master-independent block storage;
/// callers cannot mutate that storage independently of the attribute rows.
#[derive(Clone, Debug)]
pub struct AttributeBlockset {
    offsets: Blockset,
    attributes: Vec<[u8; 16]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum AttributeCommand {
    Set {
        block: u8,
        offsets: [u8; 16],
        attributes: [u8; 16],
    },
    Paint {
        block: u8,
        at: Point,
        tile: u16,
        palette: u8,
        #[serde(default)]
        x_flip: bool,
        #[serde(default)]
        y_flip: bool,
        #[serde(default)]
        priority: bool,
        #[serde(default)]
        brush: BlockBrush,
        #[serde(default)]
        order: BankOrder,
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
    SetCollisions {
        block: u8,
        collision: CollisionRow,
    },
}

impl AttributeBlockset {
    /// Short whole-row attribute files are valid in the reference; missing rows
    /// retain zero attributes. Partial rows and excess rows are errors.
    pub fn from_bytes(offsets: &[u8], attributes: &[u8]) -> Result<Self, Error> {
        let offsets = Blockset::from_bytes(offsets)?;
        if !attributes.len().is_multiple_of(16) || attributes.len() > offsets.blocks().len() * 16 {
            return Err(Error::AttributeLength);
        }
        let mut rows = vec![[0; 16]; offsets.blocks().len()];
        for (row, source) in rows.iter_mut().zip(attributes.as_chunks::<16>().0) {
            *row = source.map(|a| a & 0xef);
        }
        Ok(Self {
            offsets,
            attributes: rows,
        })
    }
    pub fn offsets(&self) -> &Blockset {
        &self.offsets
    }
    pub fn attributes(&self) -> &[[u8; 16]] {
        &self.attributes
    }
    pub fn attribute_bytes(&self) -> Vec<u8> {
        self.attributes.iter().flatten().copied().collect()
    }
    pub fn is_modified(&self) -> bool {
        self.offsets.is_modified()
    }
    pub fn mark_saved(&mut self) {
        self.offsets.mark_saved();
    }
    pub fn with_collisions(mut self, rows: Vec<CollisionRow>) -> Result<Self, Error> {
        self.offsets = self.offsets.with_collisions(rows)?;
        Ok(self)
    }
    pub fn block(&self, id: u8) -> Result<[TileReference; 16], Error> {
        let offsets = self.offsets.block(id)?;
        Ok(std::array::from_fn(|i| {
            TileReference::from_bytes(offsets[i], self.attributes[usize::from(id)][i])
        }))
    }
    /// Match the original automatic 512-tile option threshold, including its
    /// strict >0x100 test. Graphics count may independently enable that option.
    pub fn enables_512_tiles(&self, order: BankOrder) -> bool {
        self.offsets
            .blocks()
            .iter()
            .zip(&self.attributes)
            .any(|(offsets, attrs)| {
                offsets
                    .iter()
                    .zip(attrs)
                    .any(|(&o, &a)| TileReference::from_bytes(o, a).index(order) > 0x100)
            })
    }
    pub fn apply_batch(&mut self, commands: &[AttributeCommand]) -> Result<(), Error> {
        let mut draft = self.clone();
        for command in commands {
            draft.apply(command)?;
        }
        *self = draft;
        Ok(())
    }
    /// Each command validates completely before touching the active state.
    pub fn apply(&mut self, command: &AttributeCommand) -> Result<(), Error> {
        match command {
            AttributeCommand::Set {
                block,
                offsets,
                attributes,
            } => {
                self.offsets.apply(&BlockCommand::Set {
                    block: *block,
                    tiles: *offsets,
                })?;
                self.attributes[usize::from(*block)] = attributes.map(|a| a & 0xef);
            }
            AttributeCommand::Copy { from, to } => {
                self.offsets.apply(&BlockCommand::Copy {
                    from: *from,
                    to: *to,
                })?;
                self.attributes[usize::from(*to)] = self.attributes[usize::from(*from)];
            }
            AttributeCommand::Swap { a, b } => {
                self.offsets.apply(&BlockCommand::Swap { a: *a, b: *b })?;
                self.attributes.swap(usize::from(*a), usize::from(*b));
            }
            AttributeCommand::Resize { count } => {
                self.offsets
                    .apply(&BlockCommand::Resize { count: *count })?;
                self.attributes.resize(usize::from(*count), [0; 16]);
            }
            AttributeCommand::SetCollisions { block, collision } => {
                self.offsets.apply(&BlockCommand::SetCollisions {
                    block: *block,
                    collision: collision.clone(),
                })?;
            }
            AttributeCommand::Paint {
                block,
                at,
                tile,
                palette,
                x_flip,
                y_flip,
                priority,
                brush,
                order,
            } => {
                let mut offsets = *self.offsets.block(*block)?;
                let mut attributes = self.attributes[usize::from(*block)];
                TileReference::from_index(*tile, *order)?;
                if *palette > 7 {
                    return Err(Error::Palette(*palette));
                }
                if at.x >= 4 || at.y >= 4 {
                    return Err(BlocksetError::Coordinate(at.x, at.y).into());
                }
                let n = brush.side();
                for (dx, dy, delta) in brush.placements() {
                    let x = at.x + if *x_flip { n - dx - 1 } else { dx };
                    let y = at.y + if *y_flip { n - dy - 1 } else { dy };
                    if x < 4 && y < 4 {
                        let reference = TileReference::from_brush_index(
                            i32::from(*tile) + i32::from(delta),
                            *order,
                        );
                        let slot = usize::from(y * 4 + x);
                        offsets[slot] = reference.offset();
                        attributes[slot] = reference.attribute()
                            | palette
                            | if *x_flip { 0x20 } else { 0 }
                            | if *y_flip { 0x40 } else { 0 }
                            | if *priority { 0x80 } else { 0 };
                    }
                }
                self.offsets.apply(&BlockCommand::Set {
                    block: *block,
                    tiles: offsets,
                })?;
                self.attributes[usize::from(*block)] = attributes;
            }
        }
        Ok(())
    }
}
