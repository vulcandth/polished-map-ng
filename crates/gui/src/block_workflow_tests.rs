use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{BlockCommand, CollisionRow, Tile, day_palettes};

fn pointer(h: &mut Harness<'_, Editor>, pos: Pos2, button: egui::PointerButton, pressed: bool) {
    h.hover_at(pos);
    h.event_modifiers(
        egui::Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Modifiers::NONE,
    );
    h.run();
}

pub(super) fn open_block(h: &mut Harness<'_, Editor>, id: u8) {
    let size = if h.state().zoom { 64.0 } else { 32.0 };
    let pos = Pos2::new(
        (f32::from(id % 4) + 0.5) * size,
        geometry::MENU_HEIGHT + geometry::TOOLBAR_HEIGHT + (f32::from(id / 4) + 0.5) * size
            - h.state().sidebar_scroll,
    );
    pointer(h, pos, egui::PointerButton::Secondary, true);
    pointer(h, pos, egui::PointerButton::Secondary, false);
    assert_eq!(h.state().selected, id);
    assert!(
        h.state().block_form.is_some(),
        "right-click must open the real block editor"
    );
}

#[test]
fn sidebar_releases_match_original_buttons_and_right_click_edits_the_released_block() {
    for zoom in [false, true] {
        for transfers in [false, true] {
            let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
            h.state_mut().zoom = zoom;
            h.state_mut().drag_and_drop = transfers;
            h.run();
            let size = if zoom { 64.0 } else { 32.0 };
            let pos = Pos2::new(size * 1.5, 47.0 + size * 0.5);
            for button in [egui::PointerButton::Primary, egui::PointerButton::Secondary] {
                h.state_mut().selected = 0;
                h.run();
                pointer(&mut h, pos, button, true);
                assert_eq!(h.state().selected, 0, "selection waits for release");
                assert!(h.state().block_form.is_none());
                pointer(&mut h, pos, button, false);
                assert_eq!(h.state().selected, 1);
                assert_eq!(
                    h.state().block_form.is_some(),
                    button == egui::PointerButton::Secondary
                );
                if button == egui::PointerButton::Secondary {
                    h.get_by_label("Cancel").click();
                    h.run();
                }
                assert!(h.state().sidebar_press.is_none());
            }
            assert!(!h.state().is_modified());
            assert_eq!(
                std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
                [0, 1]
            );
        }
    }
}

#[test]
fn sidebar_release_cannot_cross_cell_boundaries_or_escape_into_blank_space() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().drag_and_drop = false;
    h.run();
    // Two pixels across the cell border is below egui's usual click-drag
    // threshold, but must not act as a click on a different original button.
    for button in [egui::PointerButton::Primary, egui::PointerButton::Secondary] {
        for (from, to) in [
            (Pos2::new(31.0, 63.0), Pos2::new(33.0, 63.0)),
            (Pos2::new(48.0, 63.0), Pos2::new(80.0, 63.0)),
            (Pos2::new(48.0, 63.0), Pos2::new(48.0, 90.0)),
            (Pos2::new(48.0, 63.0), Pos2::new(200.0, 63.0)),
        ] {
            pointer(&mut h, from, button, true);
            pointer(&mut h, to, button, false);
            assert_eq!(h.state().selected, 0);
            assert!(h.state().block_form.is_none() && h.state().sidebar_press.is_none());
        }
    }
    h.state_mut().prepare_resize();
    h.run();
    for pressed in [true, false] {
        pointer(
            &mut h,
            Pos2::new(48.0, 63.0),
            egui::PointerButton::Secondary,
            pressed,
        );
    }
    assert!(h.state().block_form.is_none() && h.state().sidebar_press.is_none());
    assert_eq!(h.state().selected, 0);
    h.get_by_label("Cancel").click();
    h.run();
    open_block(&mut h, 1);
    h.get_by_label("Cancel").click();
    h.run();
    assert!(!h.state().is_modified());
}

#[test]
fn sidebar_right_release_can_return_to_its_source_and_pointer_loss_cancels_the_press() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let pos = Pos2::new(48.0, 63.0);
    pointer(&mut h, pos, egui::PointerButton::Secondary, true);
    h.hover_at(Pos2::new(200.0, 63.0));
    h.run();
    pointer(&mut h, pos, egui::PointerButton::Secondary, false);
    assert_eq!(h.state().selected, 1);
    assert!(
        h.state().block_form.is_some(),
        "source does not require event_is_click for this callback"
    );
    h.get_by_label("Cancel").click();
    h.run();
    h.state_mut().selected = 0;
    pointer(&mut h, pos, egui::PointerButton::Secondary, true);
    h.input_mut().events.push(egui::Event::PointerGone);
    h.run();
    assert!(h.state().sidebar_press.is_none());
    pointer(&mut h, pos, egui::PointerButton::Secondary, false);
    assert!(h.state().block_form.is_none());
    assert_eq!(h.state().selected, 0);
}

#[test]
fn scrolled_sidebar_releases_respect_the_visible_clip_boundary() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let ctx = h.ctx.clone();
    assert!(
        h.state_mut()
            .apply_block_commands(&ctx, &[BlockCommand::Resize { count: 64 }])
    );
    h.set_size(Vec2::new(335.0, 262.0));
    h.run();
    h.hover_at(Pos2::new(80.0, 100.0));
    h.run();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: Vec2::new(0.0, -13.0),
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    assert_eq!(h.state().sidebar_scroll, 13.0);
    h.state_mut().selected = 1;
    // Row zero now spans y=34..66 but only y>=47 belongs to the sidebar.
    pointer(
        &mut h,
        Pos2::new(16.0, 50.0),
        egui::PointerButton::Secondary,
        true,
    );
    pointer(
        &mut h,
        Pos2::new(16.0, 45.0),
        egui::PointerButton::Secondary,
        false,
    );
    assert_eq!(h.state().selected, 1);
    assert!(h.state().block_form.is_none());
    for pressed in [true, false] {
        pointer(
            &mut h,
            Pos2::new(16.0, 50.0),
            egui::PointerButton::Secondary,
            pressed,
        );
    }
    assert_eq!(h.state().selected, 0);
    assert!(h.state().block_form.is_some());
}

fn fixture() -> (tempfile::TempDir, Harness<'static, Editor>) {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("Map.2x1.blk");
    let blocks = dir.path().join("blocks.bin");
    let collisions = dir.path().join("collisions.bin");
    std::fs::write(&map, [0, 2]).unwrap();
    std::fs::write(&blocks, [[0; 16], [1; 16], [2; 16]].concat()).unwrap();
    std::fs::write(&collisions, [[0; 4], [1; 4], [2; 4]].concat()).unwrap();
    let loaded = Loaded {
        recent_path: None,
        trailing_blocks: 0,
        palette_plan: None,
        events: None,
        event_warning: None,
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
    h.get_by_label(if label == "Resize Blockset…" {
        "Tools"
    } else {
        "Edit"
    })
    .click();
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
