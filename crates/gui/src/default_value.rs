//! Middle-release defaults from master widgets.cpp Default_{Hex_,}Spinner
//! and Default_Slider. LGPL-3.0-only; see upstream.lock.toml.
use super::*;

pub(super) struct DefaultValue<T> {
    initial: T,
    armed: bool,
    focus_on_press: bool,
}

#[cfg(test)]
mod tests;

impl<T: Copy> DefaultValue<T> {
    pub fn new(initial: T) -> Self {
        Self {
            initial,
            armed: false,
            focus_on_press: false,
        }
    }

    pub fn slider(initial: T) -> Self {
        Self {
            focus_on_press: true,
            ..Self::new(initial)
        }
    }

    /// A press captures this control; release restores even after dragging away.
    /// Store this state with the form so closing it also abandons the capture.
    pub fn restore(&mut self, ui: &egui::Ui, response: &egui::Response, value: &mut T) -> bool {
        if !response.enabled() || !ui.input(|i| i.focused) || ui.ctx().any_popup_open() {
            self.armed = false;
            return false;
        }
        let bounds = response.rect.intersect(ui.clip_rect());
        let mut restored = false;
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
                    self.armed = true;
                    if self.focus_on_press {
                        response.request_focus();
                    }
                }
                egui::Event::PointerButton {
                    button: egui::PointerButton::Middle,
                    pressed: false,
                    ..
                } => {
                    if self.armed {
                        *value = self.initial;
                        restored = true;
                    }
                    self.armed = false;
                }
                egui::Event::PointerGone | egui::Event::WindowFocused(false) => self.armed = false,
                _ => {}
            }
        }
        if !ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle)) {
            self.armed = false;
        }
        if restored {
            ui.ctx().request_repaint();
        }
        restored
    }
}
