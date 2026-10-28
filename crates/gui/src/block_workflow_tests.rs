use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{BlockCommand, CollisionRow, Tile, day_palettes};

fn fixture() -> (tempfile::TempDir, Harness<'static, Editor>) {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("Map.2x1.blk");
    let blocks = dir.path().join("blocks.bin");
    let collisions = dir.path().join("collisions.bin");
    std::fs::write(&map, [0, 2]).unwrap();
    std::fs::write(&blocks, [[0; 16], [1; 16], [2; 16]].concat()).unwrap();
    std::fs::write(&collisions, [[0; 4], [1; 4], [2; 4]].concat()).unwrap();
    let loaded = Loaded {
        palette_plan: None,
        document: MapDocument::open(&map, &OpenOptions::default()).unwrap(),
        graphics: Graphics::new(
            (0..3).map(|i| Tile::new([i; 64]).unwrap()).collect(),
            vec![day_palettes()[0]; 3],
            vec![[0; 16], [1; 16], [2; 16]],
        )
        .unwrap(),
        blockset: Some(BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap()),
        tileset: None,
        allow_256: true,
    };
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new(cc));
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(loaded)).unwrap();
    harness.state_mut().loading = Some(rx);
    harness.run();
    (dir, harness)
}

fn assert_blocks(h: &Harness<'_, Editor>, ids: &[u8]) {
    let loaded = h.state().loaded.as_ref().unwrap();
    let blocks = loaded.blockset.as_ref().unwrap().blockset();
    assert_eq!(
        blocks.blocks(),
        ids.iter().map(|&id| [id; 16]).collect::<Vec<_>>()
    );
    assert_eq!(
        blocks.collisions().unwrap(),
        ids.iter()
            .map(|&id| CollisionRow::Binary([id; 4]))
            .collect::<Vec<_>>()
    );
    assert_eq!(loaded.document.map().blocks(), [0, 2]);
    assert!(!loaded.document.map().is_modified());
    assert!(!loaded.document.map().can_undo());
    assert_eq!(h.state().textures.len(), ids.len());
    assert_eq!(loaded.graphics.block_count(), ids.len());
    for (i, &id) in ids.iter().enumerate() {
        let image = loaded.graphics.render_block(i as u8).unwrap();
        assert_eq!(
            &image.pixels[..3],
            &loaded.graphics.render_tile(id).unwrap().pixels[..3]
        );
    }
}

fn menu(h: &mut Harness<'_, Editor>, label: &str) {
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label(label).click();
    h.run();
}

#[test]
fn clipboard_snapshot_paste_and_live_id_swap_preserve_collisions_and_map() {
    let (dir, mut h) = fixture();
    // Copy remembers block zero's value. Swap later uses the live source ID.
    menu(&mut h, "Copy Block");
    assert!(!h.state().is_modified());
    h.state_mut().selected = 1;
    menu(&mut h, "Swap Block");
    assert_blocks(&h, &[1, 0, 2]);
    h.state_mut().selected = 2;
    menu(&mut h, "Paste Block");
    assert_blocks(&h, &[1, 0, 0]);
    // Native clipboard events operate on the internal snapshot, not external text.
    h.state_mut().selected = 0;
    h.input_mut().events.push(egui::Event::Copy);
    h.run();
    h.state_mut().selected = 2;
    h.input_mut()
        .events
        .push(egui::Event::Paste("ignored external text".into()));
    h.run();
    assert_blocks(&h, &[1, 0, 1]);
    h.state_mut().selected = 1;
    h.input_mut().events.push(egui::Event::Cut);
    h.run();
    assert_blocks(&h, &[0, 1, 1]);
    h.state_mut().save_blockset();
    assert!(h.state().error.is_none());
    assert_eq!(
        std::fs::read(dir.path().join("blocks.bin")).unwrap(),
        [[0; 16], [1; 16], [1; 16]].concat()
    );
    assert_eq!(
        std::fs::read(dir.path().join("collisions.bin")).unwrap(),
        [[0; 4], [1; 4], [1; 4]].concat()
    );
}

#[test]
fn sidebar_drag_swaps_ctrl_drag_copies_and_invalid_drops_do_nothing() {
    let (_dir, mut h) = fixture();
    for (from, to, ctrl, expected) in [
        (
            Pos2::new(16.0, 63.0),
            Pos2::new(48.0, 63.0),
            false,
            [1, 0, 2],
        ),
        (
            Pos2::new(16.0, 63.0),
            Pos2::new(80.0, 63.0),
            true,
            [1, 0, 1],
        ),
        (
            Pos2::new(16.0, 63.0),
            Pos2::new(112.0, 63.0),
            false,
            [1, 0, 1],
        ),
        (
            Pos2::new(16.0, 63.0),
            Pos2::new(256.0, 159.0),
            false,
            [1, 0, 1],
        ),
    ] {
        h.hover_at(from);
        h.drag_at(from);
        h.run();
        h.hover_at(to);
        h.run();
        let modifiers = if ctrl {
            egui::Modifiers::CTRL
        } else {
            egui::Modifiers::NONE
        };
        h.event_modifiers(
            egui::Event::PointerButton {
                pos: to,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers,
            },
            modifiers,
        );
        h.run();
        assert_blocks(&h, &expected);
        assert!(h.state().sidebar_drag.is_none());
    }
    assert_eq!(h.state().selected, 2);
}

#[test]
fn resize_blockset_cancel_noop_shrink_grow_and_clipboard_invalidation() {
    let (_dir, mut h) = fixture();
    menu(&mut h, "Resize Blockset…");
    #[cfg(feature = "visual-tests")]
    {
        h.state_mut().status = "Synthetic blockset resize".into();
        let original = h.state().block_count_windows_buttons;
        for windows in [true, false] {
            h.state_mut().block_count_windows_buttons = windows;
            h.remove_cursor();
            h.run();
            h.snapshot(if windows {
                "block-count-windows"
            } else {
                "block-count-unix"
            });
        }
        h.state_mut().block_count_windows_buttons = original;
        h.run();
    }
    h.get_by_label("OK").click();
    h.run();
    assert!(!h.state().is_modified());
    h.state_mut().selected = 2;
    menu(&mut h, "Copy Block");
    menu(&mut h, "Resize Blockset…");
    h.get_by_label("Block count").set_value_accesskit(1.0);
    h.run();
    h.get_by_label("Cancel").click();
    h.run();
    assert_blocks(&h, &[0, 1, 2]);
    assert!(h.state().block_clipboard.is_some());
    menu(&mut h, "Resize Blockset…");
    h.get_by_label("Block count").set_value_accesskit(1.0);
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_blocks(&h, &[0]);
    assert_eq!(h.state().selected, 0);
    assert!(h.state().block_clipboard.is_none());
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::B);
    h.run();
    h.get_by_label("Block count").set_value_accesskit(256.0);
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_blocks(&h, &[0; 256]);
    h.state_mut().save_blockset();
    assert!(h.state().error.is_none());
    assert!(!h.state().is_modified());
}

#[test]
fn failed_block_preview_validation_preserves_document_and_textures() {
    let (_dir, mut h) = fixture();
    let ctx = h.ctx.clone();
    assert!(!h.state_mut().apply_block_commands(
        &ctx,
        &[BlockCommand::Set {
            block: 0,
            tiles: [255; 16]
        }]
    ));
    assert_blocks(&h, &[0, 1, 2]);
    assert!(!h.state().is_modified());
    assert!(h.state().error.is_some());
}

#[test]
fn private_paste_works_with_empty_system_clipboard_and_does_not_repeat_on_release() {
    let (_dir, mut h) = fixture();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::C);
    h.run();
    h.state_mut().selected = 1;
    // egui-winit produces only this release when the OS clipboard has no contents.
    h.key_up_modifiers(egui::Modifiers::COMMAND, egui::Key::V);
    h.run();
    assert_blocks(&h, &[0, 0, 2]);
    h.state_mut().selected = 2;
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::C);
    h.run();
    h.state_mut().selected = 1;
    h.input_mut()
        .events
        .push(egui::Event::Paste("system text".into()));
    h.run();
    assert_blocks(&h, &[0, 2, 2]);
    // A following key release must not paste a second time.
    h.state_mut().selected = 0;
    h.key_up_modifiers(egui::Modifiers::COMMAND, egui::Key::V);
    h.run();
    assert_blocks(&h, &[0, 2, 2]);
}
