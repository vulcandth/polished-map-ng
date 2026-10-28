//! Logical coordinates from master Block_Window::initialize (466×304).
use super::*;
use polished_map_core::{BlockBrush, BlockCommand, CollisionRow};

pub(super) struct BlockForm {
    id: u8,
    tiles: [u8; 16],
    collision: Option<CollisionRow>,
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
                    collision: doc
                        .blockset()
                        .collisions()
                        .map(|rows| rows[usize::from(self.selected)].clone()),
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
            let labels = [
                "Collision top left",
                "Collision top right",
                "Collision bottom left",
                "Collision bottom right",
            ];
            match &mut form.collision {
                Some(CollisionRow::Binary(values)) => {
                    let font = egui::FontId::proportional(13.0);
                    let width = ["AA", "FF"]
                        .into_iter()
                        .map(|s| {
                            ui.painter()
                                .layout_no_wrap(s.into(), font.clone(), text)
                                .size()
                                .x
                        })
                        .fold(0.0, f32::max)
                        + 22.0;
                    for (i, value) in values.iter_mut().enumerate() {
                        let x = 300.0 + (i % 2) as f32 * (32.0 + width);
                        let y = 144.0 + (i / 2) as f32 * 26.0;
                        quadrant_icon(ui, rect(x - 22.0, y + 4.0, 14.0, 14.0), i);
                        ui.put(
                            rect(x, y, width, 22.0),
                            egui::DragValue::new(value)
                                .range(0..=255)
                                .hexadecimal(2, false, true),
                        )
                        .widget_info(|| {
                            egui::WidgetInfo::labeled(egui::Role::SpinButton, true, labels[i])
                        });
                    }
                }
                values => {
                    let mut empty = std::array::from_fn::<_, 4, _>(|_| String::new());
                    let enabled = values.is_some();
                    let fields = if let Some(CollisionRow::Asm(values)) = values {
                        values
                    } else {
                        &mut empty
                    };
                    for (i, value) in fields.iter_mut().enumerate() {
                        let y = 144.0 + i as f32 * 26.0;
                        quadrant_icon(ui, rect(278.0, y + 4.0, 14.0, 14.0), i);
                        ui.put(
                            rect(300.0, y, 156.0, 22.0),
                            egui::TextEdit::singleline(value).interactive(enabled),
                        )
                        .widget_info(|| {
                            egui::WidgetInfo::labeled(egui::Role::TextInput, enabled, labels[i])
                        });
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
            let mut commands = vec![BlockCommand::Set {
                block: form.id,
                tiles: form.tiles,
            }];
            if let Some(collision) = &form.collision {
                commands.push(BlockCommand::SetCollisions {
                    block: form.id,
                    collision: collision.clone(),
                });
            }
            if !self.apply_block_commands(ctx, &commands) {
                self.block_form = Some(form);
            }
        } else if !cancel {
            self.block_form = Some(form);
        }
    }
}

/// Original draw_selection_border at unzoomed tile/chip scale.
pub(super) fn selection_border(ui: &egui::Ui, cell: Rect) {
    map_buttons::selection(ui.painter(), cell, false);
}

fn quadrant_icon(ui: &egui::Ui, rect: Rect, quadrant: usize) {
    let color = ui.visuals().text_color();
    ui.painter()
        .rect_stroke(rect, 0.0, (1.0, color), egui::StrokeKind::Inside);
    let cell = Rect::from_min_size(
        rect.min + Vec2::new((quadrant % 2) as f32 * 7.0, (quadrant / 2) as f32 * 7.0),
        Vec2::splat(7.0),
    );
    ui.painter().rect_filled(cell.shrink(1.0), 0.0, color);
}
