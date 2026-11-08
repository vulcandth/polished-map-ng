//! Pinned master/plusplus Main_Window::change_tileset_cb/force_add_sub_metatiles and
//! Tileset_Options_Dialog. LGPL-3.0-only. Failed replacements preserve the session.
use super::*;
use polished_map_project::TilesetChoice;

pub(super) struct TilesetChoiceForm {
    choices: Vec<TilesetChoice>,
    selected: usize,
    pub(super) preview: choice_preview::Preview,
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

    pub(super) fn has_tileset_assets(&self) -> bool {
        self.loaded
            .as_ref()
            .is_some_and(|l| l.plus_assets.is_some() || l.tileset.is_some() && l.blockset.is_some())
    }

    fn tileset_assets_modified(&self) -> bool {
        let Some(loaded) = &self.loaded else {
            return false;
        };
        if let Some(assets) = &loaded.plus_assets {
            return assets.is_modified();
        }
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
        // Plusplus changes the global visibility option before the dirty guard.
        if let Some(a) = self.loaded.as_mut().and_then(|l| l.plus_assets.as_mut()) {
            a.allow_512 = self.allow_512;
        }
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
            .and_then(|l| l.tileset_location())
            .map(|(_, name)| name.to_owned());
        if let Some(name) = name
            && let Err(error) = self.change_loaded_tileset(ctx, &name)
        {
            self.error = Some(error);
        }
    }

    fn prepare_tileset_choice(&mut self) {
        let Some((root, name)) = self.loaded.as_ref().and_then(|l| l.tileset_location()) else {
            return;
        };
        match self
            .profile()
            .tileset_options(root, None)
            .map(|options| options.choices().to_vec())
        {
            Ok(choices) if !choices.is_empty() => {
                let selected = choices.iter().position(|c| c.name == name).unwrap_or(0);
                self.tileset_choice_form = Some(TilesetChoiceForm {
                    choices,
                    selected,
                    preview: Default::default(),
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
        let monochrome = self.monochrome_preference();
        let loaded = self.loaded.as_mut().ok_or("No map is loaded")?;
        if let Some(previous) = &loaded.plus_assets {
            let assets = previous
                .replacement(name, self.allow_512, self.bank_order)
                .map_err(|e| e.to_string())?;
            let graphics = assets.graphics(&assets.palettes);
            let textures = documents::block_textures(ctx, &graphics)?;
            self.allow_512 = assets.allow_512;
            loaded.plus_assets = Some(assets);
            loaded.graphics = graphics;
            self.textures = textures;
            self.finish_tileset_change(name);
            return Ok(());
        }
        let previous = loaded.tileset.as_ref().ok_or("No tileset is loaded")?;
        let (tileset, blocks) = previous
            .prepare_replacement_in_mode(
                name,
                polished_map_project::MasterTileMode {
                    monochrome,
                    allow_256: self.allow_256,
                },
            )
            .map_err(|e| e.to_string())?;
        let graphics: MapGraphics = tileset
            .graphics(blocks.blockset())
            .map_err(|e| e.to_string())?
            .into();
        let textures = documents::block_textures(ctx, &graphics)?;
        // Preserve all map IDs/history, even when some IDs no longer have definitions.
        self.allow_priority = tileset.allow_priority();
        self.monochrome = tileset.monochrome();
        loaded.allow_256 = tileset.allow_256();
        loaded.tileset = Some(tileset);
        loaded.blockset = Some(blocks);
        loaded.graphics = graphics;
        self.textures = textures;
        self.finish_tileset_change(name);
        Ok(())
    }

    fn finish_tileset_change(&mut self, name: &str) {
        let count = self.textures.len();
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
            // Bounded reviewed client: selector, scrollable preview, actions.
            let (area, _) =
                ui.allocate_exact_size(Vec2::new(280.0, choice_preview::HEIGHT), Sense::hover());
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
                        .height(choice_preview::MENU_HEIGHT)
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
            let preview_area = rect(10.0, 44.0, 260.0, 210.0);
            ui.scope_builder(egui::UiBuilder::new().max_rect(preview_area), |ui| {
                ui.set_clip_rect(preview_area.intersect(ui.clip_rect()));
                form.preview.show(
                    ui,
                    choice_preview::Request::new(
                        self,
                        Some(&form.choices[form.selected].name),
                        false,
                    ),
                );
            });
            let (ok, no) = if form.windows_buttons {
                (96.0, 190.0)
            } else {
                (190.0, 96.0)
            };
            accept = ui
                .put(
                    rect(ok, choice_preview::BUTTON_Y, 80.0, 22.0),
                    egui::Button::new("OK"),
                )
                .clicked();
            cancel = ui
                .put(
                    rect(no, choice_preview::BUTTON_Y, 80.0, 22.0),
                    egui::Button::new("Cancel"),
                )
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

#[cfg(test)]
mod review_tests {
    use super::*;

    #[test]
    fn long_tileset_list_scrolls_without_growing_the_dialog() {
        let (_dir, mut h) = crate::roof_editor_tests::fixture();
        h.state_mut().request_tileset_change();
        let form = h.state_mut().tileset_choice_form.as_mut().unwrap();
        let original = form.choices[0].clone();
        form.choices = (0..200)
            .map(|i| {
                let mut choice = original.clone();
                choice.name = format!("tile{i:03}");
                choice.label = choice.name.clone();
                choice
            })
            .collect();
        form.selected = 0;
        h.run();
        let bounds = h.state().tileset_choice_form.as_ref().unwrap().bounds;
        choice_preview::scroll_to_last_choice(&mut h, "tile000", "tile199", bounds);
        assert_eq!(
            h.state().tileset_choice_form.as_ref().unwrap().selected,
            199
        );
        assert_eq!(
            h.state().tileset_choice_form.as_ref().unwrap().bounds,
            bounds
        );
        assert_eq!(
            h.state()
                .loaded
                .as_ref()
                .unwrap()
                .tileset_location()
                .unwrap()
                .1,
            "demo"
        );
    }
}
