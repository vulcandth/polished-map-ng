//! master Main_Window::{save_cb,save_map,save_metatileset,save_tileset,
//! save_roof,save_event_script,export_palettes}. LGPL-3.0-only.
use super::*;
use std::collections::VecDeque;

#[cfg(test)]
pub(super) mod tests;

#[derive(Clone, Copy)]
pub(super) enum SaveStep {
    Tileset,
    Roof,
    Blocks,
    Events,
    Map,
}

#[derive(Clone)]
pub(super) struct Message {
    pub text: String,
    pub acknowledge_meta128: bool,
}

#[derive(Default)]
pub(super) struct SaveUi {
    pub messages: VecDeque<Message>,
    pub steps: VecDeque<SaveStep>,
}

impl SaveUi {
    pub fn busy(&self) -> bool {
        !self.messages.is_empty() || !self.steps.is_empty()
    }
}

pub(super) fn basename(path: &std::path::Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

impl Editor {
    pub(super) fn saved_message(&mut self, name: impl std::fmt::Display) {
        self.save_ui.messages.push_back(Message {
            text: format!("Saved {name}!"),
            acknowledge_meta128: false,
        });
    }

    pub(super) fn blockset_saved_message(&mut self) {
        let Some(doc) = self.loaded.as_ref().and_then(|l| l.blockset.as_ref()) else {
            return;
        };
        let name = basename(doc.path());
        let message = if let Some(path) = doc.collision_path() {
            format!("{name}\nand {}", basename(path))
        } else {
            name.clone()
        };
        let count = doc.blockset().blocks().len();
        let warn = doc.collision_path().is_some() && count > 128 && !self.meta128;
        self.saved_message(message);
        if warn {
            self.save_ui.messages.push_back(Message {
                text: format!("Warning: {name} has {count} blocks.\n\nBe sure to fix the 128-block limit:\nhttps://github.com/pret/pokecrystal/blob/master/docs/bugs_and_glitches.md\n(\"LoadMetatiles wraps around past 128 blocks\")"),
                acknowledge_meta128: true,
            });
        }
    }

    /// Run exactly one domain, then wait for its feedback before advancing.
    pub(super) fn advance_save(&mut self) {
        if self.error.is_some() {
            self.save_ui.steps.clear();
            return;
        }
        if !self.save_ui.messages.is_empty() || self.overworld_warning.is_some() {
            return;
        }
        let Some(step) = self.save_ui.steps.pop_front() else {
            return;
        };
        match step {
            SaveStep::Tileset => self.save_tileset(),
            SaveStep::Roof => self.save_roof(),
            SaveStep::Blocks => self.save_blockset(),
            SaveStep::Events => self.save_events(),
            SaveStep::Map => self.save_map(),
        }
        // A failed or canceled native Save As must not advance to another domain.
        if self.error.is_some() || self.save_ui.messages.is_empty() {
            self.save_ui.steps.clear();
        }
    }

    pub(super) fn save_feedback_dialog(&mut self, ctx: &egui::Context) {
        let Some(message) = self.save_ui.messages.front().cloned() else {
            return;
        };
        let kind = if message.acknowledge_meta128 {
            message_dialog::Kind::Warning
        } else {
            message_dialog::Kind::Success
        };
        if message_dialog::show(ctx, "save-feedback", kind, &message.text) {
            self.save_ui.messages.pop_front();
            if message.acknowledge_meta128 {
                self.meta128 = true;
            }
            // Advance on the next frame so the dismissal cannot accept another
            // modal or trigger a parent action with the same key/pointer event.
            ctx.request_repaint();
        }
    }
}
