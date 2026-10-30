//! master Overworld_Map_Size_Dialog and Main_Window::save_map. LGPL-3.0-only.
use super::*;

const DESCRIPTION: &str = "The overworld map block buffer in WRAM stores the entire\n\
current map, padded by 3 blocks on each side.\n\n\
Its standard label in wram.asm is wOverworldMapBlocks,\n\
wOverworldMap, or OverworldMap.\n\n\
Its default size is 1300 bytes, so every map requires that\n\
(width + 6) × (height + 6) ≤ 1300.";

pub(super) struct OverworldForm {
    size: u16,
    default: default_value::DefaultValue<u16>,
    windows_buttons: bool,
    #[cfg(test)]
    bounds: Rect,
}

impl Editor {
    pub(super) fn prepare_overworld(&mut self) {
        self.overworld_form = Some(OverworldForm {
            size: self.overworld_map_size,
            default: default_value::DefaultValue::new(self.overworld_map_size),
            windows_buttons: cfg!(target_os = "windows"),
            #[cfg(test)]
            bounds: Rect::NOTHING,
        });
    }

    /// Called after a successful dirty-map write, or any successful Save As.
    /// A capacity warning never rejects or rolls back that completed save.
    pub(super) fn check_overworld_buffer(&mut self) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let map = loaded.document.map();
        let bytes = map.overworld_buffer_bytes();
        if bytes > u32::from(self.overworld_map_size) {
            self.overworld_warning = Some(format!(
                "A {} x {} map will overflow a {}-byte buffer!\n\n\
Make sure the overworld map block buffer in WRAM\n\
(wOverworldMapBlocks, wOverworldMap, or OverworldMap)\n\
has at least {bytes} bytes.",
                map.width(),
                map.height(),
                self.overworld_map_size,
            ));
        }
    }

    pub(super) fn overworld_dialog(&mut self, ctx: &egui::Context) {
        if let Some(message) = &self.overworld_warning {
            if message_dialog::show(
                ctx,
                "overworld-warning",
                message_dialog::Kind::Warning,
                message,
            ) {
                self.overworld_warning = None;
            }
            return;
        }
        let Some(mut form) = self.overworld_form.take() else {
            return;
        };
        let (mut accept, mut cancel) = (false, false);
        egui::Modal::new(egui::Id::unique("overworld-map-size")).show(ctx, |ui| {
            ui.heading("Overworld Map Size");
            let font = egui::FontId::proportional(13.0);
            let color = ui.visuals().text_color();
            let measured = ui
                .painter()
                .layout_no_wrap(DESCRIPTION.into(), font.clone(), color);
            // Option_Dialog::refresh + refresh_content: text metrics, 22px
            // controls, 10px margins and 16px gap before the bottom buttons.
            let description_height = measured.size().y + 22.0 - font.size;
            let buttons_y = 10.0 + description_height + 44.0 + 16.0;
            let (area, _) =
                ui.allocate_exact_size(Vec2::new(380.0, buttons_y + 32.0), Sense::hover());
            #[cfg(test)]
            {
                form.bounds = area;
            }
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(rect(10.0, 10.0, 360.0, description_height))
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
                |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(DESCRIPTION).font(font.clone()))
                            .halign(egui::Align::Min)
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                },
            );
            let label_font = egui::FontId::monospace(13.0);
            let label_width = ui
                .painter()
                .layout_no_wrap("ds:".into(), label_font.clone(), color)
                .size()
                .x
                + 4.0;
            let spinner_width = ui
                .painter()
                .layout_no_wrap("9999".into(), font, color)
                .size()
                .x
                + 4.0
                + 22.0;
            let spinner_y = 10.0 + description_height + 22.0;
            ui.put(
                rect(10.0, spinner_y, label_width, 22.0),
                egui::Label::new(egui::RichText::new("ds:").font(label_font)),
            );
            let response = ui.put(
                rect(10.0 + label_width, spinner_y, spinner_width, 22.0),
                numeric_input::IntegerInput::new(&mut form.size, 1..=4096),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::SpinButton, true, "Overworld map buffer size")
            });
            form.default.restore(ui, &response, &mut form.size);
            let (ok, no) = if form.windows_buttons {
                (196.0, 290.0)
            } else {
                (290.0, 196.0)
            };
            accept = ui
                .put(rect(ok, buttons_y, 80.0, 22.0), egui::Button::new("OK"))
                .clicked();
            cancel = ui
                .put(rect(no, buttons_y, 80.0, 22.0), egui::Button::new("Cancel"))
                .clicked();
            accept |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        });
        if cancel {
            return;
        }
        if accept {
            self.overworld_map_size = form.size;
        } else {
            self.overworld_form = Some(form);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    use std::fs;

    fn open(h: &mut Harness<'_, Editor>) {
        h.get_by_label("Options").click();
        h.run();
        h.get_by_role_and_label(egui::Role::MenuItem, "Overworld Map Size…")
            .click();
        h.run();
    }

    #[test]
    fn size_dialog_geometry_range_cancel_accept_and_modal_guards() {
        #[cfg(feature = "visual-tests")]
        let mut snapshots = egui_kittest::SnapshotResults::new();
        for windows in [false, true] {
            let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
            assert_eq!(h.state().overworld_map_size, 1300);
            open(&mut h);
            h.state_mut()
                .overworld_form
                .as_mut()
                .unwrap()
                .windows_buttons = windows;
            h.remove_cursor();
            h.run();
            let area = h.state().overworld_form.as_ref().unwrap().bounds;
            assert_eq!(area.width(), 380.0);
            assert_eq!(
                h.get_by_label(DESCRIPTION).rect().left() - area.left(),
                10.0
            );
            let ok = h.get_by_label("OK").rect();
            let cancel = h.get_by_label("Cancel").rect();
            assert_eq!(ok.size(), Vec2::new(80.0, 22.0));
            assert_eq!(cancel.size(), Vec2::new(80.0, 22.0));
            assert_eq!(ok.left() - area.left(), if windows { 196.0 } else { 290.0 });
            assert_eq!(area.bottom() - ok.bottom(), 10.0);
            assert_eq!(cancel.top(), ok.top());
            let spin = h.get_by_label("Overworld map buffer size").rect();
            assert_eq!(spin.height(), 22.0);
            assert_eq!(ok.top() - spin.bottom(), 16.0);
            #[cfg(feature = "visual-tests")]
            snapshots.add(h.try_snapshot(if windows {
                "overworld-size-windows"
            } else {
                "overworld-size-unix"
            }));
            h.get_by_label("Overworld map buffer size")
                .set_value_accesskit(4097.0);
            h.run();
            assert_eq!(h.state().overworld_form.as_ref().unwrap().size, 4096);
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
            h.run();
            assert!(h.state().loaded.is_some());
            h.key_press(egui::Key::Escape);
            h.run();
            assert!(h.state().overworld_form.is_none());
            assert_eq!(h.state().overworld_map_size, 1300);
            open(&mut h);
            h.get_by_label("Overworld map buffer size")
                .set_value_accesskit(0.0);
            h.run();
            assert_eq!(h.state().overworld_form.as_ref().unwrap().size, 1);
            h.key_press(egui::Key::Enter);
            h.run();
            assert!(h.state().overworld_form.is_none());
            assert_eq!(h.state().overworld_map_size, 1);
            assert!(!h.state().is_modified());
            assert!(
                h.state().overworld_warning.is_none(),
                "changing capacity does not warn until a write"
            );
            h.state_mut().close_map();
            open(&mut h);
            assert_eq!(h.state().overworld_form.as_ref().unwrap().size, 1);
            h.get_by_label("Overworld map buffer size")
                .set_value_accesskit(4096.0);
            h.run();
            h.get_by_label("OK").click();
            h.run();
            assert_eq!(h.state().overworld_map_size, 4096);
        }
        #[cfg(feature = "visual-tests")]
        snapshots.unwrap();
    }

    #[test]
    fn capacity_warning_follows_successful_writes_only_and_preserves_saved_bytes_and_history() {
        for limit in [55, 56, 57] {
            let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
            h.state_mut().overworld_map_size = limit;
            h.state_mut().apply(Command::Paint {
                points: vec![Point { x: 0, y: 0 }],
                block: 1,
            });
            h.get_by_label("File").click();
            h.run();
            h.get_by_role_and_label(egui::Role::MenuItem, "Save Map")
                .click();
            h.run();
            assert!(h.state().error.is_none());
            assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [1, 1]);
            assert!(!h.state().is_modified());
            assert!(h.state().loaded.as_ref().unwrap().document.map().can_undo());
            assert_eq!(h.state().overworld_warning.is_some(), limit == 55);
            if limit == 55 {
                let expected = "A 2 x 1 map will overflow a 55-byte buffer!\n\nMake sure the overworld map block buffer in WRAM\n(wOverworldMapBlocks, wOverworldMap, or OverworldMap)\nhas at least 56 bytes.";
                assert_eq!(h.state().overworld_warning.as_deref(), Some(expected));
                h.get_by_label(expected);
                #[cfg(feature = "visual-tests")]
                {
                    h.remove_cursor();
                    h.run();
                    h.snapshot("overworld-save-warning");
                }
                h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
                h.run();
                assert_eq!(
                    h.state().loaded.as_ref().unwrap().document.map().blocks(),
                    &[1, 1]
                );
                h.key_press(egui::Key::Escape);
                h.run();
                assert!(h.state().overworld_warning.is_none());
            }
            crate::save_feedback::tests::dismiss(&mut h);
            // A clean ordinary Save does not trigger the warning again.
            h.state_mut().overworld_map_size = 1;
            h.state_mut().save_map();
            assert!(h.state().overworld_warning.is_none());
            crate::save_feedback::tests::dismiss(&mut h);
            // Save As always writes, even for a clean map.
            let copy = dir.path().join("Copy.blk");
            h.state_mut().save_as_path(copy.clone());
            h.run();
            assert!(h.state().overworld_warning.is_some());
            assert_eq!(fs::read(copy).unwrap(), [1, 1]);
            h.key_press(egui::Key::Enter);
            h.run();
            assert!(h.state().overworld_warning.is_none());
            crate::save_feedback::tests::dismiss(&mut h);
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
            h.run();
            assert_eq!(
                h.state().loaded.as_ref().unwrap().document.map().blocks(),
                &[0, 1]
            );
        }
    }

    #[test]
    fn failed_map_save_and_asset_only_save_do_not_emit_capacity_warning() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        h.state_mut().overworld_map_size = 1;
        h.state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .tileset
            .as_mut()
            .unwrap()
            .apply_batch(&[polished_map_core::TileCommand::Paint {
                tile: 0,
                points: vec![Point { x: 0, y: 0 }],
                hue: 3,
            }])
            .unwrap();
        h.state_mut().save();
        assert!(h.state().error.is_none() && h.state().overworld_warning.is_none());
        assert!(!h.state().is_modified());
        crate::save_feedback::tests::dismiss(&mut h);
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        let path = dir.path().join("Map.2x1.blk");
        fs::write(&path, [0, 0]).unwrap();
        h.state_mut().save_map();
        assert!(h.state().error.is_some() && h.state().overworld_warning.is_none());
        assert!(h.state().is_modified());
        assert_eq!(fs::read(path).unwrap(), [0, 0]);
        h.state_mut().error = None;
        h.state_mut().save_as_path(dir.path().join("Bad.map"));
        assert!(h.state().error.is_some() && h.state().overworld_warning.is_none());
        assert!(h.state().is_modified());
    }
}
