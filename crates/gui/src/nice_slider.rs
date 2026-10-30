//! Palette slider from master widgets.cpp::OS_Slider/Default_Slider and
//! FLTK 1.4.5 Fl_Slider.cxx::handle. LGPL-3.0-only adaptation; see NOTICE.md.
//! The source deliberately paints a 13-point grip but uses a 16-point drag
//! extent at height 22. Keep these separate instead of adjusting hit geometry.
use super::*;

#[derive(Clone, Copy)]
struct Capture {
    button: egui::PointerButton,
    offcenter: i32,
    frame: u64,
}

impl Default for Capture {
    fn default() -> Self {
        Self {
            button: egui::PointerButton::Primary,
            offcenter: 0,
            frame: 0,
        }
    }
}

fn position(value: u8, width: i32, grip: i32) -> i32 {
    (f64::from(value) / 31.0 * f64::from(width - grip) + 0.5) as i32
}

fn move_value(
    value: u8,
    width: i32,
    grip: i32,
    mouse: i32,
    offcenter: &mut i32,
    pressed: bool,
) -> u8 {
    let travel = width - grip;
    if travel <= 0 {
        return value;
    }
    if pressed {
        *offcenter = mouse - position(value, width, grip);
        if (0..=grip).contains(offcenter) {
            return value;
        }
        *offcenter = (*offcenter).clamp(0, grip);
    }
    let mut x = mouse - *offcenter;
    let mut retry = pressed;
    loop {
        if x < 0 {
            x = 0;
            *offcenter = mouse.max(0);
        } else if x > travel {
            x = travel;
            *offcenter = (mouse - x).min(grip);
        }
        let scaled = f64::from(x) * 31.0 / f64::from(travel);
        // FL/math.h provides floor(v+.5) for Windows; Unix uses rint with
        // its default ties-to-even mode. Values are nonnegative here.
        let next = if cfg!(windows) {
            (scaled + 0.5).floor()
        } else {
            scaled.round_ties_even()
        } as u8;
        if retry && next == value {
            *offcenter = grip / 2;
            // The original retries with the same xx; only the capture offset
            // changes for the following drag event. Do not recenter this click.
            retry = false;
        } else {
            return next.min(31);
        }
    }
}

pub(super) struct NiceSlider<'a>(pub &'a mut u8);

impl egui::Widget for NiceSlider<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 22.0),
            Sense::click_and_drag(),
        );
        let old = *self.0;
        let mut value = old.min(31);
        let frame = ui.ctx().cumulative_frame_nr();
        let mut capture = ui
            .data_mut(|d| d.remove_temp::<Capture>(response.id))
            .filter(|drag| frame <= drag.frame.saturating_add(1));
        let active = ui.is_enabled() && ui.input(|i| i.focused) && !ui.ctx().any_popup_open();
        let width = rect.width() as i32;
        let height = rect.height() as i32;
        let hit_grip = height / 2 + 5;
        if active {
            for event in ui.input(|i| i.events.clone()) {
                match event {
                    egui::Event::PointerButton {
                        pos,
                        button,
                        pressed: true,
                        ..
                    } if matches!(
                        button,
                        egui::PointerButton::Primary | egui::PointerButton::Secondary
                    ) && capture.is_none()
                        && pos.is_finite()
                        && rect.intersect(ui.clip_rect()).contains(pos)
                        && ui.ctx().layer_id_at(pos) == Some(ui.layer_id()) =>
                    {
                        response.request_focus();
                        let mut drag = Capture {
                            button,
                            offcenter: 0,
                            frame,
                        };
                        value = move_value(
                            value,
                            width,
                            hit_grip,
                            (pos.x - rect.left()) as i32,
                            &mut drag.offcenter,
                            true,
                        );
                        capture = Some(drag);
                    }
                    egui::Event::PointerMoved(pos) if pos.is_finite() => {
                        if let Some(drag) = &mut capture {
                            value = move_value(
                                value,
                                width,
                                hit_grip,
                                (pos.x - rect.left()) as i32,
                                &mut drag.offcenter,
                                false,
                            );
                        }
                    }
                    egui::Event::PointerButton {
                        button,
                        pressed: false,
                        ..
                    } => {
                        if capture.is_some_and(|drag| drag.button == button) {
                            capture = None;
                        }
                    }
                    egui::Event::PointerGone | egui::Event::WindowFocused(false) => capture = None,
                    _ => {}
                }
            }
            if response.has_focus() {
                ui.memory_mut(|m| {
                    m.set_focus_lock_filter(
                        response.id,
                        egui::EventFilter {
                            horizontal_arrows: true,
                            ..Default::default()
                        },
                    )
                });
                ui.input_mut(|i| {
                    for event in &mut i.events {
                        if let egui::Event::Key {
                            key, pressed: true, ..
                        } = event
                        {
                            if *key == egui::Key::ArrowLeft {
                                value = value.saturating_sub(1);
                            }
                            if *key == egui::Key::ArrowRight {
                                value = (value + 1).min(31);
                            }
                        }
                    }
                    i.events.retain(|event| {
                        !matches!(
                            event,
                            egui::Event::Key {
                                key: egui::Key::ArrowLeft | egui::Key::ArrowRight,
                                pressed: true,
                                ..
                            }
                        )
                    });
                });
            }
            ui.input(|i| {
                use egui::accesskit::{Action, ActionData};
                for request in i.accesskit_action_requests(response.id, Action::SetValue) {
                    if let Some(ActionData::NumericValue(n)) = request.data
                        && n.is_finite()
                    {
                        value = n.round().clamp(0.0, 31.0) as u8;
                    }
                }
                for _ in 0..i.num_accesskit_action_requests(response.id, Action::Increment) {
                    value = (value + 1).min(31);
                }
                for _ in 0..i.num_accesskit_action_requests(response.id, Action::Decrement) {
                    value = value.saturating_sub(1);
                }
            });
        } else {
            capture = None;
        }
        if let Some(mut drag) = capture
            && ui.input(|i| i.pointer.button_down(drag.button))
        {
            drag.frame = frame;
            ui.data_mut(|d| d.insert_temp(response.id, drag));
        }
        *self.0 = value;
        if old != value {
            response.mark_changed();
            ui.ctx().request_repaint();
        }
        response.widget_info(|| egui::WidgetInfo::slider(ui.is_enabled(), f64::from(value), ""));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_min_numeric_value(0.0);
            node.set_max_numeric_value(31.0);
            node.set_numeric_value_step(1.0);
            node.set_orientation(egui::accesskit::Orientation::Horizontal);
            for action in [
                egui::accesskit::Action::SetValue,
                egui::accesskit::Action::Increment,
                egui::accesskit::Action::Decrement,
            ] {
                node.add_action(action);
            }
        });
        let dark = ui.visuals().dark_mode;
        // Avoid egui applying its own disabled-opacity transform a second time.
        let painter = egui::Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.clip_rect().intersect(rect),
        );
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_gray(control_paint::background(dark)),
        );
        control_paint::paint(
            &painter,
            rect.min + Vec2::new(0.0, (height / 2 - 2) as f32),
            control_paint::track(dark, ui.is_enabled(), width),
        );
        let grip = height / 2 + 2;
        let origin = rect.min + Vec2::new(position(value, width, grip) as f32, 0.0);
        control_paint::paint(
            &painter,
            origin,
            control_paint::button(dark, ui.is_enabled(), grip, height),
        );
        if active && response.has_focus() {
            let inset = if dark { 1 } else { 2 };
            let w = grip - 2 * inset - 1;
            let h = height - 2 * inset - 1;
            let color = if dark { Color32::WHITE } else { Color32::BLACK };
            // FLTK's generic box focus rectangle: one-pixel alternating dots.
            for x in 0..w {
                for y in [0, h - 1] {
                    if (x + y) % 2 == 0 {
                        painter.rect_filled(
                            Rect::from_min_size(
                                origin + Vec2::new((inset + x) as f32, (inset + y) as f32),
                                Vec2::splat(1.0),
                            ),
                            0.0,
                            color,
                        );
                    }
                }
            }
            for y in 1..h - 1 {
                for x in [0, w - 1] {
                    if (x + y) % 2 == 0 {
                        painter.rect_filled(
                            Rect::from_min_size(
                                origin + Vec2::new((inset + x) as f32, (inset + y) as f32),
                                Vec2::splat(1.0),
                            ),
                            0.0,
                            color,
                        );
                    }
                }
            }
        }
        response
    }
}

#[cfg(test)]
mod tests;
