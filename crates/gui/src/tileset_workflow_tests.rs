use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{BlockCommand, ColorSlot, PaletteCommand, PaletteSet, TileCommand};
use std::fs;

fn fixture() -> (tempfile::TempDir, Harness<'static, Editor>) {
    let (dir, h) = super::roof_editor_tests::fixture();
    let root = dir.path();
    fs::create_dir_all(root.join("constants")).unwrap();
    fs::write(
        root.join("constants/tileset_constants.asm"),
        "\tconst TILESET_OTHER\n",
    )
    .unwrap();
    fs::write(root.join("gfx/tilesets/01.2bpp"), [255; 32 * 16]).unwrap();
    fs::write(
        root.join("gfx/tilesets/01_palette_map.asm"),
        "\ttilepal 0, RED\n".repeat(32),
    )
    .unwrap();
    fs::write(root.join("data/tilesets/01_metatiles.bin"), [0; 16]).unwrap();
    (dir, h)
}
fn open(h: &mut Harness<'_, Editor>) {
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::H);
    h.run();
}
fn choose(h: &mut Harness<'_, Editor>, old: &str, next: &str) {
    h.get_by_label(old).click();
    h.run();
    h.get_by_label(next).click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
}
fn current<'a>(h: &'a Harness<'_, Editor>) -> &'a MasterTilesetDocument {
    h.state().loaded.as_ref().unwrap().tileset.as_ref().unwrap()
}

#[test]
fn tileset_switch_preserves_map_history_and_palette_edits_and_clears_invalid_selection() {
    let (dir, mut h) = fixture();
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.state_mut().selected = 1;
    h.input_mut().events.push(egui::Event::Copy);
    h.run();
    assert!(h.state().block_clipboard.is_some());
    h.state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .tileset
        .as_mut()
        .unwrap()
        .palettes_mut()
        .apply_batch(&[PaletteCommand::SetRgb {
            slot: ColorSlot {
                set: PaletteSet::Day,
                palette: 1,
                hue: 3,
            },
            rgb: [34, 56, 78],
        }])
        .unwrap();
    let bank = current(&h).palettes().bank().clone();
    open(&mut h);
    assert!(
        !h.state().confirm_tileset_change,
        "map and palette edits survive without warning"
    );
    choose(&mut h, "demo", "01: OTHER");
    assert!(h.state().error.is_none());
    let loaded = h.state().loaded.as_ref().unwrap();
    assert_eq!(
        loaded.document.map().blocks(),
        &[1, 1],
        "missing definition does not replace the map ID"
    );
    assert!(loaded.document.map().is_modified());
    assert!(loaded.document.map().can_undo());
    assert_eq!(
        &loaded.graphics.render_block(0).unwrap().pixels[..3],
        &[34, 56, 78]
    );
    assert_eq!(current(&h).name(), "01");
    assert_eq!(current(&h).palettes().bank(), &bank);
    assert!(current(&h).palettes().is_modified());
    assert_eq!(current(&h).roof().unwrap().name(), "demo");
    assert!(!current(&h).tileset().is_modified());
    assert!(!current(&h).roof().unwrap().roof().tileset().is_modified());
    assert!(h.state().block_clipboard.is_none());
    assert_eq!(h.state().selected, 0);
    assert!(!dir.path().join("gfx/tilesets/01.png").exists());
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
    h.run();
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        &[0, 1]
    );
    h.input_mut().events.push(egui::Event::Copy);
    h.run();
    open(&mut h);
    choose(&mut h, "01: OTHER", "demo");
    assert_eq!(h.state().textures.len(), 2);
    assert!(
        h.state().block_clipboard.is_some(),
        "an in-range copied block remains available after a switch"
    );
    h.state_mut().selected = 1;
    h.input_mut()
        .events
        .push(egui::Event::Paste("ignored".into()));
    h.run();
    assert_eq!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .blockset
            .as_ref()
            .unwrap()
            .blockset()
            .blocks()[1],
        [0; 16]
    );
}

#[test]
fn every_replaced_dirty_domain_warns_and_both_cancel_stages_preserve_edits() {
    let (_dir, mut h) = fixture();
    for domain in 0..3 {
        let loaded = h.state_mut().loaded.as_mut().unwrap();
        let paint = TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        };
        match domain {
            0 => loaded
                .blockset
                .as_mut()
                .unwrap()
                .blockset_mut()
                .apply_batch(&[BlockCommand::Set {
                    block: 0,
                    tiles: [1; 16],
                }])
                .unwrap(),
            1 => loaded
                .tileset
                .as_mut()
                .unwrap()
                .apply_batch(&[paint])
                .unwrap(),
            _ => loaded
                .tileset
                .as_mut()
                .unwrap()
                .roof_mut()
                .unwrap()
                .roof_mut()
                .apply_batch(&[paint])
                .unwrap(),
        }
        for proceed in [false, true] {
            open(&mut h);
            assert!(h.state().confirm_tileset_change);
            if proceed {
                h.get_by_label("Change Tileset").click();
                h.run();
                assert!(h.state().tileset_choice_form.is_some());
            }
            h.get_by_label("Cancel").click();
            h.run();
            assert!(h.state().is_modified());
            assert_eq!(current(&h).name(), "demo");
        }
        // Accepting the unchanged selection reloads assets and clears their edits.
        open(&mut h);
        h.get_by_label("Change Tileset").click();
        h.run();
        h.get_by_label("OK").click();
        h.run();
        assert!(!h.state().is_modified());
    }
}

#[test]
fn missing_replacement_preserves_live_assets_and_source_sized_selector() {
    let (dir, mut h) = fixture();
    h.get_by_label("Tools").click();
    h.run();
    let shortcut = h.ctx.format_shortcut(&egui::KeyboardShortcut::new(
        egui::Modifiers::COMMAND,
        egui::Key::H,
    ));
    h.get_by_label(&format!("Change Tileset… {shortcut}"))
        .click();
    h.run();
    for windows in [true, false] {
        h.state_mut()
            .tileset_choice_form
            .as_mut()
            .unwrap()
            .windows_buttons = windows;
        h.run();
        let area = h.state().tileset_choice_form.as_ref().unwrap().bounds;
        assert_eq!(area.size(), Vec2::new(280.0, 80.0));
        for (label, x) in [
            ("OK", if windows { 96.0 } else { 190.0 }),
            ("Cancel", if windows { 190.0 } else { 96.0 }),
        ] {
            let rect = h.get_by_label(label).rect();
            assert_eq!(rect.size(), Vec2::new(80.0, 22.0));
            assert_eq!(rect.min - area.min, Vec2::new(x, 48.0));
        }
        #[cfg(feature = "visual-tests")]
        {
            h.remove_cursor();
            h.run();
            h.snapshot(if windows {
                "tileset-choice-windows"
            } else {
                "tileset-choice-unix"
            });
        }
    }
    fs::write(dir.path().join("data/tilesets/01_metatiles.bin"), [0; 15]).unwrap();
    let old = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .graphics
        .render_block(0)
        .unwrap();
    choose(&mut h, "demo", "01: OTHER");
    assert!(h.state().error.is_some());
    assert_eq!(current(&h).name(), "demo");
    assert_eq!(h.state().textures.len(), 2);
    assert_eq!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .graphics
            .render_block(0)
            .unwrap(),
        old
    );
    assert!(!h.state().is_modified());
}
