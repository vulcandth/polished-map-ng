//! Pinned plusplus colors.cpp and Main_Window::open_map/load_palettes.
//! Reuses master RGB editing/export, roof overlays and import continuations,
//! restricting the session to eight game palettes. LGPL-3.0-only.
use super::MapDocument;
use crate::{MasterGraphicsOptions, ProjectError, RoofPaletteLayout, SaveTarget, discovery::Rules};
use polished_map_core::{ColorSlot, Palette, PaletteCommand, PaletteSet};
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct PaletteDocument {
    inner: crate::PaletteDocument,
}

fn validate_slot(slot: ColorSlot) -> Result<(), ProjectError> {
    if slot.palette >= 8 {
        return Err(polished_map_core::plusplus::Error::Palette(slot.palette).into());
    }
    Ok(())
}

impl PaletteDocument {
    pub fn colors(&self, set: PaletteSet) -> [Palette; 8] {
        std::array::from_fn(|i| self.inner.bank().colors(set)[i])
    }
    pub fn color(&self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        validate_slot(slot)?;
        Ok(self.inner.bank().color(slot)?)
    }
    pub fn copy_color(&self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        validate_slot(slot)?;
        Ok(self.inner.bank().copy_color(slot)?)
    }
    pub fn selected(&self) -> PaletteSet {
        self.inner.selected()
    }
    pub fn select(&mut self, set: PaletteSet) {
        self.inner.select(set);
    }
    pub fn is_modified(&self) -> bool {
        self.inner.is_modified()
    }
    pub fn mark_modified(&mut self) {
        self.inner.mark_modified();
    }
    pub fn content_hash(&self) -> String {
        self.inner.content_hash()
    }
    pub fn verify_source(&self) -> Result<(), ProjectError> {
        self.inner.verify_source()
    }
    pub fn load(&mut self, path: &Path) -> Result<(), ProjectError> {
        self.inner.load_with_rules(path, Rules::PlusPlus)
    }
    pub fn load_roof_colors(
        &mut self,
        path: &Path,
        group: usize,
        layout: RoofPaletteLayout,
    ) -> Result<(), ProjectError> {
        self.inner.load_roof_colors(path, group, layout)
    }
    /// Validate both ends of transfers before executing any command.
    pub fn apply_batch(&mut self, commands: &[PaletteCommand]) -> Result<(), ProjectError> {
        for command in commands {
            match command {
                PaletteCommand::SetRgb { slot, .. }
                | PaletteCommand::Channel { slot, .. }
                | PaletteCommand::Hex { slot, .. } => validate_slot(*slot)?,
                PaletteCommand::Copy { from, to } | PaletteCommand::Swap { a: from, b: to } => {
                    validate_slot(*from)?;
                    validate_slot(*to)?;
                }
            }
        }
        self.inner.apply_batch(commands)
    }
    /// Export all six sets, with RGB5 quantization and shared conflict protection.
    pub fn export(&mut self, target: &SaveTarget) -> Result<(), ProjectError> {
        self.inner.export(target, false)
    }
}

#[derive(Clone, Debug)]
pub struct PaletteOptions {
    pub palette: Option<PaletteSet>,
    pub dark_theme: bool,
    pub special_palettes: bool,
    pub roof_colors: bool,
    pub roof_layout: RoofPaletteLayout,
}

impl Default for PaletteOptions {
    fn default() -> Self {
        Self {
            palette: None,
            dark_theme: false,
            special_palettes: true,
            roof_colors: true,
            roof_layout: RoofPaletteLayout::DayNite,
        }
    }
}

/// Ordered default/special import, contextual selection, then roof overlay.
/// Callers retain the same consent/skip/error boundaries as the master session.
#[derive(Clone, Debug)]
pub struct MapPalettePlan {
    inner: crate::MapPalettePlan,
}

impl MapPalettePlan {
    pub fn for_map(
        doc: &MapDocument,
        tileset: &str,
        options: &PaletteOptions,
    ) -> Result<Self, ProjectError> {
        let shared = MasterGraphicsOptions {
            palette: options.palette,
            dark_theme: options.dark_theme,
            special_palettes: options.special_palettes,
            roof_colors: options.roof_colors,
            roof_layout: options.roof_layout,
            ..Default::default()
        };
        Ok(Self {
            inner: crate::MapPalettePlan::for_map(&doc.inner, tileset, &shared)?,
        })
    }
    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }
    pub fn next_is_roof_colors(&self) -> bool {
        self.inner.next_is_roof_colors()
    }
    pub fn next_import_path(&self) -> Option<&Path> {
        self.inner.next_import_path()
    }
    pub fn advance(
        &mut self,
        palettes: &mut PaletteDocument,
        load: bool,
    ) -> Result<(), ProjectError> {
        self.inner.advance(&mut palettes.inner, load)
    }
}
