//! master Main_Window::{load_roof_colors,load_roof_colors_cb,roof_*_cb}.
//! LGPL-3.0-only. Manual imports deliberately omit the automatic outside/mono guard.
use super::*;
use polished_map_project::{RoofPaletteLayout, roof_palette_path};

const LAYOUTS: [(RoofPaletteLayout, &str); 5] = [
    (RoofPaletteLayout::Custom, "Custom"),
    (RoofPaletteLayout::DayNite, "Morn + Day, Night"),
    (RoofPaletteLayout::MornDayNite, "Morn, Day, Night"),
    (
        RoofPaletteLayout::DayNiteCustom,
        "Morn + Day, Night, Custom",
    ),
    (
        RoofPaletteLayout::MornDayNiteCustom,
        "Morn, Day, Night, Custom",
    ),
];

impl Editor {
    pub(super) fn roof_palette_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Roof Palettes", |ui| {
            for (layout, label) in LAYOUTS {
                if ui
                    .radio_value(&mut self.roof_palette_layout, layout, label)
                    .clicked()
                {
                    ui.close();
                }
            }
        });
    }

    pub(super) fn can_load_roof_colors(&self) -> bool {
        self.loaded
            .as_ref()
            .is_some_and(|l| l.document.metadata().group != 0)
    }

    pub(super) fn request_roof_colors(&mut self, ctx: &egui::Context) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let metadata = loaded.document.metadata();
        if metadata.group == 0 {
            self.error = Some("The map's group is unknown!".into());
            return;
        }
        let Some(root) = &metadata.root else {
            self.error = Some("The map's project root is unknown!".into());
            return;
        };
        let path = roof_palette_path(root);
        if self.palette_document().is_modified() {
            self.pending_roof_colors = Some(path);
        } else {
            self.load_roof_colors_path(ctx, &path);
        }
    }

    fn load_roof_colors_path(&mut self, ctx: &egui::Context, path: &std::path::Path) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let group = loaded.document.metadata().group;
        let mut draft = self.palette_document().clone();
        let result = draft
            .load_roof_colors(path, group, self.roof_palette_layout)
            .map_err(|e| e.to_string())
            .and_then(|()| self.commit_palettes(ctx, draft));
        match result {
            Ok(()) => {
                let message = format!("Loaded roof colors for map group {group}!");
                self.status = message.clone();
                self.roof_color_success = Some(message);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn roof_color_dialog(&mut self, ctx: &egui::Context) {
        if let Some(message) = &self.roof_color_success {
            let mut done = false;
            egui::Modal::new(egui::Id::unique("roof-color-success")).show(ctx, |ui| {
                ui.heading("Success");
                ui.label(message);
                done = ui.button("OK").clicked();
                done |= ui.input_mut(|i| {
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                        || i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                });
            });
            if done {
                self.roof_color_success = None;
            }
            return;
        }
        let Some(path) = &self.pending_roof_colors else {
            return;
        };
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let message = format!("The palettes have been edited!\n\nLoad {name} anyway?");
        let mut accept = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::unique("roof-color-confirm")).show(ctx, |ui| {
            ui.heading("Warning");
            ui.label(message);
            ui.horizontal(|ui| {
                for label in if self.discard_windows_buttons {
                    ["OK", "Cancel"]
                } else {
                    ["Cancel", "OK"]
                } {
                    if ui.button(label).clicked() {
                        if label == "OK" {
                            accept = true;
                        } else {
                            cancel = true;
                        }
                    }
                }
            });
            accept |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        });
        if cancel {
            self.pending_roof_colors = None;
        } else if accept {
            let path = self
                .pending_roof_colors
                .take()
                .expect("pending roof color import");
            self.load_roof_colors_path(ctx, &path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };
    use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet, TileCommand};
    use std::{
        fs,
        time::{Duration, Instant},
    };

    fn fixture(
        monochrome: bool,
        environment: &str,
    ) -> (tempfile::TempDir, Harness<'static, Editor>) {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, monochrome);
        let root = dir.path();
        fs::create_dir_all(root.join("constants")).unwrap();
        fs::create_dir_all(root.join("data/maps")).unwrap();
        fs::write(root.join("Next.blk"), [0, 1]).unwrap();
        fs::write(
            root.join("constants/map_constants.asm"),
            "\tnewgroup\n\tmap_const NEXT, 2, 1\n",
        )
        .unwrap();
        fs::write(
            root.join("data/maps/maps.asm"),
            format!(
                "map Next, TILESET_DEMO, {environment}, LANDMARK_NEXT, MUSIC, FALSE, PALETTE_AUTO\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("gfx/tilesets/demo_palette_map.asm"),
            "\ttilepal 0, ROOF, ROOF, ROOF\n",
        )
        .unwrap();
        fs::write(
            root.join("gfx/tilesets/roofs.pal"),
            (0..32)
                .map(|v| format!("\tRGB {v},{v},{v}\n"))
                .collect::<String>(),
        )
        .unwrap();
        let loaded = h.state_mut().loaded.as_mut().unwrap();
        loaded.document = MapDocument::open_in_project(
            &root.join("Next.blk"),
            &OpenOptions::default(),
            Some(root),
        )
        .unwrap();
        loaded.tileset = Some(
            MasterTilesetDocument::open_in_project(
                root,
                "demo",
                &MasterGraphicsOptions {
                    monochrome,
                    ..Default::default()
                },
            )
            .unwrap(),
        );
        let palettes = h.state().palette_document().clone();
        let ctx = h.ctx.clone();
        h.state_mut().commit_palettes(&ctx, palettes).unwrap();
        h.state_mut().status = "Synthetic roof colors".into();
        h.run();
        (dir, h)
    }

    fn invoke(h: &mut Harness<'_, Editor>) {
        h.get_by_label("Data").click();
        h.run();
        let node = h.get_by_label("Load Roof Colors");
        assert!(!node.accesskit_node().is_disabled());
        node.click();
        h.run();
    }

    #[test]
    fn manual_roof_colors_load_in_indoor_and_monochrome_maps_without_dirtying_documents() {
        for monochrome in [false, true] {
            let (dir, mut h) = fixture(monochrome, "INDOOR");
            let table = fs::read(dir.path().join("gfx/tilesets/roofs.pal")).unwrap();
            let before = h
                .state()
                .loaded
                .as_ref()
                .unwrap()
                .graphics
                .render_block(0)
                .unwrap()
                .pixels;
            invoke(&mut h);
            assert!(h.state().error.is_none());
            assert!(h.state().pending_roof_colors.is_none());
            assert_eq!(
                h.state().roof_color_success.as_deref(),
                Some("Loaded roof colors for map group 1!")
            );
            h.get_by_label("Success");
            let palettes = h.state().palette_document();
            assert_eq!(palettes.bank().colors(PaletteSet::Day)[6][2], [33; 3]);
            assert_eq!(palettes.bank().colors(PaletteSet::Day)[6][1], [41; 3]);
            assert!(!palettes.is_modified());
            assert!(!h.state().is_modified());
            assert!(!h.state().loaded.as_ref().unwrap().document.map().can_undo());
            let after = h
                .state()
                .loaded
                .as_ref()
                .unwrap()
                .graphics
                .render_block(0)
                .unwrap()
                .pixels;
            if monochrome {
                assert_eq!(after, before);
            } else {
                assert_ne!(after, before);
            }
            // The success modal also guards document shortcuts.
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
            h.run();
            assert!(h.state().loaded.is_some());
            h.key_press(egui::Key::Escape);
            h.run();
            assert!(h.state().roof_color_success.is_none());
            assert_eq!(
                fs::read(dir.path().join("gfx/tilesets/roofs.pal")).unwrap(),
                table
            );
            assert_eq!(fs::read(dir.path().join("Next.blk")).unwrap(), [0, 1]);
        }
    }

    #[test]
    fn dirty_palette_consent_preserves_edits_and_history_on_cancel_error_and_accept() {
        let (dir, mut h) = fixture(false, "TOWN");
        let ctx = h.ctx.clone();
        let mut palettes = h.state().palette_document().clone();
        palettes
            .apply_batch(&[PaletteCommand::SetRgb {
                slot: ColorSlot {
                    set: PaletteSet::Day,
                    palette: 0,
                    hue: 0,
                },
                rgb: [17, 34, 51],
            }])
            .unwrap();
        h.state_mut().commit_palettes(&ctx, palettes).unwrap();
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        h.state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .tileset
            .as_mut()
            .unwrap()
            .apply_batch(&[TileCommand::Paint {
                tile: 0,
                points: vec![Point { x: 0, y: 0 }],
                hue: 3,
            }])
            .unwrap();
        let before = h.state().palette_document().content_hash();
        h.state_mut().discard_windows_buttons = true;
        invoke(&mut h);
        h.get_by_label("The palettes have been edited!\n\nLoad roofs.pal anyway?");
        #[cfg(feature = "visual-tests")]
        {
            h.remove_cursor();
            h.run();
            h.snapshot("roof-colors-confirm");
        }
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
        h.run();
        assert!(!h.state().confirm_discard && h.state().loaded.is_some());
        h.get_by_label("Cancel").click();
        h.run();
        assert!(h.state().pending_roof_colors.is_none());
        assert_eq!(h.state().palette_document().content_hash(), before);
        // Invalid preferred table is an error, not a partial import or legacy fallback.
        let path = dir.path().join("gfx/tilesets/roofs.pal");
        let table = fs::read(&path).unwrap();
        fs::write(&path, "\tRGB 32,0,0\n").unwrap();
        invoke(&mut h);
        h.key_press(egui::Key::Enter);
        h.run();
        assert!(h.state().error.is_some() && h.state().pending_roof_colors.is_none());
        assert_eq!(h.state().palette_document().content_hash(), before);
        h.get_by_label("OK").click();
        h.run();
        fs::write(&path, &table).unwrap();
        invoke(&mut h);
        h.key_press(egui::Key::Enter);
        h.run();
        assert!(h.state().error.is_none());
        let palettes = h.state().palette_document();
        assert!(palettes.is_modified());
        assert_eq!(palettes.selected(), PaletteSet::Day);
        assert_eq!(palettes.bank().colors(PaletteSet::Day)[0][0], [17, 34, 51]);
        assert_eq!(palettes.bank().colors(PaletteSet::Day)[6][2], [33; 3]);
        let loaded = h.state().loaded.as_ref().unwrap();
        assert!(loaded.document.map().is_modified() && loaded.document.map().can_undo());
        assert!(loaded.tileset.as_ref().unwrap().tileset().is_modified());
        assert!(!loaded.blockset.as_ref().unwrap().blockset().is_modified());
        #[cfg(feature = "visual-tests")]
        {
            h.remove_cursor();
            h.run();
            h.snapshot("roof-colors-success");
        }
        h.get_by_label("OK").click();
        h.run();
        assert_eq!(fs::read(&path).unwrap(), table);
        assert_eq!(fs::read(dir.path().join("Next.blk")).unwrap(), [0, 1]);
    }

    #[test]
    fn roof_layout_choices_defer_recoloring_and_control_manual_and_next_open_imports() {
        let (dir, mut h) = fixture(false, "TOWN");
        let before = h.state().palette_document().content_hash();
        for (layout, label) in LAYOUTS {
            h.get_by_label("Options").click();
            h.run();
            h.get_by_label("Roof Palettes").click();
            h.run();
            h.get_by_label(label).click();
            h.run();
            assert_eq!(h.state().roof_palette_layout, layout);
            assert_eq!(h.state().palette_document().content_hash(), before);
            assert!(!h.state().is_modified());
            // Closing the popup keeps the next iteration independent of submenu policy.
            h.key_press(egui::Key::Escape);
            h.run();
        }
        #[cfg(feature = "visual-tests")]
        {
            h.get_by_label("Options").click();
            h.run();
            h.get_by_label("Roof Palettes").click();
            h.run();
            h.remove_cursor();
            h.run();
            h.snapshot("roof-palette-layouts");
            h.key_press(egui::Key::Escape);
            h.run();
        }
        invoke(&mut h);
        assert_eq!(
            h.state()
                .palette_document()
                .bank()
                .colors(PaletteSet::Custom)[6][2],
            [115; 3]
        );
        h.get_by_label("OK").click();
        h.run();
        h.state_mut().close_map();
        assert_eq!(
            h.state().roof_palette_layout,
            RoofPaletteLayout::MornDayNiteCustom
        );
        let ctx = h.ctx.clone();
        h.state_mut().start_load(
            OpenForm {
                path: dir.path().join("Next.blk"),
                root: dir.path().to_string_lossy().into_owned(),
                tileset: "demo".into(),
                width: 2,
                height: 1,
                ..Default::default()
            },
            &ctx,
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        while h.state().loading.is_some() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
            h.step();
        }
        assert!(h.state().loading.is_none() && h.state().error.is_none());
        h.run();
        assert_eq!(
            h.state()
                .palette_document()
                .bank()
                .colors(PaletteSet::Custom)[6][2],
            [115; 3]
        );
        assert_eq!(
            h.state().palette_document().bank().colors(PaletteSet::Day)[6][2],
            [82; 3]
        );
        assert!(h.state().loaded.is_some() && h.state().pending_open.is_none());
    }
}
