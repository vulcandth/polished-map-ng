use super::*;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn confirmation_source_geometry_keyboard_and_platform_order() {
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for windows in [false, true] {
        for scale in [1.0, 2.0] {
            for key in [egui::Key::Enter, egui::Key::Escape] {
                let mut h = Harness::builder()
                    .with_size(Vec2::new(800.0, 600.0))
                    .with_pixels_per_point(scale)
                    .build_ui_state(
                        |ui, state: &mut (bool, bool)| {
                            if !state.0 && !state.1 {
                                *state = confirm(
                                    ui.ctx(),
                                    "confirmation-test",
                                    "Synthetic.blk has unsaved changes!\n\nExit anyway?",
                                    windows,
                                    ["Proceed", "Keep editing"],
                                );
                            }
                        },
                        (false, false),
                    );
                h.run();
                let ok = h.get_by_label("Proceed").rect();
                let cancel = h.get_by_label("Keep editing").rect();
                assert_eq!(ok.size(), Vec2::new(80.0, 22.0));
                assert_eq!(cancel.size(), ok.size());
                assert_eq!(ok.top(), cancel.top());
                assert_eq!((ok.left() - cancel.left()).abs(), 94.0);
                assert_eq!(ok.left() < cancel.left(), windows);
                // Both labels are visually the source OK/Cancel; descriptive
                // accessibility names must not force larger client controls.
                for text in ["OK", "Cancel"] {
                    assert!(h.output().shapes.iter().any(
                        |s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.text() == text)
                    ));
                }
                #[cfg(feature = "visual-tests")]
                if key == egui::Key::Enter {
                    h.remove_cursor();
                    h.run();
                    snapshots.add(h.try_snapshot(format!(
                        "confirmation-{}-{}x",
                        if windows { "windows" } else { "unix" },
                        scale
                    )));
                }
                h.key_press(key);
                h.run();
                assert_eq!(
                    *h.state(),
                    (key == egui::Key::Enter, key == egui::Key::Escape)
                );
            }
        }
    }
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}

#[test]
fn source_client_geometry_preserves_icon_gap_minimum_size_and_button_anchor() {
    for (body, font, size, button) in [
        (
            Vec2::ZERO,
            12.0,
            Vec2::new(280.0, 102.0),
            Pos2::new(190.0, 70.0),
        ),
        (
            Vec2::new(150.0, 13.0),
            13.0,
            Vec2::new(280.0, 102.0),
            Pos2::new(190.0, 70.0),
        ),
        (
            Vec2::new(250.0, 80.0),
            12.0,
            Vec2::new(342.0, 132.0),
            Pos2::new(252.0, 100.0),
        ),
        (
            Vec2::new(620.0, 120.0),
            13.0,
            Vec2::new(713.0, 172.0),
            Pos2::new(623.0, 140.0),
        ),
        (
            Vec2::new(200.2, 50.2),
            12.0,
            Vec2::new(293.0, 103.0),
            Pos2::new(203.0, 71.0),
        ),
    ] {
        let layout = Layout::measured(body, font);
        assert_eq!(layout.size, size);
        assert_eq!(layout.ok.min, button);
        assert_eq!(layout.ok.size(), Vec2::new(80.0, 22.0));
        assert_eq!(layout.body.min, Pos2::new(70.0, 10.0));
        assert_eq!(layout.body.width(), size.x - 80.0);
        assert_eq!(
            layout.icon,
            Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(50.0))
        );
    }
}

#[test]
fn rendered_messages_use_platform_metrics_at_both_dpi_and_wrap_long_text() {
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for (font, dark, scale, _name) in [
        (12.0, false, 1.0, "message-error-windows"),
        (13.0, false, 1.0, "message-error-unix"),
        (12.0, true, 1.0, "message-error-dark"),
        (13.0, true, 2.0, "message-error-unix-2x"),
    ] {
        let message = "Could not read synthetic-map.blk!\n\nThe existing map and unsaved edits are preserved.";
        let mut h = Harness::builder()
            .with_size(Vec2::new(800.0, 600.0))
            .with_pixels_per_point(scale)
            .with_theme(if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            })
            .build_ui_state(
                |ui, state: &mut (bool, Rect)| {
                    if !state.0 {
                        *state =
                            show_with_font(ui.ctx(), "message-test", Kind::Error, message, font);
                    }
                },
                (false, Rect::NOTHING),
            );
        h.run();
        let area = h.state().1;
        let ok = h.get_by_label("OK").rect();
        let body = h.get_by_label(message).rect();
        assert_eq!(ok.size(), Vec2::new(80.0, 22.0));
        assert_eq!(area.right() - ok.right(), 10.0);
        assert_eq!(area.bottom() - ok.bottom(), 10.0);
        assert!((body.left() - area.left() - 70.0).abs() <= 0.5);
        assert!((body.top() - area.top() - 10.0).abs() <= 0.5);
        let painted = h
            .output()
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.text() == message => Some(t.pos),
                _ => None,
            })
            .expect("painted message");
        assert!((painted.x - area.left() - 70.0).abs() <= 0.5);
        assert!((painted.y - area.top() - 10.0).abs() <= 0.5);
        assert!((area.center().x - 400.0).abs() <= 0.5);
        h.remove_cursor();
        h.run();
        #[cfg(feature = "visual-tests")]
        snapshots.add(h.try_snapshot(_name));
        h.get_by_label("OK").click();
        h.run();
        assert!(h.state().0);
    }
    let message =
        "A long message with whitespace that should wrap at the source measurement width. "
            .repeat(12);
    let mut h = Harness::builder()
        .with_size(Vec2::new(900.0, 700.0))
        .build_ui_state(
            |ui, state: &mut (bool, Rect)| {
                *state = show_with_font(ui.ctx(), "long-message", Kind::Warning, &message, 13.0);
            },
            (false, Rect::NOTHING),
        );
    h.run();
    let area = h.state().1;
    assert!(area.width() <= 713.0 && area.height() > 102.0);
    assert!(h.get_by_label(&message).rect().bottom() < h.get_by_label("OK").rect().top());
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}

fn pending(editor: &Editor, kind: usize) -> bool {
    match kind {
        0 => editor.error.is_some(),
        1 => editor.open_palette_warning.is_some(),
        2 => editor.print_success.is_some(),
        3 => editor.roof_color_success.is_some(),
        4 => editor.overworld_warning.is_some(),
        _ => unreachable!(),
    }
}

#[test]
fn all_message_routes_dismiss_by_enter_escape_or_click_without_parent_actions() {
    for kind in 0..5 {
        for key in [Some(egui::Key::Enter), Some(egui::Key::Escape), None] {
            for scale in [1.0, 2.0] {
                let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
                h.ctx.set_pixels_per_point(scale);
                h.run();
                h.set_size(Vec2::new(800.0, 600.0));
                h.state_mut().apply(Command::Paint {
                    points: vec![Point { x: 0, y: 0 }],
                    block: 1,
                });
                let palette_hash = h.state().palette_document().content_hash();
                let message = Some("Synthetic message".into());
                match kind {
                    0 => h.state_mut().error = message,
                    1 => h.state_mut().open_palette_warning = message,
                    2 => h.state_mut().print_success = message,
                    3 => h.state_mut().roof_color_success = message,
                    4 => h.state_mut().overworld_warning = message,
                    _ => unreachable!(),
                }
                h.run();
                assert!(h.get_by_label("File").accesskit_node().is_disabled());
                h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
                h.run();
                h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
                h.run();
                assert!(pending(h.state(), kind));
                assert!(!h.state().confirm_discard && !h.state().pending_exit);
                if let Some(key) = key {
                    h.key_press(key);
                } else {
                    h.get_by_label("OK").click();
                }
                h.run();
                assert!(!pending(h.state(), kind));
                let map = h.state().loaded.as_ref().unwrap().document.map();
                assert_eq!(map.blocks(), [1, 1]);
                assert!(map.is_modified() && map.can_undo());
                assert_eq!(h.state().palette_document().content_hash(), palette_hash);
                assert_eq!(
                    std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
                    [0, 1]
                );
            }
        }
    }
}

#[test]
fn acknowledging_error_does_not_skip_a_queued_warning() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().error = Some("First error".into());
    h.state_mut().open_palette_warning = Some("Next warning".into());
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(h.state().error.is_none() && h.state().open_palette_warning.is_some());
    h.get_by_label("Next warning");
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(h.state().open_palette_warning.is_none());
}
