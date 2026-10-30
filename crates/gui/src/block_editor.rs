//! Logical coordinates from master Block_Window::initialize (466×304).
use super::*;
use polished_map_core::{BlockBrush, BlockCommand, CollisionRow};

pub(super) struct BlockForm {
    id: u8,
    tiles: [u8; 16],
    collision: Option<CollisionRow>,
    defaults: [default_value::DefaultValue<u8>; 4],
    selected: u8,
    textures: Vec<egui::TextureHandle>,
    allow_256: bool,
    priority: [bool; 256],
    show_priority: bool,
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
                    defaults: match doc
                        .blockset()
                        .collisions()
                        .map(|rows| &rows[usize::from(self.selected)])
                    {
                        Some(CollisionRow::Binary(values)) => *values,
                        _ => [0; 4],
                    }
                    .map(default_value::DefaultValue::new),
                    collision: doc
                        .blockset()
                        .collisions()
                        .map(|rows| rows[usize::from(self.selected)].clone()),
                    selected: 0,
                    textures,
                    allow_256: loaded.allow_256,
                    priority: std::array::from_fn(|id| loaded.graphics.tile_priority(id as u8)),
                    show_priority: self.show_priority,
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
        let priority = priority::Textures::get(ctx);
        dialog_host::client(ctx, egui::Id::unique("edit-block"), |ui| {
            dialog_host::heading(ui, "Edit Block");
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
                if form.show_priority && form.priority[id] {
                    priority.tile(
                        ui.painter(),
                        cell,
                        polished_map_core::PriorityPattern::Large,
                    );
                }
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
                if form.show_priority && form.priority[usize::from(form.tiles[i])] {
                    priority.tile(ui.painter(), cell, polished_map_core::PriorityPattern::Chip);
                }
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
                        // utils.cpp::text_width(s, 2) adds two points per side.
                        + 4.0 + 22.0;
                    for (i, value) in values.iter_mut().enumerate() {
                        let x = 300.0 + (i % 2) as f32 * (32.0 + width);
                        let y = 144.0 + (i / 2) as f32 * 26.0;
                        quadrant_icon(ui, rect(x, y, width, 22.0), i, true);
                        let response = ui.put(
                            rect(x, y, width, 22.0),
                            numeric_input::IntegerInput::new(&mut *value, 0..=255).hex(),
                        );
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::Role::SpinButton, true, labels[i])
                        });
                        form.defaults[i].restore(ui, &response, value);
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
                        quadrant_icon(ui, rect(300.0, y, 156.0, 22.0), i, enabled);
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

fn quadrant_icon(ui: &egui::Ui, field: Rect, quadrant: usize, enabled: bool) {
    use icons::Icon;
    // OS_Input/OS_Hex_Spinner use outside FL_ALIGN_LEFT. FLTK 1.4.5
    // Fl_Group::draw_outside_label right-aligns the image three points before
    // the field; fl_draw centers the 16-point image in its 22-point height.
    let bounds = Rect::from_min_size(
        Pos2::new(field.left() - 19.0, field.center().y - 8.0),
        Vec2::splat(16.0),
    );
    let icon = [
        Icon::CollisionTopLeft,
        Icon::CollisionTopRight,
        Icon::CollisionBottomLeft,
        Icon::CollisionBottomRight,
    ][quadrant];
    icons::paint(ui, bounds, icon, enabled && ui.is_enabled());
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::{NodeT, Queryable};

    #[test]
    fn collision_labels_follow_source_layout_and_availability() {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        crate::block_workflow_tests::open_block(&mut h, 0);
        assert!(h.state().block_form.as_ref().unwrap().collision.is_none());
        // Snapshot button order is explicit, independent of the CI host.
        h.state_mut().block_form.as_mut().unwrap().windows_buttons = true;
        for collision in [
            None,
            Some(CollisionRow::Asm(std::array::from_fn(|_| "FLOOR".into()))),
            Some(CollisionRow::Binary([0x01, 0x23, 0x45, 0x67])),
        ] {
            let enabled = collision.is_some();
            let binary = matches!(collision, Some(CollisionRow::Binary(_)));
            h.state_mut().block_form.as_mut().unwrap().collision = collision;
            h.remove_cursor();
            h.run();
            let origin = h.get_by_label("Tile $00").rect().min - Vec2::new(11.0, 37.0);
            let first = h.get_by_label("Collision top left").rect();
            for (i, name) in [
                "Collision top left",
                "Collision top right",
                "Collision bottom left",
                "Collision bottom right",
            ]
            .into_iter()
            .enumerate()
            {
                let node = h.get_by_label(name);
                assert_eq!(node.accesskit_node().is_disabled(), !enabled, "{name}");
                let x = 300.0
                    + if binary {
                        (i % 2) as f32 * (32.0 + first.width())
                    } else {
                        0.0
                    };
                let y = 144.0 + if binary { i / 2 } else { i } as f32 * 26.0;
                assert_eq!(node.rect().min - origin, Vec2::new(x, y), "{name}");
                assert_eq!(node.rect().height(), 22.0, "{name}");
            }
            #[cfg(feature = "visual-tests")]
            {
                // Source XPM geometry: outer box at (1..=13, 1..=13),
                // central lines at x/y=7; pink fills one 5×5 quadrant.
                // Check the image in the actual dialog, independently of the
                // icon renderer or its decoded asset accessor.
                let image = h.render().unwrap();
                for i in 0..4 {
                    let x = 281.0
                        + if binary {
                            (i % 2) as f32 * (32.0 + first.width())
                        } else {
                            0.0
                        };
                    let y = 147.0 + if binary { i / 2 } else { i } as f32 * 26.0;
                    for py in 1..=13 {
                        for px in 1..=13 {
                            let rgb = if [1, 7, 13].contains(&px)
                                && [1, 7, 13].contains(&py)
                                && ((px != 7 && py != 7) || (px == 7 && py == 7))
                            {
                                Some([32_u32, 32, 72])
                            } else if [1, 7, 13].contains(&px) || [1, 7, 13].contains(&py) {
                                Some([0, 0, 24])
                            } else if (px > 7) as usize == i % 2 && (py > 7) as usize == i / 2 {
                                Some([249, 89, 89])
                            } else {
                                None
                            };
                            if let Some(rgb) = rgb {
                                let mut expected = rgb.map(|v| v as u8);
                                if !enabled {
                                    let gray = (31 * rgb[0] + 61 * rgb[1] + 8 * rgb[2]) / 100;
                                    expected = [((gray + 206) / 2) as u8; 3];
                                }
                                assert_eq!(
                                    &image
                                        .get_pixel(
                                            (origin.x + x) as u32 + px,
                                            (origin.y + y) as u32 + py
                                        )
                                        .0[..3],
                                    &expected,
                                    "quadrant {i}, pixel ({px}, {py}), enabled {enabled}, binary {binary}",
                                );
                            }
                        }
                    }
                }
                if !enabled {
                    h.snapshot("block-dialog-disabled-collisions");
                }
            }
        }
        h.get_by_label("Cancel").click();
        h.run();
        assert!(!h.state().is_modified());
        assert!(!h.state().loaded.as_ref().unwrap().document.map().can_undo());
    }
}
