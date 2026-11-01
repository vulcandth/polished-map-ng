//! Pinned plusplus Block_Window::{initialize,refresh,select,change_chip_cb}.
//! Branch-specific occurrence controls reuse the master dialog, collision fields,
//! modal hosting and backend brush algorithm. LGPL-3.0-only.
use super::*;
use polished_map_core::plusplus::{AttributeBlockset, AttributeCommand, BankOrder, TileReference};

const PALETTES: [&str; 8] = [
    "GRAY", "RED", "GREEN", "WATER", "YELLOW", "BROWN", "ROOF", "TEXT",
];

#[cfg(test)]
mod tests;

pub(super) struct Draft {
    pub attributes: [u8; 16],
    order: BankOrder,
    palette: u8,
    x_flip: bool,
    y_flip: bool,
    priority: bool,
    count: u16,
    reveal: bool,
    scroll: workspace_pan::SheetScroll,
    atlas: egui::TextureHandle,
}

pub(super) fn prepare(
    loaded: &Loaded,
    id: u8,
    attributes: [bool; 3],
    ctx: &egui::Context,
) -> Result<BlockForm, String> {
    let assets = loaded.plus_assets.as_ref().ok_or("No plusplus assets")?;
    let MapGraphics::PlusPlus(graphics) = &loaded.graphics else {
        return Err("Plusplus blocks require plusplus graphics".into());
    };
    let row = assets
        .blocks
        .blockset()
        .block(id)
        .map_err(|e| e.to_string())?;
    let collision = assets
        .blocks
        .blockset()
        .offsets()
        .collisions()
        .map(|c| c[usize::from(id)].clone());
    // One atlas holds all eight palette variants. Flips only change UVs, so
    // changing brush attributes never needs to allocate hundreds of textures.
    let mut pixels = vec![0; 128 * 2048 * 3];
    for palette in 0..8 {
        for index in 0..512 {
            let base = TileReference::from_index(index, assets.order).map_err(|e| e.to_string())?;
            let rgb = graphics.render_tile(TileReference::from_bytes(
                base.offset(),
                base.attribute() | palette,
            ));
            for y in 0..8 {
                let at = ((usize::from(palette) * 256 + usize::from(index) / 16 * 8 + y) * 128
                    + usize::from(index) % 16 * 8)
                    * 3;
                pixels[at..at + 24].copy_from_slice(&rgb.pixels[y * 24..y * 24 + 24]);
            }
        }
    }
    let count = if assets.allow_512 || row.iter().any(|t| t.index(assets.order) >= 256) {
        512
    } else {
        256
    };
    Ok(BlockForm {
        id,
        tiles: row.map(|t| t.offset()),
        defaults: match &collision {
            Some(CollisionRow::Binary(values)) => *values,
            _ => [0; 4],
        }
        .map(default_value::DefaultValue::new),
        collision,
        selected: 0,
        plus: Some(Draft {
            attributes: row.map(|t| t.attribute()),
            order: assets.order,
            palette: 0,
            x_flip: attributes[0],
            y_flip: attributes[1],
            priority: attributes[2],
            count,
            reveal: false,
            scroll: Default::default(),
            atlas: ctx.load_texture(
                "plusplus-block-tiles",
                egui::ColorImage::from_rgb([128, 2048], &pixels),
                egui::TextureOptions::NEAREST,
            ),
        }),
        textures: Vec::new(),
        allow_256: false,
        priority: [false; 256],
        // Chip::draw always displays occurrence priority inside this editor.
        show_priority: true,
        windows_buttons: cfg!(target_os = "windows"),
        #[cfg(test)]
        client_bounds: Rect::NOTHING,
    })
}

impl Draft {
    pub fn brush_attributes(&self) -> [bool; 3] {
        [self.x_flip, self.y_flip, self.priority]
    }
    fn reference(&self, index: u16, attributes: bool) -> TileReference {
        let base = TileReference::from_index(index, self.order)
            .expect("selected tile is in the visible sheet");
        TileReference::from_bytes(
            base.offset(),
            base.attribute()
                | self.palette
                | if attributes && self.x_flip { 0x20 } else { 0 }
                | if attributes && self.y_flip { 0x40 } else { 0 }
                | if attributes && self.priority { 0x80 } else { 0 },
        )
    }
    pub fn paint(&self, ui: &egui::Ui, cell: Rect, offset: u8, attribute: u8) {
        let t = TileReference::from_bytes(offset, attribute);
        let index = usize::from(t.index(self.order));
        let x = (index % 16 * 8) as f32;
        let y = (usize::from(t.palette()) * 256 + index / 16 * 8) as f32;
        let u = if t.x_flip() {
            [x + 8.0, x]
        } else {
            [x, x + 8.0]
        };
        let v = if t.y_flip() {
            [y + 8.0, y]
        } else {
            [y, y + 8.0]
        };
        ui.painter().image(
            self.atlas.id(),
            cell,
            Rect::from_min_max(
                Pos2::new(u[0] / 128.0, v[0] / 2048.0),
                Pos2::new(u[1] / 128.0, v[1] / 2048.0),
            ),
            Color32::WHITE,
        );
    }
}

impl BlockForm {
    pub(super) fn tile_label(&self) -> String {
        if let Some(plus) = &self.plus {
            let t = plus.reference(self.selected, false);
            format!("Tile: ${}:{:02X}", u8::from(t.bank1()), t.offset())
        } else {
            format!("Tile: ${:02X}", self.selected)
        }
    }
    pub(super) fn pick(&mut self, slot: usize) {
        if let Some(plus) = &mut self.plus {
            let t = TileReference::from_bytes(self.tiles[slot], plus.attributes[slot]);
            self.selected = t.index(plus.order);
            plus.palette = t.palette();
            plus.x_flip = t.x_flip();
            plus.y_flip = t.y_flip();
            plus.priority = t.priority();
            plus.reveal = true;
        } else {
            self.selected = u16::from(self.tiles[slot]);
        }
    }
    pub(super) fn paint(&mut self, brush: BlockBrush, at: Point) -> Result<(), String> {
        if let Some(plus) = &mut self.plus {
            let mut model = AttributeBlockset::from_bytes(&self.tiles, &plus.attributes)
                .map_err(|e| e.to_string())?;
            model
                .apply(&AttributeCommand::Paint {
                    block: 0,
                    at,
                    tile: self.selected,
                    palette: plus.palette,
                    x_flip: plus.x_flip,
                    y_flip: plus.y_flip,
                    priority: plus.priority,
                    brush,
                    order: plus.order,
                })
                .map_err(|e| e.to_string())?;
            self.tiles = model.offsets().blocks()[0];
            plus.attributes = model.attributes()[0];
            Ok(())
        } else {
            let tile =
                u8::try_from(self.selected).map_err(|_| "Master tile selection exceeds 255")?;
            brush
                .paint(&mut self.tiles, at, tile, self.allow_256)
                .map_err(|e| e.to_string())
        }
    }
    pub(super) fn attribute_shortcuts(&mut self, ctx: &egui::Context) {
        let Some(plus) = &mut self.plus else { return };
        let mut handled = false;
        ctx.input_mut(|i| {
            i.events.retain(|event| {
                let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat,
                    modifiers,
                    ..
                } = event
                else {
                    return true;
                };
                if !modifiers.alt {
                    return true;
                }
                let palette = [
                    egui::Key::Num1,
                    egui::Key::Num2,
                    egui::Key::Num3,
                    egui::Key::Num4,
                    egui::Key::Num5,
                    egui::Key::Num6,
                    egui::Key::Num7,
                    egui::Key::Num8,
                ]
                .iter()
                .position(|k| k == key);
                let known =
                    palette.is_some() || matches!(key, egui::Key::X | egui::Key::Y | egui::Key::P);
                if known && !repeat {
                    if let Some(palette) = palette {
                        plus.palette = palette as u8;
                    }
                    match key {
                        egui::Key::X => plus.x_flip = !plus.x_flip,
                        egui::Key::Y => plus.y_flip = !plus.y_flip,
                        egui::Key::P => plus.priority = !plus.priority,
                        _ => {}
                    }
                }
                handled |= known;
                !known
            });
            // macOS Option may generate text alongside the key event. The
            // attribute shortcut must not also insert text in a collision field.
            if handled {
                i.events.retain(|e| !matches!(e, egui::Event::Text(_)));
            }
        });
    }
    pub(super) fn plusplus_controls(
        &mut self,
        ui: &mut egui::Ui,
        area: Rect,
        overlays: &priority::Textures,
    ) {
        let Some(plus) = &mut self.plus else { return };
        let rect = |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
        let height = if plus.count == 512 { 386.0 } else { 258.0 };
        // FLTK's frame contributes one point per side, with a 15-point scrollbar.
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(rect(11.0, 37.0, 271.0, height - 2.0)),
            |ui| {
                let mut scroll = std::mem::take(&mut plus.scroll);
                scroll.show(
                    ui,
                    "plusplus-block-tiles",
                    f32::from(plus.count),
                    |ui, panning| {
                        let (sheet, _) = ui.allocate_exact_size(
                            Vec2::new(256.0, f32::from(plus.count / 16) * 16.0),
                            Sense::hover(),
                        );
                        for index in 0..plus.count {
                            let cell = Rect::from_min_size(
                                sheet.min
                                    + Vec2::new(
                                        f32::from(index % 16) * 16.0,
                                        f32::from(index / 16) * 16.0,
                                    ),
                                Vec2::splat(16.0),
                            );
                            if plus.reveal && index == self.selected {
                                ui.scroll_to_rect(cell, None);
                            }
                            if !ui.is_rect_visible(cell) {
                                continue;
                            }
                            let t = plus.reference(index, false);
                            plus.paint(ui, cell, t.offset(), t.attribute());
                            let response = ui.interact(
                                cell,
                                ui.scope_id().with(("raw-tile", index)),
                                Sense::click(),
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::Role::Button,
                                    true,
                                    format!("Tile ${}:{:02X}", u8::from(t.bank1()), t.offset()),
                                )
                            });
                            if self.selected == index {
                                selection_border(ui, cell);
                            }
                            if response.clicked() && !panning {
                                self.selected = index;
                            }
                        }
                        plus.reveal = false;
                    },
                );
                plus.scroll = scroll;
            },
        );
        let font = egui::FontId::proportional(13.0);
        ui.style_mut().override_font_id = Some(font.clone());
        ui.spacing_mut().interact_size.y = 22.0;
        let text = ui.visuals().text_color();
        let width = |s: &str| {
            ui.painter()
                .layout_no_wrap(s.into(), font.clone(), text)
                .size()
                .x
                .ceil()
                + 6.0
        };
        let offset = width("Palette:");
        let flip_width = width("X flip").max(width("Y flip"));
        map_options::label(ui, rect(293.0, 176.0, offset, 22.0), "Palette:");
        let response = ui
            .scope_builder(
                egui::UiBuilder::new().max_rect(rect(293.0 + offset, 176.0, 174.0 - offset, 22.0)),
                |ui| {
                    egui::ComboBox::from_id_salt("occurrence-palette")
                        .selected_text(PALETTES[usize::from(plus.palette)])
                        .width(174.0 - offset)
                        .show_ui(ui, |ui| {
                            for (i, name) in PALETTES.iter().enumerate() {
                                ui.selectable_value(&mut plus.palette, i as u8, *name);
                            }
                        })
                        .response
                },
            )
            .inner;
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::ComboBox, true, "Tile palette"));
        ui.put(
            rect(293.0, 202.0, 22.0 + flip_width, 22.0),
            egui::Checkbox::new(&mut plus.x_flip, "X flip"),
        );
        ui.put(
            rect(319.0 + flip_width, 202.0, 22.0 + flip_width, 22.0),
            egui::Checkbox::new(&mut plus.y_flip, "Y flip"),
        );
        ui.put(
            rect(293.0, 228.0, 174.0, 22.0),
            egui::Checkbox::new(&mut plus.priority, "Priority (above sprites)"),
        );
        map_options::label(ui, rect(293.0, 253.0, 174.0, 22.0), "Collision:");
        let t = plus.reference(self.selected, true);
        let preview = rect(294.0, 37.0, 32.0, 32.0);
        plus.paint(ui, preview, t.offset(), t.attribute());
        if self.show_priority && t.priority() {
            overlays.tile(
                ui.painter(),
                preview,
                polished_map_core::PriorityPattern::Chip,
            );
        }
    }
}
