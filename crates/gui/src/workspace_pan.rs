//! Pinned master widgets.cpp::Workspace::handle, LGPL-3.0-only.
//! The workspace consumes middle presses before dispatching to its children.
use super::*;

#[derive(Default)]
pub(super) struct WorkspacePan {
    anchor: Option<(Pos2, Vec2)>,
}

#[derive(Default)]
pub(super) struct PanFrame {
    pub handled: bool,
    pub offset: Option<Vec2>,
}

/// Workspace bounds exclude each visible perpendicular scrollbar. Visibility
/// is coupled: introducing one scrollbar may make the other necessary.
fn maximum(content: Vec2, outer: Vec2, offset: Vec2, bar: f32, vertical_only: bool) -> Vec2 {
    let mut horizontal = !vertical_only && (content.x > outer.x || offset.x > 0.0);
    let mut vertical = vertical_only || content.y > outer.y || offset.y > 0.0;
    for _ in 0..2 {
        horizontal |= !vertical_only && content.x > outer.x - if vertical { bar } else { 0.0 };
        vertical |= content.y > outer.y - if horizontal { bar } else { 0.0 };
    }
    Vec2::new(
        if vertical_only {
            0.0
        } else {
            (content.x - outer.x + if vertical { bar } else { 0.0 }).max(0.0)
        },
        (content.y - outer.y + if horizontal { bar } else { 0.0 }).max(0.0),
    )
}

fn integer_position(pos: Pos2) -> Pos2 {
    Pos2::new(pos.x.trunc(), pos.y.trunc())
}

impl WorkspacePan {
    pub fn reset(&mut self) {
        self.anchor = None;
    }

    pub fn update(
        &mut self,
        ui: &mut egui::Ui,
        current: Vec2,
        content: Vec2,
        vertical_only: bool,
    ) -> PanFrame {
        let mut frame = PanFrame::default();
        if !ui.is_enabled() || ui.ctx().any_popup_open() || !ui.input(|i| i.focused) {
            self.reset();
            return frame;
        }
        let bounds = ui.max_rect().intersect(ui.clip_rect());
        let limit = maximum(
            content,
            ui.max_rect().size(),
            current,
            ui.spacing().scroll.bar_width,
            vertical_only,
        );
        frame.handled = self.anchor.is_some();
        for event in ui.input(|i| i.events.clone()) {
            match event {
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Middle,
                    pressed: true,
                    ..
                } if pos.is_finite()
                    && bounds.contains(pos)
                    && ui.ctx().layer_id_at(pos) == Some(ui.layer_id()) =>
                {
                    self.anchor = Some((
                        integer_position(pos),
                        Vec2::new(current.x.trunc(), current.y.trunc()),
                    ));
                    frame.handled = true;
                }
                egui::Event::PointerMoved(pos) if pos.is_finite() => {
                    if let Some((origin, offset)) = self.anchor {
                        frame.offset = Some(
                            (offset + (origin - integer_position(pos))).clamp(Vec2::ZERO, limit),
                        );
                    }
                }
                egui::Event::PointerButton {
                    button: egui::PointerButton::Middle,
                    pressed: false,
                    ..
                }
                | egui::Event::PointerGone => self.reset(),
                _ => {}
            }
        }
        // Lost releases and focus changes cannot leave a stale capture behind.
        if !ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle)) {
            self.reset();
        }
        if frame.handled {
            // No drag threshold, rubber band or inertial fling in Workspace.
            ui.spacing_mut().scroll.kinetic.rubber_band = false;
            ui.spacing_mut().scroll.kinetic.decay_time = 0.0;
            frame.offset.get_or_insert(current);
        }
        if self.anchor.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
        }
        frame
    }
}

#[cfg(test)]
mod tests;
