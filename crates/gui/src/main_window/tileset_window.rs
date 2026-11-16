//! Pinned master/plusplus Tileset_Window/Roof_Window and map-buttons.cpp.
//! Shared pixel controls and separate format drafts; LGPL-3.0-only.
mod clipboard;
mod dialog;
use super::*;
use polished_map_core::attributed::RawTileCommand as TileCommand;
#[cfg(test)]
mod attributed_tests;
mod draft;
mod pixels;
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
    drag_press: Option<(u16, Pos2)>,
    last_pixel: Option<usize>,
    pixel_primary_down: bool,
    paste_handled: bool,
    pending_image: Option<PendingImage>,
    swap_held: bool,
    pub(super) windows_buttons: bool,
    #[cfg(test)]
    pub(super) bounds: Rect,
    #[cfg(test)]
    client_bounds: Rect,
}

struct PendingImage {
    ctx: egui::Context,
    viewport: egui::ViewportId,
    id: u64,
}
impl Drop for PendingImage {
    fn drop(&mut self) {
        egui_winit::image_clipboard::cancel(&self.ctx, self.viewport, self.id);
    }
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
        let height = if self.draft.is_attributed() && !self.roof {
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

    fn shortcuts(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if self.pending_image.is_some() {
            return Ok(());
        }
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
                            (egui::Key::C, true) => self.copy_image(ctx)?,
                            (egui::Key::V, true) => self.paste_image(ctx)?,
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
                            && self.draft.classic().is_some_and(|d| !d.monochrome())
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
            if self.pending_image.is_some() {
                break;
            }
        }
        Ok(())
    }
}

impl Editor {
    pub(super) fn can_edit_tiles(&self) -> bool {
        self.loaded
            .as_ref()
            .is_some_and(|l| l.tileset.is_some() || l.attributed_assets.is_some())
    }
    pub(super) fn prepare_tileset(&mut self) {
        self.prepare_tile_form(false);
    }
    pub(super) fn prepare_tile_form(&mut self, roof: bool) {
        let Some(loaded) = &self.loaded else { return };
        let draft = if let Some(assets) = &loaded.attributed_assets {
            if roof {
                let Some(doc) = &assets.roof else { return };
                Draft::AttributedRoof(Box::new(doc.clone()))
            } else {
                Draft::AttributedTiles {
                    document: Box::new(assets.tiles.clone()),
                    allow_512: assets.allow_512,
                    order: assets.order,
                }
            }
        } else if let Some(doc) = &loaded.tileset {
            if roof && doc.roof().is_none() {
                return;
            }
            Draft::Classic {
                document: Box::new(doc.clone()),
                roof,
            }
        } else {
            return;
        };
        let show_priority = self.show_priority && !roof && !draft.is_attributed();
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
            drag_press: None,
            last_pixel: None,
            pixel_primary_down: false,
            paste_handled: false,
            pending_image: None,
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
        let storage = if form.draft.is_attributed() {
            documents::Storage::Attributed
        } else {
            documents::Storage::Classic
        };
        if loaded.document.storage() != storage
            || loaded.graphics.storage() != storage
            || !self.application.accepts(storage)
        {
            return Err("Tile draft belongs to another editor profile".into());
        }
        let graphics: MapGraphics = match &form.draft {
            Draft::Classic { document, .. } => document
                .graphics(
                    loaded
                        .blockset
                        .as_ref()
                        .ok_or("No master blockset")?
                        .blockset(),
                )
                .map_err(|e| e.to_string())?
                .into(),
            Draft::AttributedTiles { document, .. } => {
                let assets = loaded
                    .attributed_assets
                    .as_ref()
                    .ok_or("No plusplus assets")?;
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
            Draft::AttributedRoof(roof) => {
                let assets = loaded
                    .attributed_assets
                    .as_ref()
                    .ok_or("No plusplus assets")?;
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
            Draft::Classic { document, .. } => loaded.tileset = Some(document.as_ref().clone()),
            Draft::AttributedTiles { document, .. } => {
                loaded
                    .attributed_assets
                    .as_mut()
                    .ok_or("No plusplus assets")?
                    .tiles = document.as_ref().clone()
            }
            Draft::AttributedRoof(document) => {
                loaded
                    .attributed_assets
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
        if self.save_attributed_tiles() {
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
}
