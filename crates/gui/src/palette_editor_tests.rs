use super::*;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use polished_map_core::{ColorSlot, PaletteSet};
use polished_map_project::prepare_palette_export;
use std::fs;

fn open(h: &mut Harness<'_, Editor>) {
    h.get_by_label("Tools").click();
    h.run();
    h.get_by_label("Edit Current Palettes…").click();
    h.run();
}
fn color(h: &Harness<'_, Editor>, palette: u8, hue: u8) -> [u8; 3] {
    h.state()
        .palette_form
        .as_ref()
        .unwrap()
        .draft
        .bank()
        .color(ColorSlot {
            set: PaletteSet::Day,
            palette,
            hue,
        })
        .unwrap()
}

#[test]
fn palette_spinners_wrap_but_slider_keys_clamp_and_cancel_discards_both() {
    let (_dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let original = h.state().palette_document().bank().clone();
    open(&mut h);
    for (channel, name) in ["R", "G", "B"].into_iter().enumerate() {
        let spin = format!("{name} channel");
        h.get_by_label(&spin).set_value_accesskit(31.0);
        h.run();
        h.get_by_label(&spin).focus();
        h.run();
        h.key_press(egui::Key::ArrowUp);
        h.run();
        assert_eq!(color(&h, 0, 0)[channel], 0);
        h.key_press(egui::Key::ArrowDown);
        h.run();
        assert_eq!(color(&h, 0, 0)[channel], 255);
        let slider = format!("{name} slider");
        h.get_by_label(&slider).focus();
        h.run();
        h.key_press(egui::Key::ArrowRight);
        h.run();
        assert_eq!(color(&h, 0, 0)[channel], 255);
        h.key_press(egui::Key::ArrowLeft);
        h.run();
        assert_eq!(color(&h, 0, 0)[channel], 247);
        h.key_press(egui::Key::ArrowUp);
        h.run();
        assert_eq!(
            color(&h, 0, 0)[channel],
            247,
            "horizontal slider ignores up/down"
        );
        h.get_by_label(&slider).set_value_accesskit(0.0);
        h.run();
        h.key_press(egui::Key::ArrowLeft);
        h.run();
        assert_eq!(color(&h, 0, 0)[channel], 0);
    }
    h.get_by_label("Cancel").click();
    h.run();
    assert_eq!(h.state().palette_document().bank(), &original);
    assert!(!h.state().is_modified());
}

#[test]
fn channel_controls_and_set_selection_refresh_colors_without_map_edits() {
    let (_dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    open(&mut h);
    hex(&mut h, "037ff9");
    h.get_by_label("R channel").set_value_accesskit(2.0);
    h.run();
    assert_eq!(color(&h, 0, 0), [16, 127, 249]);
    h.get_by_label("G slider").set_value_accesskit(31.0);
    h.run();
    assert_eq!(color(&h, 0, 0), [16, 255, 249]);
    h.get_by_label("OK").click();
    h.run();
    h.get_by_label("View").click();
    h.run();
    h.get_by_label("Palettes").click();
    h.run();
    h.get_by_label("Night").click();
    h.run();
    assert_eq!(h.state().palette_document().selected(), PaletteSet::Nite);
    assert_eq!(
        h.state().palette_document().bank().colors(PaletteSet::Day)[0][0],
        [16, 255, 249]
    );
    assert!(h.state().palette_document().is_modified());
    assert!(!h.state().is_modified());
}

#[test]
fn toolbar_palette_sets_share_menu_state_and_preserve_edits_and_history() {
    let (_dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    // Preserve an existing palette edit and a map undo/redo boundary while
    // switching all six sets. Palette selection itself must create neither.
    open(&mut h);
    h.get_by_label("GRAY color 3").click();
    h.run();
    hex(&mut h, "123456");
    h.get_by_label("OK").click();
    h.run();
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .document
        .map_mut()
        .undo();
    h.run();
    let palette_hash = h.state().palette_document().content_hash();
    let palette_colors = h.state().palette_document().bank().clone();
    let map_modified = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .document
        .map()
        .is_modified();
    let initial_bounds = h.get_by_label("Map palette set").rect();
    assert_eq!(initial_bounds.height(), 22.0);
    assert_eq!(
        (initial_bounds.top(), initial_bounds.bottom()),
        (23.0, 45.0)
    );
    let label = h.get_by_label("Palettes:").rect();
    assert_eq!(label.height(), 24.0);
    assert_eq!(label.right(), initial_bounds.left());
    assert_eq!(
        h.get_by_label("Load palettes").rect().left() - initial_bounds.right(),
        4.0
    );
    assert_eq!(
        h.get_by_label("Edit current palettes").rect().left() - initial_bounds.right(),
        28.0
    );
    for (set, name) in PaletteSet::ALL
        .into_iter()
        .zip(["Morn", "Day", "Night", "Dark", "Indoor", "Custom"])
    {
        h.get_by_label("Map palette set").click();
        h.run();
        h.get_by_label(name).click();
        h.run();
        assert_eq!(h.state().palette_document().selected(), set);
        assert_eq!(
            h.get_by_label("Map palette set")
                .accesskit_node()
                .value()
                .as_deref(),
            Some(name)
        );
        assert_eq!(h.get_by_label("Map palette set").rect(), initial_bounds);
        assert_eq!(h.state().palette_document().bank(), &palette_colors);
        assert!(h.state().palette_document().is_modified());
        let loaded = h.state().loaded.as_ref().unwrap();
        assert_eq!(loaded.document.map().is_modified(), map_modified);
        assert!(!loaded.document.map().can_undo());
        assert!(loaded.document.map().can_redo());
        assert!(!loaded.tileset.as_ref().unwrap().tileset().is_modified());
        assert_eq!(
            loaded.graphics.render_block(0).unwrap().pixels[..3],
            h.state().palette_document().bank().colors(set)[0][1],
        );
        h.get_by_label("View").click();
        h.run();
        h.get_by_label("Palettes").click();
        h.run();
        assert_eq!(
            h.get_by_label(name).accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        h.get_by_label("Day").click();
        h.run();
        assert_eq!(
            h.get_by_label("Map palette set")
                .accesskit_node()
                .value()
                .as_deref(),
            Some("Day")
        );
    }
    h.get_by_label("Map palette set").focus();
    h.key_press(egui::Key::Space);
    h.run();
    assert!(h.query_by_label("Custom").is_some());
    #[cfg(feature = "visual-tests")]
    {
        h.remove_cursor();
        h.run();
        h.snapshot("toolbar-palette-list-light");
        h.ctx.set_theme(egui::Theme::Dark);
        h.state_mut().dark = true;
        h.run();
        h.snapshot("toolbar-palette-list-dark");
        h.ctx.set_theme(egui::Theme::Light);
        h.state_mut().dark = false;
        h.run();
    }
    h.key_press(egui::Key::Escape);
    h.run();
    assert_eq!(h.state().palette_document().selected(), PaletteSet::Day);
    open(&mut h);
    assert!(
        h.get_by_label("Map palette set")
            .accesskit_node()
            .is_disabled()
    );
    h.get_by_label("Map palette set").click();
    h.run();
    assert!(h.query_by_label("Custom").is_none());
    h.get_by_label("Cancel").click();
    h.run();
    h.get_by_label("Redo").click();
    h.run();
    assert!(h.state().is_modified());
    assert_eq!(h.state().palette_document().content_hash(), palette_hash);
}

#[test]
fn toolbar_palette_selector_works_without_a_map() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .build_eframe(|cc| Editor::new(cc));
    h.get_by_label("Map palette set").click();
    h.run();
    h.get_by_label("Night").click();
    h.run();
    assert_eq!(h.state().palette_document().selected(), PaletteSet::Nite);
    assert!(!h.state().palette_document().is_modified());
    assert!(!h.state().is_modified());
    assert!(h.state().loaded.is_none());
}

#[test]
fn palette_editor_operates_before_loading_a_map() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .build_eframe(|cc| Editor::new(cc));
    open(&mut h);
    hex(&mut h, "123");
    assert_eq!(color(&h, 0, 0), [0, 1, 35]);
    h.get_by_label("OK").click();
    h.run();
    assert!(h.state().loaded.is_none());
    assert_eq!(
        h.state().palette_document().bank().colors(PaletteSet::Day)[0][0],
        [0, 1, 35]
    );
    assert!(h.state().palette_document().is_modified());
}
fn hex(h: &mut Harness<'_, Editor>, value: &str) {
    let input = h.get_by_label("Hex color");
    input.click();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Hex color").type_text(value);
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
}

#[test]
fn palette_draft_accept_cancel_export_and_load_preserve_map_and_tile_state() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let old = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .graphics
        .render_block(0)
        .unwrap();
    for accept in [false, true] {
        open(&mut h);
        h.get_by_label("GRAY color 3").click();
        h.run();
        hex(&mut h, "037ff9");
        assert!(
            h.state().palette_form.is_some(),
            "hex Enter must not accept the dialog"
        );
        assert_eq!(color(&h, 0, 1), [3, 127, 249]);
        h.get_by_label(if accept { "OK" } else { "Cancel" }).click();
        h.run();
        assert_eq!(h.state().palette_document().is_modified(), accept);
        assert!(
            !h.state().is_modified(),
            "palette export is separate from map Save"
        );
        let loaded = h.state().loaded.as_ref().unwrap();
        assert!(!loaded.document.map().can_undo());
        assert!(!loaded.tileset.as_ref().unwrap().tileset().is_modified());
        let current = loaded.graphics.render_block(0).unwrap();
        if accept {
            assert_eq!(current.pixels[..3], [3, 127, 249]);
        } else {
            assert_eq!(current, old);
        }
    }
    let target = prepare_palette_export(&dir.path().join("colors.pal")).unwrap();
    h.state_mut().export_palette_target(&target).unwrap();
    h.run();
    assert!(!h.state().palette_document().is_modified());
    let text = fs::read_to_string(target.path()).unwrap();
    assert_eq!(text.lines().count(), 54);
    assert!(text.contains("00,15,31"));
    open(&mut h);
    h.get_by_label("OK").click();
    h.run();
    assert!(
        h.state().palette_document().is_modified(),
        "unchanged acceptance marks palettes edited"
    );
    let ctx = h.ctx.clone();
    h.state_mut()
        .request_palette_load(&ctx, target.path().into());
    h.run();
    h.get_by_label("Cancel").click();
    h.run();
    assert!(h.state().palette_document().is_modified());
    h.state_mut()
        .request_palette_load(&ctx, target.path().into());
    h.run();
    h.get_by_label("Load").click();
    h.run();
    assert!(!h.state().palette_document().is_modified());
    assert_eq!(
        h.state().palette_document().bank().colors(PaletteSet::Day)[0][1],
        [0, 123, 255]
    );
    let before = h.state().palette_document().content_hash();
    fs::write(target.path(), "\tRGB 32,0,0\n").unwrap();
    assert!(
        h.state_mut()
            .load_palette_path(&ctx, target.path())
            .is_err()
    );
    assert_eq!(h.state().palette_document().content_hash(), before);
}

#[test]
fn palette_private_copy_quantizes_snapshot_and_swap_uses_live_source() {
    let (_dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    open(&mut h);
    hex(&mut h, "037ff9");
    h.get_by_label("GRAY color 1").click();
    h.run();
    h.input_mut().events.push(egui::Event::Copy);
    h.run();
    hex(&mut h, "112233");
    h.get_by_label("RED color 2").click();
    h.run();
    h.input_mut()
        .events
        .push(egui::Event::Paste("ignored".into()));
    h.run();
    assert_eq!(color(&h, 1, 2), [0, 123, 255]);
    h.input_mut().events.push(egui::Event::Cut);
    h.run();
    assert_eq!(color(&h, 1, 2), [17, 34, 51]);
    assert_eq!(color(&h, 0, 0), [0, 123, 255]);
    h.input_mut().events.push(egui::Event::Cut);
    h.run();
    assert_eq!(color(&h, 1, 2), [17, 34, 51]);
    h.get_by_label("Cancel").click();
    h.run();
    assert!(!h.state().palette_document().is_modified());
}

#[test]
fn color_palette_form_geometry_and_export_follow_source() {
    palette_geometry(false);
}

#[test]
fn monochrome_palette_form_geometry_and_export_follow_source() {
    palette_geometry(true);
}

fn palette_geometry(monochrome: bool) {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, monochrome);
    open(&mut h);
    let name = if monochrome { "MONOCHROME" } else { "GRAY" };
    let first = h.get_by_label(&format!("{name} color 1")).rect();
    let next = h.get_by_label(&format!("{name} color 2")).rect();
    assert_eq!(first.size(), Vec2::splat(20.0));
    assert_eq!(next.min - first.min, Vec2::new(21.0, 0.0));
    if !monochrome {
        assert_eq!(
            h.get_by_label("TEXT color 1").rect().min - first.min,
            Vec2::new(0.0, 147.0)
        );
    }
    assert!(h.state().palette_form.as_ref().unwrap().bounds.height() < 245.0);
    #[cfg(feature = "visual-tests")]
    for windows in [true, false] {
        h.state_mut().palette_form.as_mut().unwrap().windows_buttons = windows;
        h.remove_cursor();
        h.run();
        h.snapshot(format!(
            "palette-{}-{}",
            if monochrome { "mono" } else { "color" },
            if windows { "windows" } else { "unix" }
        ));
    }
    if monochrome {
        hex(&mut h, "ff0000");
        assert_eq!(color(&h, 8, 0), [255, 0, 0]);
    }
    h.get_by_label("OK").click();
    h.run();
    let target = prepare_palette_export(&dir.path().join("colors.pal")).unwrap();
    h.state_mut().export_palette_target(&target).unwrap();
    assert_eq!(
        fs::read_to_string(target.path()).unwrap().lines().count(),
        if monochrome { 6 } else { 54 }
    );
}
