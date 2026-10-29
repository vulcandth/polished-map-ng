use super::*;
use egui_kittest::kittest::Queryable;
use polished_map_core::BlockCommand;

#[test]
fn reassignment_is_bijective_and_pruning_keeps_surviving_ids() {
    let mut keys = BlockHotkeys::default();
    keys.assign(1, 7);
    keys.assign(2, 8);
    keys.assign(2, 7);
    assert_eq!(
        keys.0,
        [
            None,
            None,
            Some(7),
            None,
            None,
            None,
            None,
            None,
            None,
            None
        ]
    );
    keys.assign(0, 255);
    keys.assign(9, 0);
    keys.prune(8);
    assert_eq!(keys.digit(255), None);
    assert_eq!(keys.digit(7), Some(2));
    assert_eq!(keys.digit(0), Some(9));
    keys.prune(7);
    assert_eq!(keys.digit(7), None);
}

#[test]
fn digit_events_assign_select_and_clear_without_editing_document() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    for (digit, key) in KEYS.into_iter().enumerate() {
        h.state_mut().selected = 1;
        h.key_press_modifiers(egui::Modifiers::CTRL, key);
        h.run();
        assert_eq!(h.state().block_hotkeys.digit(1), Some(digit));
        h.state_mut().selected = 0;
        h.key_press(key);
        h.run();
        assert_eq!(h.state().selected, 1);
        h.key_press_modifiers(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, key);
        h.run();
        assert_eq!(h.state().block_hotkeys.digit(1), None);
        h.state_mut().selected = 0;
        h.key_press(key);
        h.run();
        assert_eq!(h.state().selected, 0);
    }
    assert!(!h.state().is_modified());
    assert_eq!(
        std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
        [0, 1]
    );
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        [0, 1]
    );
}

#[test]
fn per_event_modifiers_control_consumption_and_mac_command_is_not_control() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let editor = h.state_mut();
    editor.selected = 1;
    let ctx = egui::Context::default();
    let key = |modifiers, pressed| egui::Event::Key {
        key: egui::Key::Num7,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers,
    };
    let mac_command = egui::Modifiers {
        mac_cmd: true,
        command: true,
        ..egui::Modifiers::NONE
    };
    let mut remaining = 0;
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![key(mac_command, true), key(egui::Modifiers::CTRL, true)],
            ..Default::default()
        },
        |ui| {
            let ctx = ui.ctx();
            editor.block_hotkey_input(ctx);
            remaining = ctx.input(|i| i.events.len());
        },
    );
    output.textures_delta.clear();
    assert_eq!(editor.block_hotkeys.digit(1), Some(7));
    assert_eq!(
        remaining, 2,
        "unbound command and assignment both propagate"
    );
    editor.selected = 0;
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![key(mac_command, true), key(mac_command, false)],
            ..Default::default()
        },
        |ui| {
            let ctx = ui.ctx();
            editor.block_hotkey_input(ctx);
            remaining = ctx.input(|i| i.events.len());
        },
    );
    output.textures_delta.clear();
    assert_eq!(
        editor.selected, 1,
        "source ignores modifiers other than Ctrl"
    );
    assert_eq!(
        remaining, 1,
        "only the selecting press is consumed, never key-up"
    );
}

#[test]
fn modal_and_text_focus_block_shortcuts_and_view_clear_shortcuts_propagate() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let original_warp_ids = h.state().event_ui.warp_ids;
    h.state_mut().selected = 1;
    for key in [egui::Key::Num3, egui::Key::Num4] {
        h.key_press_modifiers(egui::Modifiers::CTRL, key);
        h.run();
        h.key_press_modifiers(
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT | egui::Modifiers::COMMAND,
            key,
        );
        h.run();
        assert_eq!(h.state().block_hotkeys.digit(1), None);
    }
    assert!(h.state().hex);
    assert_ne!(h.state().event_ui.warp_ids, original_warp_ids);
    h.state_mut().block_hotkeys.assign(5, 1);
    h.state_mut().selected = 0;
    h.state_mut().about = Some(Default::default());
    h.run();
    h.key_press(egui::Key::Num5);
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::Num6);
    h.run();
    assert_eq!(h.state().selected, 0);
    assert_eq!(h.state().block_hotkeys.digit(0), None);
    h.get_by_label("OK").click();
    h.run();

    // A separate focused text field exercises the same main-window focus guard.
    let ctx = h.ctx.clone();
    let text_id = egui::Id::unique("hotkey-focus-test");
    ctx.memory_mut(|m| m.request_focus(text_id));
    let mut text = String::new();
    let editor = h.state_mut();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        ui.add(egui::TextEdit::singleline(&mut text).id(text_id));
    });
    output.textures_delta.clear();
    assert!(ctx.egui_wants_keyboard_input());
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Num5,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ui| {
            editor.show(ui);
        },
    );
    output.textures_delta.clear();
    assert_eq!(editor.selected, 0);
}

#[test]
fn resize_and_document_lifetime_preserve_only_valid_bindings() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let ctx = h.ctx.clone();
    let editor = h.state_mut();
    editor.block_hotkeys.assign(1, 0);
    editor.block_hotkeys.assign(2, 1);
    assert!(editor.apply_block_commands(&ctx, &[BlockCommand::Swap { a: 0, b: 1 }]));
    assert_eq!(editor.block_hotkeys.digit(0), Some(1));
    assert_eq!(editor.block_hotkeys.digit(1), Some(2));
    editor.sidebar_reveal = Some(1);
    assert!(editor.apply_block_commands(&ctx, &[BlockCommand::Resize { count: 1 }]));
    assert_eq!(editor.block_hotkeys.digit(0), Some(1));
    assert_eq!(editor.block_hotkeys.digit(1), None);
    assert_eq!(editor.sidebar_reveal, None);
    editor.close_map();
    assert!(editor.block_hotkeys.0.iter().all(Option::is_none));

    let (_other_dir, mut other) = crate::tileset_editor_tests::fixture(false, false);
    editor.block_hotkeys.assign(8, 0);
    editor.pending_open = other.state_mut().loaded.take();
    editor.advance_open_palettes(&ctx);
    assert!(editor.error.is_none());
    assert!(editor.loaded.is_some());
    assert!(editor.block_hotkeys.0.iter().all(Option::is_none));
}

#[test]
fn half_block_reveal_thresholds_and_label_offsets_match_source() {
    for size in [32.0, 64.0] {
        // Row 7 starts at 7*size; viewport is four rows tall.
        assert_eq!(reveal_offset(28, size, 4.0 * size, 3.5 * size), 4.0 * size);
        assert_eq!(
            reveal_offset(28, size, 4.0 * size, 3.5 * size + 1.0),
            3.5 * size + 1.0
        );
        assert_eq!(reveal_offset(28, size, 4.0 * size, 7.5 * size), 7.0 * size);
        assert_eq!(
            reveal_offset(28, size, 4.0 * size, 7.5 * size - 1.0),
            7.5 * size - 1.0
        );
        assert_eq!(reveal_offset(255, size, 4.0 * size, 0.0), 60.0 * size);
    }
    let area = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::splat(32.0));
    assert_eq!(
        label_position(area, 99, false, true, false),
        Pos2::new(40.0, 21.0)
    );
    assert_eq!(
        label_position(area, 100, false, true, false),
        Pos2::new(40.0, 35.0)
    );
    assert_eq!(
        label_position(area, 100, false, true, true),
        Pos2::new(40.0, 21.0)
    );
    assert_eq!(
        label_position(area, 100, false, false, false),
        Pos2::new(40.0, 21.0)
    );
    assert_eq!(
        label_position(area, 100, true, true, false),
        Pos2::new(39.0, 22.0)
    );
}

#[test]
fn hotkey_selection_reveals_last_and_first_rows_at_both_zoom_sizes() {
    for zoom in [false, true] {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let ctx = h.ctx.clone();
        assert!(
            h.state_mut()
                .apply_block_commands(&ctx, &[BlockCommand::Resize { count: 256 }])
        );
        h.state_mut().zoom = zoom;
        h.state_mut().block_hotkeys.assign(9, 255);
        h.state_mut().block_hotkeys.assign(0, 0);
        h.set_size(Vec2::new(600.0, 326.0));
        h.run();
        h.key_press(egui::Key::Num9);
        h.run();
        assert_eq!(h.state().selected, 255);
        let size = if zoom { 64.0 } else { 32.0 };
        assert_eq!(h.state().sidebar_scroll, 64.0 * size - 256.0);
        h.key_press(egui::Key::Num0);
        h.run();
        assert_eq!(h.state().selected, 0);
        assert_eq!(h.state().sidebar_scroll, 0.0);
    }
}

#[test]
fn bound_alt_digit_selects_before_recent_maps_and_failed_open_keeps_bindings() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let recent = dir.path().join("Recent.2x1.blk");
    std::fs::write(&recent, [1, 0]).unwrap();
    h.state_mut().recent_maps.remember(recent.clone());
    h.state_mut().block_hotkeys.assign(1, 1);
    h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num1);
    h.run();
    assert_eq!(h.state().selected, 1);
    assert!(h.state().open_form.is_none());
    h.key_press_modifiers(
        egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
        egui::Key::Num1,
    );
    h.run();
    h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num1);
    h.run();
    assert_eq!(h.state().open_form.as_ref().unwrap().path, recent);
    h.get_by_label("Cancel").click();
    h.run();
    h.state_mut().block_hotkeys.assign(1, 1);
    let (tx, rx) = mpsc::channel();
    tx.send(Err("Synthetic open failure".into())).unwrap();
    h.state_mut().loading = Some(rx);
    h.run();
    assert!(h.state().error.is_some());
    assert_eq!(h.state().block_hotkeys.digit(1), Some(1));
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        [0, 1]
    );
}

#[test]
fn canvas_right_pick_reveals_the_selected_block_without_editing() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let ctx = h.ctx.clone();
    assert!(
        h.state_mut()
            .apply_block_commands(&ctx, &[BlockCommand::Resize { count: 256 }])
    );
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 255,
    });
    h.set_size(Vec2::new(600.0, 326.0));
    h.run();
    // Sidebar width 144, map margin three blocks, half a block to the center.
    let pos = Pos2::new(144.0 + 112.0, 47.0 + 112.0);
    h.hover_at(pos);
    h.run();
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks()[0],
        255
    );
    for pressed in [true, false] {
        h.hover_at(pos);
        h.event_modifiers(
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Secondary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Modifiers::NONE,
        );
        h.run();
        assert_eq!(
            h.state().selected,
            255,
            "picking happens on press, before release"
        );
    }
    assert_eq!(
        h.state().selected,
        255,
        "cursor {:?}, canvas scroll {:?}, event mode {}, error {:?}",
        h.state().ruler_cursor,
        h.state().canvas_scroll,
        h.state().event_ui.mode,
        h.state().error
    );
    assert_eq!(h.state().sidebar_scroll, 1792.0);
    // The only undoable edit is the fixture's initial paint.
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
    h.run();
    let map = h.state().loaded.as_ref().unwrap().document.map();
    assert_eq!(map.blocks(), [0, 1]);
    assert!(!map.can_undo());
}

#[cfg(feature = "visual-tests")]
#[test]
fn hotkey_badge_atlas_covers_ids_zoom_hex_and_dpi() {
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for scale in [1.0, 2.0] {
        let mut h = egui_kittest::Harness::builder()
            .with_size(Vec2::new(720.0, 350.0))
            .with_pixels_per_point(scale)
            .with_theme(egui::Theme::Light)
            .build_ui(|ui| {
                ui.painter().rect_filled(ui.max_rect(), 0.0, Color32::WHITE);
                for (group, (ids, hex, zoom)) in [
                    (false, false, false),
                    (true, false, false),
                    (true, true, false),
                    (true, false, true),
                ]
                .into_iter()
                .enumerate()
                {
                    let editor = Editor {
                        ids,
                        hex,
                        zoom,
                        ..Default::default()
                    };
                    let origin = Pos2::new(10.0, 10.0 + group as f32 * 80.0);
                    for (digit, id) in [0, 1, 9, 10, 99, 100, 101, 175, 254, 255]
                        .into_iter()
                        .enumerate()
                    {
                        let area = Rect::from_min_size(
                            origin + Vec2::new(digit as f32 * 70.0, 0.0),
                            Vec2::splat(if zoom { 64.0 } else { 32.0 }),
                        );
                        ui.painter().rect_filled(area, 0.0, Color32::from_gray(110));
                        map_buttons::decorate(ui.painter(), area, zoom, true, digit == 0, false);
                        if ids {
                            map_labels::draw_id(ui.painter(), area, id, hex, zoom, false);
                        }
                        draw(ui.painter(), area, digit, id, &editor);
                    }
                }
            });
        h.run();
        snapshots.add(h.try_snapshot(format!("block-hotkeys-atlas-{scale}x")));
    }
    snapshots.unwrap();
}
