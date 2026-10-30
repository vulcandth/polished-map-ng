use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{CollisionRow, ColorSlot, PaletteSet};

fn pointer<T>(h: &mut Harness<'_, T>, pos: Pos2, pressed: bool) {
    h.hover_at(pos);
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Middle,
        pressed,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
}

fn reset(h: &mut Harness<'_, Editor>, label: &str) {
    let pos = h.get_by_label(label).rect().center();
    pointer(h, pos, true);
    pointer(h, pos, false);
    h.run();
}

#[test]
fn middle_reset_captures_release_and_cancels_on_disabled_focus_or_pointer_loss() {
    struct State {
        reset: DefaultValue<u16>,
        value: u16,
        enabled: bool,
        calls: usize,
    }
    let mut h = Harness::builder()
        .with_size(Vec2::new(400.0, 200.0))
        .build_ui_state(
            |ui, state: &mut State| {
                let response = ui.add_enabled(
                    state.enabled,
                    egui::DragValue::new(&mut state.value).range(1..=255),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::SpinButton, state.enabled, "Number")
                });
                if state.reset.restore(ui, &response, &mut state.value) {
                    state.calls += 1;
                }
            },
            State {
                reset: DefaultValue::new(37),
                value: 90,
                enabled: true,
                calls: 0,
            },
        );
    let pos = h.get_by_label("Number").rect().center();
    pointer(&mut h, pos, true);
    assert_eq!(h.state().value, 90, "press does not reset");
    pointer(&mut h, Pos2::new(300.0, 100.0), false);
    assert_eq!((h.state().value, h.state().calls), (37, 1));
    h.run();
    assert_eq!(h.state().calls, 1, "one callback per release");
    for interruption in 0..3 {
        h.state_mut().value = 90;
        pointer(&mut h, pos, true);
        match interruption {
            0 => h.state_mut().enabled = false,
            1 => h.event(egui::Event::PointerGone),
            _ => h.event(egui::Event::WindowFocused(false)),
        }
        h.run();
        pointer(&mut h, pos, false);
        assert_eq!(h.state().value, 90);
        assert_eq!(h.state().calls, 1);
        h.state_mut().enabled = true;
        h.event(egui::Event::WindowFocused(true));
        h.run();
    }
    pointer(&mut h, Pos2::new(300.0, 100.0), true);
    pointer(&mut h, pos, false);
    assert_eq!(h.state().value, 90, "outside press cannot arm reset");
}

#[test]
fn middle_reset_restores_resize_and_block_count_open_values_without_editing() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().prepare_resize();
    h.run();
    for (label, value) in [("Width", 12.0), ("Height", 19.0)] {
        h.get_by_label(label).set_value_accesskit(value);
        h.run();
        reset(&mut h, label);
    }
    let form = h.state().resize_form.as_ref().unwrap();
    assert_eq!((form.width, form.height), (2, 1));
    h.get_by_label("Cancel").click();
    h.run();
    h.state_mut().prepare_block_count();
    h.run();
    h.get_by_label("Block count").set_value_accesskit(256.0);
    h.run();
    reset(&mut h, "Block count");
    assert_eq!(h.state().block_count_form, Some(2));
    h.get_by_label("OK").click();
    h.run();
    assert!(!h.state().is_modified());
    assert!(!h.state().loaded.as_ref().unwrap().document.map().can_undo());
}

#[test]
fn middle_reset_numeric_text_edit_does_not_reapply_stale_text() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().prepare_resize();
    h.run();
    h.get_by_label("Width").click();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Width").type_text("19");
    h.run();
    reset(&mut h, "Width");
    h.run();
    h.get_by_label("Height").click();
    h.run();
    assert_eq!(h.state().resize_form.as_ref().unwrap().width, 2);
    h.get_by_label("Cancel").click();
    h.run();
    assert!(!h.state().is_modified());
}

#[test]
fn middle_reset_overworld_uses_session_value_and_reopening_discards_old_capture() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().overworld_map_size = 2048;
    h.state_mut().prepare_overworld();
    h.run();
    h.get_by_label("Overworld map buffer size")
        .set_value_accesskit(3000.0);
    h.run();
    reset(&mut h, "Overworld map buffer size");
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(h.state().overworld_map_size, 2048);
    h.state_mut().prepare_overworld();
    h.run();
    let pos = h.get_by_label("Overworld map buffer size").rect().center();
    pointer(&mut h, pos, true);
    h.key_press(egui::Key::Escape);
    h.run();
    h.state_mut().prepare_overworld();
    h.run();
    h.get_by_label("Overworld map buffer size")
        .set_value_accesskit(3000.0);
    h.run();
    pointer(&mut h, pos, false);
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(h.state().overworld_map_size, 3000);
    assert!(!h.state().is_modified());
}

#[test]
fn middle_reset_event_decimal_and_hex_coordinates_preserve_original_fields() {
    use crate::event_workflows::{EventAction, EventChange};
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let path = dir.path().join("Map.asm");
    let source = "\twarp_event 1, 2, MAP, 1\n\tperson_event SPRITE_RED, $00, $00, FLAGS\n";
    std::fs::write(&path, source).unwrap();
    h.state_mut()
        .request_event_change(EventChange::Load(path.clone()));
    h.run();
    for index in 0..2 {
        let before = h
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .events
            .as_ref()
            .unwrap()
            .script()
            .events()
            .entries()[index]
            .definition()
            .clone();
        let ctx = h.ctx.clone();
        h.state_mut()
            .event_action(EventAction::Options(index), &ctx);
        h.run();
        for label in ["Event X", "Event Y"] {
            h.get_by_label(label).set_value_accesskit(249.0);
            h.run();
            reset(&mut h, label);
        }
        h.get_by_label("OK").click();
        h.run();
        assert_eq!(
            h.state()
                .loaded
                .as_ref()
                .unwrap()
                .events
                .as_ref()
                .unwrap()
                .script()
                .events()
                .entries()[index]
                .definition(),
            &before
        );
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    assert!(
        !h.state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .map()
            .is_modified()
    );
}

#[test]
fn middle_reset_collision_quadrants_restore_the_loaded_bytes() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let blocks = dir.path().join("blocks.bin");
    let collisions = dir.path().join("collision.bin");
    std::fs::write(&blocks, [0; 16]).unwrap();
    std::fs::write(&collisions, [1, 32, 128, 255]).unwrap();
    h.state_mut().loaded.as_mut().unwrap().blockset =
        Some(BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap());
    crate::block_workflow_tests::open_block(&mut h, 0);
    for label in [
        "Collision top left",
        "Collision top right",
        "Collision bottom left",
        "Collision bottom right",
    ] {
        h.get_by_label(label).set_value_accesskit(0.0);
        h.run();
        reset(&mut h, label);
    }
    h.get_by_label("OK").click();
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
            .collisions()
            .unwrap(),
        &[CollisionRow::Binary([1, 32, 128, 255])]
    );
    assert_eq!(std::fs::read(collisions).unwrap(), [1, 32, 128, 255]);
}

#[test]
fn middle_reset_palette_channels_keep_selection_defaults_across_hex_and_channel_edits() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().prepare_palettes();
    h.run();
    let slot = ColorSlot {
        set: PaletteSet::Day,
        palette: 0,
        hue: 0,
    };
    let original = h
        .state()
        .palette_form
        .as_ref()
        .unwrap()
        .draft
        .bank()
        .color(slot)
        .unwrap();
    h.get_by_label("Hex color").click();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Hex color").type_text("123456");
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    for label in ["R channel", "G slider", "B channel"] {
        h.get_by_label(label).set_value_accesskit(3.0);
        h.run();
        reset(&mut h, label);
    }
    let quantized = original.map(|c| (u16::from(c / 8) * 33 / 4) as u8);
    assert_eq!(
        h.state()
            .palette_form
            .as_ref()
            .unwrap()
            .draft
            .bank()
            .color(slot)
            .unwrap(),
        quantized
    );
    // Selecting the same swatch again captures its current channels.
    h.get_by_label("R channel").set_value_accesskit(7.0);
    h.run();
    h.get_by_label("GRAY color 1").click();
    h.run();
    h.get_by_label("R slider").set_value_accesskit(20.0);
    h.run();
    reset(&mut h, "R slider");
    assert_eq!(
        h.state()
            .palette_form
            .as_ref()
            .unwrap()
            .draft
            .bank()
            .color(slot)
            .unwrap()[0],
        57
    );
    h.get_by_label("Cancel").click();
    h.run();
    assert_eq!(
        h.state().palette_document().bank().color(slot).unwrap(),
        original
    );
    assert!(!h.state().is_modified());
}

#[test]
fn middle_reset_palette_clipboard_and_drag_capture_the_transferred_color() {
    for action in 0..4 {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        h.state_mut().prepare_palettes();
        h.run();
        let a = ColorSlot {
            set: PaletteSet::Day,
            palette: 0,
            hue: 0,
        };
        let b = ColorSlot { hue: 2, ..a };
        h.state_mut()
            .palette_form
            .as_mut()
            .unwrap()
            .draft
            .apply_batch(&[
                polished_map_core::PaletteCommand::SetRgb {
                    slot: a,
                    rgb: [17, 34, 51],
                },
                polished_map_core::PaletteCommand::SetRgb {
                    slot: b,
                    rgb: [201, 202, 203],
                },
            ])
            .unwrap();
        h.run();
        if action < 2 {
            h.get_by_label("GRAY color 1").click();
            h.run();
            h.event(egui::Event::Copy);
            h.run();
            h.get_by_label("GRAY color 2").click();
            h.run();
            h.event(if action == 0 {
                egui::Event::Paste(String::new())
            } else {
                egui::Event::Cut
            });
            h.run();
        } else {
            let from = h.get_by_label("GRAY color 1").rect().center();
            let to = h.get_by_label("GRAY color 2").rect().center();
            let modifiers = if action == 2 {
                egui::Modifiers::CTRL
            } else {
                egui::Modifiers::NONE
            };
            h.hover_at(from);
            h.event_modifiers(
                egui::Event::PointerButton {
                    pos: from,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers,
                },
                modifiers,
            );
            h.run();
            h.hover_at(from + Vec2::new(8.0, 0.0));
            h.run();
            h.hover_at(to);
            h.run();
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
        }
        let received = h
            .state()
            .palette_form
            .as_ref()
            .unwrap()
            .draft
            .bank()
            .color(b)
            .unwrap();
        assert!(
            received[0] < 20,
            "transfer must actually reach the target, action {action}"
        );
        h.get_by_label("R channel").set_value_accesskit(31.0);
        h.run();
        reset(&mut h, "R slider");
        let result = h
            .state()
            .palette_form
            .as_ref()
            .unwrap()
            .draft
            .bank()
            .color(b)
            .unwrap();
        assert_eq!(
            result,
            [
                (u16::from(received[0] / 8) * 33 / 4) as u8,
                received[1],
                received[2]
            ],
            "action {action}"
        );
    }
}
