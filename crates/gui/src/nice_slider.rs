//! Palette slider from master widgets.cpp::OS_Slider/Default_Slider and
//! FLTK 1.4.5 Fl_Slider.cxx::handle. LGPL-3.0-only adaptation; see NOTICE.md.
//! Retains source interaction with the earlier rounded egui slider appearance.
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
        let inner = ui.horizontal(|ui| self.show(ui));
        inner.inner | inner.response
    }
}

impl NiceSlider<'_> {
    fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let thickness = ui
            .text_style_height(&egui::TextStyle::Body)
            .max(ui.spacing().interact_size.y);
        let mut response = ui.allocate_response(
            Vec2::new(ui.spacing().slider_width, thickness),
            Sense::click_and_drag(),
        );
        let rect = response.rect;
        let old = *self.0;
        let mut value = old.min(31);
        let frame = ui.ctx().cumulative_frame_nr();
        let mut capture = ui
            .data_mut(|d| d.remove_temp::<Capture>(response.id))
            .filter(|drag| frame <= drag.frame.saturating_add(1));
        let active = ui.is_enabled() && ui.input(|i| i.focused) && !ui.ctx().any_popup_open();
        let width = rect.width() as i32;
        // Keep the original 22-point source control's drag extent independently
        // of the theme's painted handle size.
        let hit_grip = 16;
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
        // Match egui 0.37's slider rail/round handle using the current theme.
        // Painting is separate from the source-compatible input above.
        let visuals = ui.style().interact(&response);
        let inactive = &ui.visuals().widgets.inactive;
        let rail = Rect::from_center_size(
            rect.center(),
            Vec2::new(rect.width(), ui.spacing().slider_rail_height.max(0.0)),
        );
        ui.painter()
            .rect_filled(rail, inactive.corner_radius, inactive.bg_fill);
        let radius = rect.height() / 2.5;
        let half_width = match ui.visuals().handle_shape {
            egui::style::HandleShape::Circle => radius,
            egui::style::HandleShape::Rect { aspect_ratio } => radius * aspect_ratio,
        };
        let center = egui::pos2(
            egui::lerp(
                (rect.left() + half_width)..=(rect.right() - half_width),
                f32::from(value) / 31.0,
            ),
            rect.center().y,
        );
        if ui.visuals().slider_trailing_fill {
            let mut fill = rail;
            fill.max.x = (center.x + f32::from(inactive.corner_radius.nw)).min(rail.right());
            ui.painter()
                .rect_filled(fill, inactive.corner_radius, ui.visuals().selection.bg_fill);
        }
        match ui.visuals().handle_shape {
            egui::style::HandleShape::Circle => {
                ui.painter().circle(
                    center,
                    radius + visuals.expansion,
                    visuals.bg_fill,
                    visuals.fg_stroke,
                );
            }
            egui::style::HandleShape::Rect { .. } => {
                ui.painter().rect(
                    Rect::from_center_size(center, 2.0 * Vec2::new(half_width, radius))
                        .expand(visuals.expansion),
                    visuals.corner_radius,
                    visuals.bg_fill,
                    visuals.fg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }
        response
    }
}

#[cfg(test)]
mod tests;
