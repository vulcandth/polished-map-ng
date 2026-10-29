//! master Main_Window::change_tileset_cb/force_add_sub_metatiles and
//! Tileset_Options_Dialog. LGPL-3.0-only. Failed replacements preserve the session.
use super::*;
use polished_map_project::TilesetChoice;

pub(super) struct TilesetChoiceForm {
    choices: Vec<TilesetChoice>,
    selected: usize,
    pub(super) windows_buttons: bool,
    #[cfg(test)]
    pub(super) bounds: Rect,
}

impl Editor {
    pub(super) fn request_tileset_change(&mut self) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let Some(tileset) = &loaded.tileset else {
            return;
        };
        let Some(blocks) = &loaded.blockset else {
            return;
        };
        // Map edits and palette colors survive this operation without prompting.
        if blocks.blockset().is_modified()
            || tileset.tileset().is_modified()
            || tileset
                .roof()
                .is_some_and(|r| r.roof().tileset().is_modified())
        {
            self.confirm_tileset_change = true;
        } else {
            self.prepare_tileset_choice();
        }
    }

    fn prepare_tileset_choice(&mut self) {
        let Some(tileset) = self.loaded.as_ref().and_then(|l| l.tileset.as_ref()) else {
            return;
        };
        match polished_map_project::tileset_choices(tileset.root()) {
            Ok(choices) if !choices.is_empty() => {
                let selected = choices
                    .iter()
                    .position(|c| c.name == tileset.name())
                    .unwrap_or(0);
                self.tileset_choice_form = Some(TilesetChoiceForm {
                    choices,
                    selected,
                    windows_buttons: cfg!(target_os = "windows"),
                    #[cfg(test)]
                    bounds: Rect::NOTHING,
                });
            }
            Ok(_) => self.error = Some("No tilesets were found in this project".into()),
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn change_loaded_tileset(&mut self, ctx: &egui::Context, name: &str) -> Result<(), String> {
        let loaded = self.loaded.as_mut().ok_or("No map is loaded")?;
        let previous = loaded.tileset.as_ref().ok_or("No tileset is loaded")?;
        let (tileset, blocks) = previous
            .prepare_replacement(name)
            .map_err(|e| e.to_string())?;
        let graphics = tileset
            .graphics(blocks.blockset())
            .map_err(|e| e.to_string())?;
        let images = (0..graphics.block_count())
            .map(|id| graphics.render_block(id as u8))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let textures: Vec<_> = images
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
        let count = textures.len();
        // Preserve all map IDs/history, even when some IDs no longer have definitions.
        self.allow_priority = tileset.allow_priority();
        loaded.tileset = Some(tileset);
        loaded.blockset = Some(blocks);
        loaded.graphics = graphics;
        self.textures = textures;
        if usize::from(self.selected) >= count {
            self.selected = 0;
            self.reset_sidebar_scroll = true;
        }
        if self
            .block_clipboard
            .as_ref()
            .is_some_and(|c| usize::from(c.id) >= count)
        {
            self.block_clipboard = None;
        }
        self.status = format!("Tileset: {name} ({count} blocks)");
        Ok(())
    }

    pub(super) fn tileset_choice_dialog(&mut self, ctx: &egui::Context) {
        if self.confirm_tileset_change {
            let mut proceed = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::unique("replace-tileset")).show(ctx, |ui| {
                ui.heading("Unsaved tileset changes");
                ui.label("The blockset, tileset or roof has unsaved changes!\n\nChange the tileset anyway?");
                ui.horizontal(|ui| {
                    proceed = ui.button("Change Tileset").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
                cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            });
            if cancel || proceed {
                self.confirm_tileset_change = false;
            }
            if proceed && !cancel {
                self.prepare_tileset_choice();
            }
            return;
        }
        let Some(mut form) = self.tileset_choice_form.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        let choosing = egui::Popup::is_any_open(ctx);
        egui::Modal::new(egui::Id::unique("change-tileset")).show(ctx, |ui| {
            ui.heading("Change Tileset");
            // Option_Dialog: 10 + 22 + 16 + 22 + 10 = 80 logical pixels.
            let (area, _) = ui.allocate_exact_size(Vec2::new(280.0, 80.0), Sense::hover());
            #[cfg(test)]
            {
                form.bounds = area;
            }
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            let font = egui::FontId::proportional(13.0);
            let width = |s: &str| {
                ui.painter()
                    .layout_no_wrap(s.into(), font.clone(), ui.visuals().text_color())
                    .size()
                    .x
            };
            let offset = 10.0 + width("Tileset:") + 4.0;
            let combo_width = (form
                .choices
                .iter()
                .map(|c| width(&c.label))
                .fold(0.0, f32::max)
                + 12.0
                + 22.0)
                .min(270.0 - offset);
            ui.put(
                rect(10.0, 10.0, offset - 14.0, 22.0),
                egui::Label::new("Tileset:"),
            );
            let current = &form.choices[form.selected].label;
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(rect(offset, 10.0, combo_width, 22.0)),
                |ui| {
                    ui.spacing_mut().interact_size.y = 22.0;
                    egui::ComboBox::from_id_salt("tileset-choice")
                        .width(combo_width)
                        .truncate()
                        .selected_text(current)
                        .show_ui(ui, |ui| {
                            for (i, choice) in form.choices.iter().enumerate() {
                                ui.selectable_value(&mut form.selected, i, &choice.label);
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
            if !choosing && !egui::Popup::is_any_open(ctx) {
                accept |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            }
        });
        if cancel {
            return;
        }
        if accept {
            if let Err(e) = self.change_loaded_tileset(ctx, &form.choices[form.selected].name) {
                self.error = Some(e);
                self.tileset_choice_form = Some(form);
            }
        } else {
            self.tileset_choice_form = Some(form);
        }
    }
}
