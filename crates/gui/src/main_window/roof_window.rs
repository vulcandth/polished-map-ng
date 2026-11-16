//! Roof_Window editing availability, draft startup and save lifecycle.
//! Shared pixel controls live in tileset_window; LGPL-3.0-only.
use super::*;

impl Editor {
    pub(super) fn can_edit_roof(&self) -> bool {
        self.loaded.as_ref().is_some_and(|l| {
            l.tileset.as_ref().is_some_and(|t| t.roof().is_some())
                || l.attributed_assets
                    .as_ref()
                    .is_some_and(|a| a.roof.is_some())
        })
    }
    pub(super) fn prepare_roof(&mut self) {
        self.prepare_tile_form(true);
    }
    pub(super) fn save_roof(&mut self) {
        if self.save_attributed_roof() {
            return;
        }
        let Some(roof) = self
            .loaded
            .as_mut()
            .and_then(|l| l.tileset.as_mut())
            .and_then(|t| t.roof_mut())
        else {
            return;
        };
        if !roof.roof().tileset().is_modified() {
            let name = roof.name().to_owned();
            self.saved_message(name);
            return;
        }
        let name = save_feedback::basename(roof.output().path());
        match roof.save() {
            Ok(()) => {
                self.status = "Roof saved".into();
                self.saved_message(name);
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
}
