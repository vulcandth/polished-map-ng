//! Implemented preferences from master Main_Window constructor/exit_cb.
//! LGPL-3.0-only. eframe owns platform storage and native window placement.
use super::*;
use polished_map_core::PaletteSet;
use polished_map_project::RoofPaletteLayout;
use serde::{Deserialize, Serialize};

const KEY: &str = "polished-map-ng.preferences";
const MAX_BYTES: usize = 4 * 1024 * 1024;
pub(super) mod legacy;

/// Versioned app settings, deliberately excluding documents, colors and UI drafts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Preferences {
    version: u32,
    dark: bool,
    transparent: bool,
    event_mode: bool,
    grid: bool,
    rulers: bool,
    zoom: bool,
    ids: bool,
    hex: bool,
    game_boy: bool,
    show_events: bool,
    show_warp_ids: bool,
    palette_set: PaletteSet,
    roof_layout: RoofPaletteLayout,
    auto_events: bool,
    auto_special_palettes: bool,
    auto_roof_colors: bool,
    allow_priority: bool,
    monochrome: bool,
    allow_256: bool,
    drag_and_drop: bool,
    overworld_map_size: u16,
    meta128: bool,
    show_priority: bool,
    print_options: polished_map_core::PrintOptions,
    resize_anchor: ResizeAnchor,
    recent_maps: recent_maps::RecentMaps,
}

impl Default for Preferences {
    fn default() -> Self {
        Self::capture(&Editor::default())
    }
}

impl Preferences {
    fn capture(editor: &Editor) -> Self {
        Self {
            version: 1,
            dark: editor.dark,
            transparent: editor.transparency.preferred(),
            event_mode: editor.event_ui.mode,
            grid: editor.grid,
            rulers: editor.rulers,
            zoom: editor.zoom,
            ids: editor.ids,
            hex: editor.hex,
            game_boy: editor.event_ui.game_boy,
            show_events: editor.event_ui.show,
            show_warp_ids: editor.event_ui.warp_ids,
            palette_set: editor.palette_document().selected(),
            roof_layout: editor.roof_palette_layout,
            auto_events: editor.event_ui.auto_load,
            auto_special_palettes: editor.auto_special_palettes,
            auto_roof_colors: editor.auto_roof_colors,
            allow_priority: editor.allow_priority,
            monochrome: editor.monochrome,
            allow_256: editor.allow_256,
            drag_and_drop: editor.drag_and_drop,
            overworld_map_size: editor.overworld_map_size,
            meta128: editor.meta128,
            show_priority: editor.show_priority,
            print_options: editor.print_options,
            resize_anchor: editor.resize_anchor,
            recent_maps: editor.recent_maps.clone(),
        }
    }

    fn apply(self, editor: &mut Editor) {
        editor.dark = self.dark;
        editor.transparency.restore(self.transparent);
        editor.event_ui.mode = self.event_mode;
        editor.grid = self.grid;
        editor.rulers = self.rulers;
        editor.zoom = self.zoom;
        editor.ids = self.ids;
        editor.hex = self.hex;
        editor.event_ui.game_boy = self.game_boy;
        editor.event_ui.show = self.show_events;
        editor.event_ui.warp_ids = self.show_warp_ids;
        editor.unloaded_palettes.select(self.palette_set);
        editor.roof_palette_layout = self.roof_layout;
        editor.event_ui.auto_load = self.auto_events;
        editor.auto_special_palettes = self.auto_special_palettes;
        editor.auto_roof_colors = self.auto_roof_colors;
        editor.allow_priority = self.allow_priority;
        editor.monochrome = self.monochrome;
        editor.allow_256 = self.allow_256;
        editor.drag_and_drop = self.drag_and_drop;
        editor.overworld_map_size = self.overworld_map_size;
        editor.meta128 = self.meta128;
        editor.show_priority = self.show_priority;
        editor.print_options = self.print_options;
        editor.resize_anchor = self.resize_anchor;
        editor.recent_maps = self.recent_maps;
    }
}

impl Editor {
    pub(super) fn restore_preferences(&mut self, storage: Option<&dyn eframe::Storage>) {
        let Some(raw) = storage.and_then(|storage| storage.get_string(KEY)) else {
            return;
        };
        let decoded = (raw.len() <= MAX_BYTES)
            .then(|| serde_json::from_str::<Preferences>(&raw).ok())
            .flatten()
            .filter(|settings| {
                settings.version == 1 && (1..=4096).contains(&settings.overworld_map_size)
            });
        if let Some(settings) = decoded {
            settings.apply(self);
        } else {
            // A downgrade or malformed record must not overwrite the user's record.
            self.preferences_writable = false;
            self.status = "Preferences could not be restored; using defaults and preserving the saved record.".into();
        }
    }

    pub(super) fn store_preferences(&mut self, storage: &mut dyn eframe::Storage) {
        if !self.preferences_writable {
            return;
        }
        match serde_json::to_string(&Preferences::capture(self)) {
            Ok(raw) if raw.len() <= MAX_BYTES => storage.set_string(KEY, raw),
            Ok(_) => {
                self.status =
                    "Preferences exceed the storage limit; preserving the saved record.".into()
            }
            Err(error) => self.status = format!("Could not encode preferences: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;
    use polished_map_core::{ColorSlot, PaletteCommand};
    use std::collections::BTreeMap;

    #[derive(Default)]
    pub(super) struct MemoryStorage {
        values: BTreeMap<String, String>,
        writes: usize,
    }
    impl eframe::Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.values.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.values.insert(key.into(), value);
            self.writes += 1;
        }
        fn remove_string(&mut self, key: &str) {
            self.values.remove(key);
        }
        fn flush(&mut self) {}
    }
    fn restore(storage: &dyn eframe::Storage) -> Harness<'_, Editor> {
        Harness::builder()
            .with_size(Vec2::new(800.0, 600.0))
            .build_eframe(|cc| {
                cc.storage = Some(storage);
                Editor::new(cc)
            })
    }

    #[test]
    fn theme_radio_choices_reuse_v1_preferences_and_restore_the_selected_theme() {
        use egui_kittest::kittest::{NodeT, Queryable};
        let mut storage = MemoryStorage::default();
        for (name, dark) in [("Dark", true), ("Greybird", false)] {
            let mut h = restore(&storage);
            h.get_by_label("View").click();
            h.run();
            h.get_by_label("Theme").click();
            h.run();
            h.get_by_role_and_label(egui::Role::RadioButton, name)
                .click();
            h.run();
            let mut editor = std::mem::take(h.state_mut());
            drop(h);
            eframe::App::save(&mut editor, &mut storage);
            let value: serde_json::Value = serde_json::from_str(&storage.values[KEY]).unwrap();
            assert_eq!(value["version"], 1);
            assert_eq!(value["dark"], dark);
            let theme = if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            };
            // Harness applies its theme after CreationContext. Check the real
            // startup choice inside the factory, then use the same test theme.
            let mut h = Harness::builder()
                .with_size(Vec2::new(800.0, 600.0))
                .with_theme(theme)
                .build_eframe(|cc| {
                    cc.storage = Some(&storage);
                    let editor = Editor::new(cc);
                    assert_eq!(cc.egui_ctx.theme(), theme);
                    editor
                });
            assert_eq!(h.state().dark, dark);
            assert_eq!(
                h.ctx.theme(),
                if dark {
                    egui::Theme::Dark
                } else {
                    egui::Theme::Light
                }
            );
            h.get_by_label("View").click();
            h.run();
            h.get_by_label("Theme").click();
            h.run();
            assert_eq!(
                h.get_by_role_and_label(egui::Role::RadioButton, name)
                    .accesskit_node()
                    .toggled(),
                Some(egui::accesskit::Toggled::True)
            );
        }
    }

    #[test]
    fn shutdown_settings_restore_at_startup_without_documents_colors_or_transient_state() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::Num7);
        h.run();
        assert_eq!(h.state().block_hotkeys.digit(0), Some(7));
        let editor = h.state_mut();
        editor.dark = true;
        editor.event_ui.mode = true;
        editor.grid = false;
        editor.rulers = true;
        editor.zoom = true;
        editor.ids = true;
        editor.hex = true;
        editor.event_ui.game_boy = true;
        editor.event_ui.show = false;
        editor.event_ui.warp_ids = false;
        editor.event_ui.auto_load = false;
        editor.auto_special_palettes = false;
        editor.auto_roof_colors = false;
        editor.show_priority = false;
        editor.monochrome = true;
        editor.allow_256 = true;
        editor.drag_and_drop = false;
        editor.overworld_map_size = 4096;
        editor.print_options = polished_map_core::PrintOptions {
            grid: true,
            ids: true,
            priority: true,
            events: true,
            warp_ids: true,
        };
        editor.roof_palette_layout = RoofPaletteLayout::MornDayNiteCustom;
        editor.resize_anchor = ResizeAnchor::BottomRight;
        let palettes = editor
            .loaded
            .as_mut()
            .unwrap()
            .tileset
            .as_mut()
            .unwrap()
            .palettes_mut();
        palettes
            .apply_batch(&[PaletteCommand::SetRgb {
                slot: ColorSlot {
                    set: PaletteSet::Custom,
                    palette: 0,
                    hue: 0,
                },
                rgb: [17, 34, 51],
            }])
            .unwrap();
        palettes.select(PaletteSet::Custom);
        editor.copy_block();
        editor.apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        editor.canvas_scroll = Vec2::new(250.0, 90.0);
        editor.prepare_resize();
        let mut storage = MemoryStorage::default();
        eframe::App::save(editor, &mut storage);
        assert_eq!(storage.writes, 1);
        let value: serde_json::Value = serde_json::from_str(&storage.values[KEY]).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "version": 1, "dark": true, "transparent": false, "event_mode": true, "grid": false,
                "rulers": true, "zoom": true, "ids": true, "hex": true,
                "game_boy": true, "show_events": false, "show_warp_ids": false,
                "palette_set": "custom", "roof_layout": "morn_day_nite_custom",
                "auto_events": false, "auto_special_palettes": false, "auto_roof_colors": false,
                "allow_priority": true,
                "monochrome": true, "allow_256": true,
                "drag_and_drop": false,
                "overworld_map_size": 4096, "meta128": false,
                "show_priority": false,
                "print_options": {"grid":true,"ids":true,"priority":true,"events":true,"warp_ids":true},
                "resize_anchor": "bottom_right",
                "recent_maps": [{"utf8": std::fs::canonicalize(dir.path().join("Map.2x1.blk")).unwrap()}],
            })
        );
        let restored = restore(&storage);
        let editor = restored.state();
        assert_eq!(restored.ctx.theme(), egui::Theme::Dark);
        assert_eq!(
            Preferences::capture(editor),
            Preferences::capture(h.state())
        );
        assert!(editor.loaded.is_none() && editor.block_clipboard.is_none());
        assert_eq!(editor.block_hotkeys.digit(0), None);
        assert_eq!(editor.sidebar_reveal, None);
        assert!(editor.resize_form.is_none() && editor.textures.is_empty());
        assert_eq!(editor.canvas_scroll, Vec2::ZERO);
        assert!(!editor.is_modified() && !editor.palette_document().is_modified());
        assert_eq!(
            editor.palette_document().bank(),
            &polished_map_core::PaletteBank::default()
        );
        assert_eq!(storage.writes, 1, "startup does not write preferences");
        assert_eq!(
            std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
            [0, 1]
        );
        assert_eq!(
            eframe::App::auto_save_interval(editor),
            std::time::Duration::MAX
        );
        assert!(!eframe::App::persist_egui_memory(editor));
    }

    #[test]
    fn missing_fields_use_source_defaults_and_all_saved_enum_values_round_trip() {
        let mut storage = MemoryStorage::default();
        storage
            .values
            .insert(KEY.into(), r#"{"version":1,"hex":true}"#.into());
        let restored = restore(&storage);
        let editor = restored.state();
        assert!(editor.hex && editor.grid);
        assert!(!editor.dark && !editor.rulers && !editor.zoom && !editor.ids);
        assert!(!editor.event_ui.mode && !editor.event_ui.game_boy);
        assert!(editor.event_ui.show && editor.event_ui.warp_ids && editor.event_ui.auto_load);
        assert!(editor.auto_special_palettes && editor.auto_roof_colors);
        assert!(!editor.allow_priority);
        assert!(!editor.monochrome && !editor.allow_256);
        assert!(editor.drag_and_drop);
        assert_eq!(editor.overworld_map_size, 1300);
        assert!(editor.show_priority);
        assert_eq!(
            editor.print_options,
            polished_map_core::PrintOptions::default()
        );
        assert_eq!(editor.palette_document().selected(), PaletteSet::Day);
        assert_eq!(editor.roof_palette_layout, RoofPaletteLayout::DayNite);
        assert_eq!(editor.resize_anchor, ResizeAnchor::Center);
        drop(restored);
        for layout in [
            RoofPaletteLayout::Custom,
            RoofPaletteLayout::DayNite,
            RoofPaletteLayout::MornDayNite,
            RoofPaletteLayout::DayNiteCustom,
            RoofPaletteLayout::MornDayNiteCustom,
        ] {
            for set in PaletteSet::ALL {
                for anchor in ResizeAnchor::ALL {
                    let mut editor = Editor {
                        roof_palette_layout: layout,
                        resize_anchor: anchor,
                        ..Default::default()
                    };
                    editor.unloaded_palettes.select(set);
                    eframe::App::save(&mut editor, &mut storage);
                    let mut next = Editor::default();
                    next.restore_preferences(Some(&storage));
                    assert_eq!(next.roof_palette_layout, layout);
                    assert_eq!(next.resize_anchor, anchor);
                    assert_eq!(next.palette_document().selected(), set);
                }
            }
        }
    }

    #[test]
    fn malformed_oversized_and_future_records_are_preserved_instead_of_overwritten() {
        for raw in [
            "{".into(),
            r#"{"version":2,"hex":true}"#.into(),
            r#"{"version":1,"grid":"yes"}"#.into(),
            r#"{"version":1,"palette_set":"unknown"}"#.into(),
            r#"{"version":1,"roof_layout":99}"#.into(),
            r#"{"version":1,"resize_anchor":"sideways"}"#.into(),
            r#"{"version":1,"new_feature":true}"#.into(),
            r#"{"print_options":{"grid":"yes"}}"#.into(),
            r#"{"print_options":{"future":true}}"#.into(),
            r#"{"overworld_map_size":0}"#.into(),
            r#"{"overworld_map_size":4097}"#.into(),
            r#"{"overworld_map_size":65536}"#.into(),
            r#"{"recent_maps":[{"utf8":""}]}"#.into(),
            r#"{"recent_maps":[{"utf8":"a"},{"utf8":"a"}]}"#.into(),
            " ".repeat(MAX_BYTES + 1),
        ] {
            let mut storage = MemoryStorage::default();
            storage.values.insert(KEY.into(), raw.clone());
            storage.values.insert("unrelated".into(), "keep".into());
            let mut restored = restore(&storage);
            assert_eq!(
                Preferences::capture(restored.state()),
                Preferences::default()
            );
            assert!(!restored.state().preferences_writable);
            assert!(restored.state().status.contains("preserving"));
            restored.state_mut().hex = true;
            // Release the harness's storage borrow before saving the restored app.
            let mut editor = std::mem::take(restored.state_mut());
            drop(restored);
            eframe::App::save(&mut editor, &mut storage);
            assert_eq!(storage.writes, 0);
            assert_eq!(storage.values[KEY], raw);
            assert_eq!(storage.values["unrelated"], "keep");
        }
    }

    #[test]
    fn opacity_preference_survives_failed_startup_application_and_tracks_accepted_changes() {
        let mut storage = MemoryStorage::default();
        storage
            .values
            .insert(KEY.into(), r#"{"version":1,"transparent":true}"#.into());
        let mut h = restore(&storage);
        h.state_mut().dispatch_transparency(|opacity| {
            assert_eq!(opacity, polished_map_platform::WindowOpacity::Transparent);
            Err(polished_map_platform::OpacityError::Unsupported(
                "unsupported desktop",
            ))
        });
        let mut editor = std::mem::take(h.state_mut());
        drop(h);
        eframe::App::save(&mut editor, &mut storage);
        let saved: serde_json::Value = serde_json::from_str(&storage.values[KEY]).unwrap();
        assert_eq!(
            saved["transparent"], true,
            "keep intent when changing desktop backends"
        );
        let mut h = restore(&storage);
        h.state_mut().dispatch_transparency(|_| Ok(()));
        h.state_mut().transparency.toggle();
        h.state_mut().dispatch_transparency(|opacity| {
            assert_eq!(opacity, polished_map_platform::WindowOpacity::Opaque);
            Ok(())
        });
        let mut editor = std::mem::take(h.state_mut());
        drop(h);
        eframe::App::save(&mut editor, &mut storage);
        let saved: serde_json::Value = serde_json::from_str(&storage.values[KEY]).unwrap();
        assert_eq!(saved["transparent"], false);
        let mut h = restore(&storage);
        h.state_mut()
            .dispatch_transparency(|_| panic!("opaque startup should not touch native styles"));
    }

    #[test]
    fn long_recent_paths_restore_without_opening_files_and_clear_survives_restart() {
        let mut editor = Editor::default();
        // Ten long paths exceed the old 16 KiB settings cap. No filesystem
        // access is needed to remember or restore paths to disconnected drives.
        for i in 0..10 {
            editor
                .recent_maps
                .remember(format!("{i}/{}/Map.blk", "a".repeat(4000)).into());
        }
        let mut storage = MemoryStorage::default();
        eframe::App::save(&mut editor, &mut storage);
        assert!(storage.values[KEY].len() > 16 * 1024);
        let mut restored = restore(&storage);
        assert_eq!(restored.state().recent_maps, editor.recent_maps);
        assert!(restored.state().loaded.is_none() && restored.state().open_form.is_none());
        let mut editor = std::mem::take(restored.state_mut());
        drop(restored);
        editor.recent_maps = recent_maps::RecentMaps::default();
        eframe::App::save(&mut editor, &mut storage);
        let next = restore(&storage);
        assert_eq!(next.state().recent_maps, recent_maps::RecentMaps::default());
    }
}
