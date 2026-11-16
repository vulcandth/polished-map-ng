//! Raw classic tile hues and validation, from pinned tile.h/tile.cpp. LGPL-3.0-only.
use crate::RasterError;

/// Internal hue order is WHITE, DARK, LIGHT, BLACK, as in upstream colors.h.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tile([u8; 64]);

impl Tile {
    /// Validate one tile's 64 row-major hue indices.
    ///
    /// # Errors
    /// Returns [`RasterError::Hue`] if any hue is outside 0–3.
    pub fn new(hues: [u8; 64]) -> Result<Self, RasterError> {
        if let Some(&hue) = hues.iter().find(|&&h| h > 3) {
            return Err(RasterError::Hue(hue));
        }
        Ok(Self(hues))
    }
    /// 64 row-major hue indices in the internal WHITE, DARK, LIGHT, BLACK order.
    pub fn hues(&self) -> &[u8; 64] {
        &self.0
    }
}
