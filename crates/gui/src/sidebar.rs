//! Block sidebar rendering, selection and dragging; upstream main-window.cpp/map-buttons.cpp.
//! LGPL-3.0-only. Source revisions are pinned in upstream.lock.toml.
use super::*;

impl Editor {
    pub(super) fn sidebar(&mut self, ui: &mut egui::Ui, block_size: f32) {
        if self.reset_sidebar_scroll || self.sidebar_reveal.is_some() {
            self.sidebar_pan.reset();
        }
        let pan = self.sidebar_pan.update(
            ui,
            Vec2::new(0.0, self.sidebar_scroll),
            Vec2::new(
                block_size * 4.0,
                block_size * self.textures.len().div_ceil(4) as f32,
            ),
            true,
        );
        if pan.handled {
            self.sidebar_press = None;
            self.sidebar_drag = None;
        }
        let priority = self
            .show_priority
            .then(|| priority::Textures::get(ui.ctx()));
        let mut scroll = egui::ScrollArea::vertical()
            .id_salt("blocks")
            .auto_shrink([false, false])
            .scroll_source(egui::scroll_area::ScrollSource {
                drag: egui::scroll_area::DragScroll::Never,
                ..Default::default()
            })
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);
        if let Some(offset) = pan.offset {
            scroll = scroll
                .scroll_offset(offset)
                .scroll_source(egui::scroll_area::ScrollSource {
                    mouse_wheel: true,
                    ..egui::scroll_area::ScrollSource::NONE
                });
        }
        if self.reset_sidebar_scroll {
            scroll = scroll.scroll_offset(Vec2::ZERO);
            self.reset_sidebar_scroll = false;
            self.sidebar_reveal = None;
        } else if let Some(id) = self.sidebar_reveal.take() {
            let offset = block_hotkeys::reveal_offset(
                id,
                block_size,
                ui.available_height(),
                self.sidebar_scroll,
            );
            scroll = scroll.scroll_offset(Vec2::new(0.0, offset));
            ui.spacing_mut().scroll.kinetic.rubber_band = false;
            ui.spacing_mut().scroll.kinetic.decay_time = 0.0;
        }
        let output = scroll.show(ui, |ui| {
            let rows = self.textures.len().div_ceil(4);
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(block_size * 4.0, block_size * rows as f32),
                Sense::click_and_drag(),
            );
            let hit = |pos: Pos2| {
                let local = pos - rect.min;
                let col = (local.x / block_size).floor() as i32;
                let row = (local.y / block_size).floor() as i32;
                let id = row * 4 + col;
                (rect.contains(pos)
                    && ui.clip_rect().contains(pos)
                    && (0..4).contains(&col)
                    && row >= 0
                    && (id as usize) < self.textures.len())
                .then_some(id as u8)
            };
            let drops = asset_drag::transfers(
                ui,
                self.drag_and_drop && !pan.handled,
                &mut self.sidebar_drag_press,
                &mut self.sidebar_drag,
                hit,
            );
            // Metatile_Button sends its release callback to the pressed button;
            // select_metatile_cb acts only if the pointer is still over it.
            // Keep that cell identity even though egui uses one sidebar region.
            let mut released = None;
            if ui.is_enabled() && !pan.handled {
                for event in ui.input(|i| i.events.clone()) {
                    match event {
                        egui::Event::PointerButton {
                            pos,
                            button,
                            pressed: true,
                            ..
                        } if button != egui::PointerButton::Middle
                            && (response.contains_pointer()
                                || response.is_pointer_button_down_on()) =>
                        {
                            self.sidebar_press = hit(pos).map(|id| (button, id));
                        }
                        egui::Event::PointerButton {
                            pos,
                            button,
                            pressed: false,
                            ..
                        } => {
                            if let Some((held, id)) = self.sidebar_press
                                && held == button
                            {
                                self.sidebar_press = None;
                                if response.contains_pointer() && hit(pos) == Some(id) {
                                    released = Some((id, button));
                                }
                            }
                        }
                        egui::Event::PointerGone => self.sidebar_press = None,
                        _ => {}
                    }
                }
            }
            if !ui.is_enabled() || !ui.input(|i| i.pointer.any_down()) {
                self.sidebar_press = None;
            }
            for (id, texture) in self.textures.iter().enumerate() {
                let cell = Rect::from_min_size(
                    rect.min
                        + Vec2::new((id % 4) as f32 * block_size, (id / 4) as f32 * block_size),
                    Vec2::splat(block_size),
                );
                if ui.is_rect_visible(cell) {
                    ui.painter().image(
                        texture.id(),
                        cell,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    if let (Some(priority), Some(loaded)) = (&priority, &self.loaded) {
                        priority.block(
                            ui.painter(),
                            cell,
                            loaded.graphics.block_priorities(id as u8),
                            self.zoom,
                        );
                    }
                    let outlined = id == usize::from(self.selected)
                        || ui.is_enabled()
                            && self.sidebar_drag.is_some_and(|from| {
                                id == usize::from(from) || ui.rect_contains_pointer(cell)
                            });
                    map_buttons::decorate(
                        ui.painter(),
                        cell,
                        self.zoom,
                        self.grid,
                        outlined,
                        ui.visuals().dark_mode,
                    );
                    if self.ids {
                        map_labels::draw_id(
                            ui.painter(),
                            cell,
                            id as u8,
                            self.hex,
                            self.zoom,
                            false,
                        );
                    }
                    if let Some(digit) = self.block_hotkeys.digit(id as u8) {
                        block_hotkeys::draw(ui.painter(), cell, digit, id as u8, self);
                    }
                }
            }
            if let Some((id, button)) = released {
                self.selected = id;
                if button == egui::PointerButton::Secondary {
                    self.prepare_block(ui.ctx());
                }
            }
            for asset_drag::Transfer { from, to, copy } in drops {
                if from == to {
                    continue;
                }
                let command = if copy {
                    polished_map_core::BlockCommand::Copy { from, to }
                } else {
                    polished_map_core::BlockCommand::Swap { a: from, b: to }
                };
                if self.apply_block_commands(ui.ctx(), &[command]) {
                    self.selected = to;
                }
            }
        });
        self.sidebar_scroll = output.state.clamped_offset().y;
    }
}
