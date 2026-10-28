//! master Main_Window::{load_palettes,export_palettes,edit_current_palettes_cb}.
//! Export is explicit and independent of map/tileset saves.
use crate::{
    ProjectError, SaveTarget,
    document::{optional_hash, persist_bytes, prepare_target},
    parse_rgb_palettes, read_bounded,
};
use polished_map_core::{PaletteBank, PaletteCommand, PaletteSet};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Clone, Debug)]
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
    pub fn new(bank: PaletteBank, selected: PaletteSet) -> Self {
        Self {
            bank,
            selected,
            modified: false,
            source: None,
        }
    }
    pub fn bank(&self) -> &PaletteBank {
        &self.bank
    }
    pub fn selected(&self) -> PaletteSet {
        self.selected
    }
    pub fn select(&mut self, selected: PaletteSet) {
        self.selected = selected;
    }
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
    pub(crate) fn overlay_roof_colors(
        &mut self,
        path: &Path,
        group: usize,
        layout: crate::RoofPaletteLayout,
    ) -> Result<(), ProjectError> {
        crate::palettes::apply_roofs(&mut self.bank, &path.to_path_buf(), group, layout)
    }
    pub fn apply_batch(&mut self, commands: &[PaletteCommand]) -> Result<(), ProjectError> {
        self.bank.edit(commands)?;
        self.modified |= !commands.is_empty();
        Ok(())
    }
    /// Import overlays the layouts recognized by upstream; other sets survive.
    /// Callers decide whether to discard pending color edits before invoking this.
    pub fn load(&mut self, path: &Path) -> Result<(), ProjectError> {
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
        self.selected = self.bank.apply(&palettes, self.selected);
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
    pub fn verify_source(&self) -> Result<(), ProjectError> {
        if let Some(source) = &self.source
            && optional_hash(&source.path)? != source.hash
        {
            return Err(ProjectError::Conflict(source.path.clone()));
        }
        Ok(())
    }
}

pub fn prepare_palette_export(path: &Path) -> Result<SaveTarget, ProjectError> {
    let path = if path.extension().is_none() {
        path.with_extension("pal")
    } else {
        path.into()
    };
    prepare_target(&path)
}
