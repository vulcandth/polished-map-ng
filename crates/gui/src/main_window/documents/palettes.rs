//! Shared palette controls over explicit master/plusplus palette documents.
//! Pinned palette-window.cpp/colors.cpp; LGPL-3.0-only. No ninth palette is
//! exposed by the plusplus adapter and commands keep backend validation.
use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
use polished_map_project::{ProjectError, RoofPaletteLayout, SaveTarget, attributed};
use std::path::Path;

#[derive(Clone, Debug)]
pub(crate) enum PaletteDocument {
    Classic(polished_map_project::PaletteDocument),
    Attributed(attributed::PaletteDocument),
}
impl Default for PaletteDocument {
    fn default() -> Self {
        Self::Classic(Default::default())
    }
}
impl From<polished_map_project::PaletteDocument> for PaletteDocument {
    fn from(d: polished_map_project::PaletteDocument) -> Self {
        Self::Classic(d)
    }
}
impl From<attributed::PaletteDocument> for PaletteDocument {
    fn from(d: attributed::PaletteDocument) -> Self {
        Self::Attributed(d)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PaletteRef<'a> {
    Classic(&'a polished_map_project::PaletteDocument),
    Attributed(&'a attributed::PaletteDocument),
}
impl PaletteRef<'_> {
    pub fn to_owned(self) -> PaletteDocument {
        match self {
            Self::Classic(d) => d.clone().into(),
            Self::Attributed(d) => d.clone().into(),
        }
    }
    pub fn selected(self) -> PaletteSet {
        match self {
            Self::Classic(d) => d.selected(),
            Self::Attributed(d) => d.selected(),
        }
    }
    pub fn is_modified(self) -> bool {
        match self {
            Self::Classic(d) => d.is_modified(),
            Self::Attributed(d) => d.is_modified(),
        }
    }
    pub fn color(self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        match self {
            Self::Classic(d) => Ok(d.bank().color(slot)?),
            Self::Attributed(d) => d.color(slot),
        }
    }
    pub fn copy_color(self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        match self {
            Self::Classic(d) => Ok(d.bank().copy_color(slot)?),
            Self::Attributed(d) => d.copy_color(slot),
        }
    }
    #[cfg(test)]
    pub fn colors(self, set: PaletteSet) -> Vec<polished_map_core::Palette> {
        match self {
            Self::Classic(d) => d.bank().colors(set).to_vec(),
            Self::Attributed(d) => d.colors(set).to_vec(),
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
            Self::Classic(d) => d.content_hash(),
            Self::Attributed(d) => d.content_hash(),
        }
    }
}
impl PaletteDocument {
    pub fn storage(&self) -> super::Storage {
        match self {
            Self::Classic(_) => super::Storage::Classic,
            Self::Attributed(_) => super::Storage::Attributed,
        }
    }
    #[cfg(test)]
    pub fn content_hash(&self) -> String {
        self.view().content_hash()
    }
    pub fn view(&self) -> PaletteRef<'_> {
        match self {
            Self::Classic(d) => PaletteRef::Classic(d),
            Self::Attributed(d) => PaletteRef::Attributed(d),
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
            Self::Classic(d) => d.select(set),
            Self::Attributed(d) => d.select(set),
        }
    }
    pub fn mark_modified(&mut self) {
        match self {
            Self::Classic(d) => d.mark_modified(),
            Self::Attributed(d) => d.mark_modified(),
        }
    }
    pub fn apply_batch(&mut self, commands: &[PaletteCommand]) -> Result<(), ProjectError> {
        match self {
            Self::Classic(d) => d.apply_batch(commands),
            Self::Attributed(d) => d.apply_batch(commands),
        }
    }
    pub fn load(&mut self, path: &Path) -> Result<(), ProjectError> {
        match self {
            Self::Classic(d) => d.load(path),
            Self::Attributed(d) => d.load(path),
        }
    }
    pub fn load_roof_colors(
        &mut self,
        path: &Path,
        group: usize,
        layout: RoofPaletteLayout,
    ) -> Result<(), ProjectError> {
        match self {
            Self::Classic(d) => d.load_roof_colors(path, group, layout),
            Self::Attributed(d) => d.load_roof_colors(path, group, layout),
        }
    }
    pub fn export(&mut self, target: &SaveTarget, monochrome: bool) -> Result<(), ProjectError> {
        match self {
            Self::Classic(d) => d.export(target, monochrome),
            Self::Attributed(d) => d.export(target),
        }
    }
}
