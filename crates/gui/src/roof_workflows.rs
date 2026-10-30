//! master main-window.cpp::change_roof_cb and option-dialogs.cpp::Roof_Options_Dialog.
//! LGPL-3.0-only. The 280×80 client area follows the original Option_Dialog math.
use super::*;

pub(super) struct RoofChoiceForm {
    names: Vec<String>,
    selected: usize,
    pub(super) windows_buttons: bool,
    #[cfg(test)]
    pub(super) bounds: Rect,
}

impl Editor {
    pub(super) fn can_change_roof(&self) -> bool {
        self.roof_choices_available
            && self
                .loaded
                .as_ref()
                .is_some_and(|l| l.tileset.is_some() && l.blockset.is_some())
    }

    pub(super) fn request_roof_change(&mut self) {
        if !self.can_change_roof() {
            return;
        }
        let Some(loaded) = &self.loaded else {
            return;
        };
        let Some(tileset) = &loaded.tileset else {
            return;
        };
        if loaded.blockset.is_none() {
            return;
        }
        if tileset
            .roof()
            .is_some_and(|r| r.roof().tileset().is_modified())
        {
            self.confirm_roof_change = true;
        } else {
            self.prepare_roof_choice();
        }
    }

    fn prepare_roof_choice(&mut self) {
        let Some(tileset) = self.loaded.as_ref().and_then(|l| l.tileset.as_ref()) else {
            return;
        };
        match polished_map_project::list_roofs(tileset.root()) {
            Ok(names) => {
                let selected = tileset
                    .roof()
                    .and_then(|r| names.iter().position(|n| n == r.name()))
                    .map_or(0, |i| i + 1);
                self.roof_choice_form = Some(RoofChoiceForm {
                    names,
                    selected,
                    windows_buttons: cfg!(target_os = "windows"),
                    #[cfg(test)]
                    bounds: Rect::NOTHING,
                });
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn change_loaded_roof(
        &mut self,
        ctx: &egui::Context,
        name: Option<&str>,
    ) -> Result<(), String> {
        let loaded = self.loaded.as_mut().ok_or("No map is loaded")?;
        let mut draft = loaded
            .tileset
            .as_ref()
            .ok_or("No tileset is loaded")?
            .clone();
        let blocks = loaded
            .blockset
            .as_ref()
            .ok_or("Block definitions are unavailable")?;
        draft.change_roof(name).map_err(|e| e.to_string())?;
        let graphics = draft
            .graphics(blocks.blockset())
            .map_err(|e| e.to_string())?;
        let images = (0..graphics.block_count())
            .map(|id| graphics.render_block(id as u8))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let textures = images
            .into_iter()
            .enumerate()
            .map(|(id, rgb)| {
                ctx.load_texture(
                    format!("block-{id}"),
                    egui::ColorImage::from_rgb([32, 32], &rgb.pixels),
                    egui::TextureOptions::NEAREST,
                )
            })
            .collect();
        loaded.tileset = Some(draft);
        loaded.graphics = graphics;
        self.textures = textures;
        self.status = format!("Roof: {}", name.unwrap_or("(none)"));
        Ok(())
    }

    pub(super) fn roof_choice_dialog(&mut self, ctx: &egui::Context) {
        if self.confirm_roof_change {
            let name = self
                .loaded
                .as_ref()
                .and_then(|l| l.tileset.as_ref())
                .and_then(|t| t.roof())
                .map_or("Roof", |r| r.name());
            let message = format!("{name}.png has unsaved changes!\n\nChange the roof anyway?");
            let (proceed, cancel) = message_dialog::confirm(
                ctx,
                "replace-roof",
                &message,
                self.discard_windows_buttons,
                ["Change Roof", "Cancel"],
            );
            if cancel || proceed {
                self.confirm_roof_change = false;
            }
            if proceed && !cancel {
                self.prepare_roof_choice();
            }
            return;
        }
        let Some(mut form) = self.roof_choice_form.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        let choosing = egui::Popup::is_any_open(ctx);
        dialog_host::client(ctx, egui::Id::unique("change-roof"), |ui| {
            dialog_host::heading(ui, "Change Roof");
            let (area, _) = ui.allocate_exact_size(Vec2::new(280.0, 80.0), Sense::hover());
            #[cfg(test)]
            {
                form.bounds = area;
            }
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            let font = egui::FontId::proportional(if form.windows_buttons { 12.0 } else { 13.0 });
            ui.style_mut().override_font_id = Some(font.clone());
            let width = |s: &str| {
                ui.painter()
                    .layout_no_wrap(s.into(), font.clone(), ui.visuals().text_color())
                    .size()
                    .x
                    .ceil()
            };
            // text_width(label, pad) in utils.cpp adds two times the padding.
            let offset = 10.0 + width("Roof:") + 4.0;
            let combo_width = form
                .names
                .iter()
                .map(|n| width(n))
                .fold(width("(none)"), f32::max)
                + 12.0
                + 22.0;
            map_options::label(ui, rect(11.0, 10.0, offset - 14.0, 22.0), "Roof:");
            // Keep long names within the fixed original client width. Full labels
            // remain available in the dropdown and hover text.
            let combo_width = combo_width.min(270.0 - offset);
            let current = form
                .selected
                .checked_sub(1)
                .map_or("(none)", |i| form.names[i].as_str());
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(rect(offset, 10.0, combo_width, 22.0)),
                |ui| {
                    ui.spacing_mut().interact_size.y = 22.0;
                    egui::ComboBox::from_id_salt("roof-choice")
                        .width(combo_width)
                        .truncate()
                        .selected_text(current)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut form.selected, 0, "(none)");
                            for (i, name) in form.names.iter().enumerate() {
                                ui.selectable_value(&mut form.selected, i + 1, name);
                            }
                        })
                        .response
                        .on_hover_text(current);
                },
            );
            let (ok, no) = if form.windows_buttons {
                (96.0, 190.0)
            } else {
                (190.0, 96.0)
            };
            accept = ui
                .put(rect(ok, 48.0, 80.0, 22.0), egui::Button::new("OK"))
                .clicked();
            cancel = ui
                .put(rect(no, 48.0, 80.0, 22.0), egui::Button::new("Cancel"))
                .clicked();
            // Enter in the open dropdown selects an item, not the entire form.
            if !choosing && !egui::Popup::is_any_open(ctx) {
                accept |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            }
        });
        if cancel {
            return;
        }
        if accept {
            let name = form.selected.checked_sub(1).map(|i| form.names[i].as_str());
            if let Err(e) = self.change_loaded_roof(ctx, name) {
                self.error = Some(e);
                self.roof_choice_form = Some(form);
            }
        } else {
            self.roof_choice_form = Some(form);
        }
    }
}
