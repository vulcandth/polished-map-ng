use super::*;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn source_drag_geometry_retains_offcenter_capture() {
    assert_eq!([0, 15, 31].map(|v| position(v, 92, 16)), [0, 37, 76]);
    let mut offset = 0;
    // Initial grip at 37..53: grabbing at 40 preserves its off-center origin.
    assert_eq!(move_value(15, 92, 16, 40, &mut offset, true), 15);
    assert_eq!(offset, 3);
    assert_eq!(move_value(15, 92, 16, 50, &mut offset, false), 19);
    // Clicking beyond its right edge moves the *edge*, not the grip center.
    assert_eq!(move_value(15, 92, 16, 70, &mut offset, true), 22);
    assert_eq!(offset, 16);
    assert_eq!(move_value(22, 92, 16, 200, &mut offset, false), 31);
    assert_eq!(move_value(31, 92, 16, -20, &mut offset, false), 0);
    assert_eq!(move_value(15, 12, 16, 10, &mut offset, true), 15);
}

#[derive(Default)]
struct State {
    value: u8,
    disabled: bool,
    changes: usize,
    hidden: bool,
}

fn fixture(dark: bool, dpi: f32) -> Harness<'static, State> {
    Harness::builder()
        .with_size(Vec2::new(200.0, 100.0))
        .with_theme(if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        })
        .with_pixels_per_point(dpi)
        .build_ui_state(
            |ui, s: &mut State| {
                ui.spacing_mut().slider_width = 92.0;
                ui.spacing_mut().interact_size.y = 22.0;
                if s.hidden {
                    return;
                }
                let response = ui
                    .add_enabled_ui(!s.disabled, |ui| {
                        ui.put(
                            Rect::from_min_size(egui::pos2(20.0, 20.0), Vec2::new(92.0, 22.0)),
                            NiceSlider(&mut s.value),
                        )
                    })
                    .inner;
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Slider, !s.disabled, "Channel")
                });
                if response.changed() {
                    s.changes += 1;
                }
            },
            State {
                value: 15,
                ..Default::default()
            },
        )
}

fn pointer(h: &mut Harness<'_, State>, x: f32, button: egui::PointerButton, pressed: bool) {
    let pos = egui::pos2(x, 31.0);
    h.event(egui::Event::PointerMoved(pos));
    h.event(egui::Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
}

#[test]
fn palette_slider_pointer_keyboard_accessibility_and_dpi() {
    for dark in [false, true] {
        for dpi in [1.0, 2.0] {
            let mut h = fixture(dark, dpi);
            assert_eq!(
                h.get_by_label("Channel").rect().size(),
                Vec2::new(92.0, 22.0)
            );
            pointer(&mut h, 60.0, egui::PointerButton::Primary, true);
            assert_eq!(h.state().value, 15);
            h.event(egui::Event::PointerMoved(egui::pos2(70.0, 31.0)));
            h.run();
            assert_eq!(h.state().value, 19);
            pointer(&mut h, 70.0, egui::PointerButton::Primary, false);
            h.key_press(egui::Key::ArrowRight);
            h.run();
            assert_eq!(h.state().value, 20);
            h.key_press(egui::Key::ArrowUp);
            h.run();
            assert_eq!(h.state().value, 20);
            h.get_by_label("Channel").set_value_accesskit(31.0);
            h.run();
            h.key_press(egui::Key::ArrowRight);
            h.run();
            assert_eq!(h.state().value, 31);
            pointer(&mut h, 20.0, egui::PointerButton::Secondary, true);
            assert_eq!(h.state().value, 0, "secondary mouse follows FLTK drag path");
            pointer(&mut h, 20.0, egui::PointerButton::Secondary, false);
            pointer(&mut h, 90.0, egui::PointerButton::Middle, true);
            h.event(egui::Event::PointerMoved(egui::pos2(110.0, 31.0)));
            h.run();
            assert_eq!(
                h.state().value,
                0,
                "middle reset belongs to DefaultValue, never the drag path"
            );
            pointer(&mut h, 110.0, egui::PointerButton::Middle, false);
            s_disabled(&mut h);
        }
    }
}

fn s_disabled(h: &mut Harness<'_, State>) {
    h.state_mut().disabled = true;
    h.run();
    assert!(h.get_by_label("Channel").accesskit_node().is_disabled());
    let changes = h.state().changes;
    pointer(h, 110.0, egui::PointerButton::Primary, true);
    pointer(h, 110.0, egui::PointerButton::Primary, false);
    h.get_by_label("Channel").set_value_accesskit(20.0);
    h.run();
    assert_eq!(h.state().changes, changes);
}

#[test]
fn capture_cancels_on_focus_loss_and_disable() {
    for disable in [false, true] {
        let mut h = fixture(false, 1.0);
        pointer(&mut h, 60.0, egui::PointerButton::Primary, true);
        if disable {
            h.state_mut().disabled = true;
        } else {
            h.event(egui::Event::WindowFocused(false));
        }
        h.run();
        h.state_mut().disabled = false;
        h.event(egui::Event::WindowFocused(true));
        h.event(egui::Event::PointerMoved(egui::pos2(105.0, 31.0)));
        h.run();
        assert_eq!(h.state().value, 15);
    }
}

#[test]
fn hidden_slider_does_not_resume_an_old_drag_when_it_reappears() {
    let mut h = fixture(false, 1.0);
    pointer(&mut h, 60.0, egui::PointerButton::Primary, true);
    h.state_mut().hidden = true;
    h.run();
    h.step();
    h.state_mut().hidden = false;
    h.event(egui::Event::PointerMoved(egui::pos2(105.0, 31.0)));
    h.run();
    assert_eq!(h.state().value, 15);
}

#[test]
#[ignore = "requires tools/slider_oracle.py; runs in visual CI"]
fn reference_slider_drag_matches_original_handler() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.scratch/reference-slider.tsv");
    let data = std::fs::read_to_string(path).unwrap();
    let mut count = 0;
    let mut value = 0;
    let mut offset = 0;
    for line in data.lines() {
        let fields: Vec<i32> = line.split('\t').map(|s| s.parse().unwrap()).collect();
        if (fields[0] == 1) != cfg!(windows) {
            continue;
        }
        let pressed = fields[4] == 0;
        if pressed {
            value = fields[2] as u8;
            offset = 0;
        }
        value = move_value(value, fields[1], 16, fields[5], &mut offset, pressed);
        assert_eq!(i32::from(value), fields[6], "{line}");
        count += 1;
    }
    assert_eq!(count, 42_432);
}

#[cfg(feature = "visual-tests")]
#[test]
fn palette_slider_visuals_both_themes_scales_and_states() {
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for dark in [false, true] {
        for dpi in [1.0, 2.0] {
            let mut h = Harness::builder()
                .with_size(Vec2::new(150.0, 160.0))
                .with_theme(if dark {
                    egui::Theme::Dark
                } else {
                    egui::Theme::Light
                })
                .with_pixels_per_point(dpi)
                .build_ui(|ui| {
                    for (row, mut value) in [0, 15, 31, 15].into_iter().enumerate() {
                        ui.add_enabled_ui(row != 3, |ui| {
                            let response = ui.put(
                                Rect::from_min_size(
                                    egui::pos2(20.0, 15.0 + 32.0 * row as f32),
                                    Vec2::new(101.0, 22.0),
                                ),
                                NiceSlider(&mut value),
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::Role::Slider,
                                    row != 3,
                                    format!("Channel {row}"),
                                )
                            });
                        });
                    }
                });
            h.remove_cursor();
            h.run();
            snapshots.add(h.try_snapshot(format!(
                "palette-slider-{}-{dpi}x",
                if dark { "dark" } else { "light" }
            )));
        }
    }
}
