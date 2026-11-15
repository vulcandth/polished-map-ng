//! Pinned plusplus colors.cpp and Main_Window::open_map/load_palettes.
//! Reuses master RGB editing/export, roof overlays and import continuations,
//! restricting the session to eight game palettes. LGPL-3.0-only.
use super::MapDocument;
use crate::{MasterGraphicsOptions, ProjectError, RoofPaletteLayout, SaveTarget, discovery::Rules};
use polished_map_core::{ColorSlot, Palette, PaletteCommand, PaletteSet};
use std::path::Path;

#[derive(Clone, Debug, Default)]
/// Palette session restricted to eight game palettes per set.
///
/// Delegates import/export and dirty-state handling to [`crate::PaletteDocument`];
/// monochrome slots and monochrome-only imports are excluded.
pub struct PaletteDocument {
    inner: crate::PaletteDocument,
}

fn validate_slot(slot: ColorSlot) -> Result<(), ProjectError> {
    if slot.palette >= 8 {
        return Err(polished_map_core::attributed::Error::Palette(slot.palette).into());
    }
    Ok(())
}

impl PaletteDocument {
    /// Copy the eight game palettes for the chosen time/environment set.
    pub fn colors(&self, set: PaletteSet) -> [Palette; 8] {
        std::array::from_fn(|i| self.inner.bank().colors(set)[i])
    }
    /// Read an RGB8 hue. Invalid palette (not 0..=7) or hue indices return an error.
    pub fn color(&self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        validate_slot(slot)?;
        Ok(self.inner.bank().color(slot)?)
    }
    /// Read a color through RGB5 clipboard quantization; invalid slots return an error.
    pub fn copy_color(&self, slot: ColorSlot) -> Result<[u8; 3], ProjectError> {
        validate_slot(slot)?;
        Ok(self.inner.bank().copy_color(slot)?)
    }
    /// Current display palette set.
    pub fn selected(&self) -> PaletteSet {
        self.inner.selected()
    }
    /// Change the display set without dirtying colors.
    pub fn select(&mut self, set: PaletteSet) {
        self.inner.select(set);
    }
    /// Whether edits or dialog acceptance have marked this session dirty.
    pub fn is_modified(&self) -> bool {
        self.inner.is_modified()
    }
    /// Mark accepted palette edits dirty even if color values did not change.
    pub fn mark_modified(&mut self) {
        self.inner.mark_modified();
    }
    /// Fingerprint RGB8 colors, selection and import source; see [`crate::PaletteDocument::content_hash`].
    pub fn content_hash(&self) -> String {
        self.inner.content_hash()
    }
    /// Check imported source content; returns read errors or a conflict if changed.
    pub fn verify_source(&self) -> Result<(), ProjectError> {
        self.inner.verify_source()
    }
    /// Import recognized game-palette layouts, preserving unaffected sets.
    ///
    /// Monochrome-only layouts leave colors/selection intact. Successful imports
    /// clear dirty state. Callers resolve pending edits before loading.
    ///
    /// # Errors
    /// Read, UTF-8, RGB parsing, and observation conflicts leave the session intact.
    pub fn load(&mut self, path: &Path) -> Result<(), ProjectError> {
        self.inner.load_with_rules(path, Rules::PlusPlus)
    }
    /// Overlay a map group’s roof hues without changing dirty state or selection.
    ///
    /// # Errors
    /// Invalid RGB data, missing groups, or reads fail without changing colors.
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
/// Palette selection and import options for attributed map opening.
pub struct PaletteOptions {
    /// Explicit set, or automatic selection from map metadata.
    pub palette: Option<PaletteSet>,
    /// Use the dark-theme automatic fallback where metadata does not select a set.
    pub dark_theme: bool,
    /// Import discovered map/landmark/tileset-specific palette files.
    pub special_palettes: bool,
    /// Include the map-group roof-color overlay step.
    pub roof_colors: bool,
    /// Set ordering in the roof-color table.
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
    /// Build an import plan using attributed discovery precedence.
    ///
    /// # Errors
    /// Rejects invalid tileset names, missing roots, or failed path resolution.
    pub fn for_map(
        doc: &MapDocument,
        tileset: &str,
        options: &PaletteOptions,
    ) -> Result<Self, ProjectError> {
        Self::with_sources(doc, tileset, options, false)
    }
    /// Build an import plan using combined supported source layouts.
    ///
    /// # Errors
    /// Rejects invalid names, missing roots, or ambiguous/failed asset resolution.
    pub fn for_unified_map(
        doc: &MapDocument,
        tileset: &str,
        options: &PaletteOptions,
    ) -> Result<Self, ProjectError> {
        Self::with_sources(doc, tileset, options, true)
    }
    fn with_sources(
        doc: &MapDocument,
        tileset: &str,
        options: &PaletteOptions,
        unified: bool,
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
            inner: crate::MapPalettePlan::with_sources(&doc.inner, tileset, &shared, unified)?,
        })
    }
    /// Whether no import or selection step remains.
    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }
    /// Whether the next step is an optional roof-color overlay.
    pub fn next_is_roof_colors(&self) -> bool {
        self.inner.next_is_roof_colors()
    }
    /// Next import path; `None` means automatic selection or a finished plan.
    pub fn next_import_path(&self) -> Option<&Path> {
        self.inner.next_import_path()
    }
    /// Execute one step, or skip a file/roof import when `load` is false.
    ///
    /// Automatic selection still runs. Read/parse errors preserve both plan and
    /// session, allowing the caller to report the failure or explicitly skip.
    pub fn advance(
        &mut self,
        palettes: &mut PaletteDocument,
        load: bool,
    ) -> Result<(), ProjectError> {
        self.inner.advance(&mut palettes.inner, load)
    }
}
