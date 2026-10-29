//! master palette-window.cpp and Main_Window palette callbacks. LGPL-3.0-only.
//! Text-measured offsets preserve the original 20px colors and 21px grid pitch.
use super::*;
use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
use polished_map_project::{PaletteDocument, SaveTarget, prepare_palette_export};

const NAMES: [&str; 9] = [
    "GRAY",
    "RED",
    "GREEN",
    "WATER",
    "YELLOW",
    "BROWN",
    "ROOF",
    "TEXT",
    "MONOCHROME",
];
const HUES: [u8; 4] = [0, 2, 1, 3];
const SET_NAMES: [&str; 6] = ["Morn", "Day", "Night", "Dark", "Indoor", "Custom"];

pub(super) struct PaletteForm {
    pub(super) draft: PaletteDocument,
    monochrome: bool,
    selected: ColorSlot,
    hex: String,
    clipboard: Option<(ColorSlot, [u8; 3])>,
    paste_handled: bool,
    swap_held: bool,
    pub(super) windows_buttons: bool,
    #[cfg(test)]
    pub(super) bounds: Rect,
}
impl PaletteForm {
    fn select(&mut self, slot: ColorSlot) {
        self.selected = slot;
        let [r, g, b] = self
            .draft
            .bank()
            .color(slot)
            .expect("validated color button");
        self.hex = format!("{r:02X}{g:02X}{b:02X}");
    }
    fn apply(&mut self, command: PaletteCommand) -> Result<(), String> {
        self.draft
            .apply_batch(&[command])
            .map_err(|e| e.to_string())?;
        self.select(self.selected);
        Ok(())
    }
    fn copy(&mut self) {
        self.clipboard = Some((
            self.selected,
            self.draft
                .bank()
                .copy_color(self.selected)
                .expect("validated color button"),
        ));
    }
    fn paste(&mut self) -> Result<(), String> {
        if let Some((_, rgb)) = self.clipboard {
            self.apply(PaletteCommand::SetRgb {
                slot: self.selected,
                rgb,
            })?;
        }
        Ok(())
    }
    fn swap(&mut self) -> Result<(), String> {
        if let Some((source, _)) = self.clipboard {
            self.apply(PaletteCommand::Swap {
                a: source,
                b: self.selected,
            })?;
        }
        Ok(())
    }
    fn shortcuts(&mut self, ctx: &egui::Context) -> Result<(), String> {
        // Text fields retain their own copy/paste behavior; swatch shortcuts apply
        // when no text editor owns keyboard input. Key-up covers an empty OS clipboard.
        if ctx.egui_wants_keyboard_input() {
            return Ok(());
        }
        for event in ctx.input(|i| i.events.clone()) {
            match event {
                egui::Event::Copy => self.copy(),
                egui::Event::Paste(_) | egui::Event::PasteImage(_) => {
                    self.paste()?;
                    self.paste_handled = true;
                }
                egui::Event::Cut if !self.swap_held => {
                    self.swap()?;
                    self.swap_held = true;
                }
                egui::Event::Key {
                    key: egui::Key::X,
                    pressed: false,
                    ..
                } => self.swap_held = false,
                egui::Event::Key {
                    key: egui::Key::V,
                    pressed: false,
                    modifiers,
                    ..
                } => {
                    if modifiers.command && !self.paste_handled {
                        self.paste()?;
                    }
                    self.paste_handled = false;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Editor {
    #[cfg(windows)]
    pub(super) fn choose_palette_color(&mut self, frame: &eframe::Frame) {
        let Some(slot) = self.native_color_request.take() else {
            return;
        };
        let Some(form) = &mut self.palette_form else {
            return;
        };
        let rgb = form
            .draft
            .bank()
            .color(slot)
            .expect("validated color button");
        match self.native_color_picker.choose(frame, rgb) {
            Ok(Some(rgb)) => {
                if let Err(e) = form.apply(PaletteCommand::SetRgb { slot, rgb }) {
                    self.error = Some(e);
                }
            }
            Ok(None) => {}
            Err(e) => self.error = Some(e),
        }
    }
    pub(super) fn palette_document(&self) -> &PaletteDocument {
        self.loaded
            .as_ref()
            .and_then(|l| l.tileset.as_ref())
            .map(|t| t.palettes())
            .unwrap_or(&self.unloaded_palettes)
    }
    pub(super) fn palette_monochrome(&self) -> bool {
        self.loaded
            .as_ref()
            .and_then(|l| l.tileset.as_ref())
            .map_or(self.monochrome, |t| t.monochrome())
    }
    pub(super) fn prepare_palettes(&mut self) {
        let draft = self.palette_document().clone();
        let monochrome = self.palette_monochrome();
        let selected = ColorSlot {
            set: draft.selected(),
            palette: if monochrome { 8 } else { 0 },
            hue: 0,
        };
        let mut form = PaletteForm {
            draft,
            monochrome,
            selected,
            hex: String::new(),
            clipboard: None,
            paste_handled: false,
            swap_held: false,
            windows_buttons: cfg!(target_os = "windows"),
            #[cfg(test)]
            bounds: Rect::NOTHING,
        };
        form.select(selected);
        self.palette_form = Some(form);
    }
    pub(super) fn commit_palettes(
        &mut self,
        ctx: &egui::Context,
        palettes: PaletteDocument,
    ) -> Result<(), String> {
        if let Some(loaded) = &mut self.loaded
            && let Some(tileset) = &loaded.tileset
        {
            let mut draft = tileset.clone();
            *draft.palettes_mut() = palettes;
            let blocks = loaded
                .blockset
                .as_ref()
                .ok_or("Block definitions are unavailable")?;
            let graphics = draft
                .graphics(blocks.blockset())
                .map_err(|e| e.to_string())?;
            let images = (0..graphics.block_count())
                .map(|id| graphics.render_block(id as u8))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            let textures = images
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
            loaded.tileset = Some(draft);
            loaded.graphics = graphics;
            self.textures = textures;
        } else {
            self.unloaded_palettes = palettes;
        }
        Ok(())
    }
    pub(super) fn load_palette_path(
        &mut self,
        ctx: &egui::Context,
        path: &std::path::Path,
    ) -> Result<(), String> {
        let mut draft = self.palette_document().clone();
        draft.load(path).map_err(|e| e.to_string())?;
        self.commit_palettes(ctx, draft)?;
        self.status = format!("Loaded palettes from {}", path.display());
        Ok(())
    }
    pub(super) fn request_palette_load(&mut self, ctx: &egui::Context, path: PathBuf) {
        if self.palette_document().is_modified() {
            self.pending_palette_load = Some(path);
        } else if let Err(e) = self.load_palette_path(ctx, &path) {
            self.error = Some(e);
        }
    }
    pub(super) fn choose_palettes(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("RGB palettes", &["pal"])
            .pick_file()
        {
            self.request_palette_load(ctx, path);
        }
    }
    pub(super) fn palette_load_dialog(&mut self, ctx: &egui::Context) {
        let mut load = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::unique("replace-palettes")).show(ctx, |ui| {
            ui.heading("Palettes have been edited");
            ui.label("Load the selected palette file anyway?");
            ui.horizontal(|ui| {
                load = ui.button("Load").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        });
        if load {
            let path = self.pending_palette_load.take().unwrap();
            if let Err(e) = self.load_palette_path(ctx, &path) {
                self.error = Some(e);
            }
        } else if cancel || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.pending_palette_load = None;
        }
    }
    pub(super) fn export_palette_target(&mut self, target: &SaveTarget) -> Result<(), String> {
        let mono = self.palette_monochrome();
        let doc = self
            .loaded
            .as_mut()
            .and_then(|l| l.tileset.as_mut())
            .map(|t| t.palettes_mut())
            .unwrap_or(&mut self.unloaded_palettes);
        doc.export(target, mono).map_err(|e| e.to_string())?;
        self.status = format!("Exported palettes to {}", target.path().display());
        Ok(())
    }
    pub(super) fn export_palettes(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("RGB palettes", &["pal"])
            .set_file_name("palettes.pal")
            .save_file()
        {
            let result = prepare_palette_export(&path)
                .map_err(|e| e.to_string())
                .and_then(|target| self.export_palette_target(&target));
            if let Err(e) = result {
                self.error = Some(e);
            }
        }
    }
    pub(super) fn palette_set_toolbar(&mut self, ui: &mut egui::Ui) {
        // Main_Window::initialize and utils.cpp::text_width(s, 3): six
        // points of text padding, 24 extra for the dropdown, then four-space.
        // Toolbar::draw centers the 22-point dropdown in its 24-point row.
        let font = egui::FontId::proportional(13.0);
        let text_width = |text: &str| {
            ui.painter()
                .layout_no_wrap(text.into(), font.clone(), ui.visuals().text_color())
                .size()
                .x
        };
        let label_width = text_width("Palettes:") + 6.0;
        let dropdown_width = text_width("Custom") + 6.0 + 24.0;
        let (label, response) =
            ui.allocate_exact_size(Vec2::new(label_width, 24.0), Sense::hover());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Label, ui.is_enabled(), "Palettes:")
        });
        ui.painter().text(
            Pos2::new(label.left() + 3.0, label.center().y),
            egui::Align2::LEFT_CENTER,
            "Palettes:",
            font.clone(),
            ui.visuals().text_color(),
        );
        let selected = self.palette_document().selected();
        let name = SET_NAMES[selected as usize];
        let response = ui
            .scope(|ui| {
                ui.style_mut().override_font_id = Some(font.clone());
                ui.spacing_mut().interact_size.y = 22.0;
                ui.spacing_mut().button_padding = Vec2::splat(3.0);
                ui.spacing_mut().icon_width = 16.0;
                ui.spacing_mut().item_spacing.x = 3.0;
                egui::ComboBox::from_id_salt("map-palette-set")
                    .width(dropdown_width)
                    .selected_text(egui::RichText::new(name).font(font))
                    .show_ui(ui, |ui| self.palette_set_menu(ui))
                    .response
            })
            .inner;
        response.widget_info(|| {
            let mut info =
                egui::WidgetInfo::labeled(egui::Role::ComboBox, ui.is_enabled(), "Map palette set");
            info.current_text_value = Some(name.into());
            info
        });
        ui.add_space(4.0);
    }
    pub(super) fn palette_set_menu(&mut self, ui: &mut egui::Ui) {
        let selected = self.palette_document().selected();
        for (set, name) in PaletteSet::ALL.into_iter().zip(SET_NAMES) {
            if ui.selectable_label(selected == set, name).clicked() {
                let mut draft = self.palette_document().clone();
                draft.select(set);
                if let Err(e) = self.commit_palettes(ui.ctx(), draft) {
                    self.error = Some(e);
                }
                ui.close();
            }
        }
    }
    pub(super) fn palette_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut form) = self.palette_form.take() else {
            return;
        };
        if let Err(e) = form.shortcuts(ctx) {
            self.error = Some(e);
        }
        let mut command = None;
        let mut accept = false;
        let mut cancel = false;
        let mut hex_enter = false;
        let _modal = egui::Modal::new(egui::Id::unique("edit-palettes")).show(ctx, |ui| {
            ui.heading("Edit Current Palettes");
            let font = egui::FontId::proportional(13.0);
            let text = ui.visuals().text_color();
            let measure = |s: &str, pad: f32, font: egui::FontId| {
                ui.painter().layout_no_wrap(s.into(), font, text).size().x + pad * 2.0
            };
            let off = if form.monochrome {
                measure("MONOCHROME:", 2.0, font.clone())
            } else {
                measure("YELLOW:", 2.0, font.clone()).max(measure("BROWN:", 2.0, font.clone()))
            };
            let (width, height, ox, oy, ss, bx, by) = if form.monochrome {
                (
                    107.0 + off,
                    192.0,
                    10.0,
                    44.0,
                    81.0 + off,
                    off - 77.0,
                    160.0,
                )
            } else {
                (
                    295.0 + off,
                    191.0,
                    107.0 + off,
                    10.0,
                    172.0,
                    111.0 + off,
                    159.0,
                )
            };
            let wo = measure("R:", 2.0, font.clone())
                .max(measure("G:", 2.0, font.clone()))
                .max(measure("B:", 2.0, font.clone()));
            let ww = measure("99", 2.0, font.clone()) + 22.0;
            let hw = measure("Hex: #", 2.0, font.clone());
            let hex_width = measure("AAAAAA", 4.0, egui::FontId::monospace(13.0)).max(measure(
                "FFFFFF",
                4.0,
                egui::FontId::monospace(13.0),
            ));
            let (area, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            ui.painter().rect_filled(
                rect(
                    10.0 + off,
                    10.0,
                    87.0,
                    if form.monochrome { 24.0 } else { 171.0 },
                ),
                0.0,
                ui.visuals().widgets.inactive.bg_fill,
            );
            for row in 0..if form.monochrome { 1 } else { 8 } {
                let palette = if form.monochrome { 8 } else { row };
                ui.painter().text(
                    area.min + Vec2::new(8.0 + off, 22.0 + 21.0 * f32::from(row)),
                    egui::Align2::RIGHT_CENTER,
                    format!("{}:", NAMES[usize::from(palette)]),
                    font.clone(),
                    text,
                );
                for (col, hue) in HUES.into_iter().enumerate() {
                    let slot = ColorSlot {
                        set: form.draft.selected(),
                        palette,
                        hue,
                    };
                    let rgb = form.draft.bank().color(slot).unwrap();
                    let r = rect(
                        12.0 + off + col as f32 * 21.0,
                        12.0 + f32::from(row) * 21.0,
                        20.0,
                        20.0,
                    );
                    let response = ui.put(
                        r,
                        egui::Button::new("")
                            .fill(Color32::from_rgb(rgb[0], rgb[1], rgb[2]))
                            .corner_radius(0.0),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::Role::Button,
                            true,
                            format!("{} color {}", NAMES[usize::from(palette)], col + 1),
                        )
                    });
                    if slot == form.selected {
                        ui.painter().rect_stroke(
                            r.shrink(1.0),
                            0.0,
                            (1.0, Color32::BLACK),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().rect_stroke(
                            r.shrink(2.0),
                            0.0,
                            (1.0, Color32::WHITE),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if response.clicked() {
                        form.select(slot);
                    }
                }
            }
            let rgb = form.draft.bank().color(form.selected).unwrap();
            for (channel, name) in ["R", "G", "B"].into_iter().enumerate() {
                let y = oy + channel as f32 * 26.0;
                ui.painter().text(
                    area.min + Vec2::new(ox + wo - 3.0, y + 11.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("{name}:"),
                    font.clone(),
                    text,
                );
                let mut value = rgb[channel] / 8;
                let spin = ui.put(
                    rect(ox + wo, y, ww, 22.0),
                    egui::DragValue::new(&mut value).range(0..=31),
                );
                spin.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::SpinButton,
                        true,
                        format!("{name} channel"),
                    )
                });
                let slider = ui.put(
                    rect(ox + wo + ww + 6.0, y, ss - wo - ww, 22.0),
                    egui::Slider::new(&mut value, 0..=31).show_value(false),
                );
                slider.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Slider, true, format!("{name} slider"))
                });
                if spin.changed() || slider.changed() {
                    command = Some(PaletteCommand::Channel {
                        slot: form.selected,
                        channel: channel as u8,
                        value,
                    });
                }
            }
            ui.painter().text(
                area.min + Vec2::new(ox + hw - 3.0, oy + 89.0),
                egui::Align2::RIGHT_CENTER,
                "Hex: #",
                font.clone(),
                text,
            );
            let hex = ui.put(
                rect(ox + hw, oy + 78.0, hex_width, 22.0),
                egui::TextEdit::singleline(&mut form.hex)
                    .char_limit(6)
                    .font(egui::TextStyle::Monospace),
            );
            hex.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Hex color"));
            if hex.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                hex_enter = true;
                command = Some(PaletteCommand::Hex {
                    slot: form.selected,
                    value: form.hex.clone(),
                });
            }
            let preview = ui.put(
                rect(ox + hw + hex_width + 4.0, oy + 78.0, 22.0, 22.0),
                egui::Button::new("")
                    .fill(Color32::from_rgb(rgb[0], rgb[1], rgb[2]))
                    .corner_radius(0.0),
            );
            preview.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Button, true, "Color preview")
            });
            if preview.clicked() {
                #[cfg(windows)]
                {
                    self.native_color_request = Some(form.selected);
                }
                #[cfg(not(windows))]
                form.select(form.selected);
            }
            let (ok, no) = if form.windows_buttons {
                (bx, bx + 94.0)
            } else {
                (bx + 94.0, bx)
            };
            accept = ui
                .put(rect(ok, by, 80.0, 22.0), egui::Button::new("OK"))
                .clicked();
            cancel = ui
                .put(rect(no, by, 80.0, 22.0), egui::Button::new("Cancel"))
                .clicked();
        });
        #[cfg(test)]
        {
            form.bounds = _modal.response.rect;
        }
        if !hex_enter && !ctx.egui_wants_keyboard_input() {
            accept |= ctx.input(|i| i.key_pressed(egui::Key::Enter));
        }
        cancel |= ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if let Some(command) = command
            && let Err(e) = form.apply(command)
        {
            self.error = Some(e);
            accept = false;
        }
        if accept && !cancel {
            form.draft.mark_modified();
            match self.commit_palettes(ctx, form.draft.clone()) {
                Ok(()) => {
                    self.status =
                        "Palettes edited; use Export Palettes to write a palette file".into()
                }
                Err(e) => {
                    self.error = Some(e);
                    self.palette_form = Some(form);
                }
            }
        } else if !cancel {
            self.palette_form = Some(form);
        }
    }
}
