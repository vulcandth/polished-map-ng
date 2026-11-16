//! One-metatile draft editing: 4x4 clipping and source brush tile placement.
//! Adapted from metatile/block-window editing semantics; LGPL-3.0-only.
use crate::{BlocksetError, Point};
use serde::{Deserialize, Serialize};

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
