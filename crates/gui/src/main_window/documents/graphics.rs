//! Storage-specific tile interpretation behind the shared canvas/Print interface.
//! The plusplus renderer retains occurrence banks/palettes/flips/priority; it
//! is never flattened into master's palette-per-tile model. LGPL-3.0-only.
use polished_map_core::{
    Events, Map, PrintError, PrintFont, PrintOptions, PrintStyle, PrintTextMask, RasterError,
    RgbImage, attributed,
};

#[derive(Clone, Debug)]
pub(crate) enum MapGraphics {
    Classic(polished_map_core::Graphics),
    Attributed(Box<attributed::Graphics>),
}

impl From<polished_map_core::Graphics> for MapGraphics {
    fn from(graphics: polished_map_core::Graphics) -> Self {
        Self::Classic(graphics)
    }
}
impl From<attributed::Graphics> for MapGraphics {
    fn from(graphics: attributed::Graphics) -> Self {
        Self::Attributed(Box::new(graphics))
    }
}

impl MapGraphics {
    pub fn storage(&self) -> super::Storage {
        match self {
            Self::Classic(_) => super::Storage::Classic,
            Self::Attributed(_) => super::Storage::Attributed,
        }
    }
    pub fn block_count(&self) -> usize {
        match self {
            Self::Classic(g) => g.block_count(),
            Self::Attributed(g) => g.block_count(),
        }
    }
    pub fn block_priorities(&self, block: u8) -> Option<[bool; 16]> {
        match self {
            Self::Classic(g) => g.block_priorities(block),
            Self::Attributed(g) => g.block_priorities(block),
        }
    }
    pub fn render_block(&self, block: u8) -> Result<RgbImage, RasterError> {
        match self {
            Self::Classic(g) => g.render_block(block),
            Self::Attributed(g) => g.render_block(block),
        }
    }
    #[cfg(test)]
    pub fn render_map(&self, map: &Map) -> Result<RgbImage, RasterError> {
        match self {
            Self::Classic(g) => g.render_map(map),
            Self::Attributed(g) => g.render_map(map),
        }
    }
    pub fn render_print(
        &self,
        map: &Map,
        events: Option<&Events>,
        options: PrintOptions,
        style: PrintStyle,
        rasterize: impl FnMut(PrintFont, &str) -> Result<PrintTextMask, PrintError>,
    ) -> Result<RgbImage, PrintError> {
        match self {
            Self::Classic(g) => g.render_print(map, events, options, style, rasterize),
            Self::Attributed(g) => g.render_print(map, events, options, style, rasterize),
        }
    }
}
