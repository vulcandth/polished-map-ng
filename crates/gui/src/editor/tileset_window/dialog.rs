//! Tile/roof editor presentation and input; editing stays in the owning draft. LGPL-3.0-only.
use super::*;

impl Editor {
    pub(in crate::editor) fn tileset_dialog(&mut self, ctx: &egui::Context) {
        let priority = priority::Textures::get(ctx);
        let Some(mut form) = self.tileset_form.take() else {
            return;
        };
        if !self.drag_and_drop {
            form.drag = None;
        }
        if let Err(e) = form
            .poll_image()
            .and_then(|()| form.shortcuts(ctx))
            .and_then(|()| form.update_texture(ctx))
        {
            self.error = Some(e);
        }
        let mut accept = false;
        let mut cancel = false;
        let mut paste_image = false;
        let modal = dialog_host::client(ctx, egui::Id::unique("edit-tileset"), |ui| {
            if form.pending_image.is_some() {
                ui.disable();
            }
            dialog_host::heading(
                ui,
                if form.roof {
                    "Edit Roof"
                } else {
                    "Edit Tileset"
                },
            );
            let plus = form.draft.is_attributed();
            let size = if form.roof {
                Vec2::new(226.0, 228.0)
            } else if plus {
                Vec2::new(441.0, 304.0)
            } else {
                Vec2::new(466.0, 304.0)
            };
            let (area, _) = ui.allocate_exact_size(size, Sense::hover());
            #[cfg(test)]
            {
                form.client_bounds = area;
            }
            let tile_x = if form.roof {
                70.0
            } else if plus {
                293.0
            } else {
                278.0
            };
            let sheet_size = if form.roof { 50.0 } else { 258.0 };
            let sheet_width = if plus && !form.roof {
                273.0
            } else {
                sheet_size
            };
            let pixel_size = if plus { 17.0 } else { 18.0 };
            // Plusplus retains the roof frame's original 146pt extent even
            // though its common PIXEL_ZOOM_FACTOR is 17 (136pt of pixels).
            let pixel_frame = if plus && !form.roof { 138.0 } else { 146.0 };
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            let font = egui::FontId::proportional(13.0);
            let text = ui.visuals().text_color();
            let heading_x = if form.roof {
                216.0
                    - ui.painter()
                        .layout_no_wrap(
                            if plus { "Tile: $0:AA" } else { "Tile: $FFF" }.into(),
                            font.clone(),
                            text,
                        )
                        .size()
                        .x
                    - 2.0
            } else {
                tile_x
            };
            for frame in [
                rect(10.0, 36.0, sheet_width, sheet_size),
                rect(tile_x, 36.0, pixel_frame, pixel_frame),
            ] {
                ui.painter().rect_stroke(
                    frame,
                    0.0,
                    ui.visuals().widgets.inactive.bg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            // Source Label uses FL_ALIGN_CLIP: long names cannot cover Tile ID.
            ui.painter()
                .with_clip_rect(rect(
                    10.0,
                    10.0,
                    if form.roof {
                        heading_x - 10.0
                    } else {
                        sheet_width
                    },
                    22.0,
                ))
                .text(
                    area.min + Vec2::new(10.0, 10.0),
                    egui::Align2::LEFT_TOP,
                    if form.roof {
                        format!("Roof: {}", form.draft.name())
                    } else {
                        format!("Tileset: {}", form.draft.name())
                    },
                    font.clone(),
                    text,
                );
            ui.painter().text(
                area.min + Vec2::new(heading_x, 10.0),
                egui::Align2::LEFT_TOP,
                format!("Tile: {}", form.draft.label(form.selected, form.roof)),
                font.clone(),
                text,
            );
            let toolbar_y = if form.roof { 160.0 } else { 10.0 };
            for (x, label, title) in [
                (
                    if form.roof {
                        16.0
                    } else if plus {
                        387.0
                    } else {
                        380.0
                    },
                    "C",
                    "Copy tile image",
                ),
                (
                    if form.roof {
                        38.0
                    } else if plus {
                        409.0
                    } else {
                        402.0
                    },
                    "P",
                    "Paste tile image",
                ),
            ] {
                let key = if label == "C" { "C" } else { "V" };
                let icon = if label == "C" {
                    icons::Icon::Copy
                } else {
                    icons::Icon::Paste
                };
                let response = ui
                    .put(
                        rect(x, toolbar_y, 22.0, 22.0),
                        icons::Button::new(icon, title).size(22.0),
                    )
                    .on_hover_text(format!("{title} (Ctrl/Cmd+Shift+{key})"));
                if response.clicked() {
                    if label == "C" {
                        if let Err(e) = form.copy_image(ctx) {
                            self.error = Some(e);
                        }
                    } else {
                        paste_image = true;
                    }
                }
            }
            if let Err(error) = form.tile_sheet(
                ui,
                area.min + Vec2::new(11.0, 37.0),
                self.drag_and_drop,
                &priority,
            ) {
                self.error = Some(error);
            }
            let pixel_grid = rect(tile_x + 1.0, 37.0, 8.0 * pixel_size, 8.0 * pixel_size);
            if let Err(error) = form.pixel_input(ui, pixel_grid, pixel_size) {
                self.error = Some(error);
            }
            let colors = form.colors();
            let pixels = form.draft.pixels(form.selected);
            for (i, &hue) in pixels.iter().enumerate() {
                let cell = rect(
                    (tile_x + 1.0) + (i % 8) as f32 * pixel_size,
                    37.0 + (i / 8) as f32 * pixel_size,
                    pixel_size,
                    pixel_size,
                );
                let response = ui.interact(
                    cell,
                    ui.scope_id().with(("pixel", i)),
                    Sense::click_and_drag(),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::Button,
                        true,
                        format!("Pixel {}, {}", i % 8, i / 8),
                    )
                });
                ui.painter().rect_filled(
                    cell,
                    0.0,
                    Color32::from_rgb(
                        colors[usize::from(hue)][0],
                        colors[usize::from(hue)][1],
                        colors[usize::from(hue)][2],
                    ),
                );
                if response.hovered() {
                    super::block_window::selection_border(ui, cell);
                }
                // Keyboard/accessibility activation has no native pointer event.
                // Pointer clicks were already handled in chronological order.
                if response.clicked()
                    && !ui.input(|input| {
                        input.events.iter().any(|event| {
                            matches!(
                                event,
                                egui::Event::PointerButton {
                                    button: egui::PointerButton::Primary,
                                    ..
                                }
                            )
                        })
                    })
                    && let Err(error) = form.paint_pixel(i, ui.input(|input| input.modifiers))
                {
                    self.error = Some(error);
                }
            }
            for (i, &hue) in SWATCHES.iter().enumerate() {
                let cell = if form.roof {
                    rect(
                        10.0 + (i % 2) as f32 * 28.0,
                        100.0 + (i / 2) as f32 * 28.0,
                        22.0,
                        22.0,
                    )
                } else if plus {
                    rect(293.0 + i as f32 * 28.0, 184.0, 22.0, 22.0)
                } else {
                    rect(434.0, 36.0 + i as f32 * 28.0, 22.0, 22.0)
                };
                let c = colors[usize::from(hue)];
                let response = ui.put(
                    cell,
                    egui::Button::new("")
                        .min_size(cell.size())
                        .corner_radius(0)
                        .fill(Color32::from_rgb(c[0], c[1], c[2])),
                );
                let digit = (i + 1).to_string();
                let galley = ui.painter().layout_no_wrap(
                    digit,
                    egui::FontId::monospace(14.0),
                    Color32::WHITE,
                );
                // Center the visible glyph, not its font's asymmetric line box.
                let origin = response.rect.center() - galley.mesh_bounds.center().to_vec2();
                for offset in [
                    Vec2::new(-1.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(0.0, -1.0),
                    Vec2::new(0.0, 1.0),
                ] {
                    ui.painter().galley_with_override_text_color(
                        origin + offset,
                        galley.clone(),
                        Color32::BLACK,
                    );
                }
                ui.painter().galley_with_override_text_color(
                    origin,
                    galley,
                    if form.hue == hue {
                        Color32::YELLOW
                    } else {
                        Color32::WHITE
                    },
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, true, format!("Swatch {}", i + 1))
                });
                if response.clicked() {
                    form.hue = hue;
                }
                if form.hue == hue {
                    super::block_window::selection_border(ui, cell);
                }
            }
            if !form.roof && !form.draft.is_attributed() {
                let attribute = form.draft.attribute(form.selected).expect("master tile");
                let mut palette = attribute & 15;
                let mut priority = attribute & 0x80 != 0;
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(rect(278.0, 192.0, 146.0, 22.0)),
                    |ui| {
                        ui.add_enabled_ui(!form.draft.classic().unwrap().monochrome(), |ui| {
                            ui.horizontal(|ui| {
                                let label = ui.label("Palette:");
                                crate::editor::widgets::dropdown::combo("tile-palette")
                                    .selected_text(PALETTES[usize::from(palette)])
                                    .width(82.0)
                                    .show_ui(ui, |ui| {
                                        for (id, label) in PALETTES.iter().enumerate().take(8) {
                                            crate::editor::widgets::dropdown::value(
                                                ui,
                                                &mut palette,
                                                id as u8,
                                                *label,
                                            );
                                        }
                                    })
                                    .response
                                    .labelled_by(label.id);
                            });
                        });
                    },
                );
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(rect(278.0, 218.0, 178.0, 22.0)),
                    |ui| {
                        ui.add_enabled_ui(
                            !form.draft.classic().unwrap().monochrome()
                                && form.draft.classic().unwrap().allow_priority()
                                && palette < 8,
                            |ui| {
                                ui.checkbox(&mut priority, "Priority (above sprites)");
                            },
                        );
                    },
                );
                let updated = palette | if priority { 0x80 } else { 0 };
                if updated != attribute {
                    if let Err(e) = form.draft.set_palette(form.selected, updated) {
                        self.error = Some(e);
                    }
                    form.refresh = true;
                }
            }
            let (ok, no) = if form.windows_buttons {
                (282.0, 376.0)
            } else {
                (376.0, 282.0)
            };
            let (ok, no, ok_y, no_y) = if form.roof {
                (ok - 240.0, no - 240.0, 196.0, 196.0)
            } else if plus {
                if form.windows_buttons {
                    (351.0, 351.0, 233.0, 272.0)
                } else {
                    (351.0, 351.0, 272.0, 233.0)
                }
            } else {
                (ok, no, 272.0, 272.0)
            };
            accept = ui
                .put(rect(ok, ok_y, 80.0, 22.0), egui::Button::new("OK"))
                .clicked();
            cancel = ui
                .put(rect(no, no_y, 80.0, 22.0), egui::Button::new("Cancel"))
                .clicked();
        });
        #[cfg(test)]
        {
            form.bounds = modal.response.rect;
        }
        // The first Enter/Escape belongs to an open palette popup.
        if !modal.any_popup_open {
            if form.pending_image.is_none() {
                accept |= ctx.input(|i| i.key_pressed(egui::Key::Enter));
            }
            cancel |= ctx.input(|i| i.key_pressed(egui::Key::Escape));
        }
        if paste_image && let Err(e) = form.paste_image(ctx) {
            self.error = Some(e);
        }
        if form.refresh {
            ctx.request_repaint();
        }
        if accept && !cancel {
            if let Err(e) = self.accept_tileset(ctx, &mut form) {
                self.error = Some(e);
                self.tileset_form = Some(form);
            }
        } else if !cancel {
            self.tileset_form = Some(form);
        }
    }
}
