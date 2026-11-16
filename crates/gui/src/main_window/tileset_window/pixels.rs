//! Pinned master/plusplus map-buttons.cpp::Pixel_Button::handle and
//! Tileset_Window/Roof_Window::change_pixel_cb. Process delivered pointer events
//! in order, including a complete gesture in one frame; do not invent pixels
//! between native motion events. LGPL-3.0-only.
use super::*;

impl TilesetForm {
    pub(super) fn paint_pixel(
        &mut self,
        index: usize,
        modifiers: egui::Modifiers,
    ) -> Result<(), String> {
        let from = self.draft.pixels(self.selected)[index];
        let at = Point {
            x: (index % 8) as u16,
            y: (index / 8) as u16,
        };
        self.apply(if modifiers.shift {
            TileCommand::Fill {
                tile: self.selected,
                at,
                hue: self.hue,
            }
        } else if modifiers.ctrl {
            TileCommand::Replace {
                tile: self.selected,
                from,
                to: self.hue,
            }
        } else if modifiers.alt {
            TileCommand::SwapHues {
                tile: self.selected,
                a: from,
                b: self.hue,
            }
        } else {
            TileCommand::Paint {
                tile: self.selected,
                points: vec![at],
                hue: self.hue,
            }
        })
    }

    pub(super) fn pixel_input(
        &mut self,
        ui: &egui::Ui,
        grid: Rect,
        size: f32,
    ) -> Result<(), String> {
        if !ui.is_enabled() || self.pending_image.is_some() || egui::Popup::is_any_open(ui.ctx()) {
            self.pixel_primary_down = false;
            self.last_pixel = None;
            return Ok(());
        }
        let hit = |pos: Pos2| {
            (grid.contains(pos)
                && pos.x < grid.right()
                && pos.y < grid.bottom()
                && ui.clip_rect().contains(pos)
                && ui.ctx().layer_id_at(pos) == Some(ui.layer_id()))
            .then(|| {
                ((pos.y - grid.top()) / size) as usize * 8 + ((pos.x - grid.left()) / size) as usize
            })
        };
        let (events, modifiers, down) =
            ui.input(|i| (i.events.clone(), i.modifiers, i.pointer.primary_down()));
        for event in events {
            let paint = match event {
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers,
                } => {
                    self.pixel_primary_down = pressed;
                    self.last_pixel = None;
                    pressed.then(|| (hit(pos), modifiers))
                }
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Secondary,
                    pressed: true,
                    ..
                } => {
                    if let Some(index) = hit(pos) {
                        self.hue = self.draft.pixels(self.selected)[index];
                    }
                    None
                }
                egui::Event::PointerMoved(pos) if self.pixel_primary_down => {
                    Some((hit(pos), modifiers))
                }
                egui::Event::PointerGone => {
                    self.last_pixel = None;
                    None
                }
                egui::Event::WindowFocused(false) => {
                    self.pixel_primary_down = false;
                    self.last_pixel = None;
                    None
                }
                _ => None,
            };
            if let Some((pixel, modifiers)) = paint
                && pixel != self.last_pixel
            {
                self.last_pixel = pixel;
                if let Some(index) = pixel {
                    self.paint_pixel(index, modifiers)?;
                }
            }
        }
        if !down {
            self.pixel_primary_down = false;
            self.last_pixel = None;
        }
        Ok(())
    }
}
