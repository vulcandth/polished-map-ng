//! Pinned master/plusplus widgets.cpp::Workspace::handle, LGPL-3.0-only.
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

/// Plusplus block/raw-tile lists use the same Workspace gesture. Callers pass
/// the inset client rectangle: its size already subtracts the source frame,
/// equivalent to plusplus's extra box_dw/box_dh term in the outer-bound formula.
#[derive(Default)]
pub(super) struct SheetScroll {
    pan: WorkspacePan,
    offset: f32,
    initialized: bool,
}

impl SheetScroll {
    pub fn show<R>(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        content_height: f32,
        add_contents: impl FnOnce(&mut egui::Ui, bool) -> R,
    ) -> R {
        ui.spacing_mut().scroll.bar_width = 15.0;
        ui.spacing_mut().scroll.bar_inner_margin = 0.0;
        ui.spacing_mut().scroll.bar_outer_margin = 0.0;
        let pan = self.pan.update(
            ui,
            Vec2::new(0.0, self.offset),
            Vec2::new(256.0, content_height),
            true,
        );
        let mut scroll = egui::ScrollArea::vertical()
            .id_salt(id)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .max_height(ui.max_rect().height())
            .auto_shrink([false, false])
            .scroll_source(egui::scroll_area::ScrollSource {
                drag: egui::scroll_area::DragScroll::Never,
                ..Default::default()
            });
        if !self.initialized {
            scroll = scroll.vertical_scroll_offset(0.0);
        } else if let Some(offset) = pan.offset {
            scroll = scroll.vertical_scroll_offset(offset.y).scroll_source(
                egui::scroll_area::ScrollSource {
                    mouse_wheel: true,
                    ..egui::scroll_area::ScrollSource::NONE
                },
            );
        }
        let output = scroll.show(ui, |ui| add_contents(ui, pan.handled));
        self.offset = output.state.clamped_offset().y;
        self.initialized = true;
        output.inner
    }

    #[cfg(test)]
    pub fn offset(&self) -> f32 {
        self.offset
    }
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
