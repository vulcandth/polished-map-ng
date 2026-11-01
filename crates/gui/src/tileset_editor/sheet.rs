//! Shared Deep_Tile_Button selection/drag behavior, with the plusplus vertical
//! sheet from Tileset_Window::initialize/refresh. LGPL-3.0-only.
use super::*;

impl TilesetForm {
    fn sheet_cell(
        &mut self,
        ui: &mut egui::Ui,
        id: u16,
        cell: Rect,
        drag: bool,
        priority: &priority::Textures,
        panning: bool,
    ) -> Option<TileCommand> {
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
        if panning {
            return None;
        }
        if response.clicked() {
            self.selected = id;
        }
        if drag && enabled && response.drag_started_by(egui::PointerButton::Primary) {
            self.drag = Some(id);
        }
        if enabled
            && response.contains_pointer()
            && ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
            && let Some(from) = self.drag
        {
            self.selected = id;
            return Some(if ui.input(|i| i.modifiers.ctrl) {
                TileCommand::Copy { from, to: id }
            } else {
                TileCommand::Swap { a: from, b: id }
            });
        }
        None
    }
    pub(super) fn tile_sheet(
        &mut self,
        ui: &mut egui::Ui,
        origin: Pos2,
        drag: bool,
        priority: &priority::Textures,
    ) -> Option<TileCommand> {
        let mut command = None;
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
                            for index in 0..count {
                                let cell = Rect::from_min_size(
                                    sheet.min
                                        + Vec2::new(
                                            (index % 16) as f32 * 16.0,
                                            (index / 16) as f32 * 16.0,
                                        ),
                                    Vec2::splat(16.0),
                                );
                                if ui.is_rect_visible(cell) {
                                    command = self
                                        .sheet_cell(ui, index as u16, cell, drag, priority, panning)
                                        .or(command.take());
                                }
                            }
                        },
                    );
                    self.sheet_scroll = scroll;
                },
            );
        } else {
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
                command = self
                    .sheet_cell(ui, id, cell, drag, priority, false)
                    .or(command.take());
            }
        }
        if ui.input(|i| !i.pointer.primary_down()) {
            self.drag = None;
        }
        command
    }
}
