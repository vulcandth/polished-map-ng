use super::*;
use egui_kittest::{Harness, kittest::Queryable};

struct State {
    value: i16,
    hex: bool,
    enabled: bool,
    commits: usize,
}
fn fixture(hex: bool) -> Harness<'static, State> {
    Harness::builder()
        .with_size(Vec2::new(400.0, 200.0))
        .build_ui_state(
            |ui, s: &mut State| {
                let mut input = IntegerInput::new(&mut s.value, -6..=249);
                if s.hex {
                    input = input.hex();
                }
                let response = ui.add_enabled(s.enabled, input);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::SpinButton, s.enabled, "Number")
                });
                if response.changed() {
                    s.commits += 1;
                }
                let _ = ui.button("Elsewhere");
            },
            State {
                value: 10,
                hex,
                enabled: true,
                commits: 0,
            },
        )
}

fn type_number(h: &mut Harness<'_, State>, text: &str) {
    h.get_by_label("Number").click();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    h.get_by_label("Number").type_text(text);
    h.run();
}

#[test]
fn integer_input_commits_on_enter_or_focus_loss_and_wraps_arrows() {
    for hex in [false, true] {
        let mut h = fixture(hex);
        type_number(&mut h, "999");
        assert_eq!(h.state().value, 10, "text is a draft");
        assert_eq!(h.state().commits, 0);
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(h.state().value, 249, "typed values clamp");
        assert!(h.get_by_label("Number").is_focused());
        h.key_press(Key::ArrowUp);
        h.run();
        assert_eq!(h.state().value, -6);
        h.key_press(Key::ArrowDown);
        h.run();
        assert_eq!(h.state().value, 249);
        type_number(&mut h, "-3");
        h.get_by_label("Elsewhere").click();
        h.run();
        assert_eq!(h.state().value, -3);
    }
}

#[test]
fn integer_input_paste_replaces_whole_field_and_rejects_invalid_text() {
    for hex in [false, true] {
        let mut h = fixture(hex);
        h.get_by_label("Number").click();
        h.run();
        h.event(egui::Event::Paste(" \t+20\n".into()));
        h.run();
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(h.state().value, if hex { 32 } else { 20 });
        for invalid in ["12junk", "3.5", "1+2", "$FF", "", "  ", "１２"] {
            h.event(egui::Event::Paste(invalid.into()));
            h.run();
            h.key_press(Key::Enter);
            h.run();
            assert_eq!(h.state().value, if hex { 32 } else { 20 }, "{invalid}");
        }
        h.event(egui::Event::Paste("-6".into()));
        h.run();
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(h.state().value, -6);
    }
}

#[test]
fn integer_input_invalid_key_preserves_selection_and_signed_hex_is_editable() {
    for hex in [false, true] {
        let mut h = fixture(hex);
        type_number(&mut h, "z");
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(
            h.state().value,
            10,
            "invalid key must not erase selected text"
        );
        type_number(&mut h, "-3");
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(h.state().value, -3);
        h.key_press(Key::ArrowUp);
        h.run();
        assert_eq!(h.state().value, -2);
    }
    assert_eq!(parse("+-3", true, &(-6..=249)), 0);
    assert_eq!(parse("01A", false, &(-6..=249)), 1);
}

#[test]
fn integer_input_preserves_in_range_spelling_until_an_arrow_or_clamp_updates_it() {
    for (hex, text, next) in [(false, "+020", "21"), (true, "+0a", "0B")] {
        let mut h = fixture(hex);
        type_number(&mut h, text);
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(h.get_by_label("Number").value().as_deref(), Some(text));
        h.get_by_label("Elsewhere").click();
        h.run();
        assert_eq!(h.get_by_label("Number").value().as_deref(), Some(text));
        h.get_by_label("Number").focus();
        h.run();
        h.key_press(Key::ArrowUp);
        h.run();
        assert_eq!(h.get_by_label("Number").value().as_deref(), Some(next));
    }
}

#[test]
fn integer_input_decimal_arrows_commit_draft_but_hex_arrows_use_stored_value() {
    for hex in [false, true] {
        let mut h = fixture(hex);
        type_number(&mut h, "20");
        h.key_press(Key::ArrowUp);
        h.run();
        assert_eq!(h.state().value, if hex { 11 } else { 21 });
    }
}

#[test]
fn integer_input_buttons_step_on_press_repeat_and_stop_outside_or_disabled() {
    let mut h = fixture(false);
    let rect = h.get_by_label("Number").rect();
    let pos = egui::pos2(rect.right() - 5.0, rect.top() + 5.0);
    h.hover_at(pos);
    h.run();
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    assert_eq!(h.state().value, 11);
    let now = h.ctx.input(|i| i.time);
    h.input_mut().time = Some(now + 0.49);
    h.step();
    assert_eq!(h.state().value, 11);
    h.input_mut().time = Some(now + 0.51);
    h.step();
    assert_eq!(h.state().value, 12);
    h.input_mut().time = Some(now + 0.62);
    h.step();
    assert_eq!(h.state().value, 13);
    h.hover_at(egui::pos2(300.0, 100.0));
    h.step();
    h.input_mut().time = Some(now + 1.2);
    h.step();
    assert_eq!(h.state().value, 13);
    h.hover_at(pos);
    h.step();
    assert_eq!(h.state().value, 14, "reenter starts a new repeat delay");
    h.state_mut().enabled = false;
    h.input_mut().time = Some(now + 2.0);
    h.step();
    assert_eq!(h.state().value, 14);
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run();
}

#[test]
fn integer_input_arrow_button_focus_keeps_keyboard_stepping_in_its_group() {
    for button in [egui::PointerButton::Primary, egui::PointerButton::Secondary] {
        let mut h = fixture(false);
        let rect = h.get_by_label("Number").rect();
        let pos = egui::pos2(rect.right() - 5.0, rect.top() + 5.0);
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton {
                pos,
                button,
                pressed,
                modifiers: Modifiers::NONE,
            });
            h.step();
        }
        assert_eq!(h.state().value, 11);
        h.key_press(Key::ArrowUp);
        h.run();
        assert_eq!(h.state().value, 12);
        h.key_press(Key::ArrowDown);
        h.run();
        assert_eq!(h.state().value, 11);
        h.key_press(Key::Space);
        h.run();
        assert_eq!(
            h.state().value,
            12,
            "Space activates the focused arrow button"
        );
    }
}

#[test]
fn integer_input_ignores_disabled_actions_and_external_change_clears_draft() {
    let mut h = fixture(false);
    type_number(&mut h, "90");
    h.state_mut().value = 37;
    h.run();
    h.get_by_label("Elsewhere").click();
    h.run();
    assert_eq!(h.state().value, 37);
    h.state_mut().enabled = false;
    h.run();
    h.get_by_label("Number").set_value_accesskit(200.0);
    h.run();
    h.key_press(Key::ArrowUp);
    h.run();
    assert_eq!(h.state().value, 37);
}

#[test]
fn integer_input_signed_hex_and_limits_have_deterministic_overflow() {
    for (text, expected) in [
        ("FFFFFFFA", -6),
        ("FFFFFFFF", -1),
        ("+F9", 249),
        ("-F", -6),
        ("FF", 249),
        ("", 0),
        ("-", 0),
        ("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", 249),
    ] {
        assert_eq!(parse(text, true, &(-6..=249)), expected, "{text}");
    }
    for range in [1..=255, 1..=256, 1..=4096, -6..=249, 0..=255, 0..=31] {
        assert_eq!(step(*range.end(), true, &range), *range.start());
        assert_eq!(step(*range.start(), false, &range), *range.end());
        assert_eq!(parse("999999", false, &range), *range.end());
        assert_eq!(parse("-999999", false, &range), *range.start());
    }
}

#[test]
fn integer_input_integrates_open_resize_block_count_overworld_and_collision_bounds() {
    use crate::*;
    fn wrap(h: &mut Harness<'_, Editor>, label: &str, max: f64, low: &str, high: &str) {
        h.get_by_label(label).set_value_accesskit(max);
        h.run();
        h.get_by_label(label).focus();
        h.run();
        h.key_press(Key::ArrowUp);
        h.run();
        assert_eq!(
            h.get_by_label(label).value().as_deref(),
            Some(low),
            "{label}"
        );
        h.key_press(Key::ArrowDown);
        h.run();
        assert_eq!(
            h.get_by_label(label).value().as_deref(),
            Some(high),
            "{label}"
        );
    }
    let (dir, mut h) = tileset_editor_tests::fixture(false, false);
    h.state_mut().prepare_new_in(dir.path().to_owned());
    h.run();
    for label in ["Width", "Height"] {
        wrap(&mut h, label, 255.0, "1", "255");
    }
    h.get_by_label("Cancel").click();
    h.run();
    h.state_mut().prepare_resize();
    h.run();
    for label in ["Width", "Height"] {
        wrap(&mut h, label, 255.0, "1", "255");
    }
    h.get_by_label("Cancel").click();
    h.run();
    h.state_mut().prepare_block_count();
    h.run();
    wrap(&mut h, "Block count", 256.0, "1", "256");
    h.get_by_label("Cancel").click();
    h.run();
    h.state_mut().prepare_overworld();
    h.run();
    wrap(&mut h, "Overworld map buffer size", 4096.0, "1", "4096");
    h.get_by_label("Cancel").click();
    h.run();
    let blocks = dir.path().join("blocks.bin");
    let collisions = dir.path().join("collision.bin");
    std::fs::write(&blocks, [0; 16]).unwrap();
    std::fs::write(&collisions, [1, 32, 128, 255]).unwrap();
    h.state_mut().loaded.as_mut().unwrap().blockset =
        Some(BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap());
    block_workflow_tests::open_block(&mut h, 0);
    for label in [
        "Collision top left",
        "Collision top right",
        "Collision bottom left",
        "Collision bottom right",
    ] {
        wrap(&mut h, label, 255.0, "00", "FF");
    }
    h.get_by_label("Cancel").click();
    h.run();
    assert_eq!(std::fs::read(collisions).unwrap(), [1, 32, 128, 255]);
    assert!(!h.state().is_modified());
}

#[test]
fn integer_input_resize_enter_commits_field_escape_discards_and_default_enter_applies() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().prepare_resize();
    h.run();
    h.get_by_label("Width").focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    h.get_by_label("Width").type_text("20");
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert!(h.state().resize_form.is_some());
    assert!(!h.state().is_modified());
    h.key_press(Key::Escape);
    h.run();
    assert!(h.state().resize_form.is_none());
    assert!(!h.state().is_modified());
    h.state_mut().prepare_resize();
    h.run();
    h.get_by_label("Width").set_value_accesskit(3.0);
    h.run();
    h.get_by_label("Resize").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert!(h.state().resize_form.is_none());
    assert_eq!(h.state().loaded.as_ref().unwrap().document.map().width(), 3);
}
