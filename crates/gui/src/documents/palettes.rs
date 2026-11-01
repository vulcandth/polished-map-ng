//! Shared palette controls over explicit master/plusplus palette documents.
//! Pinned palette-window.cpp/colors.cpp; LGPL-3.0-only. No ninth palette is
//! exposed by the plusplus adapter and commands keep backend validation.
use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
use polished_map_project::{ProjectError, RoofPaletteLayout, SaveTarget, plusplus};
use std::path::Path;

#[derive(Clone, Debug)]
pub(crate) enum PaletteDocument {
    Master(polished_map_project::PaletteDocument),
    PlusPlus(plusplus::PaletteDocument),
}
impl Default for PaletteDocument {
    fn default() -> Self {
        Self::Master(Default::default())
    }
}
impl From<polished_map_project::PaletteDocument> for PaletteDocument {
    fn from(d: polished_map_project::PaletteDocument) -> Self {
        Self::Master(d)
    }
}
impl From<plusplus::PaletteDocument> for PaletteDocument {
    fn from(d: plusplus::PaletteDocument) -> Self {
        Self::PlusPlus(d)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PaletteRef<'a> {
    Master(&'a polished_map_project::PaletteDocument),
    PlusPlus(&'a plusplus::PaletteDocument),
}
impl PaletteRef<'_> {
    pub fn to_owned(self) -> PaletteDocument {
        match self {
            Self::Master(d) => d.clone().into(),
            Self::PlusPlus(d) => d.clone().into(),
        }
    }
    pub fn selected(self) -> PaletteSet {
        match self {
            Self::Master(d) => d.selected(),
            Self::PlusPlus(d) => d.selected(),
        }
    }
    pub fn is_modified(self) -> bool {
        match self {
            Self::Master(d) => d.is_modified(),
            Self::PlusPlus(d) => d.is_modified(),
        }
    }
    pub fn color(self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        match self {
            Self::Master(d) => Ok(d.bank().color(slot)?),
            Self::PlusPlus(d) => d.color(slot),
        }
    }
    pub fn copy_color(self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        match self {
            Self::Master(d) => Ok(d.bank().copy_color(slot)?),
            Self::PlusPlus(d) => d.copy_color(slot),
        }
    }
    #[cfg(test)]
    pub fn colors(self, set: PaletteSet) -> Vec<polished_map_core::Palette> {
        match self {
            Self::Master(d) => d.bank().colors(set).to_vec(),
            Self::PlusPlus(d) => d.colors(set).to_vec(),
        }
    }
    #[cfg(test)]
    pub fn color_snapshot(self) -> Vec<Vec<polished_map_core::Palette>> {
        PaletteSet::ALL
            .into_iter()
            .map(|set| self.colors(set))
            .collect()
    }
    #[cfg(test)]
    pub fn content_hash(self) -> String {
        match self {
            Self::Master(d) => d.content_hash(),
            Self::PlusPlus(d) => d.content_hash(),
        }
    }
}
impl PaletteDocument {
    #[cfg(test)]
    pub fn content_hash(&self) -> String {
        self.view().content_hash()
    }
    pub fn view(&self) -> PaletteRef<'_> {
        match self {
            Self::Master(d) => PaletteRef::Master(d),
            Self::PlusPlus(d) => PaletteRef::PlusPlus(d),
        }
    }
    pub fn selected(&self) -> PaletteSet {
        self.view().selected()
    }
    pub fn color(&self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        self.view().color(slot)
    }
    pub fn copy_color(&self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        self.view().copy_color(slot)
    }
    pub fn select(&mut self, set: PaletteSet) {
        match self {
            Self::Master(d) => d.select(set),
            Self::PlusPlus(d) => d.select(set),
        }
    }
    pub fn mark_modified(&mut self) {
        match self {
            Self::Master(d) => d.mark_modified(),
            Self::PlusPlus(d) => d.mark_modified(),
        }
    }
    pub fn apply_batch(&mut self, commands: &[PaletteCommand]) -> Result<(), ProjectError> {
        match self {
            Self::Master(d) => d.apply_batch(commands),
            Self::PlusPlus(d) => d.apply_batch(commands),
        }
    }
    pub fn load(&mut self, path: &Path) -> Result<(), ProjectError> {
        match self {
            Self::Master(d) => d.load(path),
            Self::PlusPlus(d) => d.load(path),
        }
    }
    pub fn load_roof_colors(
        &mut self,
        path: &Path,
        group: usize,
        layout: RoofPaletteLayout,
    ) -> Result<(), ProjectError> {
        match self {
            Self::Master(d) => d.load_roof_colors(path, group, layout),
            Self::PlusPlus(d) => d.load_roof_colors(path, group, layout),
        }
    }
    pub fn export(&mut self, target: &SaveTarget, monochrome: bool) -> Result<(), ProjectError> {
        match self {
            Self::Master(d) => d.export(target, monochrome),
            Self::PlusPlus(d) => d.export(target),
        }
    }
}
