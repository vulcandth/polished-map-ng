//! Editor modal dispatch and input ordering. LGPL-3.0-only.
use super::*;

impl Editor {
    pub(super) fn dialogs(&mut self, ctx: &egui::Context) {
        if ctx.embed_viewports() {
            self.dialog_contents(ctx);
            return;
        }
        let kind = self.dialog_kind();
        dialog_host::active(ctx, kind);
        if let Some(kind) = kind {
            dialog_host::show(ctx, dialog_host::main_id(kind), |ctx| {
                // A failure belongs above the retained editor. Render that
                // editor inactive; do not discard/recreate its form or history.
                let error = if kind != "error" {
                    self.error.take()
                } else {
                    None
                };
                if error.is_some() {
                    dialog_host::error_parent(ctx);
                }
                self.dialog_contents(ctx);
                if kind != "error" {
                    if let Some(message) = error.or_else(|| self.error.take()) {
                        if !dialog_host::error(ctx, &message) {
                            self.error = Some(message);
                        }
                    } else {
                        dialog_host::clear_error(ctx);
                    }
                }
            });
        }
        dialog_host::active(ctx, self.dialog_kind());
        if let Some(reason) = dialog_host::take_error(ctx) {
            self.error.get_or_insert_with(|| reason.to_owned());
        }
    }

    pub(super) fn dialog_kind(&self) -> Option<&'static str> {
        if self.open_palette_warning.is_some() {
            Some("open-palette-warning")
        } else if self.about.is_some() {
            Some("about")
        } else if self.print_form.is_some() || self.print_success.is_some() {
            Some("print")
        } else if self.overworld_warning.is_some() || self.overworld_form.is_some() {
            Some("overworld")
        } else if !self.save_ui.messages.is_empty() {
            Some("save-feedback")
        } else if self.confirm_discard {
            Some("discard")
        } else if self.pending_roof_colors.is_some() || self.roof_color_success.is_some() {
            Some("roof-colors")
        } else if self.event_ui.change.is_some() || self.event_ui.form.is_some() {
            Some("event")
        } else if self.block_form.is_some() {
            Some("block")
        } else if self.tileset_form.is_some() {
            Some("tileset")
        } else if self.palette_form.is_some() {
            Some("palette")
        } else if self.confirm_roof_change || self.roof_choice_form.is_some() {
            Some("roof-choice")
        } else if self.confirm_tileset_change || self.tileset_choice_form.is_some() {
            Some("tileset-choice")
        } else if self.pending_palette_load.is_some() {
            Some("palette-load")
        } else if self.pending_open.is_some() {
            Some("pending-open")
        } else if self.block_count_form.is_some() {
            Some("block-count")
        } else if self.resize_form.is_some() {
            Some("resize")
        } else if self.open_form.is_some() {
            Some("map-options")
        } else if self.error.is_some() {
            Some("error")
        } else {
            None
        }
    }

    pub(super) fn dialog_contents(&mut self, ctx: &egui::Context) {
        if let Some(message) = self.error.clone() {
            if message_dialog::show(ctx, "error", message_dialog::Kind::Error, &message) {
                self.error = None;
            }
            return;
        }
        if let Some(message) = self.open_palette_warning.clone() {
            if message_dialog::show(
                ctx,
                "open-palette-warning",
                message_dialog::Kind::Warning,
                &message,
            ) {
                self.open_palette_warning = None;
            }
            return;
        }
        if let Some(mut about) = self.about.take() {
            if !about.show(ctx, self.application) {
                self.about = Some(about);
            }
            return;
        }
        if self.print_form.is_some() || self.print_success.is_some() {
            self.print_dialog(ctx);
            return;
        }
        if self.overworld_warning.is_some() || self.overworld_form.is_some() {
            self.overworld_dialog(ctx);
            return;
        }
        if !self.save_ui.messages.is_empty() {
            self.save_feedback_dialog(ctx);
            return;
        }
        if self.confirm_discard {
            self.discard_dialog(ctx);
            return;
        }
        if self.pending_roof_colors.is_some() || self.roof_color_success.is_some() {
            self.roof_color_dialog(ctx);
            return;
        }
        if self.event_ui.change.is_some() || self.event_ui.form.is_some() {
            self.event_dialog(ctx);
            return;
        }
        if self.block_form.is_some() {
            self.block_dialog(ctx);
            return;
        }
        if self.tileset_form.is_some() {
            self.tileset_dialog(ctx);
            return;
        }
        if self.palette_form.is_some() {
            self.palette_dialog(ctx);
            return;
        }
        if self.confirm_roof_change || self.roof_choice_form.is_some() {
            self.roof_choice_dialog(ctx);
            return;
        }
        if self.confirm_tileset_change || self.tileset_choice_form.is_some() {
            self.tileset_choice_dialog(ctx);
            return;
        }
        if self.pending_palette_load.is_some() {
            self.palette_load_dialog(ctx);
            return;
        }
        if self.pending_open.is_some() {
            if self.pending_map_warning.is_some() {
                self.map_length_dialog(ctx);
            } else {
                self.open_palette_dialog(ctx);
            }
            return;
        }
        if self.block_count_form.is_some() {
            self.block_count_dialog(ctx);
            return;
        }
        self.resize_dialog(ctx);
        self.map_options_dialog(ctx);
    }
}
