//! Internal transfers from pinned master/plusplus map-buttons.cpp's
//! Metatile_Button, Deep_Tile_Button and Color_Button handlers. LGPL-3.0-only.
//! Preserve delivered event order; a complete drag can arrive in one frame.
use crate::egui::{Event, PointerButton, Pos2, Ui};

pub(super) struct Transfer<T> {
    pub from: T,
    pub to: T,
    pub copy: bool,
}

pub(super) fn transfers<T: Copy + Eq>(
    ui: &Ui,
    enabled: bool,
    press: &mut Option<(T, Pos2)>,
    dragging: &mut Option<T>,
    hit: impl Fn(Pos2) -> Option<T>,
) -> Vec<Transfer<T>> {
    let mut output = Vec::new();
    if !enabled || !ui.is_enabled() || ui.ctx().any_popup_open() || !ui.input(|i| i.focused) {
        *press = None;
        *dragging = None;
        return output;
    }
    let hit = |pos: Pos2| {
        (pos.is_finite()
            && ui.clip_rect().contains(pos)
            && ui.ctx().layer_id_at(pos) == Some(ui.layer_id()))
        .then(|| hit(pos))
        .flatten()
    };
    for event in ui.input(|i| i.events.clone()) {
        match event {
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                ..
            } => {
                *press = hit(pos).map(|id| (id, pos));
                *dragging = None;
            }
            Event::PointerMoved(pos) if pos.is_finite() => {
                if let Some((id, origin)) = *press
                    && pos != origin
                {
                    *dragging = Some(id);
                }
            }
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers,
            } => {
                if let Some(from) = dragging.take()
                    && let Some(to) = hit(pos)
                {
                    output.push(Transfer {
                        from,
                        to,
                        copy: modifiers.ctrl,
                    });
                }
                *press = None;
            }
            Event::PointerGone | Event::WindowFocused(false) => {
                *press = None;
                *dragging = None;
            }
            _ => {}
        }
    }
    if !ui.input(|i| i.pointer.primary_down()) {
        *press = None;
        *dragging = None;
    }
    output
}
