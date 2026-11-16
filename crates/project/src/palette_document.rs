//! master Main_Window::{load_palettes,export_palettes,edit_current_palettes_cb}.
//! Export is explicit and independent of map/tileset saves.
use crate::{
    ProjectError, SaveTarget,
    file_io::{optional_hash, persist_bytes, prepare_target},
    parse_rgb_palettes, read_bounded,
};
use polished_map_core::{PaletteBank, PaletteCommand, PaletteSet};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Clone, Debug)]
/// Session colors, selected set, dirty state and optional imported source.
///
/// Exports are explicit and separate from map/tileset saves. The default
/// uses the upstream palette bank with Day selected.
pub struct PaletteDocument {
    bank: PaletteBank,
    selected: PaletteSet,
    modified: bool,
    source: Option<SaveTarget>,
}

impl Default for PaletteDocument {
    fn default() -> Self {
        Self::new(PaletteBank::default(), PaletteSet::Day)
    }
}
impl PaletteDocument {
    /// Create a clean palette session without an imported source.
    pub fn new(bank: PaletteBank, selected: PaletteSet) -> Self {
        Self {
            bank,
            selected,
            modified: false,
            source: None,
        }
    }
    /// Read all six color sets.
    pub fn bank(&self) -> &PaletteBank {
        &self.bank
    }
    /// Current display palette set.
    pub fn selected(&self) -> PaletteSet {
        self.selected
    }
    /// Change the display set without marking colors dirty.
    pub fn select(&mut self, selected: PaletteSet) {
        self.selected = selected;
    }
    /// Whether palette edits or dialog acceptance have marked the session dirty.
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    /// Dialog acceptance is dirty even if the values did not change.
    pub fn mark_modified(&mut self) {
        self.modified = true;
    }
    pub(crate) fn accept_missing_import(&mut self) {
        // Main_Window::load_palettes clears edited state even if no file was read.
        self.modified = false;
    }
    /// Import a map group's LIGHT/DARK roof colors using the selected table
    /// layout. This leaves the selected set, other colors, import source and
    /// edited flag intact. Failure changes nothing. Callers handle consent for
    /// edited palettes; this API has no GUI or display dependency.
    pub fn load_roof_colors(
        &mut self,
        path: &Path,
        group: usize,
        layout: crate::RoofPaletteLayout,
    ) -> Result<(), ProjectError> {
        crate::colors::apply_roofs(&mut self.bank, path, group, layout)
    }
    /// Atomically validate/apply colors and mark nonempty batches dirty.
    ///
    /// # Errors
    /// Invalid slots or color values leave the bank and dirty state unchanged.
    pub fn apply_batch(&mut self, commands: &[PaletteCommand]) -> Result<(), ProjectError> {
        self.bank.edit(commands)?;
        self.modified |= !commands.is_empty();
        Ok(())
    }
    /// Import overlays the layouts recognized by upstream; other sets survive.
    /// Callers decide whether to discard pending color edits before invoking this.
    pub fn load(&mut self, path: &Path) -> Result<(), ProjectError> {
        self.load_with_rules(path, crate::config::Rules::Master)
    }

    pub(crate) fn load_with_rules(
        &mut self,
        path: &Path,
        rules: crate::config::Rules,
    ) -> Result<(), ProjectError> {
        let source = prepare_target(path)?;
        let raw = read_bounded(path, 4 * 1024 * 1024)?;
        let text = std::str::from_utf8(&raw).map_err(|e| ProjectError::Format {
            path: path.into(),
            message: e.to_string(),
        })?;
        let palettes = parse_rgb_palettes(text).map_err(|message| ProjectError::Format {
            path: path.into(),
            message,
        })?;
        if optional_hash(path)? != source.hash {
            return Err(ProjectError::Conflict(path.into()));
        }
        // Plusplus has eight game palettes; 1..6-palette monochrome imports
        // are a master-only format and do not change its selection or colors.
        if rules == crate::config::Rules::Master || palettes.len() > 6 {
            self.selected = self.bank.apply(&palettes, self.selected);
        }
        self.source = Some(source);
        self.modified = false;
        Ok(())
    }
    /// Fingerprint of this exact RGB8 bank, selection and optional import source.
    /// Exports quantize to RGB5, so reopening need not yield this draft fingerprint.
    pub fn content_hash(&self) -> String {
        let mut hash = Sha256::new();
        hash.update([self.selected as u8]);
        for set in PaletteSet::ALL {
            for palette in self.bank.colors(set) {
                for color in palette {
                    hash.update(color);
                }
            }
        }
        if let Some(source) = &self.source {
            let path = source.path.as_os_str().as_encoded_bytes();
            hash.update((path.len() as u64).to_le_bytes());
            hash.update(path);
            if let Some(value) = source.hash {
                hash.update(value);
            }
        }
        hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }
    /// Existing destination authorization is the caller's responsibility, as for Save As.
    /// Verify observed input and output contents immediately before publication.
    pub fn export(&mut self, target: &SaveTarget, monochrome: bool) -> Result<(), ProjectError> {
        self.verify_source()?;
        let text = self.bank.export_rgb(monochrome);
        persist_bytes(&target.path, target.hash, text.as_bytes())?;
        if let Some(source) = &mut self.source
            && source.path == target.path
        {
            source.hash = Some(Sha256::digest(text.as_bytes()).into());
        }
        self.modified = false;
        Ok(())
    }
    /// Check the last imported palette source, if any.
    ///
    /// # Errors
    /// Returns read errors or a conflict if observed content changed.
    pub fn verify_source(&self) -> Result<(), ProjectError> {
        if let Some(source) = &self.source
            && optional_hash(&source.path)? != source.hash
        {
            return Err(ProjectError::Conflict(source.path.clone()));
        }
        Ok(())
    }
}

/// Observe a palette export destination, adding `.pal` when no suffix exists.
///
/// # Errors
/// Returns invalid-path or bounded observation errors; no files are written.
pub fn prepare_palette_export(path: &Path) -> Result<SaveTarget, ProjectError> {
    let path = if path.extension().is_none() {
        path.with_extension("pal")
    } else {
        path.into()
    };
    prepare_target(&path)
}
