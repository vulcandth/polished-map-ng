use super::*;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn original_width_height_and_indicator_vectors() {
    let metrics = Metrics::measured(
        &[
            (Vec2::new(100.0, 16.0), false, Vec2::new(29.0, 12.0)),
            (Vec2::new(110.0, 15.0), true, Vec2::ZERO),
            (Vec2::new(75.0, 14.0), false, Vec2::new(60.0, 20.0)),
        ],
        20.0,
    );
    assert_eq!(metrics.width, 213.0);
    assert_eq!(metrics.row, 20.0);
    assert_eq!(metrics.height(), 61.0);
    assert_eq!(metrics.key_width, 20.0);
    assert_eq!(metrics.indicator(), 14.0);
    let radio = Metrics::measured(&[(Vec2::new(80.0, 15.0), false, Vec2::ZERO)], 100.0);
    assert_eq!(radio.width, 112.0);
    assert_eq!(radio.row, 19.0);
    assert_eq!(radio.height(), 20.0);
    assert_eq!(radio.indicator(), 13.0);
}

#[test]
fn shortcut_columns_keep_shift_command_and_function_keys() {
    for (source, windows, mac) in [
        ("CS+S", ("Ctrl+Shift+", "S"), ("⇧⌘", "S")),
        ("C+=", ("Ctrl+", "="), ("⌘", "=")),
        ("A+F4", ("Alt+", "F4"), ("⌥", "F4")),
        ("F11", ("", "F11"), ("", "F11")),
    ] {
        assert_eq!(
            shortcut_parts(source, false),
            (windows.0.into(), windows.1.into())
        );
        assert_eq!(shortcut_parts(source, true), (mac.0.into(), mac.1.into()));
    }
}

#[test]
fn popup_rows_reserve_images_and_align_shortcuts_at_both_font_sizes_and_dpi_scales() {
    for size in [12.0, 13.0] {
        for dpi in [1.0, 2.0] {
            let mut h = Harness::builder()
                .with_size(Vec2::new(800.0, 600.0))
                .with_pixels_per_point(dpi)
                .with_context_setup(move |ctx| test_font_size(ctx, size))
                .build_eframe(|cc| Editor::new(cc));
            h.get_by_label("File").click();
            h.remove_cursor();
            h.run();
            let mut previous: Option<Rect> = None;
            let mut width: Option<f32> = None;
            for item in specs::group("File") {
                let node = h.get_by_role_and_label(egui::Role::MenuItem, item.name);
                let rect = node.rect();
                if let Some(previous) = previous {
                    assert_eq!(rect.top(), previous.bottom(), "divider must not add a row");
                }
                if let Some(width) = width {
                    assert_eq!(rect.width(), width);
                }
                assert_eq!(rect.height(), if item.name == "Exit" { 19.0 } else { 20.0 });
                previous = Some(rect);
                width = Some(rect.width());
            }
            assert_eq!(
                h.get_by_role_and_label(egui::Role::MenuItem, "Save As…")
                    .accesskit_node()
                    .data()
                    .keyboard_shortcut(),
                Some("Ctrl+Shift+S")
            );

            // Text paint positions are checked independently of widget bounds.
            let mut key_x = Vec::new();
            for shape in &h.output().shapes {
                // Toolbar text placeholders can share a shortcut letter, but
                // their paint is clipped to the independent 26-point toolbar.
                if shape.clip_rect.height() <= geometry::TOOLBAR_HEIGHT {
                    continue;
                }
                if let egui::Shape::Text(text) = &shape.shape {
                    if ["N", "O", "W", "S", "P", "F4"].contains(&text.galley.text()) {
                        key_x.push(text.pos.x);
                    }
                    if text.galley.text() == " Open...         " {
                        let button = h
                            .get_by_role_and_label(egui::Role::MenuItem, "Open…")
                            .rect();
                        assert_eq!(text.pos.x - button.left(), 19.0);
                    }
                }
            }
            assert_eq!(key_x.len(), 7);
            assert!(key_x.iter().all(|x| *x == key_x[0]));
        }
    }
}

#[test]
fn unix_and_mac_menus_have_glyphs_checked_state_and_keyboard_submenus() {
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for (mac, dark, dpi, _name) in [
        (false, true, 2.0, "popup-unix-dark-2x"),
        (true, false, 1.0, "popup-mac-shortcuts"),
    ] {
        let mut h = Harness::builder()
            .with_size(Vec2::new(800.0, 600.0))
            .with_pixels_per_point(dpi)
            .with_theme(if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            })
            .with_os(if mac {
                egui::os::OperatingSystem::Mac
            } else {
                egui::os::OperatingSystem::Nix
            })
            .with_context_setup(move |ctx| {
                test_font_size(ctx, 13.0);
            })
            .build_eframe(|cc| Editor::new(cc));
        h.state_mut().dark = dark;
        h.run();
        if mac {
            assert!(h.ctx.fonts_mut(|fonts| fonts.has_glyphs(
                &egui::FontId::new(13.0, egui::FontFamily::Name("Menu Symbols".into())),
                "⇧⌥⌘"
            )));
        }
        h.get_by_label("View").click();
        h.remove_cursor();
        h.run();
        assert_eq!(
            h.get_by_label("Show Warp IDs")
                .accesskit_node()
                .data()
                .keyboard_shortcut(),
            Some(if mac { "⇧⌘3" } else { "Ctrl+Shift+3" })
        );
        assert!(
            h.get_by_role_and_label(egui::Role::CheckBox, "Show Priority")
                .accesskit_node()
                .is_disabled()
        );
        #[cfg(feature = "visual-tests")]
        snapshots.add(h.try_snapshot(_name));
        h.get_by_label("Palettes").focus();
        h.run();
        h.key_press(egui::Key::ArrowRight);
        h.run();
        assert_eq!(
            h.get_by_role_and_label(egui::Role::RadioButton, "Day")
                .accesskit_node()
                .toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        h.get_by_role_and_label(egui::Role::RadioButton, "Night")
            .focus();
        h.run();
        h.key_press(egui::Key::Enter);
        h.run();
        assert_eq!(
            h.state().palette_document().selected(),
            polished_map_core::PaletteSet::Nite
        );
        assert!(h.query_by_label("Show Warp IDs").is_none());
    }
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}

#[test]
fn keyboard_menu_activation_opens_about_without_dismissing_it_in_the_same_frame() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .build_eframe(|cc| Editor::new(cc));
    h.get_by_label("Help").click();
    h.run();
    h.get_by_role_and_label(egui::Role::MenuItem, "About")
        .focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(h.state().about.is_some());
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(h.state().about.is_none());
}

#[test]
fn recent_labels_keep_literal_ellipsis_and_duplicate_names_have_distinct_rows() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(500.0, 300.0))
        .build_ui_state(
            |ui, chosen: &mut Option<usize>| {
                let items = [
                    Spec::recent("Town…&.blk", "A+1", false),
                    Spec::recent("Town…&.blk", "A+2", true),
                    Spec::recent("Clear Recent", "", false),
                ];
                scope(ui, "", &items, |ui| {
                    for i in 0..items.len() {
                        if row_at(ui, true, i, None).clicked() {
                            *chosen = Some(i);
                        }
                    }
                });
            },
            None,
        );
    let rows: Vec<_> = h
        .get_all_by_role_and_label(egui::Role::MenuItem, "Town…&.blk")
        .collect();
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0].rect(), rows[1].rect());
    assert_eq!(
        rows[1].accesskit_node().data().keyboard_shortcut(),
        Some("Alt+2")
    );
    rows[1].click();
    h.run();
    assert_eq!(*h.state(), Some(1));
    assert!(h.output().shapes.iter().any(|s| matches!(&s.shape,
        egui::Shape::Text(t) if t.galley.text() == " Town…&.blk         ")));
}
