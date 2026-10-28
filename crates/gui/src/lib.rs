//! FLTK-derived logical geometry and egui adapter. LGPL-3.0-only.
mod block_editor;
mod block_workflows;
pub mod geometry;
mod palette_editor;
#[cfg(test)]
mod palette_editor_tests;
mod tileset_editor;
use block_editor::BlockForm;

#[cfg(test)]
mod block_workflow_tests;
#[cfg(test)]
mod roof_editor_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tileset_editor_tests;

use std::{path::PathBuf, sync::mpsc};

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Vec2};
use polished_map_core::{Command, Graphics, Point, ResizeAnchor};
use polished_map_project::{
    BlocksetDocument, MapDocument, MasterGraphicsOptions, MasterTilesetDocument, OpenOptions,
    discover_map, list_tilesets, prepare_blockset_save_as, prepare_collision_save_as,
    prepare_save_as, write_png,
};

struct Loaded {
    document: MapDocument,
    graphics: Graphics,
    blockset: Option<BlocksetDocument>,
    tileset: Option<MasterTilesetDocument>,
    allow_256: bool,
}
type LoadResult = Result<Loaded, String>;

#[derive(Default)]
struct OpenForm {
    new_map: bool,
    path: PathBuf,
    root: String,
    tileset: String,
    roof: String,
    width: u16,
    height: u16,
    allow_256: bool,
}

struct ResizeForm {
    width: u16,
    height: u16,
    anchor: ResizeAnchor,
}

pub struct Editor {
    loaded: Option<Loaded>,
    textures: Vec<egui::TextureHandle>,
    loading: Option<mpsc::Receiver<LoadResult>>,
    export: Option<mpsc::Receiver<Result<(), String>>>,
    open_form: Option<OpenForm>,
    resize_form: Option<ResizeForm>,
    block_form: Option<BlockForm>,
    tileset_form: Option<tileset_editor::TilesetForm>,
    palette_form: Option<palette_editor::PaletteForm>,
    unloaded_palettes: polished_map_project::PaletteDocument,
    pending_palette_load: Option<PathBuf>,
    #[cfg(windows)]
    native_color_request: Option<polished_map_core::ColorSlot>,
    #[cfg(windows)]
    native_color_picker: polished_map_platform::ColorPicker,
    block_count_form: Option<u16>,
    block_count_windows_buttons: bool,
    block_clipboard: Option<block_workflows::BlockClipboard>,
    paste_shortcut_handled: bool,
    sidebar_drag: Option<u8>,
    reset_sidebar_scroll: bool,
    resize_anchor: ResizeAnchor,
    reset_canvas_scroll: bool,
    error: Option<String>,
    selected: u8,
    stroke: Vec<Point>,
    zoom: bool,
    grid: bool,
    dark: bool,
    confirm_discard: bool,
    pending_exit: bool,
    pending_new: bool,
    exit_approved: bool,
    status: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            loaded: None,
            textures: Vec::new(),
            loading: None,
            export: None,
            open_form: None,
            resize_form: None,
            block_form: None,
            tileset_form: None,
            palette_form: None,
            unloaded_palettes: polished_map_project::PaletteDocument::default(),
            pending_palette_load: None,
            #[cfg(windows)]
            native_color_request: None,
            #[cfg(windows)]
            native_color_picker: polished_map_platform::ColorPicker::default(),
            block_count_form: None,
            block_count_windows_buttons: cfg!(target_os = "windows"),
            block_clipboard: None,
            paste_shortcut_handled: false,
            sidebar_drag: None,
            reset_sidebar_scroll: false,
            resize_anchor: ResizeAnchor::default(),
            reset_canvas_scroll: false,
            error: None,
            selected: 0,
            stroke: Vec::new(),
            zoom: false,
            grid: true,
            dark: false,
            confirm_discard: false,
            pending_exit: false,
            pending_new: false,
            exit_approved: false,
            status: "Open a map to begin. Development build: master parity is incomplete.".into(),
        }
    }
}

impl Editor {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let editor = Self::default();
        cc.egui_ctx.set_theme(egui::Theme::Light);
        cc.egui_ctx.all_styles_mut(|s| {
            s.spacing.item_spacing = Vec2::new(4.0, 2.0);
            s.spacing.button_padding = Vec2::new(4.0, 2.0);
            s.spacing.scroll.bar_width = 16.0;
            s.spacing.scroll.floating = false;
            s.text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
            s.text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
        });
        editor
    }

    fn is_modified(&self) -> bool {
        self.loaded.as_ref().is_some_and(|l| {
            l.document.map().is_modified()
                || l.blockset
                    .as_ref()
                    .is_some_and(|b| b.blockset().is_modified())
                || l.tileset
                    .as_ref()
                    .is_some_and(|t| t.tileset().is_modified())
                || l.tileset
                    .as_ref()
                    .and_then(|t| t.roof())
                    .is_some_and(|r| r.roof().tileset().is_modified())
        })
    }

    fn choose_open(&mut self) {
        self.pending_new = false;
        if self.is_modified() {
            self.confirm_discard = true;
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Map blocks", &["blk", "map"])
            .pick_file()
        {
            self.prepare_open(path);
        }
    }

    fn choose_new(&mut self) {
        self.pending_new = true;
        if self.is_modified() {
            self.confirm_discard = true;
            return;
        }
        self.start_new_options();
    }

    fn start_new_options(&mut self) {
        let root = self
            .loaded
            .as_ref()
            .and_then(|l| l.document.metadata().root.clone())
            .or_else(|| rfd::FileDialog::new().pick_folder());
        self.pending_new = false;
        if let Some(root) = root {
            self.prepare_new_in(root);
        }
    }

    pub fn prepare_new_in(&mut self, root: PathBuf) {
        match list_tilesets(&root) {
            Ok(tilesets) => {
                self.open_form = Some(OpenForm {
                    new_map: true,
                    path: root.join("NewMap.blk"),
                    root: root.to_string_lossy().into_owned(),
                    tileset: tilesets.first().cloned().unwrap_or_default(),
                    width: 10,
                    height: 9,
                    ..Default::default()
                })
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub fn prepare_open(&mut self, path: PathBuf) {
        let metadata = match discover_map(&path, None) {
            Ok(metadata) => metadata,
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        self.open_form = Some(OpenForm {
            root: metadata
                .root
                .as_deref()
                .or_else(|| path.parent())
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned(),
            tileset: metadata
                .tileset
                .or(metadata.tileset_hint)
                .unwrap_or_default(),
            width: metadata.dimensions.map_or(0, |d| d.0),
            height: metadata.dimensions.map_or(0, |d| d.1),
            path,
            ..Default::default()
        });
    }

    fn start_load(&mut self, form: OpenForm, ctx: &egui::Context) {
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        let dark_theme = self.dark;
        std::thread::spawn(move || {
            let result = (|| {
                let root = PathBuf::from(&form.root);
                let document = if form.new_map {
                    MapDocument::new_map(&root, form.width, form.height)
                } else {
                    MapDocument::open_in_project(
                        &form.path,
                        &OpenOptions {
                            width: (form.width != 0).then_some(form.width),
                            height: (form.height != 0).then_some(form.height),
                        },
                        Some(&root),
                    )
                }
                .map_err(|e| e.to_string())?;
                let tileset = if form.tileset.is_empty() {
                    document
                        .metadata()
                        .tileset
                        .as_deref()
                        .ok_or("Choose a tileset")?
                } else {
                    &form.tileset
                };
                let source = MasterTilesetDocument::open_for_map(
                    &document,
                    tileset,
                    &MasterGraphicsOptions {
                        allow_256: form.allow_256,
                        dark_theme,
                        roof: (!form.roof.is_empty()).then_some(form.roof),
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())?;
                let blockset =
                    BlocksetDocument::open_in_project(&root, tileset).map_err(|e| e.to_string())?;
                let graphics = source
                    .graphics(blockset.blockset())
                    .map_err(|e| e.to_string())?;
                Ok(Loaded {
                    document,
                    graphics,
                    blockset: Some(blockset),
                    tileset: Some(source),
                    allow_256: form.allow_256,
                })
            })();
            let _ = sender.send(result);
            ctx.request_repaint();
        });
        self.loading = Some(receiver);
        self.status = "Loading map…".into();
    }

    fn poll_workers(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.loading.as_ref().and_then(|rx| match rx.try_recv() {
            Ok(r) => Some(r),
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("Loading worker stopped unexpectedly".into()))
            }
            Err(_) => None,
        }) {
            self.loading = None;
            match result {
                Ok(loaded) => {
                    self.textures.clear();
                    for id in 0..loaded.graphics.block_count() {
                        match loaded.graphics.render_block(id as u8) {
                            Ok(rgb) => self.textures.push(ctx.load_texture(
                                format!("block-{id}"),
                                egui::ColorImage::from_rgb([32, 32], &rgb.pixels),
                                egui::TextureOptions::NEAREST,
                            )),
                            Err(e) => {
                                self.error = Some(e.to_string());
                                return;
                            }
                        }
                    }
                    self.status = if loaded.document.has_backing_file() {
                        loaded.document.path().display().to_string()
                    } else {
                        "New map (unsaved)".into()
                    };
                    self.loaded = Some(loaded);
                    self.selected = 0;
                    self.block_clipboard = None;
                    self.sidebar_drag = None;
                    self.reset_sidebar_scroll = true;
                    self.stroke.clear();
                }
                Err(error) => {
                    self.status = "Open failed".into();
                    self.error = Some(error);
                }
            }
        }
        if let Some(result) = self.export.as_ref().and_then(|rx| match rx.try_recv() {
            Ok(r) => Some(r),
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("Export worker stopped unexpectedly".into()))
            }
            Err(_) => None,
        }) {
            self.export = None;
            match result {
                Ok(()) => self.status = "PNG exported".into(),
                Err(e) => self.error = Some(e),
            }
        }
    }

    fn apply(&mut self, command: Command) {
        if let Some(loaded) = &mut self.loaded {
            match loaded.document.map_mut().apply(&command) {
                Ok(true) if matches!(command, Command::Resize { .. }) => {
                    self.reset_canvas_scroll = true
                }
                Err(error) => self.error = Some(error.to_string()),
                _ => {}
            }
        }
    }

    fn save(&mut self) {
        if self.loaded.as_ref().is_some_and(|l| {
            !l.document.has_backing_file()
                || l.document
                    .path()
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("map"))
        }) {
            self.save_as();
            if self
                .loaded
                .as_ref()
                .is_some_and(|l| !l.document.map().is_modified())
            {
                self.save_blockset();
                if self.error.is_none() {
                    self.save_tileset();
                    if self.error.is_none() {
                        self.save_roof();
                    }
                }
            }
            return;
        }
        if let Some(loaded) = &mut self.loaded {
            match loaded.document.save() {
                Ok(()) => self.status = "Map saved".into(),
                Err(e) => {
                    self.error = Some(e.to_string());
                    return;
                }
            }
        }
        self.save_blockset();
        if self.error.is_none() {
            self.save_tileset();
            if self.error.is_none() {
                self.save_roof();
            }
        }
    }

    fn save_blockset(&mut self) {
        let Some(doc) = self.loaded.as_mut().and_then(|l| l.blockset.as_mut()) else {
            return;
        };
        if !doc.blockset().is_modified() {
            return;
        }
        let result = if doc.needs_save_as() {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("Binary blockset", &["bin", "bst"])
                .set_file_name(
                    doc.path()
                        .with_extension("bin")
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                )
                .save_file()
            else {
                return;
            };
            if let Some(collision) = doc.collision_path() {
                let Some(collision_path) = rfd::FileDialog::new()
                    .set_title("Save Collision Copy")
                    .set_file_name(collision.file_name().unwrap_or_default().to_string_lossy())
                    .save_file()
                else {
                    return;
                };
                prepare_blockset_save_as(&path).and_then(|target| {
                    prepare_collision_save_as(&collision_path, doc.binary_collisions())
                        .and_then(|collision| doc.save_as_pair(&target, &collision))
                })
            } else {
                prepare_blockset_save_as(&path).and_then(|target| doc.save_as(&target))
            }
        } else {
            doc.save()
        };
        match result {
            Ok(()) => self.status = "Blockset saved".into(),
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn save_as(&mut self) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let source = loaded.document.path();
        let name = source.with_extension("blk");
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Map blocks", &["blk"])
            .set_directory(source.parent().unwrap_or(source))
            .set_file_name(name.file_name().unwrap_or_default().to_string_lossy())
            .save_file()
        else {
            return;
        };
        // The native Save dialog supplies the overwrite decision. Observe the target
        // immediately after it returns, then recheck before atomic replacement.
        match prepare_save_as(&path)
            .and_then(|target| self.loaded.as_mut().unwrap().document.save_as(&target))
        {
            Ok(()) => self.status = format!("Saved {}", path.display()),
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    fn prepare_resize(&mut self) {
        if let Some(loaded) = &self.loaded {
            self.resize_form = Some(ResizeForm {
                width: u16::from(loaded.document.map().width()),
                height: u16::from(loaded.document.map().height()),
                anchor: self.resize_anchor,
            });
        }
    }

    fn export_png(&mut self, ctx: &egui::Context) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name("map.png")
            .save_file()
        else {
            return;
        };
        if path.exists() {
            self.error = Some("Choose a new PNG output path.".into());
            return;
        }
        let map = loaded.document.map().clone();
        let graphics = loaded.graphics.clone();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = graphics
                .render_map(&map)
                .map_err(|e| e.to_string())
                .and_then(|image| write_png(&path, &image).map_err(|e| e.to_string()));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        self.export = Some(rx);
        self.status = "Exporting PNG…".into();
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        if let Some(message) = self.error.clone() {
            egui::Modal::new(egui::Id::unique("error")).show(ctx, |ui| {
                ui.heading("Unable to complete operation");
                ui.label(message);
                if ui.button("OK").clicked() {
                    self.error = None;
                }
            });
            return;
        }
        if self.confirm_discard {
            egui::Modal::new(egui::Id::unique("unsaved")).show(ctx, |ui| {
                ui.heading("Unsaved changes");
                ui.label("Save changes before continuing?");
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save();
                        if !self.is_modified() {
                            self.confirm_discard = false;
                            self.continue_after_discard(ctx);
                        }
                    }
                    if ui.button("Discard").clicked() {
                        self.confirm_discard = false;
                        self.continue_after_discard(ctx);
                    }
                    if ui.button("Cancel").clicked() {
                        self.confirm_discard = false;
                        self.pending_exit = false;
                        self.pending_new = false;
                    }
                });
            });
            return;
        }
        if self.block_form.is_some() {
            self.block_dialog(ctx);
            return;
        }
        if self.tileset_form.is_some() {
            self.tileset_dialog(ctx);
            return;
        }
        if self.palette_form.is_some() {
            self.palette_dialog(ctx);
            return;
        }
        if self.pending_palette_load.is_some() {
            self.palette_load_dialog(ctx);
            return;
        }
        if self.block_count_form.is_some() {
            self.block_count_dialog(ctx);
            return;
        }
        if let Some(mut form) = self.resize_form.take() {
            let mut accept = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::unique("resize-map")).show(ctx, |ui| {
                ui.heading("Resize Map");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for (label, value) in
                            [("Width", &mut form.width), ("Height", &mut form.height)]
                        {
                            ui.horizontal(|ui| {
                                let label = ui.label(label);
                                ui.add(egui::DragValue::new(value).range(1..=255))
                                    .labelled_by(label.id);
                            });
                        }
                    });
                    ui.add_space(20.0);
                    // Original Resize_Dialog: nine 24×24 buttons separated by 2 px.
                    egui::Grid::new("resize-anchor")
                        .spacing(Vec2::splat(2.0))
                        .show(ui, |ui| {
                            let labels = [
                                "Top left",
                                "Top center",
                                "Top right",
                                "Middle left",
                                "Center",
                                "Middle right",
                                "Bottom left",
                                "Bottom center",
                                "Bottom right",
                            ];
                            for (i, anchor) in ResizeAnchor::ALL.into_iter().enumerate() {
                                let selected = anchor == form.anchor;
                                let response = ui.add_sized(
                                    [24.0, 24.0],
                                    egui::Button::new("").selected(selected),
                                );
                                response.widget_info(|| {
                                    egui::WidgetInfo::labeled(egui::Role::Button, true, labels[i])
                                });
                                let active = ResizeAnchor::ALL
                                    .iter()
                                    .position(|&a| a == form.anchor)
                                    .unwrap_or(4);
                                let direction = Vec2::new(
                                    (i % 3) as f32 - (active % 3) as f32,
                                    (i / 3) as f32 - (active / 3) as f32,
                                );
                                let color = ui.visuals().text_color();
                                if selected {
                                    ui.painter().rect_filled(
                                        Rect::from_center_size(
                                            response.rect.center(),
                                            Vec2::splat(6.0),
                                        ),
                                        0.0,
                                        color,
                                    );
                                } else if direction.x.abs() <= 1.0 && direction.y.abs() <= 1.0 {
                                    let direction = direction.normalized() * 8.0;
                                    ui.painter().arrow(
                                        response.rect.center() - direction * 0.5,
                                        direction,
                                        egui::Stroke::new(1.0, color),
                                    );
                                }
                                if response.on_hover_text(labels[i]).clicked() {
                                    form.anchor = anchor;
                                }
                                if i % 3 == 2 {
                                    ui.end_row();
                                }
                            }
                        });
                });
                ui.label("New cells use block 0. Resizing clears undo history.");
                ui.label("Map constants and event scripts are not updated yet.");
                ui.horizontal(|ui| {
                    accept = ui.button("Resize").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if accept {
                self.resize_anchor = form.anchor;
                self.stroke.clear();
                self.apply(Command::Resize {
                    width: form.width,
                    height: form.height,
                    anchor: form.anchor,
                });
            } else if !cancel {
                self.resize_form = Some(form);
            }
        }
        if let Some(mut form) = self.open_form.take() {
            let mut accept = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::unique("open-options")).show(ctx, |ui| {
                ui.heading(if form.new_map { "New Map" } else { "Open Map" });
                if !form.new_map {
                    ui.label(form.path.display().to_string());
                }
                let label = ui.label("Project folder");
                ui.text_edit_singleline(&mut form.root)
                    .labelled_by(label.id);
                let label = ui.label("Tileset name");
                ui.text_edit_singleline(&mut form.tileset)
                    .labelled_by(label.id);
                let label = ui.label("Roof name (optional)");
                ui.text_edit_singleline(&mut form.roof)
                    .labelled_by(label.id);
                ui.horizontal(|ui| {
                    let label = ui.label("Width");
                    ui.add(egui::DragValue::new(&mut form.width).range(0..=255))
                        .labelled_by(label.id);
                    let label = ui.label("Height");
                    ui.add(egui::DragValue::new(&mut form.height).range(0..=255))
                        .labelled_by(label.id);
                });
                if !form.new_map {
                    ui.label("0 uses dimensions from the filename or project metadata");
                }
                ui.checkbox(&mut form.allow_256, "Allow all 256 tiles");
                ui.horizontal(|ui| {
                    accept = ui
                        .button(if form.new_map {
                            "Create map"
                        } else {
                            "Open map"
                        })
                        .clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if accept {
                self.start_load(form, ctx);
            } else if !cancel {
                self.open_form = Some(form);
            }
        }
    }

    fn continue_after_discard(&mut self, ctx: &egui::Context) {
        if self.pending_exit {
            self.exit_approved = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if self.pending_new {
            self.start_new_options();
        } else if let Some(path) = rfd::FileDialog::new()
            .add_filter("Map blocks", &["blk", "map"])
            .pick_file()
        {
            self.prepare_open(path);
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        if ui.button("Open…").clicked() {
            self.choose_open();
        }
        if ui
            .add_enabled(self.loaded.is_some(), egui::Button::new("Save"))
            .clicked()
        {
            self.save();
        }
        ui.separator();
        if let Some(loaded) = &mut self.loaded {
            if ui
                .add_enabled(loaded.document.map().can_undo(), egui::Button::new("Undo"))
                .clicked()
            {
                loaded.document.map_mut().undo();
            }
            if ui
                .add_enabled(loaded.document.map().can_redo(), egui::Button::new("Redo"))
                .clicked()
            {
                loaded.document.map_mut().redo();
            }
        }
        ui.separator();
        ui.toggle_value(&mut self.grid, "Grid");
        ui.toggle_value(&mut self.zoom, "Zoom");
        if ui
            .add_enabled(
                self.loaded.is_some() && self.export.is_none(),
                egui::Button::new("Print…"),
            )
            .clicked()
        {
            self.export_png(ui.ctx());
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui, block_size: f32) {
        let mut scroll = egui::ScrollArea::vertical()
            .id_salt("blocks")
            .auto_shrink([false, false])
            .scroll_source(egui::scroll_area::ScrollSource {
                drag: egui::scroll_area::DragScroll::Never,
                ..Default::default()
            })
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);
        if self.reset_sidebar_scroll {
            scroll = scroll.scroll_offset(Vec2::ZERO);
            self.reset_sidebar_scroll = false;
        }
        scroll.show(ui, |ui| {
            let rows = self.textures.len().div_ceil(4);
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(block_size * 4.0, block_size * rows as f32),
                Sense::click_and_drag(),
            );
            for (id, texture) in self.textures.iter().enumerate() {
                let cell = Rect::from_min_size(
                    rect.min
                        + Vec2::new((id % 4) as f32 * block_size, (id / 4) as f32 * block_size),
                    Vec2::splat(block_size),
                );
                if ui.is_rect_visible(cell) {
                    ui.painter().image(
                        texture.id(),
                        cell,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    if id == usize::from(self.selected) {
                        ui.painter().rect_stroke(
                            cell,
                            0.0,
                            (2.0, Color32::YELLOW),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }
            let hit = |pos: Pos2| {
                let local = pos - rect.min;
                let col = (local.x / block_size).floor() as i32;
                let row = (local.y / block_size).floor() as i32;
                let id = row * 4 + col;
                (rect.contains(pos)
                    && ui.clip_rect().contains(pos)
                    && (0..4).contains(&col)
                    && row >= 0
                    && (id as usize) < self.textures.len())
                .then_some(id as u8)
            };
            if response.drag_started_by(egui::PointerButton::Primary) {
                self.sidebar_drag = ui.input(|i| i.pointer.press_origin()).and_then(hit);
            }
            let drop = if response.drag_stopped_by(egui::PointerButton::Primary) {
                let from = self.sidebar_drag.take();
                from.zip(response.interact_pointer_pos().and_then(hit))
            } else {
                None
            };
            if (response.clicked() || response.secondary_clicked())
                && let Some(pos) = response.interact_pointer_pos()
            {
                let local = pos - rect.min;
                let col = (local.x / block_size) as usize;
                let id = (local.y / block_size) as usize * 4 + col;
                if col < 4 && id < self.textures.len() {
                    self.selected = id as u8;
                    if response.secondary_clicked() {
                        self.prepare_block(ui.ctx());
                    }
                }
            }
            if let Some((from, to)) = drop
                && from != to
            {
                let command = if ui.input(|i| i.modifiers.ctrl) {
                    polished_map_core::BlockCommand::Copy { from, to }
                } else {
                    polished_map_core::BlockCommand::Swap { a: from, b: to }
                };
                if self.apply_block_commands(ui.ctx(), &[command]) {
                    self.selected = to;
                }
            }
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui, layout: &geometry::Layout) {
        let Some(loaded) = &self.loaded else {
            ui.centered_and_justified(|ui| {
                ui.label("Open a map to edit blocks.");
            });
            return;
        };
        let map = loaded.document.map();
        let size = layout.block_size;
        let mut command = None;
        let mut scroll = egui::ScrollArea::both()
            .id_salt("map")
            .auto_shrink([false, false])
            .scroll_source(egui::scroll_area::ScrollSource {
                drag: egui::scroll_area::DragScroll::Never,
                ..Default::default()
            });
        if self.reset_canvas_scroll {
            scroll = scroll.scroll_offset(Vec2::ZERO);
            self.reset_canvas_scroll = false;
        }
        scroll.show(ui, |ui| {
            let extent = Vec2::new(
                (f32::from(map.width()) + 6.0) * size,
                (f32::from(map.height()) + 6.0) * size,
            );
            let (rect, response) = ui.allocate_exact_size(extent, Sense::click_and_drag());
            let clip = ui.clip_rect();
            // Iterate only rows/columns intersecting the viewport, including the map margin.
            let origin = rect.min + Vec2::splat(geometry::MAP_MARGIN * size);
            let min_x = ((clip.left() - origin.x) / size).floor().max(0.0) as usize;
            let min_y = ((clip.top() - origin.y) / size).floor().max(0.0) as usize;
            let max_x = (((clip.right() - origin.x) / size).ceil().max(0.0) as usize)
                .min(usize::from(map.width()));
            let max_y = (((clip.bottom() - origin.y) / size).ceil().max(0.0) as usize)
                .min(usize::from(map.height()));
            for y in min_y..max_y {
                for x in min_x..max_x {
                    let point = Point {
                        x: x as u16,
                        y: y as u16,
                    };
                    let id = if self.stroke.contains(&point) {
                        self.selected
                    } else {
                        map.blocks()[y * usize::from(map.width()) + x]
                    };
                    let cell = Rect::from_min_size(
                        origin + Vec2::new(x as f32 * size, y as f32 * size),
                        Vec2::splat(size),
                    );
                    if let Some(texture) = self.textures.get(usize::from(id)) {
                        ui.painter().image(
                            texture.id(),
                            cell,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    } else {
                        ui.painter()
                            .rect_filled(cell, 0.0, Color32::from_rgb(0xab, 0xcd, 0xef));
                    }
                    if self.grid {
                        ui.painter().rect_stroke(
                            cell,
                            0.0,
                            (1.0, Color32::from_black_alpha(96)),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }
            if let Some(pos) = response
                .interact_pointer_pos()
                .or_else(|| response.hover_pos())
            {
                let relative = pos - rect.min;
                if let Some((x, y)) = layout.hit(relative.x, relative.y, map.width(), map.height())
                {
                    let point = Point { x, y };
                    let old =
                        map.blocks()[usize::from(y) * usize::from(map.width()) + usize::from(x)];
                    self.status = format!("Block: ${old:02X}    X: {x}    Y: {y}");
                    if response.secondary_clicked() && usize::from(old) < self.textures.len() {
                        self.selected = old;
                    }
                    let (pressed, down, modifiers) = ui.input(|i| {
                        (
                            i.pointer.primary_pressed(),
                            i.pointer.primary_down(),
                            i.modifiers,
                        )
                    });
                    if pressed && response.hovered() {
                        if modifiers.shift {
                            command = Some(Command::Fill {
                                at: point,
                                block: self.selected,
                            });
                        } else if modifiers.ctrl {
                            command = Some(Command::Replace {
                                from: old,
                                to: self.selected,
                            });
                        } else if modifiers.alt {
                            command = Some(Command::Swap {
                                a: old,
                                b: self.selected,
                            });
                        }
                    }
                    if down
                        && (response.hovered() || response.dragged())
                        && !modifiers.ctrl
                        && !modifiers.shift
                        && !modifiers.alt
                    {
                        if let Some(&last) = self.stroke.last() {
                            append_line(&mut self.stroke, last, point);
                        } else {
                            self.stroke.push(point);
                        }
                    }
                    if response.clicked()
                        && !down
                        && self.stroke.is_empty()
                        && !modifiers.ctrl
                        && !modifiers.shift
                        && !modifiers.alt
                    {
                        command = Some(Command::Paint {
                            points: vec![point],
                            block: self.selected,
                        });
                    }
                }
            }
        });
        if let Some(command) = command {
            self.apply(command);
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.poll_workers(&ctx);
        if ctx.input(|i| i.viewport().close_requested())
            && self.is_modified()
            && !self.exit_approved
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending_exit = true;
            self.confirm_discard = true;
        }
        let blocked = self.open_form.is_some()
            || self.resize_form.is_some()
            || self.block_form.is_some()
            || self.tileset_form.is_some()
            || self.palette_form.is_some()
            || self.pending_palette_load.is_some()
            || self.block_count_form.is_some()
            || self.confirm_discard
            || self.error.is_some()
            || self.loading.is_some();
        if !blocked && !ctx.egui_wants_keyboard_input() {
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::L,
                )
            }) {
                self.prepare_palettes();
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::L)) {
                self.choose_palettes(&ctx);
            }
            self.block_shortcuts(&ctx);
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::T)) {
                self.prepare_tileset();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F)) {
                self.prepare_roof();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::N)) {
                self.choose_new();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
                self.choose_open();
            }
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::S,
                )
            }) {
                self.save_as();
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
                self.save();
            }
            if let Some(l) = &mut self.loaded {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) {
                    l.document.map_mut().undo();
                }
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y)) {
                    l.document.map_mut().redo();
                }
            }
        }
        let bounds = ui.max_rect();
        let layout = geometry::Layout::new(bounds.width(), bounds.height(), self.zoom, false, 16.0);
        ui.painter()
            .rect_filled(bounds, 0.0, ui.visuals().panel_fill);
        ui.add_enabled_ui(!blocked, |ui| {
            let menu =
                Rect::from_min_size(bounds.min, Vec2::new(bounds.width(), geometry::MENU_HEIGHT));
            ui.scope_builder(egui::UiBuilder::new().max_rect(menu), |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New…").clicked() {
                            ui.close();
                            self.choose_new();
                        }
                        if ui.button("Open…").clicked() {
                            ui.close();
                            self.choose_open();
                        }
                        if ui.button("Save").clicked() {
                            ui.close();
                            self.save();
                        }
                        if ui
                            .add_enabled(
                                self.loaded.as_ref().is_some_and(|l| l.blockset.is_some()),
                                egui::Button::new("Save Blockset"),
                            )
                            .clicked()
                        {
                            ui.close();
                            self.save_blockset();
                        }
                        if ui
                            .add_enabled(
                                self.loaded.as_ref().is_some_and(|l| l.tileset.is_some()),
                                egui::Button::new("Save Tileset"),
                            )
                            .clicked()
                        {
                            ui.close();
                            self.save_tileset();
                        }
                        if ui
                            .add_enabled(
                                self.loaded
                                    .as_ref()
                                    .and_then(|l| l.tileset.as_ref())
                                    .and_then(|t| t.roof())
                                    .is_some(),
                                egui::Button::new("Save Roof"),
                            )
                            .clicked()
                        {
                            ui.close();
                            self.save_roof();
                        }
                        if ui
                            .add_enabled(self.loaded.is_some(), egui::Button::new("Save As…"))
                            .clicked()
                        {
                            ui.close();
                            self.save_as();
                        }
                        if ui.button("Print…").clicked() {
                            ui.close();
                            self.export_png(&ctx);
                        }
                        if ui.button("Load Palettes…").clicked() {
                            ui.close();
                            self.choose_palettes(&ctx);
                        }
                        if ui.button("Export Palettes…").clicked() {
                            ui.close();
                            self.export_palettes();
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.button("Edit Current Palettes…").clicked() {
                            ui.close();
                            self.prepare_palettes();
                        }
                        self.block_menu(ui);
                        if ui
                            .add_enabled(
                                self.loaded.as_ref().is_some_and(|l| l.blockset.is_some()),
                                egui::Button::new("Edit Block…"),
                            )
                            .clicked()
                        {
                            ui.close();
                            self.prepare_block(&ctx);
                        }
                        if ui
                            .add_enabled(
                                self.loaded.as_ref().is_some_and(|l| l.tileset.is_some()),
                                egui::Button::new("Edit Tileset…"),
                            )
                            .clicked()
                        {
                            ui.close();
                            self.prepare_tileset();
                        }
                        if ui
                            .add_enabled(
                                self.loaded
                                    .as_ref()
                                    .and_then(|l| l.tileset.as_ref())
                                    .and_then(|t| t.roof())
                                    .is_some(),
                                egui::Button::new("Edit Roof…"),
                            )
                            .clicked()
                        {
                            ui.close();
                            self.prepare_roof();
                        }
                        if ui
                            .add_enabled(self.loaded.is_some(), egui::Button::new("Resize Map…"))
                            .clicked()
                        {
                            ui.close();
                            self.prepare_resize();
                        }
                    });
                    ui.menu_button("View", |ui| {
                        ui.menu_button("Palettes", |ui| self.palette_set_menu(ui));
                        ui.checkbox(&mut self.grid, "Grid");
                        ui.checkbox(&mut self.zoom, "Zoom");
                        if ui.checkbox(&mut self.dark, "Dark appearance").changed() {
                            ctx.set_theme(if self.dark {
                                egui::Theme::Dark
                            } else {
                                egui::Theme::Light
                            });
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        ui.label("Polished Map NG — development");
                        ui.label("Derived from Polished Map by Rangi42.");
                        ui.label("LGPL v3. See repository notices.");
                    });
                });
            });
            let toolbar = Rect::from_min_size(
                bounds.min + Vec2::new(0.0, geometry::MENU_HEIGHT),
                Vec2::new(bounds.width(), geometry::TOOLBAR_HEIGHT),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(toolbar), |ui| {
                ui.horizontal(|ui| self.toolbar(ui));
            });
            let rect = |r: geometry::Rect| {
                Rect::from_min_size(
                    bounds.min + Vec2::new(r.x, r.y),
                    Vec2::new(r.width, r.height),
                )
            };
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(rect(layout.sidebar)),
                |ui| {
                    self.sidebar(ui, layout.block_size);
                },
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect(layout.canvas)), |ui| {
                self.canvas(ui, &layout);
            });
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect(layout.status)), |ui| {
                ui.horizontal(|ui| {
                    ui.label(if self.is_modified() {
                        "Modified"
                    } else {
                        "Saved"
                    });
                    ui.separator();
                    ui.label(&self.status);
                });
            });
        });
        if ctx.input(|i| i.pointer.any_released()) && !self.stroke.is_empty() {
            let points = std::mem::take(&mut self.stroke);
            self.apply(Command::Paint {
                points,
                block: self.selected,
            });
        }
        self.dialogs(&ctx);
    }
}

fn append_line(points: &mut Vec<Point>, from: Point, to: Point) {
    let dx = i32::from(to.x) - i32::from(from.x);
    let dy = i32::from(to.y) - i32::from(from.y);
    let steps = dx.abs().max(dy.abs());
    for step in 1..=steps {
        let p = Point {
            x: (i32::from(from.x) + dx * step / steps) as u16,
            y: (i32::from(from.y) + dy * step / steps) as u16,
        };
        if !points.contains(&p) {
            points.push(p);
        }
    }
}

impl eframe::App for Editor {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
        #[cfg(windows)]
        self.choose_palette_color(_frame);
    }
}
