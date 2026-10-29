//! Pinned master Main_Window::{handle_hotkey,select_metatile} and
//! Metatile_Button::draw. LGPL-3.0-only. Bindings belong to the open map session,
//! not the document, undo history, or saved preferences.
use super::*;

const KEYS: [egui::Key; 10] = [
    egui::Key::Num0,
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
    egui::Key::Num9,
];

#[derive(Default)]
pub(super) struct BlockHotkeys([Option<u8>; 10]);

impl BlockHotkeys {
    fn assign(&mut self, digit: usize, block: u8) {
        // One digit per block and one block per digit, as in the two upstream maps.
        for value in &mut self.0 {
            if *value == Some(block) {
                *value = None;
            }
        }
        self.0[digit] = Some(block);
    }

    pub(super) fn prune(&mut self, count: usize) {
        for value in &mut self.0 {
            if value.is_some_and(|id| usize::from(id) >= count) {
                *value = None;
            }
        }
    }

    pub(super) fn digit(&self, block: u8) -> Option<usize> {
        self.0.iter().position(|&value| value == Some(block))
    }
}

impl Editor {
    pub(super) fn block_hotkey_input(&mut self, ctx: &egui::Context) {
        if self.loaded.is_none() {
            return;
        }
        ctx.input_mut(|input| {
            input.events.retain(|event| {
                let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                else {
                    return true;
                };
                let Some(digit) = KEYS.iter().position(|candidate| candidate == key) else {
                    return true;
                };
                // Deliberately physical Ctrl on every platform, not COMMAND.
                if modifiers.ctrl {
                    if modifiers.shift {
                        self.block_hotkeys.0[digit] = None;
                    } else if usize::from(self.selected) < self.textures.len() {
                        self.block_hotkeys.assign(digit, self.selected);
                    }
                    // Source returns 0 here: e.g. Ctrl+Shift+3/4 also toggles View.
                    true
                } else if let Some(id) = self.block_hotkeys.0[digit]
                    && usize::from(id) < self.textures.len()
                {
                    self.selected = id;
                    self.sidebar_reveal = Some(id);
                    false
                } else {
                    true
                }
            });
        });
    }
}

/// Source half-block thresholds are inclusive. ScrollArea clamps to its content.
pub(super) fn reveal_offset(id: u8, size: f32, height: f32, offset: f32) -> f32 {
    let top = f32::from(id / 4) * size;
    if top >= offset + height - size / 2.0 {
        top + size - height
    } else if top + size <= offset + size / 2.0 {
        top
    } else {
        offset
    }
}

fn label_position(area: Rect, id: u8, zoom: bool, ids: bool, hex: bool) -> Pos2 {
    let z = if zoom { 2.0 } else { 1.0 };
    // update_layout gives the source button width ms+1; draw offsets x by -z-2.
    let lower = if !zoom && ids && !hex && id >= 100 {
        14.0
    } else {
        0.0
    };
    Pos2::new(area.right() - z - 1.0, area.top() + z + lower)
}

pub(super) fn draw(painter: &egui::Painter, area: Rect, digit: usize, id: u8, editor: &Editor) {
    let position = label_position(area, id, editor.zoom, editor.ids, editor.hex);
    let text = digit.to_string();
    // Same bundled font as map IDs; native Courier Bold rasterization is pending.
    let font = egui::FontId::monospace(14.0);
    for delta in [
        Vec2::new(-1.0, -1.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, 1.0),
    ] {
        painter.text(
            position + delta,
            egui::Align2::RIGHT_TOP,
            &text,
            font.clone(),
            Color32::WHITE,
        );
    }
    painter.text(position, egui::Align2::RIGHT_TOP, text, font, Color32::RED);
}

#[cfg(test)]
mod tests;
