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
        self.pending_tileset_reload = false;
        if !self.has_tileset_assets() {
            return;
        }
        if self.tileset_assets_modified() {
            self.confirm_tileset_change = true;
        } else {
            self.prepare_tileset_choice();
        }
    }

    fn has_tileset_assets(&self) -> bool {
        self.loaded
            .as_ref()
            .is_some_and(|l| l.tileset.is_some() && l.blockset.is_some())
    }

    fn tileset_assets_modified(&self) -> bool {
        let Some(loaded) = &self.loaded else {
            return false;
        };
        let Some(tileset) = &loaded.tileset else {
            return false;
        };
        let Some(blocks) = &loaded.blockset else {
            return false;
        };
        // Map edits and palette colors survive this operation without prompting.
        blocks.blockset().is_modified()
            || tileset.tileset().is_modified()
            || tileset
                .roof()
                .is_some_and(|r| r.roof().tileset().is_modified())
    }

    /// Source monochrome_cb / allow_256_tiles_cb call change_tileset_cb(NULL):
    /// retain the changed preference even on Cancel, but skip the chooser.
    pub(super) fn request_tile_mode_reload(&mut self, ctx: &egui::Context) {
        if !self.has_tileset_assets() {
            return;
        }
        if self.tileset_assets_modified() {
            self.pending_tileset_reload = true;
            self.confirm_tileset_change = true;
        } else {
            self.reload_tile_mode(ctx);
        }
    }

    fn reload_tile_mode(&mut self, ctx: &egui::Context) {
        let name = self
            .loaded
            .as_ref()
            .and_then(|l| l.tileset.as_ref())
            .map(|t| t.name().to_owned());
        if let Some(name) = name
            && let Err(error) = self.change_loaded_tileset(ctx, &name)
        {
            self.error = Some(error);
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
            .prepare_replacement_in_mode(
                name,
                polished_map_project::MasterTileMode {
                    monochrome: self.monochrome,
                    allow_256: self.allow_256,
                },
            )
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
        self.monochrome = tileset.monochrome();
        loaded.allow_256 = tileset.allow_256();
        loaded.tileset = Some(tileset);
        loaded.blockset = Some(blocks);
        loaded.graphics = graphics;
        self.textures = textures;
        self.block_hotkeys.prune(count);
        self.sidebar_reveal = self.sidebar_reveal.filter(|&id| usize::from(id) < count);
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
            let message = format!(
                "{} has unsaved changes!\n\nChange the tileset anyway?",
                self.modified_filename(),
            );
            let (proceed, cancel) = message_dialog::confirm(
                ctx,
                "replace-tileset",
                &message,
                self.discard_windows_buttons,
                ["Change Tileset", "Cancel"],
            );
            if cancel || proceed {
                self.confirm_tileset_change = false;
            }
            if proceed && !cancel {
                if self.pending_tileset_reload {
                    self.reload_tile_mode(ctx);
                } else {
                    self.prepare_tileset_choice();
                }
            }
            if cancel || proceed {
                self.pending_tileset_reload = false;
            }
            return;
        }
        let Some(mut form) = self.tileset_choice_form.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        let choosing = egui::Popup::is_any_open(ctx);
        dialog_host::client(ctx, egui::Id::unique("change-tileset"), |ui| {
            dialog_host::heading(ui, "Change Tileset");
            // Option_Dialog: 10 + 22 + 16 + 22 + 10 = 80 logical pixels.
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
            let offset = 10.0 + width("Tileset:") + 4.0;
            let combo_width = (form
                .choices
                .iter()
                .map(|c| width(&c.label))
                .fold(0.0, f32::max)
                + 12.0
                + 22.0)
                .min(270.0 - offset);
            map_options::label(ui, rect(11.0, 10.0, offset - 14.0, 22.0), "Tileset:");
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
