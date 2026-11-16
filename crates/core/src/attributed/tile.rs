//! Plusplus tile.cpp bank/offset/attribute interpretation. LGPL-3.0-only.
use super::Error;
use serde::{Deserialize, Serialize};

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
    pub(super) fn from_brush_index(index: i32, order: BankOrder) -> Self {
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
