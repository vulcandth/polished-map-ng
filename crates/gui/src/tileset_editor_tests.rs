use super::*;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use polished_map_core::TileCommand;
use std::fs;

fn fixture(allow_256: bool, monochrome: bool) -> (tempfile::TempDir, Harness<'static, Editor>) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("gfx/tilesets")).unwrap();
    fs::create_dir_all(root.join("data/tilesets")).unwrap();
    fs::write(root.join("Map.2x1.blk"), [0, 1]).unwrap();
    fs::write(
        root.join("gfx/tilesets/demo.2bpp"),
        [[0, 255].repeat(8), [255, 0].repeat(8), vec![255; 16]].concat(),
    )
    .unwrap();
    fs::write(
        root.join("gfx/tilesets/demo_palette_map.asm"),
        "\ttilepal 0, GRAY, RED, PRIORITY_GREEN, TEXT\n",
    )
    .unwrap();
    fs::write(
        root.join("data/tilesets/demo_metatiles.bin"),
        [[0; 16], [1; 16]].concat(),
    )
    .unwrap();
    let document = MapDocument::open(&root.join("Map.2x1.blk"), &OpenOptions::default()).unwrap();
    let blocks = BlocksetDocument::open_in_project(root, "demo").unwrap();
    let tileset = MasterTilesetDocument::open_in_project(
        root,
        "demo",
        &MasterGraphicsOptions {
            allow_256,
            monochrome,
            ..Default::default()
        },
    )
    .unwrap();
    let graphics = tileset.graphics(blocks.blockset()).unwrap();
    let mut h = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new(cc));
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(Loaded {
        document,
        blockset: Some(blocks),
        tileset: Some(tileset),
        graphics,
        allow_256,
    }))
    .unwrap();
    h.state_mut().loading = Some(rx);
    h.run();
    h.state_mut().status = "Synthetic tileset".into();
    h.run();
    (dir, h)
}

fn open(h: &mut Harness<'_, Editor>) {
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Edit Tileset…").click();
    h.run();
}
fn tile<'a>(h: &'a Harness<'_, Editor>, id: usize) -> &'a [u8; 64] {
    h.state()
        .tileset_form
        .as_ref()
        .unwrap()
        .draft
        .tileset()
        .tiles()[id]
        .hues()
}

#[test]
fn tileset_draft_cancel_accept_preview_save_and_conflict_preserve_map_history() {
    let (dir, mut h) = fixture(false, false);
    let source = fs::read(dir.path().join("gfx/tilesets/demo.2bpp")).unwrap();
    for accept in [false, true] {
        open(&mut h);
        h.get_by_label("Swatch 4").click();
        h.run();
        h.get_by_label("Pixel 0, 0").click();
        h.run();
        assert_eq!(tile(&h, 0)[0], 3);
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num4);
        h.run();
        assert_eq!(
            h.state()
                .tileset_form
                .as_ref()
                .unwrap()
                .draft
                .tileset()
                .attributes()[0],
            3
        );
        assert!(!h.state().is_modified()); // draft is isolated until OK
        h.get_by_label(if accept { "OK" } else { "Cancel" }).click();
        h.run();
        let loaded = h.state().loaded.as_ref().unwrap();
        assert_eq!(
            loaded.tileset.as_ref().unwrap().tileset().tiles()[0].hues()[0],
            if accept { 3 } else { 1 }
        );
        assert!(!loaded.document.map().is_modified());
        assert!(!loaded.document.map().can_undo());
        assert!(!loaded.blockset.as_ref().unwrap().blockset().is_modified());
        assert_eq!(h.state().is_modified(), accept);
        if accept {
            assert_eq!(
                &loaded.graphics.render_block(0).unwrap().pixels[..3],
                &polished_map_core::day_palettes()[3][3]
            );
        }
    }
    h.get_by_label("File").click();
    h.run();
    h.get_by_label("Save Tileset").click();
    h.run();
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert!(!h.state().is_modified());
    let reopened = MasterTilesetDocument::open_in_project(
        dir.path(),
        "demo",
        &MasterGraphicsOptions::default(),
    )
    .unwrap();
    assert_eq!(reopened.tileset().tiles()[0].hues()[0], 3);
    assert_eq!(reopened.tileset().attributes()[0], 3);
    assert_eq!(
        fs::read(dir.path().join("gfx/tilesets/demo.2bpp")).unwrap(),
        source
    );
    open(&mut h);
    h.get_by_label("OK").click();
    h.run(); // unchanged acceptance is still dirty
    assert!(h.state().is_modified());
    fs::write(dir.path().join("gfx/tilesets/demo.png"), "external").unwrap();
    h.state_mut().save_tileset();
    assert!(h.state().error.as_ref().unwrap().contains("changed"));
    assert!(h.state().is_modified());
    assert_eq!(
        fs::read(dir.path().join("gfx/tilesets/demo.png")).unwrap(),
        b"external"
    );
    h.state_mut().error = None;
    h.state_mut().choose_new();
    assert!(h.state().confirm_discard);
}

#[test]
fn pixel_gestures_palette_priority_and_delete_match_shared_commands() {
    let (_dir, mut h) = fixture(false, false);
    open(&mut h);
    h.get_by_role(egui::Role::ComboBox).click();
    h.run();
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(
        h.state().tileset_form.is_some(),
        "Escape first dismisses the palette popup"
    );
    h.get_by_label("Swatch 4").click();
    h.run();
    h.get_by_label("Pixel 0, 0").click();
    h.run();
    let expected = h
        .state()
        .tileset_form
        .as_ref()
        .unwrap()
        .draft
        .tileset()
        .clone();
    for (modifiers, op) in [
        (
            egui::Modifiers::SHIFT,
            TileCommand::Fill {
                tile: 0,
                at: Point { x: 1, y: 0 },
                hue: 2,
            },
        ),
        (
            egui::Modifiers::CTRL,
            TileCommand::Replace {
                tile: 0,
                from: 1,
                to: 2,
            },
        ),
        (
            egui::Modifiers::ALT,
            TileCommand::SwapHues {
                tile: 0,
                a: 1,
                b: 2,
            },
        ),
    ] {
        // Restore through public commands; event behavior is compared to the
        // separately C++-verified model, not a second implementation of pixels.
        h.state_mut()
            .tileset_form
            .as_mut()
            .unwrap()
            .draft
            .apply_batch(&[TileCommand::Set {
                tile: 0,
                value: expected.snapshot(0).unwrap(),
            }])
            .unwrap();
        h.get_by_label("Swatch 2").click();
        h.run(); // light=2, not dark=1
        let mut wanted = expected.clone();
        wanted.apply(&op).unwrap();
        h.get_by_label("Pixel 1, 0").click_modifiers(modifiers);
        h.run();
        assert_eq!(tile(&h, 0), wanted.tiles()[0].hues());
    }
    h.get_by_label("Pixel 0, 0").click_secondary();
    h.run(); // pick black
    h.get_by_label("Pixel 7, 7").click();
    h.run();
    assert_eq!(tile(&h, 0)[63], 3);
    h.get_by_label("Priority (above sprites)").click();
    h.run();
    assert_eq!(
        h.state()
            .tileset_form
            .as_ref()
            .unwrap()
            .draft
            .tileset()
            .attributes()[0],
        0x80
    );
    h.key_press(egui::Key::Delete);
    h.run();
    assert_eq!(tile(&h, 0), &[0; 64]);
    assert_eq!(
        h.state()
            .tileset_form
            .as_ref()
            .unwrap()
            .draft
            .tileset()
            .attributes()[0],
        9
    );
    assert!(
        h.get_by_label("Priority (above sprites)")
            .accesskit_node()
            .is_disabled()
    );
    h.get_by_label("Cancel").click();
    h.run();
    assert!(!h.state().is_modified());
}

#[test]
fn tile_clipboard_snapshot_live_swap_empty_clipboard_and_drag_are_isolated() {
    let (_dir, mut h) = fixture(false, false);
    open(&mut h);
    h.input_mut().events.push(egui::Event::Copy);
    h.run();
    h.get_by_label("Tileset tile $01").click();
    h.run();
    h.input_mut().events.push(egui::Event::Cut);
    h.run();
    assert_eq!(tile(&h, 0), &[2; 64]);
    assert_eq!(tile(&h, 1), &[1; 64]);
    h.input_mut().events.push(egui::Event::Cut);
    h.run();
    assert_eq!(tile(&h, 0), &[2; 64]); // native auto-repeat must not swap back
    h.input_mut().events.push(egui::Event::Key {
        key: egui::Key::X,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND,
    });
    h.run();
    h.get_by_label("Tileset tile $02").click();
    h.run();
    h.input_mut()
        .events
        .push(egui::Event::Paste("external text is ignored".into()));
    h.run();
    assert_eq!(tile(&h, 2), &[1; 64]);
    h.input_mut().events.push(egui::Event::Key {
        key: egui::Key::V,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND,
    });
    h.run();
    h.get_by_label("Tileset tile $00").click();
    h.run();
    h.input_mut().events.push(egui::Event::Key {
        key: egui::Key::V,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND,
    });
    h.run();
    assert_eq!(tile(&h, 0), &[1; 64]);
    for (from, to, ctrl) in [
        ("Tileset tile $00", "Tileset tile $03", false),
        ("Tileset tile $03", "Tileset tile $01", true),
    ] {
        let start = h.get_by_label(from).rect().center();
        let end = h.get_by_label(to).rect().center();
        h.hover_at(start);
        h.drag_at(start);
        h.run();
        h.hover_at(end);
        h.run();
        let modifiers = if ctrl {
            egui::Modifiers::CTRL
        } else {
            egui::Modifiers::NONE
        };
        h.event_modifiers(
            egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers,
            },
            modifiers,
        );
        h.run();
    }
    assert_eq!(tile(&h, 0), &[0; 64]);
    assert_eq!(tile(&h, 3), &[1; 64]);
    assert_eq!(tile(&h, 1), &[1; 64]);
    assert!(!h.state().is_modified());
}

#[test]
fn tileset_geometry_and_mode_restrictions_follow_source() {
    for (allow_256, mono) in [(false, false), (false, true), (true, true)] {
        let (_dir, mut h) = fixture(allow_256, mono);
        open(&mut h);
        let first = h.get_by_label("Tileset tile $00").rect();
        assert!(
            h.state().tileset_form.as_ref().unwrap().bounds.height() < 370.0,
            "positioned cells must not expand the 304-point client area"
        );
        let last = h.get_by_label("Tileset tile $FF").rect();
        assert_eq!(first.size(), Vec2::splat(16.0));
        assert_eq!(last.min - first.min, Vec2::splat(240.0));
        let pixel = h.get_by_label("Pixel 0, 0").rect();
        assert_eq!(pixel.size(), Vec2::splat(18.0));
        assert_eq!(pixel.min - first.min, Vec2::new(268.0, 0.0));
        assert_eq!(
            h.get_by_label("Pixel 7, 7").rect().min - pixel.min,
            Vec2::splat(126.0)
        );
        assert_eq!(
            h.get_by_label("Tileset tile $60")
                .accesskit_node()
                .is_disabled(),
            !allow_256
        );
        assert_eq!(
            h.get_by_label("Tileset tile $80")
                .accesskit_node()
                .is_disabled(),
            !allow_256 && mono
        );
        if !allow_256 && !mono {
            #[cfg(feature = "visual-tests")]
            for windows in [true, false] {
                h.state_mut().tileset_form.as_mut().unwrap().windows_buttons = windows;
                h.remove_cursor();
                h.run();
                h.snapshot(if windows {
                    "tileset-dialog-windows"
                } else {
                    "tileset-dialog-unix"
                });
            }
        }
    }
}

#[test]
fn image_clipboard_events_use_grayscale_output_and_shared_clipped_import() {
    let (_dir, mut h) = fixture(false, false);
    open(&mut h);
    h.input_mut().events.push(egui::Event::Key {
        key: egui::Key::C,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
    });
    h.step();
    let copied = h
        .output()
        .platform_output
        .commands
        .iter()
        .find_map(|command| {
            if let egui::OutputCommand::CopyImage(image) = command {
                Some(image)
            } else {
                None
            }
        })
        .expect("image-copy shortcut must issue an image output command");
    assert_eq!(copied.size, [8, 8]);
    assert!(
        copied
            .pixels
            .iter()
            .all(|pixel| pixel.to_array() == [85, 85, 85, 255])
    );
    h.run();
    h.get_by_label("Tileset tile $5F").click();
    h.run();
    let source = egui::ColorImage::filled([8, 16], Color32::from_rgb(240, 30, 10));
    h.event_modifiers(
        egui::Event::PasteImage(std::sync::Arc::new(source)),
        egui::Modifiers::SHIFT,
    );
    h.run();
    // Origin row 5 skips reserved rows 6/7; grayscale rounds (31R+61G+8B)/100.
    assert_eq!(tile(&h, 0x5f), &[1; 64]);
    assert_eq!(tile(&h, 0x8f), &[1; 64]);
    assert_eq!(tile(&h, 0x6f), &[0; 64]);
    h.get_by_label("OK").click();
    h.run();
    let loaded = h.state().loaded.as_ref().unwrap();
    assert_eq!(
        loaded.tileset.as_ref().unwrap().tileset().tiles()[0x8f].hues(),
        &[1; 64]
    );
    assert!(!loaded.document.map().is_modified());
}
