use super::*;
use egui_kittest::Harness;

fn legacy(body: &str) -> LegacyPreferences {
    LegacyPreferences::parse(format!("; FLTK preferences file format 1.0\n; vendor: Rangi\n; application: Polished Map\n[.]\n{body}").as_bytes()).unwrap()
}

#[test]
fn every_master_setting_maps_to_the_ng_record_without_document_state() {
    let input = legacy(
        "theme:9\nmode:1\ngrid:0\nrulers:1\nzoom:1\nids:1\nhex:1\npriority:0\ngameboy:1\nevent:0\nwarp-ids:0\ntransparent:1\npalettes:5\nmonochrome:1\nprioritize:1\nall256:1\nroof-palettes:4\nevents:0\nspecial:0\nroofs:0\ndrag:0\noverworld-map:4096\nmeta128:1\nprint-grid:1\nprint-ids:1\nprint-priority:1\nprint-events:1\nprint-warp-ids:1\nresize-anchor:8\nrecent0:one.blk\nrecent1:two.blk\nrecent2:one.blk\nx:-100\ny:70\nw:920\nh:680\nfullscreen:1\nmaximized:1\n",
    );
    let imported = Imported::from_legacy(&input, "original.prefs".into()).unwrap();
    assert_eq!(imported.size, [920.0, 680.0]);
    assert_eq!(imported.position, Some([-100.0, 70.0]));
    assert!(imported.state.fullscreen && imported.state.maximized && !imported.theme_fallback);
    let p = &imported.settings;
    for flag in [
        p.dark,
        p.event_mode,
        p.rulers,
        p.zoom,
        p.ids,
        p.hex,
        p.game_boy,
        p.transparent,
        p.monochrome,
        p.allow_priority,
        p.allow_256,
        p.meta128,
        p.print_options.grid,
        p.print_options.ids,
        p.print_options.priority,
        p.print_options.events,
        p.print_options.warp_ids,
    ] {
        assert!(flag);
    }
    for flag in [
        p.grid,
        p.show_priority,
        p.show_events,
        p.show_warp_ids,
        p.auto_events,
        p.auto_special_palettes,
        p.auto_roof_colors,
        p.drag_and_drop,
    ] {
        assert!(!flag);
    }
    assert_eq!(p.palette_set, PaletteSet::Custom);
    assert_eq!(p.roof_layout, RoofPaletteLayout::MornDayNiteCustom);
    assert_eq!(p.resize_anchor, ResizeAnchor::BottomRight);
    assert_eq!(p.overworld_map_size, 4096);
    let mut expected = recent_maps::RecentMaps::default();
    expected.remember("two.blk".into());
    expected.remember("one.blk".into());
    assert_eq!(p.recent_maps, expected);
    let mut editor = Editor::default();
    imported.settings.apply(&mut editor);
    assert!(!editor.is_modified() && editor.loaded.is_none() && editor.palette_form.is_none());
    assert_eq!(Preferences::capture(&editor).recent_maps, expected);
}

#[test]
fn defaults_enums_themes_and_invalid_values_are_explicit() {
    assert_eq!(
        Imported::from_legacy(&legacy(""), "defaults".into())
            .unwrap()
            .settings,
        Preferences::default()
    );
    for palette in 0..6 {
        for roof in 0..5 {
            for anchor in 0..9 {
                let p = Imported::from_legacy(
                    &legacy(&format!(
                        "palettes:{palette}\nroof-palettes:{roof}\nresize-anchor:{anchor}\n"
                    )),
                    "test".into(),
                )
                .unwrap();
                assert_eq!(p.settings.palette_set, PaletteSet::ALL[palette]);
                assert_eq!(p.settings.resize_anchor, ResizeAnchor::ALL[anchor]);
                let encoded = serde_json::to_string(&p.settings).unwrap();
                assert_eq!(
                    serde_json::from_str::<Preferences>(&encoded).unwrap(),
                    p.settings
                );
            }
        }
    }
    for theme in 0..12 {
        let p = Imported::from_legacy(&legacy(&format!("theme:{theme}\n")), "test".into()).unwrap();
        assert_eq!(p.theme_fallback, !matches!(theme, 4 | 9));
        assert_eq!(p.settings.dark, theme == 9);
    }
    for body in [
        "palettes:6",
        "mode:-1",
        "roof-palettes:5",
        "resize-anchor:9",
        "overworld-map:0",
        "overworld-map:4097",
        "w:0",
        "h:999999",
        "theme:12",
        "grid:2147483648",
        "recent0:bad\\",
    ] {
        assert!(
            Imported::from_legacy(&legacy(body), "test".into()).is_err(),
            "{body}"
        );
    }
}

#[test]
fn ng_file_precedence_first_launch_save_and_failure_retention() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("polishedmap.prefs");
    let bytes = b"; FLTK preferences file format 1.0\n; vendor: Rangi\n; application: polishedmap\n[.]\ntheme:9\ngrid:0\nmeta128:1\n";
    std::fs::write(&original, bytes).unwrap();
    let ng = dir.path().join("app.ron");
    let user = LegacyUserLocation::Windows {
        roaming_appdata: None,
    };
    let startup = NativeStartup::from_result(NativeStartup::load(dir.path(), &user, Some(&ng)));
    let mut h = Harness::builder().build_eframe(|cc| {
        let mut editor = Editor::new(cc);
        startup.apply(&mut editor, None, &cc.egui_ctx);
        editor
    });
    assert!(h.state().dark && h.state().meta128 && !h.state().grid);
    let mut storage = super::super::tests::MemoryStorage::default();
    eframe::App::save(h.state_mut(), &mut storage);
    let encoded = eframe::Storage::get_string(&storage, KEY).unwrap();
    let saved: Preferences = serde_json::from_str(&encoded).unwrap();
    assert!(saved.dark && saved.meta128);
    assert_eq!(std::fs::read(&original).unwrap(), bytes);
    for existing in [b"valid NG record".as_slice(), b"malformed NG record"] {
        std::fs::write(&ng, existing).unwrap();
        assert!(
            NativeStartup::load(dir.path(), &user, Some(&ng))
                .unwrap()
                .is_none()
        );
        assert_eq!(std::fs::read(&ng).unwrap(), existing);
    }
    let mut editor = Editor::default();
    NativeStartup::from_result(Err(LegacyPreferenceError::Invalid("bad file"))).apply(
        &mut editor,
        None,
        &h.ctx,
    );
    assert!(!editor.preferences_writable && editor.status.contains("preserving"));
    let previous = eframe::Storage::get_string(&storage, KEY);
    eframe::App::save(&mut editor, &mut storage);
    assert_eq!(eframe::Storage::get_string(&storage, KEY), previous);
}

#[test]
fn fullscreen_waits_for_visibility_wins_over_maximize_and_is_requested_once() {
    for fullscreen in [false, true] {
        let mut h = Harness::builder().build_eframe(|cc| Editor::new(cc));
        h.state_mut().startup_window_state = Some(StartupWindowState {
            fullscreen,
            maximized: true,
        });
        h.input_mut()
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .minimized = Some(true);
        h.step();
        assert!(h.state().startup_window_state.is_some());
        h.input_mut()
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .minimized = Some(false);
        h.step();
        let commands = &h.output().viewport_output[&egui::ViewportId::ROOT].commands;
        assert!(commands.contains(&if fullscreen {
            egui::ViewportCommand::Fullscreen(true)
        } else {
            egui::ViewportCommand::Maximized(true)
        }));
        assert!(!commands.contains(&if fullscreen {
            egui::ViewportCommand::Maximized(true)
        } else {
            egui::ViewportCommand::Fullscreen(true)
        }));
        assert!(h.state().startup_window_state.is_none());
        h.step();
        assert!(
            !h.output().viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .any(|c| matches!(
                    c,
                    egui::ViewportCommand::Fullscreen(_) | egui::ViewportCommand::Maximized(_)
                ))
        );
    }
}
