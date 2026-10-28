//! master Tileset_Window::{initialize,apply_modifications} and map-buttons.cpp.
//! Source client area is 466×304, sheet cells 16px and editable pixels 18px.
use super::*;
use polished_map_core::{TileCommand, TileSnapshot, UNDEFINED_COLORS};

const PALETTES: [&str; 10] = [
    "GRAY",
    "RED",
    "GREEN",
    "WATER",
    "YELLOW",
    "BROWN",
    "ROOF",
    "TEXT",
    "MONOCHROME",
    "UNDEFINED",
];
const SWATCHES: [u8; 4] = [0, 2, 1, 3];

pub(super) struct TilesetForm {
    pub(super) draft: MasterTilesetDocument,
    pub(super) selected: u8,
    hue: u8,
    clipboard: Option<(u8, TileSnapshot)>,
    sheet: Option<egui::TextureHandle>,
    refresh: bool,
    drag: Option<u8>,
    last_pixel: Option<usize>,
    paste_handled: bool,
    swap_held: bool,
    pub(super) windows_buttons: bool,
    #[cfg(test)]
    pub(super) bounds: Rect,
}

impl TilesetForm {
    fn enabled(&self, id: u8) -> bool {
        self.draft.allow_256()
            || (id < 0x60 || (!self.draft.monochrome() && (0x80..0xe0).contains(&id)))
    }

    fn apply(&mut self, command: TileCommand) -> Result<(), String> {
        self.draft
            .apply_batch(&[command])
            .map_err(|e| e.to_string())?;
        self.refresh = true;
        Ok(())
    }

    fn colors(&self) -> polished_map_core::Palette {
        let attr = self.draft.tileset().attributes()[usize::from(self.selected)];
        if attr == 9 {
            UNDEFINED_COLORS
        } else {
            self.draft.colors()[usize::from(attr & 15)]
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if !self.refresh {
            return Ok(());
        }
        let mut pixels = vec![0; 128 * 128 * 3];
        for id in 0..=255u8 {
            let rgb = self
                .draft
                .tileset()
                .render_tile(id, self.draft.colors())
                .map_err(|e| e.to_string())?;
            for y in 0..8 {
                let start = ((usize::from(id) / 16 * 8 + y) * 128 + usize::from(id) % 16 * 8) * 3;
                pixels[start..start + 24].copy_from_slice(&rgb.pixels[y * 24..y * 24 + 24]);
            }
        }
        let image = egui::ColorImage::from_rgb([128, 128], &pixels);
        if let Some(sheet) = &mut self.sheet {
            sheet.set(image, egui::TextureOptions::NEAREST);
        } else {
            self.sheet =
                Some(ctx.load_texture("tileset-draft", image, egui::TextureOptions::NEAREST));
        }
        self.refresh = false;
        Ok(())
    }

    fn copy(&mut self) -> Result<(), String> {
        self.clipboard = Some((
            self.selected,
            self.draft
                .tileset()
                .snapshot(self.selected)
                .map_err(|e| e.to_string())?,
        ));
        Ok(())
    }

    fn paste(&mut self) -> Result<(), String> {
        if let Some((_, value)) = &self.clipboard {
            self.apply(TileCommand::Set {
                tile: self.selected,
                value: value.clone(),
            })?;
        }
        Ok(())
    }

    fn swap(&mut self) -> Result<(), String> {
        if let Some((source, _)) = &self.clipboard {
            self.apply(TileCommand::Swap {
                a: *source,
                b: self.selected,
            })?;
        }
        Ok(())
    }

    fn copy_image(&self, ctx: &egui::Context) {
        // Tile::draw_for_clipboard deliberately exports monochrome samples.
        let pixels: Vec<_> = self.draft.tileset().tiles()[usize::from(self.selected)]
            .hues()
            .iter()
            .flat_map(|&h| [[255, 85, 170, 0][usize::from(h)]; 3])
            .collect();
        ctx.copy_image(egui::ColorImage::from_rgb([8, 8], &pixels));
    }

    fn import_rgb(&mut self, width: usize, height: usize, pixels: Vec<u8>) -> Result<(), String> {
        let width = u16::try_from(width).map_err(|_| "Clipboard image is too wide")?;
        let height = u16::try_from(height).map_err(|_| "Clipboard image is too tall")?;
        self.apply(TileCommand::ImportRgb {
            tile: self.selected,
            width,
            height,
            pixels,
            allow_256: self.draft.allow_256(),
        })
    }

    fn paste_image(&mut self) -> Result<(), String> {
        // egui exposes image-copy output but no image-paste request command.
        // Read only on the original image-paste button/shortcut, never on polling.
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        let image = match clipboard.get_image() {
            Ok(image) => image,
            Err(arboard::Error::ContentNotAvailable) => return Ok(()),
            Err(e) => return Err(e.to_string()),
        };
        if image.width > 2048 || image.height > 2048 {
            return Err("Clipboard image exceeds 2048×2048".into());
        }
        let pixels = image
            .bytes
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect();
        self.import_rgb(image.width, image.height, pixels)
    }

    fn shortcuts(&mut self, ctx: &egui::Context) -> Result<(), String> {
        let events = ctx.input(|i| i.events.clone());
        for event in events {
            match event {
                egui::Event::Copy => self.copy()?,
                egui::Event::Cut => {
                    if !self.swap_held {
                        self.swap()?;
                        self.swap_held = true;
                    }
                }
                egui::Event::PasteImage(image) if ctx.input(|i| i.modifiers.shift) => {
                    if image.width() > 2048 || image.height() > 2048 {
                        return Err("Clipboard image exceeds 2048×2048".into());
                    }
                    let pixels = image
                        .pixels
                        .iter()
                        .flat_map(|color| {
                            let rgba = color.to_srgba_unmultiplied();
                            [rgba[0], rgba[1], rgba[2]]
                        })
                        .collect();
                    self.import_rgb(image.width(), image.height(), pixels)?;
                    self.paste_handled = true;
                }
                egui::Event::Paste(_) | egui::Event::PasteImage(_) => {
                    // Plain Ctrl/Cmd+V always uses the private tile snapshot.
                    self.paste()?;
                    self.paste_handled = true;
                }
                egui::Event::Key {
                    key,
                    pressed,
                    repeat,
                    modifiers,
                    ..
                } => {
                    if key == egui::Key::X && !pressed {
                        self.swap_held = false;
                    }
                    if key == egui::Key::V && !pressed {
                        if modifiers.command && !modifiers.shift && !self.paste_handled {
                            self.paste()?;
                        }
                        self.paste_handled = false;
                    }
                    if !pressed || repeat {
                        continue;
                    }
                    if modifiers.command && !modifiers.alt {
                        match (key, modifiers.shift) {
                            (egui::Key::C, true) => self.copy_image(ctx),
                            (egui::Key::V, true) => self.paste_image()?,
                            (egui::Key::C, false) => self.copy()?,
                            (egui::Key::V, false) => {
                                self.paste()?;
                                self.paste_handled = true;
                            }
                            (egui::Key::X, false) if !self.swap_held => {
                                self.swap()?;
                                self.swap_held = true;
                            }
                            _ => {}
                        }
                    } else if key == egui::Key::Delete {
                        self.apply(TileCommand::Delete {
                            tile: self.selected,
                        })?;
                    } else if let Some(index) = [
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
                    .position(|&k| k == key)
                    {
                        if modifiers.alt && !self.draft.monochrome() {
                            let priority = self.draft.tileset().attributes()
                                [usize::from(self.selected)]
                                & 0x80;
                            self.apply(TileCommand::Palette {
                                tile: self.selected,
                                palette: index as u8 | priority,
                            })?;
                        } else if !modifiers.alt && index < 4 {
                            self.hue = SWATCHES[index];
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Editor {
    pub(super) fn prepare_tileset(&mut self) {
        if let Some(doc) = self.loaded.as_ref().and_then(|l| l.tileset.as_ref()) {
            self.tileset_form = Some(TilesetForm {
                draft: doc.clone(),
                selected: 0,
                hue: 0,
                clipboard: None,
                sheet: None,
                refresh: true,
                drag: None,
                last_pixel: None,
                paste_handled: false,
                swap_held: false,
                windows_buttons: cfg!(target_os = "windows"),
                #[cfg(test)]
                bounds: Rect::NOTHING,
            });
        }
    }

    fn accept_tileset(
        &mut self,
        ctx: &egui::Context,
        form: &mut TilesetForm,
    ) -> Result<(), String> {
        let Some(loaded) = &mut self.loaded else {
            return Ok(());
        };
        let Some(blockset) = &loaded.blockset else {
            return Ok(());
        };
        // Upstream accepts all 256 draft tiles and marks dirty even without edits.
        let value = form
            .draft
            .tileset()
            .snapshot(0)
            .map_err(|e| e.to_string())?;
        form.apply(TileCommand::Set { tile: 0, value })?;
        let graphics = form
            .draft
            .graphics(blockset.blockset())
            .map_err(|e| e.to_string())?;
        let images = (0..graphics.block_count())
            .map(|id| graphics.render_block(id as u8))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        self.textures = images
            .into_iter()
            .enumerate()
            .map(|(id, rgb)| {
                ctx.load_texture(
                    format!("block-{id}"),
                    egui::ColorImage::from_rgb([32, 32], &rgb.pixels),
                    egui::TextureOptions::NEAREST,
                )
            })
            .collect();
        loaded.graphics = graphics;
        loaded.tileset = Some(form.draft.clone());
        self.status = "Tileset changed".into();
        Ok(())
    }

    pub(super) fn save_tileset(&mut self) {
        let Some(loaded) = &mut self.loaded else {
            return;
        };
        let (Some(doc), Some(blocks)) = (&mut loaded.tileset, &loaded.blockset) else {
            return;
        };
        if !doc.tileset().is_modified() {
            return;
        }
        match blocks
            .verify_unchanged()
            .and_then(|()| doc.save(blocks.blockset()))
        {
            Ok(()) => self.status = "Tileset saved".into(),
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    pub(super) fn tileset_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut form) = self.tileset_form.take() else {
            return;
        };
        if let Err(e) = form.shortcuts(ctx).and_then(|()| form.update_texture(ctx)) {
            self.error = Some(e);
        }
        let mut accept = false;
        let mut cancel = false;
        let mut command = None;
        let mut paste_image = false;
        let modal = egui::Modal::new(egui::Id::unique("edit-tileset")).show(ctx, |ui| {
            ui.heading("Edit Tileset");
            let (area, _) = ui.allocate_exact_size(Vec2::new(466.0, 304.0), Sense::hover());
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            let font = egui::FontId::proportional(13.0);
            let text = ui.visuals().text_color();
            for frame in [
                rect(10.0, 36.0, 258.0, 258.0),
                rect(278.0, 36.0, 146.0, 146.0),
            ] {
                ui.painter().rect_stroke(
                    frame,
                    0.0,
                    ui.visuals().widgets.inactive.bg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            ui.painter().text(
                area.min + Vec2::new(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!("Tileset: {}", form.draft.name()),
                font.clone(),
                text,
            );
            ui.painter().text(
                area.min + Vec2::new(278.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!("Tile: ${:02X}", form.selected),
                font.clone(),
                text,
            );
            for (x, label, title) in [
                (380.0, "C", "Copy tile image"),
                (402.0, "P", "Paste tile image"),
            ] {
                let key = if label == "C" { "C" } else { "V" };
                let response = ui
                    .put(rect(x, 10.0, 22.0, 22.0), egui::Button::new(""))
                    .on_hover_text(format!("{title} (Ctrl/Cmd+Shift+{key})"));
                response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, title));
                // Code-drawn clipboard/page symbols; no unaudited upstream assets.
                let front = rect(x + 8.0, 17.0, 9.0, 11.0);
                if label == "C" {
                    ui.painter().rect_stroke(
                        rect(x + 5.0, 14.0, 9.0, 11.0),
                        0.0,
                        (1.0, text),
                        egui::StrokeKind::Inside,
                    );
                } else {
                    ui.painter().rect_stroke(
                        rect(x + 5.0, 14.0, 12.0, 15.0),
                        0.0,
                        (1.0, text),
                        egui::StrokeKind::Inside,
                    );
                }
                ui.painter()
                    .rect_filled(front, 0.0, ui.visuals().panel_fill);
                ui.painter()
                    .rect_stroke(front, 0.0, (1.0, text), egui::StrokeKind::Inside);
                if response.clicked() {
                    if label == "C" {
                        form.copy_image(ctx);
                    } else {
                        paste_image = true;
                    }
                }
            }
            for id in 0..=255u8 {
                let cell = rect(
                    11.0 + f32::from(id % 16) * 16.0,
                    37.0 + f32::from(id / 16) * 16.0,
                    16.0,
                    16.0,
                );
                let enabled = form.enabled(id);
                // Positioned cells must not advance the parent layout cursor.
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
                        format!("Tileset tile ${id:02X}"),
                    )
                });
                if let Some(sheet) = &form.sheet {
                    let uv = Rect::from_min_size(
                        Pos2::new(f32::from(id % 16) / 16.0, f32::from(id / 16) / 16.0),
                        Vec2::splat(1.0 / 16.0),
                    );
                    ui.painter().image(sheet.id(), cell, uv, Color32::WHITE);
                }
                if form.selected == id {
                    super::block_editor::selection_border(ui, cell);
                }
                if response.clicked() {
                    form.selected = id;
                }
                if response.drag_started() {
                    form.drag = Some(id);
                }
                if enabled
                    && response.contains_pointer()
                    && ui.input(|i| i.pointer.any_released())
                    && let Some(from) = form.drag
                {
                    command = Some(if ui.input(|i| i.modifiers.ctrl) {
                        TileCommand::Copy { from, to: id }
                    } else {
                        TileCommand::Swap { a: from, b: id }
                    });
                    form.selected = id;
                }
            }
            if ui.input(|i| !i.pointer.primary_down()) {
                form.drag = None;
            }
            let colors = form.colors();
            let pixels = *form.draft.tileset().tiles()[usize::from(form.selected)].hues();
            for (i, &hue) in pixels.iter().enumerate() {
                let cell = rect(
                    279.0 + (i % 8) as f32 * 18.0,
                    37.0 + (i / 8) as f32 * 18.0,
                    18.0,
                    18.0,
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
                    super::block_editor::selection_border(ui, cell);
                }
                if response.secondary_clicked() {
                    form.hue = hue;
                }
                if (response.clicked()
                    || (response.contains_pointer() && ui.input(|i| i.pointer.primary_down())))
                    && form.last_pixel != Some(i)
                {
                    form.last_pixel = Some(i);
                    let at = Point {
                        x: (i % 8) as u16,
                        y: (i / 8) as u16,
                    };
                    let modifiers = ui.input(|i| i.modifiers);
                    command = Some(if modifiers.shift {
                        TileCommand::Fill {
                            tile: form.selected,
                            at,
                            hue: form.hue,
                        }
                    } else if modifiers.ctrl {
                        TileCommand::Replace {
                            tile: form.selected,
                            from: hue,
                            to: form.hue,
                        }
                    } else if modifiers.alt {
                        TileCommand::SwapHues {
                            tile: form.selected,
                            a: hue,
                            b: form.hue,
                        }
                    } else {
                        TileCommand::Paint {
                            tile: form.selected,
                            points: vec![at],
                            hue: form.hue,
                        }
                    });
                }
            }
            if ui.input(|i| !i.pointer.primary_down()) {
                form.last_pixel = None;
            }
            for (i, &hue) in SWATCHES.iter().enumerate() {
                let cell = rect(434.0, 36.0 + i as f32 * 28.0, 22.0, 22.0);
                let c = colors[usize::from(hue)];
                let response = ui.put(
                    cell,
                    egui::Button::new("").fill(Color32::from_rgb(c[0], c[1], c[2])),
                );
                let digit = (i + 1).to_string();
                for offset in [
                    Vec2::new(-1.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(0.0, -1.0),
                    Vec2::new(0.0, 1.0),
                ] {
                    ui.painter().text(
                        cell.center() + offset,
                        egui::Align2::CENTER_CENTER,
                        &digit,
                        egui::FontId::monospace(14.0),
                        Color32::BLACK,
                    );
                }
                ui.painter().text(
                    cell.center(),
                    egui::Align2::CENTER_CENTER,
                    &digit,
                    egui::FontId::monospace(14.0),
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
                    super::block_editor::selection_border(ui, cell);
                }
            }
            let attribute = form.draft.tileset().attributes()[usize::from(form.selected)];
            let mut palette = attribute & 15;
            let mut priority = attribute & 0x80 != 0;
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(rect(278.0, 192.0, 146.0, 22.0)),
                |ui| {
                    ui.add_enabled_ui(!form.draft.monochrome(), |ui| {
                        ui.horizontal(|ui| {
                            let label = ui.label("Palette:");
                            egui::ComboBox::from_id_salt("tile-palette")
                                .selected_text(PALETTES[usize::from(palette)])
                                .width(82.0)
                                .show_ui(ui, |ui| {
                                    for (id, label) in PALETTES.iter().enumerate().take(8) {
                                        ui.selectable_value(&mut palette, id as u8, *label);
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
                        !form.draft.monochrome() && form.draft.allow_priority() && palette < 8,
                        |ui| {
                            ui.checkbox(&mut priority, "Priority (above sprites)");
                        },
                    );
                },
            );
            let updated = palette | if priority { 0x80 } else { 0 };
            if updated != attribute {
                command = Some(TileCommand::Palette {
                    tile: form.selected,
                    palette: updated,
                });
            }
            let (ok, no) = if form.windows_buttons {
                (282.0, 376.0)
            } else {
                (376.0, 282.0)
            };
            accept = ui
                .put(rect(ok, 272.0, 80.0, 22.0), egui::Button::new("OK"))
                .clicked();
            cancel = ui
                .put(rect(no, 272.0, 80.0, 22.0), egui::Button::new("Cancel"))
                .clicked();
        });
        #[cfg(test)]
        {
            form.bounds = modal.response.rect;
        }
        // The first Enter/Escape belongs to an open palette popup.
        if !modal.any_popup_open {
            accept |= ctx.input(|i| i.key_pressed(egui::Key::Enter));
            cancel |= ctx.input(|i| i.key_pressed(egui::Key::Escape));
        }
        if let Some(command) = command
            && let Err(e) = form.apply(command)
        {
            self.error = Some(e);
        }
        if paste_image && let Err(e) = form.paste_image() {
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
