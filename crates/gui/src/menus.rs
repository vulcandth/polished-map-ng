//! Main_Window::initialize menu groups at pinned master. LGPL-3.0-only.
//! Unported actions/themes/preferences remain listed in docs/compatibility.md.
use super::*;

fn action(ui: &mut egui::Ui, enabled: bool, name: &str) -> bool {
    menu_popup::action(ui, enabled, name)
}

impl Editor {
    pub(super) fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::MenuBar::new()
            .config(menu_popup::config())
            .ui(ui, |ui| {
                menu_popup::menu(ui, "File", |ui| {
                    if action(ui, true, "New…") {
                        self.choose_new();
                    }
                    if action(ui, true, "Open…") {
                        self.choose_open();
                    }
                    self.recent_menu(ui);
                    let loaded = self.loaded.is_some();
                    if action(ui, loaded, "Close") {
                        self.request_close_map();
                    }
                    let loaded = self.loaded.is_some();
                    if action(ui, loaded, "Save") {
                        self.save();
                    }
                    if action(ui, loaded, "Save As…") {
                        self.save_as();
                    }
                    if action(ui, loaded, "Save Map") {
                        self.save_map();
                    }
                    let blocks = self.loaded.as_ref().is_some_and(|l| l.blockset.is_some());
                    if action(ui, blocks, "Save Blockset") {
                        self.save_blockset();
                    }
                    let tiles = self.loaded.as_ref().is_some_and(|l| l.tileset.is_some());
                    if action(ui, tiles, "Save Tileset") {
                        self.save_tileset();
                    }
                    let roof = self
                        .loaded
                        .as_ref()
                        .and_then(|l| l.tileset.as_ref())
                        .and_then(|t| t.roof())
                        .is_some();
                    if action(ui, roof, "Save Roof") {
                        self.save_roof();
                    }
                    if action(
                        ui,
                        loaded && self.print_worker.is_none() && self.print_request.is_none(),
                        "Print…",
                    ) {
                        self.prepare_print();
                    }
                    if action(ui, true, "Exit") {
                        self.request_exit(&ctx);
                    }
                });
                menu_popup::menu(ui, "Data", |ui| {
                    self.event_data_menu(ui);
                    if action(ui, true, "Load Palettes…") {
                        self.choose_palettes(&ctx);
                    }
                    if action(ui, true, "Export Palettes…") {
                        self.export_palettes();
                    }
                    if action(ui, self.can_load_roof_colors(), "Load Roof Colors") {
                        self.request_roof_colors(&ctx);
                    }
                });
                menu_popup::menu(ui, "Edit", |ui| {
                    let undo = self
                        .loaded
                        .as_ref()
                        .is_some_and(|l| l.document.map().can_undo());
                    if action(ui, undo, "Undo")
                        && let Some(loaded) = &mut self.loaded
                    {
                        loaded.document.map_mut().undo();
                    }
                    let redo = self
                        .loaded
                        .as_ref()
                        .is_some_and(|l| l.document.map().can_redo());
                    if action(ui, redo, "Redo")
                        && let Some(loaded) = &mut self.loaded
                    {
                        loaded.document.map_mut().redo();
                    }
                    self.block_menu(ui);
                    // Temporary explicit entry while the complete source block
                    // context-menu/gesture port is unfinished; see deviations.
                    let blocks = self.loaded.as_ref().is_some_and(|l| l.blockset.is_some());
                    if action(ui, blocks, "Edit Block…") {
                        self.prepare_block(&ctx);
                    }
                });
                menu_popup::menu(ui, "View", |ui| {
                    // Complete native Theme submenu is a separate parity item.
                    if menu_popup::checkbox(ui, true, &mut self.dark, "Dark appearance").changed() {
                        ctx.set_theme(if self.dark {
                            egui::Theme::Dark
                        } else {
                            egui::Theme::Light
                        });
                    }
                    menu_popup::checkbox(ui, true, &mut self.grid, "Grid");
                    menu_popup::checkbox(ui, true, &mut self.rulers, "Rulers");
                    menu_popup::checkbox(ui, true, &mut self.zoom, "Zoom");
                    menu_popup::checkbox(ui, true, &mut self.ids, "Block IDs");
                    menu_popup::checkbox(ui, true, &mut self.hex, "Hexadecimal");
                    menu_popup::checkbox(
                        ui,
                        self.allow_priority,
                        &mut self.show_priority,
                        "Show Priority",
                    );
                    menu_popup::checkbox(ui, true, &mut self.event_ui.game_boy, "Game Boy Screen");
                    menu_popup::checkbox(ui, true, &mut self.event_ui.show, "Show Events");
                    menu_popup::checkbox(ui, true, &mut self.event_ui.warp_ids, "Show Warp IDs");
                    menu_popup::submenu(ui, "Palettes", |ui| self.palette_set_menu(ui, true));
                    super::fullscreen::menu(ui);
                });
                menu_popup::menu(ui, "Mode", |ui| {
                    if menu_popup::radio(ui, !self.event_ui.mode, "Blocks").clicked() {
                        self.set_event_mode(false);
                        ui.close();
                    }
                    if menu_popup::radio(ui, self.event_ui.mode, "Events").clicked() {
                        self.set_event_mode(true);
                        ui.close();
                    }
                    if action(ui, true, "Switch Mode") {
                        self.set_event_mode(!self.event_ui.mode);
                    }
                });
                menu_popup::menu(ui, "Tools", |ui| {
                    let blocks = self.loaded.as_ref().is_some_and(|l| l.blockset.is_some());
                    if action(ui, blocks, "Resize Blockset…") {
                        self.prepare_block_count();
                    }
                    if action(ui, self.loaded.is_some(), "Resize Map…") {
                        self.prepare_resize();
                    }
                    let change = self
                        .loaded
                        .as_ref()
                        .is_some_and(|l| l.tileset.is_some() && l.blockset.is_some());
                    if action(ui, change, "Change Tileset…") {
                        self.request_tileset_change();
                    }
                    let tiles = self.loaded.as_ref().is_some_and(|l| l.tileset.is_some());
                    if action(ui, tiles, "Edit Tileset…") {
                        self.prepare_tileset();
                    }
                    if action(ui, change, "Change Roof…") {
                        self.request_roof_change();
                    }
                    let roof = self
                        .loaded
                        .as_ref()
                        .and_then(|l| l.tileset.as_ref())
                        .and_then(|t| t.roof())
                        .is_some();
                    if action(ui, roof, "Edit Roof…") {
                        self.prepare_roof();
                    }
                    if action(ui, true, "Edit Current Palettes…") {
                        self.prepare_palettes();
                    }
                });
                menu_popup::menu(ui, "Options", |ui| {
                    if menu_popup::checkbox(ui, true, &mut self.monochrome, "Monochrome").changed()
                    {
                        self.request_tile_mode_reload(&ctx);
                    }
                    if menu_popup::checkbox(ui, true, &mut self.allow_priority, "Tile Priority")
                        .changed()
                        && let Some(tileset) = self.loaded.as_mut().and_then(|l| l.tileset.as_mut())
                    {
                        tileset.set_allow_priority(self.allow_priority);
                    }
                    if menu_popup::checkbox(ui, true, &mut self.allow_256, "256 Tiles").changed() {
                        self.request_tile_mode_reload(&ctx);
                    }
                    self.roof_palette_menu(ui);
                    menu_popup::checkbox(
                        ui,
                        true,
                        &mut self.event_ui.auto_load,
                        "Auto-Load Events",
                    );
                    menu_popup::checkbox(
                        ui,
                        true,
                        &mut self.auto_special_palettes,
                        "Auto-Load Special Palettes",
                    );
                    menu_popup::checkbox(
                        ui,
                        true,
                        &mut self.auto_roof_colors,
                        "Auto-Load Roof Colors",
                    );
                    if menu_popup::checkbox(ui, true, &mut self.drag_and_drop, "Drag and Drop")
                        .changed()
                    {
                        self.sidebar_drag = None;
                    }
                    if action(ui, true, "Overworld Map Size…") {
                        self.prepare_overworld();
                    }
                });
                menu_popup::menu(ui, "Help", |ui| {
                    if action(ui, true, "Help") {
                        self.help_window.open();
                    }
                    if action(ui, true, "About") {
                        self.about = Some(Default::default());
                    }
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    #[test]
    fn source_menu_groups_keep_commands_in_order_and_guard_unavailable_data() {
        let mut h = Harness::builder()
            .with_size(Vec2::new(800.0, 600.0))
            .build_eframe(|cc| Editor::new(cc));
        let mut right = 0.0;
        for name in [
            "File", "Data", "Edit", "View", "Mode", "Tools", "Options", "Help",
        ] {
            let rect = h.get_by_label(name).rect();
            assert!(rect.left() >= right, "{name}");
            assert!(
                rect.top() >= 0.0 && rect.bottom() <= geometry::MENU_HEIGHT,
                "{name}"
            );
            right = rect.right();
        }
        for (menu, names) in [
            (
                "File",
                &[
                    "Close",
                    "Save",
                    "Save As…",
                    "Save Map",
                    "Save Blockset",
                    "Save Tileset",
                    "Save Roof",
                    "Print…",
                ][..],
            ),
            ("Edit", &["Undo", "Redo", "Edit Block…"][..]),
            (
                "Tools",
                &[
                    "Resize Blockset…",
                    "Resize Map…",
                    "Edit Tileset…",
                    "Edit Roof…",
                ][..],
            ),
        ] {
            h.get_by_label(menu).click();
            h.run();
            let mut bottom = 0.0;
            for name in names {
                let node = h.get_by_role_and_label(egui::Role::MenuItem, name);
                assert!(node.accesskit_node().is_disabled(), "{name}");
                assert!(node.rect().top() >= bottom, "{name}");
                bottom = node.rect().bottom();
            }
            if menu == "File" {
                assert!(h.query_by_label("Load Event Script…").is_none());
                assert!(
                    !h.get_by_role_and_label(egui::Role::MenuItem, "Exit")
                        .accesskit_node()
                        .is_disabled()
                );
            }
            if menu == "Edit" {
                assert!(h.query_by_label("Resize Blockset…").is_none());
            }
            if menu == "Tools" {
                assert!(
                    !h.get_by_label("Edit Current Palettes…")
                        .accesskit_node()
                        .is_disabled()
                );
                assert!(h.query_by_label("Auto-Load Events").is_none());
            }
            h.key_press(egui::Key::Escape);
            h.run();
        }
        h.get_by_label("Data").click();
        h.run();
        let mut bottom = 0.0;
        for name in [
            "Load Event Script…",
            "Save Event Script",
            "View Event Script",
            "Reload Event Script",
            "Unload Event Script",
            "Load Palettes…",
            "Export Palettes…",
            "Load Roof Colors",
        ] {
            let node = h.get_by_label(name);
            assert!(node.rect().top() >= bottom, "{name}");
            bottom = node.rect().bottom();
            assert_eq!(
                node.accesskit_node().is_disabled(),
                name.contains("Event Script") || name == "Load Roof Colors",
                "{name}"
            );
        }
        h.key_press(egui::Key::Escape);
        h.run();
        h.get_by_label("Options").click();
        h.run();
        h.get_by_label("Auto-Load Events").click();
        h.run();
        assert!(!h.state().event_ui.auto_load);
        assert!(!h.state().is_modified());
    }

    #[test]
    fn edit_history_and_mode_menu_actions_use_existing_state_transitions() {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        menu_popup::test_font_size(&h.ctx, 12.0);
        // A loaded map alone is insufficient: this fixture has no known group.
        h.get_by_label("Data").click();
        h.run();
        assert!(
            h.get_by_label("Load Roof Colors")
                .accesskit_node()
                .is_disabled()
        );
        h.key_press(egui::Key::Escape);
        h.run();
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        h.run();
        for (name, expected) in [("Undo", 0), ("Redo", 1)] {
            h.get_by_label("Edit").click();
            h.run();
            h.get_by_role_and_label(egui::Role::MenuItem, name).click();
            h.run();
            assert_eq!(
                h.state().loaded.as_ref().unwrap().document.map().blocks(),
                [expected, 1]
            );
        }
        for (name, mode) in [("Events", true), ("Blocks", false), ("Switch Mode", true)] {
            h.get_by_label("Mode").click();
            h.run();
            h.get_by_label(name).click();
            h.run();
            assert_eq!(h.state().event_ui.mode, mode);
            assert!(h.state().event_ui.drag.is_none());
            assert_eq!(
                h.state().loaded.as_ref().unwrap().document.map().blocks(),
                [1, 1]
            );
            assert!(h.state().loaded.as_ref().unwrap().document.map().can_undo());
        }
        h.get_by_label("Mode").click();
        h.run();
        assert_eq!(
            h.get_by_label("Events").accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        assert_eq!(
            h.get_by_label("Blocks").accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::False)
        );
        h.key_press(egui::Key::Escape);
        h.run();
        #[cfg(feature = "visual-tests")]
        for menu in [
            "File", "Data", "Edit", "View", "Mode", "Tools", "Options", "Help",
        ] {
            h.get_by_label(menu).click();
            h.remove_cursor();
            h.run();
            // Inspected CI 40801569788: these three popups differ only by
            // <=2/255 RGB at glyph/rounded-control edges (six pixels >1/255
            // in total). CI 40833843444 adds File/Options: max 2/255,
            // three pixels >1/255 at underlying toolbar text edges.
            // Keep the default threshold for every other menu.
            let options = if matches!(menu, "File" | "Data" | "View" | "Mode" | "Options") {
                egui_kittest::SnapshotOptions::default()
                    .threshold(egui_kittest::OsThreshold::<f32>::default().linux(3.0))
            } else {
                egui_kittest::SnapshotOptions::default()
            };
            h.snapshot_options(format!("main-menu-{}", menu.to_lowercase()), &options);
            h.key_press(egui::Key::Escape);
            h.run();
        }
    }
}
