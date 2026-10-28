use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use std::fs;

pub(super) fn fixture() -> (tempfile::TempDir, Harness<'static, Editor>) {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let root = dir.path();
    fs::create_dir_all(root.join("gfx/tilesets/roofs")).unwrap();
    fs::write(root.join("gfx/tilesets/demo.2bpp"), [0; 32 * 16]).unwrap();
    fs::write(
        root.join("gfx/tilesets/demo_palette_map.asm"),
        "\ttilepal 0, GRAY\n".repeat(32),
    )
    .unwrap();
    let tiles: Vec<_> = (0..9)
        .map(|i| polished_map_core::Tile::new([i % 4; 64]).unwrap())
        .collect();
    fs::write(
        root.join("gfx/tilesets/roofs/demo.2bpp"),
        polished_map_project::codecs::encode_2bpp(&tiles),
    )
    .unwrap();
    fs::write(
        root.join("data/tilesets/demo_metatiles.bin"),
        [[0x10; 16], [0x0a; 16]].concat(),
    )
    .unwrap();
    let tileset = MasterTilesetDocument::open_in_project(
        root,
        "demo",
        &MasterGraphicsOptions {
            roof: Some("demo".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let blockset = BlocksetDocument::open_in_project(root, "demo").unwrap();
    let graphics = tileset.graphics(blockset.blockset()).unwrap();
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(Loaded {
        palette_plan: None,
        document: MapDocument::open(&root.join("Map.2x1.blk"), &OpenOptions::default()).unwrap(),
        tileset: Some(tileset),
        blockset: Some(blockset),
        graphics,
        allow_256: false,
    }))
    .unwrap();
    h.state_mut().loading = Some(rx);
    h.run();
    h.state_mut().status = "Synthetic roof".into();
    h.run();
    (dir, h)
}
fn open(h: &mut Harness<'_, Editor>) {
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Edit Roof…").click();
    h.run();
}
fn pixels<'a>(h: &'a Harness<'_, Editor>, id: usize) -> &'a [u8; 64] {
    h.state()
        .tileset_form
        .as_ref()
        .unwrap()
        .draft
        .roof()
        .unwrap()
        .roof()
        .tileset()
        .tiles()[id]
        .hues()
}

#[test]
fn roof_draft_accept_save_conflict_and_dirty_guard_leave_base_and_map_unchanged() {
    let (dir, mut h) = fixture();
    for accept in [false, true] {
        open(&mut h);
        assert_eq!(h.state().tileset_form.as_ref().unwrap().selected, 6);
        h.get_by_label("Swatch 4").click();
        h.run();
        h.get_by_label("Pixel 0, 0").click();
        h.run();
        assert_eq!(pixels(&h, 6)[0], 3);
        h.get_by_label(if accept { "OK" } else { "Cancel" }).click();
        h.run();
        let loaded = h.state().loaded.as_ref().unwrap();
        let doc = loaded.tileset.as_ref().unwrap();
        assert_eq!(
            doc.roof().unwrap().roof().tileset().tiles()[6].hues()[0],
            if accept { 3 } else { 2 }
        );
        assert!(!doc.tileset().is_modified());
        assert!(!loaded.document.map().is_modified());
        assert!(!loaded.document.map().can_undo());
        assert!(!loaded.blockset.as_ref().unwrap().blockset().is_modified());
        assert_eq!(h.state().is_modified(), accept);
        assert_eq!(
            loaded.graphics.render_block(0).unwrap().pixels[..3],
            polished_map_core::day_palettes()[0][if accept { 3 } else { 2 }]
        );
    }
    let base = fs::read(dir.path().join("gfx/tilesets/demo.2bpp")).unwrap();
    let palette = fs::read(dir.path().join("gfx/tilesets/demo_palette_map.asm")).unwrap();
    h.state_mut().save();
    h.run();
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert!(!h.state().is_modified());
    assert!(!dir.path().join("gfx/tilesets/demo.png").exists());
    assert_eq!(
        fs::read(dir.path().join("gfx/tilesets/demo.2bpp")).unwrap(),
        base
    );
    assert_eq!(
        fs::read(dir.path().join("gfx/tilesets/demo_palette_map.asm")).unwrap(),
        palette
    );
    let reopened =
        polished_map_project::RoofDocument::open_in_project(dir.path(), "demo", [0; 9]).unwrap();
    assert_eq!(reopened.roof().tileset().tiles()[6].hues()[0], 3);
    open(&mut h);
    h.get_by_label("OK").click();
    h.run();
    assert!(h.state().is_modified());
    fs::write(dir.path().join("gfx/tilesets/roofs/demo.png"), b"external").unwrap();
    h.get_by_label("File").click();
    h.run();
    h.get_by_label("Save Roof").click();
    h.run();
    assert!(h.state().error.as_ref().unwrap().contains("changed"));
    h.state_mut().error = None;
    h.state_mut().choose_new();
    assert!(h.state().confirm_discard);
}

#[test]
fn roof_clipboards_use_roof_pixels_and_import_only_into_the_selected_tile() {
    let (_dir, mut h) = fixture();
    open(&mut h);
    h.input_mut().events.push(egui::Event::Copy);
    h.run();
    h.get_by_label("Roof tile $0A").click();
    h.run();
    h.input_mut().events.push(egui::Event::Cut);
    h.run();
    assert_eq!(pixels(&h, 0), &[2; 64]);
    assert_eq!(pixels(&h, 6), &[0; 64]);
    h.get_by_label("Roof tile $0B").click();
    h.run();
    h.input_mut()
        .events
        .push(egui::Event::Paste("ignored".into()));
    h.run();
    assert_eq!(pixels(&h, 1), &[2; 64]);
    h.get_by_label("Roof tile $12").click();
    h.run();
    h.event_modifiers(
        egui::Event::PasteImage(std::sync::Arc::new(egui::ColorImage::filled(
            [24, 24],
            Color32::WHITE,
        ))),
        egui::Modifiers::SHIFT,
    );
    h.run();
    assert_eq!(pixels(&h, 8), &[0; 64]);
    assert_eq!(pixels(&h, 7), &[3; 64]);
    h.key_press(egui::Key::Delete);
    h.run();
    assert_eq!(pixels(&h, 8), &[0; 64]);
    assert!(h.state().error.is_none());
    h.get_by_label("Cancel").click();
    h.run();
    assert!(!h.state().is_modified());
}

#[test]
fn roof_layout_uses_reversed_rows_and_original_client_dimensions() {
    let (_dir, mut h) = fixture();
    open(&mut h);
    let top = h.get_by_label("Roof tile $10").rect();
    let bottom = h.get_by_label("Roof tile $0A").rect();
    assert_eq!(top.size(), Vec2::splat(16.0));
    assert_eq!(bottom.min - top.min, Vec2::new(0.0, 32.0));
    assert_eq!(
        h.get_by_label("Roof tile $12").rect().min - top.min,
        Vec2::new(32.0, 0.0)
    );
    assert_eq!(
        h.get_by_label("Pixel 0, 0").rect().min - top.min,
        Vec2::new(60.0, 0.0)
    );
    assert!(h.state().tileset_form.as_ref().unwrap().bounds.width() < 245.0);
    assert!(h.state().tileset_form.as_ref().unwrap().bounds.height() < 270.0);
    #[cfg(feature = "visual-tests")]
    for windows in [true, false] {
        h.state_mut().tileset_form.as_mut().unwrap().windows_buttons = windows;
        h.remove_cursor();
        h.run();
        h.snapshot(if windows {
            "roof-dialog-windows"
        } else {
            "roof-dialog-unix"
        });
    }
}
