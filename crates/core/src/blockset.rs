//! Master metatileset.cpp and block-window.cpp editing rules. LGPL-3.0-only.
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Point;

#[derive(Debug, Error, PartialEq, Eq)]
/// Invalid block geometry, IDs or collision rows.
pub enum BlocksetError {
    #[error("expected 1..=256 complete 16-byte blocks")]
    /// Offsets must contain 1–256 complete 16-byte blocks.
    Length,
    #[error("block {0} is outside the blockset")]
    /// The requested block ID is not loaded.
    Block(u8),
    #[error("tile coordinate ({0}, {1}) is outside the 4×4 block")]
    /// The tile-cell coordinate lies outside the block's 4×4 grid.
    Coordinate(u16, u16),
    #[error("collision rows must match the block count and use one format")]
    /// Collision rows differ in format or do not correspond one-to-one with blocks.
    CollisionLayout,
    #[error("collision field contains a delimiter, NUL or line break")]
    /// An assembly field contains a delimiter, line break or NUL.
    CollisionText,
    #[error("collision fields are limited to 4096 bytes each")]
    /// An assembly collision field exceeds 4096 UTF-8 bytes.
    CollisionTextLength,
}

/// Quadrant order: top left, top right, bottom left, bottom right.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "format",
    content = "values",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CollisionRow {
    /// Uninterpreted assembly collision expressions; delimiters are validated on attachment/edit.
    Asm([String; 4]),
    /// Four binary collision codes in quadrant order.
    Binary([u8; 4]),
}

impl CollisionRow {
    /// Whether the row uses binary codes rather than assembly expressions.
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }
    fn empty(&self) -> Self {
        if self.is_binary() {
            Self::Binary([0; 4])
        } else {
            Self::Asm(std::array::from_fn(|_| String::new()))
        }
    }
    fn validate(&self) -> Result<(), BlocksetError> {
        if let Self::Asm(values) = self
            && values.iter().any(|v| v.len() > 4096)
        {
            return Err(BlocksetError::CollisionTextLength);
        }
        if let Self::Asm(values) = self
            && values
                .iter()
                .any(|v| v.contains([',', ';', '\r', '\n', '\0']))
        {
            return Err(BlocksetError::CollisionText);
        }
        Ok(())
    }
}

/// Block definitions have their own sticky dirty flag and no upstream undo stack.
#[derive(Clone, Debug)]
pub struct Blockset {
    blocks: Vec<[u8; 16]>,
    modified: bool,
    collisions: Option<Vec<CollisionRow>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
/// Block edits; apply atomically with [`Blockset::apply_batch`] when grouping commands.
pub enum BlockCommand {
    /// Replace one existing collision row.
    SetCollisions {
        /// Destination block ID.
        block: u8,
        /// Replacement row in the already attached collision format.
        collision: CollisionRow,
    },
    /// Paint the selected brush into one block.
    Paint {
        /// Destination block ID.
        block: u8,
        /// Starting tile-cell coordinate within the 4×4 block.
        at: Point,
        /// Selected raw tile ID.
        tile: u8,
        #[serde(default)]
        /// Modifier-controlled footprint and source stride.
        brush: BlockBrush,
        #[serde(default)]
        /// Permit reserved tile slots instead of applying classic 192-tile skips.
        allow_256: bool,
    },
    /// Replace all 16 raw tile IDs in one block.
    Set {
        /// Destination block ID.
        block: u8,
        /// Replacement row-major 4×4 tile IDs.
        tiles: [u8; 16],
    },
    /// Copy a complete block and its optional collision row.
    Copy {
        /// Source block ID.
        from: u8,
        /// Destination block ID.
        to: u8,
    },
    /// Exchange two blocks and their optional collision rows.
    Swap {
        /// First block ID.
        a: u8,
        /// Second block ID.
        b: u8,
    },
    /// Grow with empty blocks/collisions or truncate to the requested count.
    Resize {
        /// New block count in 1–256.
        count: u16,
    },
}

impl Blockset {
    /// Load complete row-major 4×4 block definitions as clean state.
    ///
    /// # Errors
    /// Rejects empty input, partial 16-byte rows, or more than 256 blocks.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BlocksetError> {
        if bytes.is_empty() || bytes.len() > 4096 || !bytes.len().is_multiple_of(16) {
            return Err(BlocksetError::Length);
        }
        Ok(Self {
            blocks: bytes.as_chunks::<16>().0.to_vec(),
            modified: false,
            collisions: None,
        })
    }
    /// All 16-byte block definitions in block-ID order.
    pub fn blocks(&self) -> &[[u8; 16]] {
        &self.blocks
    }
    /// Attach collision rows to the current definitions.
    ///
    /// # Errors
    /// Requires one row per block, a single row format, and valid assembly fields.
    pub fn with_collisions(mut self, rows: Vec<CollisionRow>) -> Result<Self, BlocksetError> {
        if rows.len() != self.blocks.len()
            || rows.iter().any(|r| r.is_binary() != rows[0].is_binary())
        {
            return Err(BlocksetError::CollisionLayout);
        }
        for row in &rows {
            row.validate()?;
        }
        self.collisions = Some(rows);
        Ok(self)
    }
    /// Attached collision rows, or `None` if no collision source was loaded.
    pub fn collisions(&self) -> Option<&[CollisionRow]> {
        self.collisions.as_deref()
    }
    /// Validate the complete draft before changing the active blockset.
    ///
    /// # Errors
    /// Any invalid command leaves the original blocks, collisions and dirty state unchanged.
    pub fn apply_batch(&mut self, commands: &[BlockCommand]) -> Result<(), BlocksetError> {
        let mut draft = self.clone();
        for command in commands {
            draft.apply(command)?;
        }
        *self = draft;
        Ok(())
    }
    /// Serialize block offsets in row-major block order; collision bytes are separate.
    pub fn bytes(&self) -> Vec<u8> {
        self.blocks.iter().flatten().copied().collect()
    }
    /// Whether a successful edit has marked the blockset dirty.
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    /// Clear the dirty flag only after offsets and attached collisions have been persisted.
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
    /// Borrow one block's 16 tile IDs.
    ///
    /// # Errors
    /// Returns [`BlocksetError::Block`] for an ID beyond the loaded blockset.
    pub fn block(&self, id: u8) -> Result<&[u8; 16], BlocksetError> {
        self.blocks
            .get(usize::from(id))
            .ok_or(BlocksetError::Block(id))
    }
    /// Validate before changing anything. Accepting even an unchanged draft marks dirty.
    ///
    /// # Errors
    /// Rejects invalid block/cell indices, resize counts or collision fields before mutation.
    pub fn apply(&mut self, command: &BlockCommand) -> Result<(), BlocksetError> {
        match command {
            BlockCommand::SetCollisions { block, collision } => {
                self.block(*block)?;
                collision.validate()?;
                let rows = self
                    .collisions
                    .as_mut()
                    .ok_or(BlocksetError::CollisionLayout)?;
                if rows[0].is_binary() != collision.is_binary() {
                    return Err(BlocksetError::CollisionLayout);
                }
                rows[usize::from(*block)] = collision.clone();
            }
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
                if let Some(rows) = &mut self.collisions {
                    rows[usize::from(*to)] = rows[usize::from(*from)].clone();
                }
            }
            BlockCommand::Swap { a, b } => {
                self.block(*a)?;
                self.block(*b)?;
                self.blocks.swap(usize::from(*a), usize::from(*b));
                if let Some(rows) = &mut self.collisions {
                    rows.swap(usize::from(*a), usize::from(*b));
                }
            }
            BlockCommand::Resize { count } => {
                if !(1..=256).contains(count) {
                    return Err(BlocksetError::Length);
                }
                self.blocks.resize(usize::from(*count), [0; 16]);
                if let Some(rows) = &mut self.collisions {
                    let empty = rows[0].empty();
                    rows.resize(usize::from(*count), empty);
                }
            }
        }
        self.modified = true;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
/// The eight source brush patterns selected by Shift/Ctrl/Alt combinations.
/// `Default` paints one cell; modifiers change footprint and source-tile strides.
pub struct BlockBrush {
    /// Double the footprint; Shift alone paints the alternating 2×2 pattern.
    pub shift: bool,
    /// Use the sheet-row stride (or reversed stride when combined with Alt).
    pub ctrl: bool,
    /// Select the alternate stride family.
    pub alt: bool,
}

impl BlockBrush {
    /// Source UI hint for this modifier combination; the one-cell brush has an empty hint.
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
    pub(crate) fn side(self) -> u16 {
        (if self.ctrl || self.alt { 2 } else { 1 }) * if self.shift { 2 } else { 1 }
    }
    /// Shared brush footprint/stride; each format owns address conversion.
    pub(crate) fn placements(self) -> impl Iterator<Item = (u16, u16, i16)> {
        let checkered = self.shift && !self.ctrl && !self.alt;
        let n = self.side() as i16;
        (0..n).flat_map(move |dy| {
            (0..n).map(move |dx| {
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
                (dx as u16, dy as u16, offset)
            })
        })
    }
    /// Paint a draft with upstream's byte wrapping, clipping and 0x60 tile skip.
    ///
    /// # Errors
    /// Returns [`BlocksetError::Coordinate`] when the starting cell is outside 4×4.
    /// The brush itself clips at block edges; `allow_256` controls reserved-slot handling.
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
        for (dx, dy, offset) in self.placements() {
            let x = at.x + dx;
            let y = at.y + dy;
            let mut id = tile.wrapping_add(offset as u8);
            if !allow_256 && tile < 0x60 && id >= 0x60 {
                id = id.wrapping_add(0x20);
            }
            if x < 4 && y < 4 && (allow_256 || id < 0xe0) {
                draft[usize::from(y * 4 + x)] = id;
            }
        }
        Ok(())
    }
}
