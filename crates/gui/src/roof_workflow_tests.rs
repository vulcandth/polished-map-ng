use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet, TileCommand};
use std::fs;

fn open(h: &mut Harness<'_, Editor>) {
    h.get_by_label("Tools").click();
    h.run();
    h.get_by_role_and_label(egui::Role::MenuItem, "Change Roof…")
        .click();
    h.run();
}
fn choose(h: &mut Harness<'_, Editor>, previous: &str, name: &str) {
    h.get_by_label(previous).click();
    h.run();
    h.get_by_label(name).click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
}
fn current<'a>(h: &'a Harness<'_, Editor>) -> &'a MasterTilesetDocument {
    h.state().loaded.as_ref().unwrap().tileset.as_ref().unwrap()
}

#[test]
fn roof_switch_updates_rendering_and_preserves_other_dirty_domains() {
    let (dir, mut h) = super::roof_editor_tests::fixture();
    fs::write(
        dir.path().join("gfx/tilesets/roofs/new.2bpp"),
        [255; 9 * 16],
    )
    .unwrap();
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    let loaded = h.state_mut().loaded.as_mut().unwrap();
    loaded
        .blockset
        .as_mut()
        .unwrap()
        .blockset_mut()
        .apply_batch(&[polished_map_core::BlockCommand::Set {
            block: 1,
            tiles: [0x0b; 16],
        }])
        .unwrap();
    let doc = loaded.tileset.as_mut().unwrap();
    doc.apply_batch(&[TileCommand::Paint {
        tile: 0,
        points: vec![Point { x: 0, y: 0 }],
        hue: 1,
    }])
    .unwrap();
    doc.palettes_mut()
        .apply_batch(&[PaletteCommand::SetRgb {
            slot: ColorSlot {
                set: PaletteSet::Day,
                palette: 0,
                hue: 3,
            },
            rgb: [23, 45, 67],
        }])
        .unwrap();
    let bank = doc.palettes().bank().clone();
    let base = doc.tileset().tiles().to_vec();
    open(&mut h);
    choose(&mut h, "demo", "new");
    assert!(h.state().error.is_none());
    assert_eq!(current(&h).roof().unwrap().name(), "new");
    assert!(!current(&h).roof().unwrap().roof().tileset().is_modified());
    assert_eq!(
        &h.state()
            .loaded
            .as_ref()
            .unwrap()
            .graphics
            .render_block(0)
            .unwrap()
            .pixels[..3],
        &[23, 45, 67]
    );
    open(&mut h);
    choose(&mut h, "new", "(none)");
    assert!(current(&h).roof().is_none());
    assert_eq!(current(&h).tileset().tiles(), base);
    assert!(current(&h).tileset().is_modified());
    assert_eq!(current(&h).palettes().bank(), &bank);
    assert!(current(&h).palettes().is_modified());
    let loaded = h.state().loaded.as_ref().unwrap();
    assert!(loaded.blockset.as_ref().unwrap().blockset().is_modified());
    assert_eq!(
        loaded.blockset.as_ref().unwrap().blockset().blocks()[1],
        [0x0b; 16]
    );
    assert!(loaded.document.map().is_modified());
    assert!(loaded.document.map().can_undo());
    assert_eq!(
        &loaded.graphics.render_block(0).unwrap().pixels[..3],
        &polished_map_core::day_palettes()[0][0]
    );
    assert!(!dir.path().join("gfx/tilesets/roofs/new.png").exists());
    open(&mut h);
    choose(&mut h, "(none)", "demo");
    assert_eq!(current(&h).roof().unwrap().name(), "demo");
}

#[test]
fn dirty_roof_warning_and_option_cancel_preserve_edits_and_reload_keeps_dirty_flag() {
    let (_dir, mut h) = super::roof_editor_tests::fixture();
    h.state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .tileset
        .as_mut()
        .unwrap()
        .roof_mut()
        .unwrap()
        .roof_mut()
        .apply_batch(&[TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        }])
        .unwrap();
    for proceed in [false, true] {
        open(&mut h);
        assert!(h.state().confirm_roof_change);
        if proceed {
            h.get_by_label("Change Roof").click();
            h.run();
            assert!(h.state().roof_choice_form.is_some());
        }
        h.get_by_label("Cancel").click();
        h.run();
        assert_eq!(
            current(&h).roof().unwrap().roof().tileset().tiles()[0].hues()[0],
            3
        );
        assert!(current(&h).roof().unwrap().roof().tileset().is_modified());
    }
    open(&mut h);
    h.get_by_label("Change Roof").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(
        current(&h).roof().unwrap().roof().tileset().tiles()[0].hues()[0],
        0
    );
    assert!(current(&h).roof().unwrap().roof().tileset().is_modified());
    open(&mut h);
    h.get_by_label("Change Roof").click();
    h.run();
    choose(&mut h, "demo", "(none)");
    assert!(!h.state().is_modified());
}

#[test]
fn roof_load_failure_is_atomic_and_source_geometry_matches_both_button_orders() {
    let (dir, mut h) = super::roof_editor_tests::fixture();
    fs::write(dir.path().join("gfx/tilesets/roofs/bad.png"), b"bad").unwrap();
    let before = current(&h).roof().unwrap().content_hash();
    let image = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .graphics
        .render_block(0)
        .unwrap();
    open(&mut h);
    for windows in [true, false] {
        h.state_mut()
            .roof_choice_form
            .as_mut()
            .unwrap()
            .windows_buttons = windows;
        h.run();
        let area = h.state().roof_choice_form.as_ref().unwrap().bounds;
        assert_eq!(area.size(), Vec2::new(280.0, 80.0));
        let ok = h.get_by_label("OK").rect();
        let cancel = h.get_by_label("Cancel").rect();
        assert_eq!(ok.size(), Vec2::new(80.0, 22.0));
        assert_eq!(cancel.size(), ok.size());
        assert_eq!(
            ok.min - area.min,
            Vec2::new(if windows { 96.0 } else { 190.0 }, 48.0)
        );
        assert_eq!(
            cancel.min - area.min,
            Vec2::new(if windows { 190.0 } else { 96.0 }, 48.0)
        );
        #[cfg(feature = "visual-tests")]
        {
            h.remove_cursor();
            h.run();
            h.snapshot(if windows {
                "roof-choice-windows"
            } else {
                "roof-choice-unix"
            });
        }
    }
    choose(&mut h, "demo", "bad");
    assert!(h.state().error.is_some());
    assert_eq!(current(&h).roof().unwrap().content_hash(), before);
    assert_eq!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .graphics
            .render_block(0)
            .unwrap(),
        image
    );
    assert!(h.state().roof_choice_form.is_some());
}

#[test]
fn roof_shortcut_and_escape_do_not_reload_or_edit_the_map() {
    let (_dir, mut h) = super::roof_editor_tests::fixture();
    h.key_press_modifiers(
        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        egui::Key::H,
    );
    h.run();
    assert!(h.state().roof_choice_form.is_some());
    h.get_by_label("demo").click();
    h.run();
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(
        h.state().roof_choice_form.is_some(),
        "Escape only closes the dropdown first"
    );
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(h.state().roof_choice_form.is_none());
    assert!(!h.state().is_modified());
    assert_eq!(current(&h).roof().unwrap().name(), "demo");
}
