use super::*;
use egui_kittest::kittest::Queryable;

#[test]
fn source_map_options_geometry_preserves_fixed_rows_and_platform_buttons() {
    let measure = |s: &str| match s {
        "Width:" => 30.0,
        "Height:" => 35.0,
        "Size:" => 22.0,
        "Tileset:" => 36.0,
        "999" => 18.0,
        "9999 x 9999" => 65.0,
        _ => panic!("unexpected metric {s}"),
    };
    for windows in [false, true] {
        for has_sizes in [false, true] {
            let layout = Layout::measured(has_sizes, windows, measure, 100.0, 55.0);
            assert_eq!(layout.size, Vec2::new(300.0, 158.0));
            assert_eq!(layout.header.min, Pos2::new(10.0, 10.0));
            assert_eq!(layout.header.size(), Vec2::new(280.0, 22.0));
            assert_eq!(layout.dimensions.min, Pos2::new(50.0, 36.0));
            assert_eq!(
                layout.dimensions.width(),
                if has_sizes { 91.0 } else { 44.0 }
            );
            assert_eq!(
                layout.height.map(|r| r.min),
                (!has_sizes).then_some(Pos2::new(139.0, 36.0))
            );
            assert_eq!(
                layout.bytes.min,
                Pos2::new(if has_sizes { 151.0 } else { 193.0 }, 36.0)
            );
            assert_eq!(layout.bytes.width(), if has_sizes { 139.0 } else { 97.0 });
            assert_eq!(layout.tileset.min, Pos2::new(50.0, 62.0));
            assert_eq!(layout.tileset.width(), 134.0);
            assert_eq!(layout.roof.min, Pos2::new(50.0, 88.0));
            assert_eq!(layout.roof.width(), 89.0);
            assert_eq!(
                layout.ok.min,
                Pos2::new(if windows { 116.0 } else { 210.0 }, 126.0)
            );
            assert_eq!(
                layout.cancel.min,
                Pos2::new(if windows { 210.0 } else { 116.0 }, 126.0)
            );
            assert_eq!(layout.ok.size(), Vec2::new(80.0, 22.0));
        }
    }
}

#[test]
fn rendered_map_options_match_measured_client_at_both_platform_metrics_and_scales() {
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for (new, windows, scale, dark, _name) in [
        (false, true, 1.0, false, "map-options-open-windows"),
        (true, true, 2.0, true, "map-options-new-windows-2x"),
        (false, false, 2.0, true, "map-options-open-unix-2x"),
        (true, false, 1.0, false, "map-options-new-unix"),
    ] {
        let (dir, mut h) = tileset_editor_tests::fixture(false, false);
        h.state_mut().dark = dark;
        h.ctx.set_theme(if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        h.set_pixels_per_point(scale);
        h.set_size(Vec2::new(800.0, 600.0));
        h.ctx
            .data_mut(|d| d.insert_temp(egui::Id::unique("test-map-options-windows"), windows));
        if new {
            h.state_mut().prepare_new_in(dir.path().to_owned());
        } else {
            h.state_mut().prepare_open(dir.path().join("Map.2x1.blk"));
        }
        h.run();
        let form = h.state().open_form.as_ref().unwrap();
        let bounds = form.client_bounds.unwrap();
        let layout = form.client_layout.unwrap();
        assert_eq!(bounds.size(), Vec2::new(300.0, 158.0));
        if let Some(height) = layout.height {
            for (name, expected) in [("Width", layout.dimensions), ("Height", height)] {
                assert_eq!(
                    h.get_by_label(name).rect(),
                    expected.translate(bounds.min.to_vec2())
                );
            }
        } else {
            assert_eq!(
                h.get_by_role_and_label(egui::Role::ComboBox, "Size").rect(),
                layout.dimensions.translate(bounds.min.to_vec2())
            );
        }
        for (label, expected) in [
            (if new { "Create map" } else { "Open map" }, layout.ok),
            ("Cancel", layout.cancel),
        ] {
            assert_eq!(
                h.get_by_label(label).rect(),
                expected.translate(bounds.min.to_vec2())
            );
        }
        for (name, expected) in [("Tileset", layout.tileset), ("Roof", layout.roof)] {
            let actual = h.get_by_role_and_label(egui::Role::ComboBox, name).rect();
            assert!(
                (actual.left() - bounds.left() - expected.left()).abs() <= 0.5,
                "{name}: {actual:?}"
            );
            assert!(
                (actual.top() - bounds.top() - expected.top()).abs() <= 0.5,
                "{name}: {actual:?}"
            );
            assert!(
                (actual.width() - expected.width()).abs() <= 0.5,
                "{name}: {actual:?}, {expected:?}"
            );
            assert_eq!(actual.height(), 22.0);
        }
        h.remove_cursor();
        h.run();
        #[cfg(feature = "visual-tests")]
        snapshots.add(h.try_snapshot(_name));
        h.key_press(egui::Key::Escape);
        h.run();
        assert!(h.state().open_form.is_none());
    }
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}

#[test]
fn map_options_popup_escape_only_closes_popup_and_next_escape_cancels() {
    let (dir, mut h) = tileset_editor_tests::fixture(false, false);
    h.state_mut().prepare_new_in(dir.path().to_owned());
    h.run();
    h.get_by_role_and_label(egui::Role::ComboBox, "Tileset")
        .click();
    h.run();
    assert!(egui::Popup::is_any_open(&h.ctx));
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(!egui::Popup::is_any_open(&h.ctx));
    assert!(h.state().open_form.is_some());
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(h.state().open_form.is_none());
}

#[test]
fn map_options_enter_accepts_once_and_stale_project_choices_cannot_submit() {
    let (dir, mut h) = tileset_editor_tests::fixture(false, false);
    h.state_mut().prepare_new_in(dir.path().to_owned());
    h.run();
    h.key_press(egui::Key::Enter);
    h.step();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while h.state().loading.is_some() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
        h.step();
    }
    h.run();
    assert!(h.state().loading.is_none());
    assert!(h.state().open_form.is_none());
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    let document = &h.state().loaded.as_ref().unwrap().document;
    assert!(!document.has_backing_file());
    assert_eq!((document.map().width(), document.map().height()), (10, 9));
    h.state_mut().prepare_new_in(dir.path().to_owned());
    h.run();
    h.state_mut()
        .open_form
        .as_mut()
        .unwrap()
        .root
        .push_str("/different");
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(h.state().loading.is_none());
    assert!(h.state().open_form.is_some());
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(h.state().open_form.is_none());
}

#[test]
fn map_options_escape_cancels_while_a_dimension_field_is_being_edited() {
    let (dir, mut h) = tileset_editor_tests::fixture(false, false);
    let old = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .document
        .path()
        .to_owned();
    h.state_mut().prepare_new_in(dir.path().to_owned());
    h.run();
    h.get_by_label("Width").click();
    h.run();
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(h.state().open_form.is_none());
    assert_eq!(h.state().loaded.as_ref().unwrap().document.path(), old);
}
