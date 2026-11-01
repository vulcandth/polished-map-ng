//! Profile-specific tile interpretation behind the shared canvas/Print interface.
//! The plusplus renderer retains occurrence banks/palettes/flips/priority; it
//! is never flattened into master's palette-per-tile model. LGPL-3.0-only.
use polished_map_core::{
    Events, Map, PrintError, PrintFont, PrintOptions, PrintStyle, PrintTextMask, RasterError,
    RgbImage, plusplus,
};

#[derive(Clone, Debug)]
pub(crate) enum MapGraphics {
    Master(polished_map_core::Graphics),
    PlusPlus(Box<plusplus::Graphics>),
}

impl From<polished_map_core::Graphics> for MapGraphics {
    fn from(graphics: polished_map_core::Graphics) -> Self {
        Self::Master(graphics)
    }
}
impl From<plusplus::Graphics> for MapGraphics {
    fn from(graphics: plusplus::Graphics) -> Self {
        Self::PlusPlus(Box::new(graphics))
    }
}

impl MapGraphics {
    pub fn profile(&self) -> super::Profile {
        match self {
            Self::Master(_) => super::Profile::Master,
            Self::PlusPlus(_) => super::Profile::PlusPlus,
        }
    }
    pub fn block_count(&self) -> usize {
        match self {
            Self::Master(g) => g.block_count(),
            Self::PlusPlus(g) => g.block_count(),
        }
    }
    pub fn block_priorities(&self, block: u8) -> Option<[bool; 16]> {
        match self {
            Self::Master(g) => g.block_priorities(block),
            Self::PlusPlus(g) => g.block_priorities(block),
        }
    }
    pub fn render_block(&self, block: u8) -> Result<RgbImage, RasterError> {
        match self {
            Self::Master(g) => g.render_block(block),
            Self::PlusPlus(g) => g.render_block(block),
        }
    }
    #[cfg(test)]
    pub fn render_map(&self, map: &Map) -> Result<RgbImage, RasterError> {
        match self {
            Self::Master(g) => g.render_map(map),
            Self::PlusPlus(g) => g.render_map(map),
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
            Self::Master(g) => g.render_print(map, events, options, style, rasterize),
            Self::PlusPlus(g) => g.render_print(map, events, options, style, rasterize),
        }
    }
}
