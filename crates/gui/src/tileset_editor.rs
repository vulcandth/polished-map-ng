//! Pinned master/plusplus Tileset_Window/Roof_Window and map-buttons.cpp.
//! Shared pixel controls and separate format drafts; LGPL-3.0-only.
use super::*;
use polished_map_core::plusplus::RawTileCommand as TileCommand;
mod draft;
#[cfg(test)]
mod plusplus_tests;
mod sheet;
use draft::{Draft, Snapshot};

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
    pub(super) draft: Draft,
    pub(super) selected: u16,
    roof: bool,
    show_priority: bool,
    hue: u8,
    clipboard: Option<(u16, Snapshot)>,
    sheet: Option<egui::TextureHandle>,
    refresh: bool,
    sheet_scroll: workspace_pan::SheetScroll,
    drag: Option<u16>,
    last_pixel: Option<usize>,
    paste_handled: bool,
    swap_held: bool,
    pub(super) windows_buttons: bool,
    #[cfg(test)]
    pub(super) bounds: Rect,
    #[cfg(test)]
    client_bounds: Rect,
}

impl TilesetForm {
    fn enabled(&self, id: u16) -> bool {
        self.draft.enabled(id)
    }

    fn apply(&mut self, command: TileCommand) -> Result<(), String> {
        self.draft.apply(command)?;
        self.refresh = true;
        Ok(())
    }

    fn colors(&self) -> polished_map_core::Palette {
        self.draft.colors(self.selected)
    }

    fn update_texture(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if !self.refresh {
            return Ok(());
        }
        let height = if self.draft.is_plus() && !self.roof {
            256
        } else {
            128
        };
        let mut pixels = vec![0; 128 * height * 3];
        for index in 0..self.draft.count() {
            let id = index as u16;
            let rgb = self.draft.render_tile(id)?;
            for y in 0..8 {
                let start = ((usize::from(id) / 16 * 8 + y) * 128 + usize::from(id) % 16 * 8) * 3;
                pixels[start..start + 24].copy_from_slice(&rgb.pixels[y * 24..y * 24 + 24]);
            }
        }
        let image = egui::ColorImage::from_rgb([128, height], &pixels);
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
        self.clipboard = Some((self.selected, self.draft.snapshot(self.selected)?));
        Ok(())
    }

    fn paste(&mut self) -> Result<(), String> {
        if let Some((_, value)) = &self.clipboard {
            self.draft.set_snapshot(self.selected, value.clone())?;
            self.refresh = true;
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
        let pixels: Vec<_> = self
            .draft
            .pixels(self.selected)
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
            allow_512: self.draft.allow_extended(),
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
                    } else if key == egui::Key::Delete && !self.roof {
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
                        if modifiers.alt
                            && self.draft.master().is_some_and(|d| !d.monochrome())
                            && !self.roof
                        {
                            let priority = self.draft.attribute(self.selected).unwrap_or(0) & 0x80;
                            self.draft
                                .set_palette(self.selected, index as u8 | priority)?;
                            self.refresh = true;
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
    pub(super) fn can_edit_tiles(&self) -> bool {
        self.loaded
            .as_ref()
            .is_some_and(|l| l.tileset.is_some() || l.plus_assets.is_some())
    }
    pub(super) fn can_edit_roof(&self) -> bool {
        self.loaded.as_ref().is_some_and(|l| {
            l.tileset.as_ref().is_some_and(|t| t.roof().is_some())
                || l.plus_assets.as_ref().is_some_and(|a| a.roof.is_some())
        })
    }
    pub(super) fn prepare_tileset(&mut self) {
        self.prepare_tile_form(false);
    }
    pub(super) fn prepare_roof(&mut self) {
        self.prepare_tile_form(true);
    }
    fn prepare_tile_form(&mut self, roof: bool) {
        let Some(loaded) = &self.loaded else { return };
        let draft = if let Some(assets) = &loaded.plus_assets {
            if roof {
                let Some(doc) = &assets.roof else { return };
                Draft::PlusRoof(Box::new(doc.clone()))
            } else {
                Draft::PlusTiles {
                    document: Box::new(assets.tiles.clone()),
                    allow_512: assets.allow_512,
                    order: assets.order,
                }
            }
        } else if let Some(doc) = &loaded.tileset {
            if roof && doc.roof().is_none() {
                return;
            }
            Draft::Master {
                document: Box::new(doc.clone()),
                roof,
            }
        } else {
            return;
        };
        let show_priority = self.show_priority && !roof && !draft.is_plus();
        self.tileset_form = Some(TilesetForm {
            draft,
            selected: if roof { 6 } else { 0 },
            roof,
            // Roof_Window::show has no priority argument. Avoid the original
            // Deep_Tile_Button's invalid Roof_Window -> Tileset_Window cast.
            show_priority,
            hue: 0,
            clipboard: None,
            sheet: None,
            refresh: true,
            sheet_scroll: Default::default(),
            drag: None,
            last_pixel: None,
            paste_handled: false,
            swap_held: false,
            windows_buttons: cfg!(target_os = "windows"),
            #[cfg(test)]
            bounds: Rect::NOTHING,
            #[cfg(test)]
            client_bounds: Rect::NOTHING,
        });
    }

    fn accept_tileset(
        &mut self,
        ctx: &egui::Context,
        form: &mut TilesetForm,
    ) -> Result<(), String> {
        let Some(loaded) = &mut self.loaded else {
            return Ok(());
        };
        // Upstream accepts the complete draft and marks dirty even without edits.
        let value = form.draft.snapshot(0)?;
        form.draft.set_snapshot(0, value)?;
        let profile = if form.draft.is_plus() {
            documents::Profile::PlusPlus
        } else {
            documents::Profile::Master
        };
        if loaded.document.profile() != profile
            || loaded.graphics.profile() != profile
            || self.profile != profile
        {
            return Err("Tile draft belongs to another editor profile".into());
        }
        let graphics: MapGraphics = match &form.draft {
            Draft::Master { document, .. } => document
                .graphics(
                    loaded
                        .blockset
                        .as_ref()
                        .ok_or("No master blockset")?
                        .blockset(),
                )
                .map_err(|e| e.to_string())?
                .into(),
            Draft::PlusTiles { document, .. } => {
                let assets = loaded.plus_assets.as_ref().ok_or("No plusplus assets")?;
                let g = document.graphics(
                    assets.blocks.blockset(),
                    assets.palettes.colors(assets.palettes.selected()),
                    assets.order,
                );
                match &assets.roof {
                    Some(roof) => g.with_roof(roof.tiles()).into(),
                    None => g.into(),
                }
            }
            Draft::PlusRoof(roof) => {
                let assets = loaded.plus_assets.as_ref().ok_or("No plusplus assets")?;
                assets
                    .tiles
                    .graphics(
                        assets.blocks.blockset(),
                        assets.palettes.colors(assets.palettes.selected()),
                        assets.order,
                    )
                    .with_roof(roof.tiles())
                    .into()
            }
        };
        let textures = documents::block_textures(ctx, &graphics)?;
        match &form.draft {
            Draft::Master { document, .. } => loaded.tileset = Some(document.as_ref().clone()),
            Draft::PlusTiles { document, .. } => {
                loaded
                    .plus_assets
                    .as_mut()
                    .ok_or("No plusplus assets")?
                    .tiles = document.as_ref().clone()
            }
            Draft::PlusRoof(document) => {
                loaded
                    .plus_assets
                    .as_mut()
                    .ok_or("No plusplus assets")?
                    .roof = Some(document.as_ref().clone())
            }
        }
        self.textures = textures;
        loaded.graphics = graphics;
        self.status = if form.roof {
            "Roof changed"
        } else {
            "Tileset changed"
        }
        .into();
        Ok(())
    }

    pub(super) fn save_tileset(&mut self) {
        if self.save_plusplus_tiles() {
            return;
        }
        let Some(loaded) = &mut self.loaded else {
            return;
        };
        let (Some(doc), Some(blocks)) = (&mut loaded.tileset, &loaded.blockset) else {
            return;
        };
        if !doc.tileset().is_modified() {
            let name = doc.name().to_owned();
            self.saved_message(name);
            return;
        }
        // The source acknowledges the main PNG once, even for compound sheets,
        // followed by its palette map. Use observed destinations, not new path discovery.
        let expected = format!("{}.png", doc.name());
        let outputs = doc.outputs();
        let graphics = outputs
            .iter()
            .find(|t| t.path().file_name() == Some(std::ffi::OsStr::new(&expected)))
            .map_or(expected, |t| save_feedback::basename(t.path()));
        let palette = (!doc.monochrome())
            .then(|| outputs.last().map(|t| save_feedback::basename(t.path())))
            .flatten();
        match blocks
            .verify_unchanged()
            .and_then(|()| doc.save(blocks.blockset()))
        {
            Ok(()) => {
                self.status = "Tileset saved".into();
                self.saved_message(graphics);
                if let Some(palette) = palette {
                    self.saved_message(palette);
                }
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    pub(super) fn save_roof(&mut self) {
        if self.save_plusplus_roof() {
            return;
        }
        let Some(roof) = self
            .loaded
            .as_mut()
            .and_then(|l| l.tileset.as_mut())
            .and_then(|t| t.roof_mut())
        else {
            return;
        };
        if !roof.roof().tileset().is_modified() {
            let name = roof.name().to_owned();
            self.saved_message(name);
            return;
        }
        let name = save_feedback::basename(roof.output().path());
        match roof.save() {
            Ok(()) => {
                self.status = "Roof saved".into();
                self.saved_message(name);
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    pub(super) fn tileset_dialog(&mut self, ctx: &egui::Context) {
        let priority = priority::Textures::get(ctx);
        let Some(mut form) = self.tileset_form.take() else {
            return;
        };
        if !self.drag_and_drop {
            form.drag = None;
        }
        if let Err(e) = form.shortcuts(ctx).and_then(|()| form.update_texture(ctx)) {
            self.error = Some(e);
        }
        let mut accept = false;
        let mut cancel = false;
        let mut command = None;
        let mut paste_image = false;
        let modal = dialog_host::client(ctx, egui::Id::unique("edit-tileset"), |ui| {
            dialog_host::heading(
                ui,
                if form.roof {
                    "Edit Roof"
                } else {
                    "Edit Tileset"
                },
            );
            let plus = form.draft.is_plus();
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
                        form.copy_image(ctx);
                    } else {
                        paste_image = true;
                    }
                }
            }
            command = form.tile_sheet(
                ui,
                area.min + Vec2::new(11.0, 37.0),
                self.drag_and_drop,
                &priority,
            );
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
                    super::block_editor::selection_border(ui, cell);
                }
                if response.secondary_clicked()
                    || (response.contains_pointer()
                        && ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Secondary)))
                {
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
            if !form.roof && !form.draft.is_plus() {
                let attribute = form.draft.attribute(form.selected).expect("master tile");
                let mut palette = attribute & 15;
                let mut priority = attribute & 0x80 != 0;
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(rect(278.0, 192.0, 146.0, 22.0)),
                    |ui| {
                        ui.add_enabled_ui(!form.draft.master().unwrap().monochrome(), |ui| {
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
                            !form.draft.master().unwrap().monochrome()
                                && form.draft.master().unwrap().allow_priority()
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
