//! Per-occurrence block attributes and bank addressing, adapted from
//! `tile.cpp`, `block-window.cpp` and `metatileset.cpp` at the plusplus revision
//! in upstream.lock.toml. Copyright Rangi42 and Polished Map NG contributors.
//! LGPL-3.0-only. Shared by the unified editor and fixed attributed launcher.
//!
//! [`AttributeBlockset`] pairs each block's 16 offsets with 16 attribute bytes.
//! [`TileReference`] interprets those bytes; [`RawTileset`] edits pixels without
//! changing occurrence attributes. [`Graphics`] combines both for CPU rendering.
//! Classic per-tile palettes remain in the crate-root [`crate::Tileset`] model.
//!
//! ```
//! use polished_map_core::attributed::{BankOrder, TileReference};
//! let tile = TileReference::from_index(0x100, BankOrder::Bank1First)?;
//! assert!(tile.bank1());
//! assert_eq!(tile.index(BankOrder::Bank1First), 0x100);
//! # Ok::<(), polished_map_core::attributed::Error>(())
//! ```

#![deny(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{BlockBrush, BlockCommand, Blockset, BlocksetError, CollisionRow, Point};

mod graphics;
// Preserve existing imports; source syntax is independent of attribute storage.
#[doc(hidden)]
pub use crate::event_catalog::{EVENT_KINDS, event_kind_from_macro};
pub use graphics::{Graphics, RawTileCommand, RawTileSnapshot, RawTileset};

/// Ordering of the upper halves of the two VRAM banks in the graphics sheet.
/// The first 256 entries are bank 0:00–7f followed by bank 1:00–7f in both modes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BankOrder {
    #[default]
    /// Bank 1 owns sheet ordinals 0x100–0x17f; this is the default ordering.
    Bank1First,
    /// Bank 0 owns sheet ordinals 0x100–0x17f.
    Bank0First,
}

#[derive(Debug, Error, PartialEq, Eq)]
/// Validation failures for occurrence attributes and raw graphics.
pub enum Error {
    #[error(transparent)]
    /// Pixel dimensions, hue values or image payload are invalid.
    Pixels(#[from] crate::TilesetError),
    #[error("expected 1..=512 raw tiles and one defined flag per tile")]
    /// Tile count is outside 1–512 or the definition flags have a different length.
    GraphicsLength,
    #[error(transparent)]
    /// A shared block/collision invariant was violated.
    Blockset(#[from] BlocksetError),
    #[error("attribute data must contain complete 16-byte rows, at most one per block")]
    /// Attribute bytes contain a partial row or more rows than the offsets.
    AttributeLength,
    #[error("tile {0} is outside the 512-entry graphics sheet")]
    /// The requested raw tile index is outside the supported or loaded sheet.
    TileIndex(u16),
    #[error("palette {0} is outside 0..=7")]
    /// An occurrence palette is outside 0–7.
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
    /// Decode an offset and attribute byte, clearing unused attribute bit 4.
    pub fn from_bytes(offset: u8, attribute: u8) -> Self {
        Self {
            offset,
            attribute: attribute & 0xef,
        }
    }
    /// Create a reference to a raw sheet ordinal, with palette zero and no flips/priority.
    ///
    /// # Errors
    /// Returns [`Error::TileIndex`] for ordinals at or above 512.
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
    /// Stored byte offset within the selected VRAM bank.
    pub fn offset(self) -> u8 {
        self.offset
    }
    /// Canonical stored attribute byte; unused bit 4 is always clear.
    pub fn attribute(self) -> u8 {
        self.attribute
    }
    /// Palette index encoded in bits 0–2.
    pub fn palette(self) -> u8 {
        self.attribute & 7
    }
    /// Whether bit 3 selects VRAM bank 1.
    pub fn bank1(self) -> bool {
        self.attribute & 8 != 0
    }
    /// Whether bit 5 mirrors the occurrence horizontally.
    pub fn x_flip(self) -> bool {
        self.attribute & 0x20 != 0
    }
    /// Whether bit 6 mirrors the occurrence vertically.
    pub fn y_flip(self) -> bool {
        self.attribute & 0x40 != 0
    }
    /// Whether bit 7 marks the occurrence as priority.
    pub fn priority(self) -> bool {
        self.attribute & 0x80 != 0
    }
    /// Resolve the stored offset/bank into a 0–511 raw-sheet ordinal.
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
/// Validated block operations that keep offsets, attributes and collisions aligned.
/// Apply with [`AttributeBlockset::apply`] or [`AttributeBlockset::apply_batch`].
pub enum AttributeCommand {
    /// Replace all 16 occurrences; unused attribute bit 4 is cleared.
    Set {
        /// Destination block ID.
        block: u8,
        /// Row-major offsets for the 4×4 block.
        offsets: [u8; 16],
        /// Row-major occurrence attribute bytes.
        attributes: [u8; 16],
    },
    /// Paint a tile brush with occurrence attributes at a block-cell coordinate.
    Paint {
        /// Destination block ID.
        block: u8,
        /// Starting cell within the 4×4 block.
        at: Point,
        /// Selected raw-sheet ordinal in 0–511.
        tile: u16,
        /// Occurrence palette index in 0–7.
        palette: u8,
        #[serde(default)]
        /// Mirror the brush footprint and tile pixels horizontally.
        x_flip: bool,
        #[serde(default)]
        /// Mirror the brush footprint and tile pixels vertically.
        y_flip: bool,
        #[serde(default)]
        /// Set the occurrence priority bit.
        priority: bool,
        #[serde(default)]
        /// Brush footprint; defaults to a single cell.
        brush: BlockBrush,
        #[serde(default)]
        /// Interpretation of raw-sheet ordinals; does not reorder stored bytes.
        order: BankOrder,
    },
    /// Copy offsets, attributes and optional collisions to another block.
    Copy {
        /// Source block ID.
        from: u8,
        /// Destination block ID.
        to: u8,
    },
    /// Exchange two complete block definitions and optional collisions.
    Swap {
        /// First block ID.
        a: u8,
        /// Second block ID.
        b: u8,
    },
    /// Resize all companion rows together; new block rows are empty.
    Resize {
        /// New block count in 1–256.
        count: u16,
    },
    /// Replace one existing collision row without changing tile occurrences.
    SetCollisions {
        /// Destination block ID.
        block: u8,
        /// Replacement collision fields matching the existing layout.
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
    /// Shared block offsets and collision rows; read-only to protect paired attributes.
    pub fn offsets(&self) -> &Blockset {
        &self.offsets
    }
    /// One canonical 16-byte attribute row for each block.
    pub fn attributes(&self) -> &[[u8; 16]] {
        &self.attributes
    }
    /// Serialize complete attribute rows in block order, with bit 4 cleared.
    pub fn attribute_bytes(&self) -> Vec<u8> {
        self.attributes.iter().flatten().copied().collect()
    }
    /// Whether an edit has marked this paired blockset dirty.
    pub fn is_modified(&self) -> bool {
        self.offsets.is_modified()
    }
    /// Clear the dirty flag after all companion files have been persisted.
    pub fn mark_saved(&mut self) {
        self.offsets.mark_saved();
    }
    /// Attach one collision row per block.
    ///
    /// # Errors
    /// Returns a blockset error if row count, field content or layout is invalid.
    pub fn with_collisions(mut self, rows: Vec<CollisionRow>) -> Result<Self, Error> {
        self.offsets = self.offsets.with_collisions(rows)?;
        Ok(self)
    }
    /// Decode a block into 16 row-major tile references.
    ///
    /// # Errors
    /// Returns a blockset index error if `id` is not loaded.
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
    /// Apply commands to a draft, committing only if every command succeeds.
    ///
    /// # Errors
    /// Any command validation error leaves the original blockset unchanged.
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
