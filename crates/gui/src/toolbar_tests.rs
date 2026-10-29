//! Master Main_Window::initialize and widgets.cpp::Toolbar::draw layout vectors.
use super::*;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

const BEFORE_PALETTES: [(&str, f32); 19] = [
    ("New", 0.0),
    ("Open…", 24.0),
    ("Load events", 48.0),
    ("Reload events", 72.0),
    ("Save", 96.0),
    ("Print…", 126.0),
    ("Undo", 156.0),
    ("Redo", 180.0),
    ("Grid", 210.0),
    ("Rulers", 234.0),
    ("Zoom", 258.0),
    ("IDs", 282.0),
    ("Hex", 306.0),
    ("Show Priority", 330.0),
    ("Game Boy screen", 354.0),
    ("Show events", 378.0),
    ("Show warp IDs", 402.0),
    ("Blocks mode", 432.0),
    ("Events mode", 456.0),
];

fn geometry(h: &Harness<'_, Editor>) {
    for (name, x) in BEFORE_PALETTES {
        assert_eq!(
            h.get_by_label(name).rect(),
            Rect::from_min_size(Pos2::new(x, 22.0), Vec2::splat(24.0)),
            "{name}"
        );
    }
    let label = h.get_by_label("Palettes:").rect();
    let dropdown = h.get_by_label("Map palette set").rect();
    assert_eq!(
        (label.left(), label.top(), label.height()),
        (486.0, 22.0, 24.0)
    );
    assert_eq!(
        (dropdown.left(), dropdown.top(), dropdown.height()),
        (label.right(), 23.0, 22.0)
    );
    for (name, dx) in [
        ("Load palettes", 4.0),
        ("Edit current palettes", 28.0),
        ("Resize blockset", 58.0),
        ("Resize map", 82.0),
        ("Change tileset", 112.0),
        ("Tileset", 136.0),
        ("Change roof", 166.0),
        ("Roof", 190.0),
    ] {
        let actual = h.get_by_label(name).rect();
        assert_eq!(actual.size(), Vec2::splat(24.0), "{name}");
        assert!(
            (actual.left() - dropdown.right() - dx).abs() < 0.001,
            "{name}: {actual:?}"
        );
        assert_eq!((actual.top(), actual.bottom()), (22.0, 46.0));
    }
}

#[test]
fn complete_toolbar_keeps_source_order_bounds_and_empty_document_guards_at_both_dpi() {
    for dpi in [1.0, 2.0] {
        let mut h = Harness::builder()
            .with_size(Vec2::new(1000.0, 400.0))
            .with_pixels_per_point(dpi)
            .build_eframe(|cc| Editor::new(cc));
        h.run();
        geometry(&h);
        for name in [
            "Load events",
            "Reload events",
            "Save",
            "Print…",
            "Undo",
            "Redo",
            "Resize blockset",
            "Resize map",
            "Change tileset",
            "Tileset",
            "Change roof",
            "Roof",
        ] {
            assert!(
                h.get_by_label(name).accesskit_node().is_disabled(),
                "{name}"
            );
        }
        for name in [
            "New",
            "Open…",
            "Grid",
            "Game Boy screen",
            "Show events",
            "Show warp IDs",
            "Blocks mode",
            "Events mode",
            "Load palettes",
            "Edit current palettes",
        ] {
            assert!(
                !h.get_by_label(name).accesskit_node().is_disabled(),
                "{name}"
            );
        }
        // Narrow windows clip the row rather than reflowing controls into the map.
        h.set_size(Vec2::new(480.0, 400.0));
        h.run();
        for (name, x) in BEFORE_PALETTES {
            assert_eq!(
                h.get_by_label(name).rect().min,
                Pos2::new(x, 22.0),
                "{name}"
            );
        }
    }
}

#[test]
fn toolbar_display_toggles_and_mode_radios_share_shortcuts_and_clear_gestures() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.set_size(Vec2::new(1000.0, 400.0));
    h.run();
    geometry(&h);
    for label in ["Game Boy screen", "Show events", "Show warp IDs"] {
        h.get_by_label(label).click();
        h.run();
    }
    assert!(
        h.state().event_ui.game_boy && !h.state().event_ui.show && !h.state().event_ui.warp_ids
    );
    let shift = egui::Modifiers::COMMAND | egui::Modifiers::SHIFT;
    for key in [egui::Key::M, egui::Key::V, egui::Key::Num3] {
        h.key_press_modifiers(shift, key);
        h.run();
    }
    assert!(!h.state().event_ui.game_boy && h.state().event_ui.show && h.state().event_ui.warp_ids);
    h.state_mut().stroke.push(Point { x: 0, y: 0 });
    h.get_by_label("Events mode").click();
    h.run();
    assert!(h.state().event_ui.mode && h.state().stroke.is_empty());
    h.get_by_label("Events mode").click();
    h.run();
    assert!(h.state().event_ui.mode, "radio selection cannot toggle off");
    h.get_by_label("Blocks mode").focus();
    h.key_press(egui::Key::Space);
    h.run();
    assert!(!h.state().event_ui.mode);
    assert!(!h.state().is_modified());
    h.state_mut().about = Some(Default::default());
    h.run();
    for label in [
        "Game Boy screen",
        "Show events",
        "Show warp IDs",
        "Blocks mode",
        "Events mode",
        "Load palettes",
        "Resize blockset",
        "Change tileset",
        "Change roof",
    ] {
        assert!(
            h.get_by_label(label).accesskit_node().is_disabled(),
            "{label}"
        );
    }
}

#[test]
fn toolbar_asset_actions_use_existing_dialogs_and_undo_redo_survive_close() {
    let (dir, mut h) = crate::roof_editor_tests::fixture();
    h.set_size(Vec2::new(1000.0, 400.0));
    h.run();
    for label in ["Resize blockset", "Change tileset", "Change roof", "Roof"] {
        h.get_by_label(label).click();
        h.run();
        assert!(h.state().error.is_none(), "{label}: {:?}", h.state().error);
        match label {
            "Resize blockset" => assert_eq!(h.state().block_count_form, Some(2)),
            "Change tileset" => assert!(h.state().tileset_choice_form.is_some()),
            "Change roof" => assert!(h.state().roof_choice_form.is_some()),
            "Roof" => {
                assert!(h.state().tileset_form.is_some());
                h.get_by_label("Roof tile $0A");
            }
            _ => unreachable!(),
        }
        h.get_by_label("Cancel").click();
        h.run();
        assert!(!h.state().is_modified());
    }
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.run();
    assert!(!h.get_by_label("Undo").accesskit_node().is_disabled());
    assert!(h.get_by_label("Redo").accesskit_node().is_disabled());
    h.get_by_label("Undo").click();
    h.run();
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        [0, 1]
    );
    h.get_by_label("Redo").click();
    h.run();
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        [1, 1]
    );
    assert_eq!(
        std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
        [0, 1]
    );
    h.state_mut().close_map();
    h.run();
    assert!(h.get_by_label("Undo").accesskit_node().is_disabled());
    assert!(h.get_by_label("Redo").accesskit_node().is_disabled());
    geometry(&h);
}

#[cfg(feature = "visual-tests")]
#[test]
fn complete_toolbar_light_and_dark_appearance() {
    let (_dir, mut h) = crate::roof_editor_tests::fixture();
    h.set_size(Vec2::new(1000.0, 260.0));
    h.remove_cursor();
    h.run();
    h.snapshot("toolbar-complete-light");
    h.state_mut().dark = true;
    h.ctx.set_theme(egui::Theme::Dark);
    h.run();
    h.snapshot("toolbar-complete-dark");
}

#[test]
fn change_roof_availability_uses_project_inventory_and_edit_roof_uses_loaded_pixels() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.set_size(Vec2::new(1000.0, 400.0));
    h.run();
    assert!(!h.state().can_change_roof());
    for label in ["Change roof", "Roof"] {
        assert!(h.get_by_label(label).accesskit_node().is_disabled());
    }
    h.get_by_label("Tools").click();
    h.run();
    assert!(
        h.get_by_role_and_label(egui::Role::MenuItem, "Change Roof…")
            .accesskit_node()
            .is_disabled()
    );
    h.key_press(egui::Key::Escape);
    h.run();
    std::fs::create_dir_all(dir.path().join("gfx/tilesets/roofs")).unwrap();
    std::fs::write(dir.path().join("gfx/tilesets/roofs/new.2bpp"), [0; 9 * 16]).unwrap();
    h.run();
    assert!(
        !h.state().can_change_roof(),
        "inventory is cached at map open, like upstream"
    );
    let ctx = h.ctx.clone();
    let editor = h.state_mut();
    editor.pending_open = editor.loaded.take();
    editor.advance_open_palettes(&ctx);
    h.run();
    assert!(h.state().can_change_roof());
    assert!(!h.get_by_label("Change roof").accesskit_node().is_disabled());
    assert!(
        h.get_by_label("Roof").accesskit_node().is_disabled(),
        "listing a roof does not load its pixels"
    );
    h.get_by_label("Change roof").click();
    h.run();
    assert!(h.state().roof_choice_form.is_some());
    h.get_by_label("Cancel").click();
    h.run();
    h.state_mut().close_map();
    assert!(!h.state().can_change_roof());
}
