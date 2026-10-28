//! Logical coordinates from master Block_Window::initialize (466×304).
use super::*;
use polished_map_core::{BlockBrush, BlockCommand};

pub(super) struct BlockForm {
    id: u8,
    tiles: [u8; 16],
    selected: u8,
    textures: Vec<egui::TextureHandle>,
    allow_256: bool,
    pub(super) windows_buttons: bool,
}

impl Editor {
    pub(super) fn prepare_block(&mut self, ctx: &egui::Context) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let Some(doc) = &loaded.blockset else {
            return;
        };
        let Ok(tiles) = doc.blockset().block(self.selected) else {
            return;
        };
        let textures = (0..=255)
            .map(|id| {
                loaded.graphics.render_tile(id).map(|rgb| {
                    ctx.load_texture(
                        format!("edit-tile-{id}"),
                        egui::ColorImage::from_rgb([8, 8], &rgb.pixels),
                        egui::TextureOptions::NEAREST,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>();
        match textures {
            Ok(textures) => {
                self.block_form = Some(BlockForm {
                    id: self.selected,
                    tiles: *tiles,
                    selected: 0,
                    textures,
                    allow_256: loaded.allow_256,
                    windows_buttons: cfg!(target_os = "windows"),
                })
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    pub(super) fn block_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut form) = self.block_form.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::unique("edit-block")).show(ctx, |ui| {
            ui.heading("Edit Block");
            let (area, _) = ui.allocate_exact_size(Vec2::new(466.0, 304.0), Sense::hover());
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            let brush = ui.input(|i| BlockBrush {
                shift: i.modifiers.shift,
                ctrl: i.modifiers.ctrl,
                alt: i.modifiers.alt,
            });
            let font = egui::FontId::proportional(13.0);
            let text = ui.visuals().text_color();
            ui.painter().text(
                area.min + Vec2::new(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                "Tileset",
                font.clone(),
                text,
            );
            ui.painter().text(
                area.min + Vec2::new(268.0, 10.0),
                egui::Align2::RIGHT_TOP,
                if brush.label().is_empty() {
                    format!("Tile: ${:02X}", form.selected)
                } else {
                    brush.label().into()
                },
                font.clone(),
                text,
            );
            ui.painter().text(
                area.min + Vec2::new(278.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!("Block: ${:02X}", form.id),
                font,
                text,
            );
            for (id, texture) in form.textures.iter().enumerate() {
                let cell = rect(
                    11.0 + (id % 16) as f32 * 16.0,
                    37.0 + (id / 16) as f32 * 16.0,
                    16.0,
                    16.0,
                );
                let response = ui.interact(cell, ui.scope_id().with(("tile", id)), Sense::click());
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, true, format!("Tile ${id:02X}"))
                });
                ui.painter().image(
                    texture.id(),
                    cell,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                if usize::from(form.selected) == id {
                    selection_border(ui, cell);
                }
                if response.clicked() {
                    form.selected = id as u8;
                }
            }
            for i in 0..16 {
                let cell = rect(
                    279.0 + (i % 4) as f32 * 24.0,
                    37.0 + (i / 4) as f32 * 24.0,
                    24.0,
                    24.0,
                );
                let response = ui.interact(
                    cell,
                    ui.scope_id().with(("chip", i)),
                    Sense::click_and_drag(),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::Button,
                        true,
                        format!("Block tile {}, {}", i % 4, i / 4),
                    )
                });
                ui.painter().image(
                    form.textures[usize::from(form.tiles[i])].id(),
                    cell,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                if response.hovered() {
                    selection_border(ui, cell);
                }
                if response.secondary_clicked() {
                    form.selected = form.tiles[i];
                }
                if response.clicked()
                    || (response.contains_pointer() && ui.input(|i| i.pointer.primary_down()))
                {
                    // Coordinates are generated from this fixed 4×4 grid.
                    if let Err(e) = brush.paint(
                        &mut form.tiles,
                        Point {
                            x: (i % 4) as u16,
                            y: (i / 4) as u16,
                        },
                        form.selected,
                        form.allow_256,
                    ) {
                        self.error = Some(e.to_string());
                    }
                }
            }
            let (ok_x, cancel_x) = if form.windows_buttons {
                (282.0, 376.0)
            } else {
                (376.0, 282.0)
            };
            accept = ui
                .put(rect(ok_x, 272.0, 80.0, 22.0), egui::Button::new("OK"))
                .clicked()
                || ui.input(|i| i.key_pressed(egui::Key::Enter));
            cancel = ui
                .put(
                    rect(cancel_x, 272.0, 80.0, 22.0),
                    egui::Button::new("Cancel"),
                )
                .clicked()
                || ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if accept && !cancel {
            if let Some(loaded) = &mut self.loaded
                && let Some(doc) = &mut loaded.blockset
            {
                let result = doc
                    .blockset_mut()
                    .apply(&BlockCommand::Set {
                        block: form.id,
                        tiles: form.tiles,
                    })
                    .map_err(|e| e.to_string())
                    .and_then(|()| {
                        loaded
                            .graphics
                            .replace_blocks(doc.blockset())
                            .map_err(|e| e.to_string())
                    })
                    .and_then(|()| {
                        loaded
                            .graphics
                            .render_block(form.id)
                            .map_err(|e| e.to_string())
                    });
                match result {
                    Ok(rgb) => {
                        self.textures[usize::from(form.id)].set(
                            egui::ColorImage::from_rgb([32, 32], &rgb.pixels),
                            egui::TextureOptions::NEAREST,
                        );
                        self.status = format!("Block ${:02X} changed", form.id);
                    }
                    Err(e) => self.error = Some(e),
                }
            }
        } else if !cancel {
            self.block_form = Some(form);
        }
    }
}

/// Original draw_selection_border at unzoomed tile/chip scale.
fn selection_border(ui: &egui::Ui, cell: Rect) {
    for (inset, color) in [
        (0.0, Color32::BLACK),
        (1.0, Color32::WHITE),
        (2.0, Color32::BLACK),
    ] {
        ui.painter().rect_stroke(
            cell.shrink(inset),
            0.0,
            (1.0, color),
            egui::StrokeKind::Inside,
        );
    }
}
