//! Shared Deep_Tile_Button selection/drag behavior, with the plusplus vertical
//! sheet from Tileset_Window::initialize/refresh. LGPL-3.0-only.
use super::*;

impl TilesetForm {
    fn sheet_cell(
        &mut self,
        ui: &mut egui::Ui,
        id: u16,
        cell: Rect,
        priority: &priority::Textures,
        panning: bool,
    ) {
        let enabled = self.enabled(id);
        let mut tile_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(cell)
                .id_salt(("tileset-tile", id)),
        );
        if !enabled {
            tile_ui.disable();
        }
        let response = tile_ui.interact(cell, tile_ui.scope_id(), Sense::click_and_drag());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::Button,
                enabled,
                format!(
                    "{} tile {}",
                    if self.roof { "Roof" } else { "Tileset" },
                    self.draft.label(id, self.roof)
                ),
            )
        });
        if let Some(sheet) = &self.sheet {
            let rows = if self.draft.is_plus() && !self.roof {
                32.0
            } else {
                16.0
            };
            let uv = Rect::from_min_size(
                Pos2::new(f32::from(id % 16) / 16.0, f32::from(id / 16) / rows),
                Vec2::new(1.0 / 16.0, 1.0 / rows),
            );
            ui.painter().image(sheet.id(), cell, uv, Color32::WHITE);
        }
        if self.show_priority && self.draft.attribute(id).is_some_and(|a| a & 0x80 != 0) {
            priority.tile(
                ui.painter(),
                cell,
                polished_map_core::PriorityPattern::Large,
            );
        }
        if self.selected == id
            || self
                .drag
                .is_some_and(|from| from == id || enabled && response.contains_pointer())
        {
            super::super::block_editor::selection_border(ui, cell);
        }
        if !panning && response.clicked() {
            self.selected = id;
        }
    }

    fn sheet_transfers(
        &mut self,
        ui: &egui::Ui,
        cells: &[(u16, Rect)],
        enabled: bool,
    ) -> Vec<TileCommand> {
        let enabled_cells: Vec<_> = cells
            .iter()
            .copied()
            .filter(|(id, _)| self.enabled(*id))
            .collect();
        asset_drag::transfers(ui, enabled, &mut self.drag_press, &mut self.drag, |pos| {
            enabled_cells
                .iter()
                .find(|(_, rect)| {
                    rect.contains(pos) && pos.x < rect.right() && pos.y < rect.bottom()
                })
                .map(|(id, _)| *id)
        })
        .into_iter()
        .filter_map(|transfer| {
            self.selected = transfer.to;
            if transfer.from == transfer.to {
                return None;
            }
            Some(if transfer.copy {
                TileCommand::Copy {
                    from: transfer.from,
                    to: transfer.to,
                }
            } else {
                TileCommand::Swap {
                    a: transfer.from,
                    b: transfer.to,
                }
            })
        })
        .collect()
    }
    pub(super) fn tile_sheet(
        &mut self,
        ui: &mut egui::Ui,
        origin: Pos2,
        drag: bool,
        priority: &priority::Textures,
    ) -> Result<(), String> {
        let mut commands = Vec::new();
        if self.draft.is_plus() && !self.roof {
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(origin, Vec2::new(271.0, 256.0))),
                |ui| {
                    let mut scroll = std::mem::take(&mut self.sheet_scroll);
                    scroll.show(
                        ui,
                        "raw-tileset-sheet",
                        self.draft.visible_count() as f32,
                        |ui, panning| {
                            if panning {
                                self.drag = None;
                            }
                            let count = self.draft.visible_count();
                            let (sheet, _) = ui.allocate_exact_size(
                                Vec2::new(256.0, count as f32),
                                Sense::hover(),
                            );
                            let cells: Vec<_> = (0..count)
                                .map(|index| {
                                    (
                                        index as u16,
                                        Rect::from_min_size(
                                            sheet.min
                                                + Vec2::new(
                                                    (index % 16) as f32 * 16.0,
                                                    (index / 16) as f32 * 16.0,
                                                ),
                                            Vec2::splat(16.0),
                                        ),
                                    )
                                })
                                .collect();
                            commands.extend(self.sheet_transfers(ui, &cells, drag && !panning));
                            for (id, cell) in cells {
                                if ui.is_rect_visible(cell) {
                                    self.sheet_cell(ui, id, cell, priority, panning);
                                }
                            }
                        },
                    );
                    self.sheet_scroll = scroll;
                },
            );
        } else {
            let mut cells = Vec::new();
            for index in 0..self.draft.count() {
                let id = index as u16;
                let cell = Rect::from_min_size(
                    origin
                        + Vec2::new(
                            f32::from(if self.roof { id % 3 } else { id % 16 }) * 16.0,
                            f32::from(if self.roof { 2 - id / 3 } else { id / 16 }) * 16.0,
                        ),
                    Vec2::splat(16.0),
                );
                cells.push((id, cell));
            }
            commands.extend(self.sheet_transfers(ui, &cells, drag));
            for (id, cell) in cells {
                self.sheet_cell(ui, id, cell, priority, false);
            }
        }
        for command in commands {
            self.apply(command)?;
        }
        Ok(())
    }
}
