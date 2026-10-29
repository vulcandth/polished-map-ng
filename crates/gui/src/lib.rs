//! FLTK-derived logical geometry and egui adapter. LGPL-3.0-only.
mod about;
mod block_editor;
mod block_hotkeys;
mod block_workflows;
#[cfg(test)]
mod drag_tests;
#[cfg(all(test, feature = "visual-tests"))]
mod event_label_tests;
#[cfg(test)]
mod event_workflow_tests;
mod event_workflows;
mod file_workflows;
mod fonts;
mod fullscreen;
pub mod geometry;
mod help;
mod icons;
mod map_buttons;
mod map_labels;
#[cfg(test)]
mod map_open_tests;
mod menu_popup;
mod menus;
mod overworld;
mod palette_editor;
#[cfg(test)]
mod palette_editor_tests;
mod palette_session;
#[cfg(test)]
mod palette_session_tests;
mod preferences;
mod printing;
mod priority;
mod recent_maps;
mod roof_colors;
#[cfg(test)]
mod roof_workflow_tests;
mod roof_workflows;
mod rulers;
mod status_bar;
mod tileset_editor;
#[cfg(test)]
mod tileset_workflow_tests;
mod tileset_workflows;
#[cfg(test)]
mod toolbar_tests;
mod transparency;
mod window_title;
mod workspace_pan;
#[cfg(test)]
mod zoom_tests;
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
    prepare_save_as,
};

/// Development application identity, also used before a map is open.
pub const WINDOW_TITLE: &str = "Polished Map NG — development";

struct Loaded {
    // Retain chosen spelling for history/title independently of canonical I/O paths.
    recent_path: Option<PathBuf>,
    trailing_blocks: usize,
    events: Option<polished_map_project::EventDocument>,
    event_warning: Option<String>,
    palette_plan: Option<polished_map_project::MapPalettePlan>,
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
    last_window_title: Option<String>,
    native_fonts: Option<std::sync::Arc<polished_map_project::NativeFonts>>,
    about: Option<about::AboutDialog>,
    help_window: help::HelpWindow,
    event_ui: event_workflows::EventUi,
    loaded: Option<Loaded>,
    pending_open: Option<Loaded>,
    pending_map_warning: Option<String>,
    pending_palette_warnings: Vec<String>,
    open_palette_warning: Option<String>,
    textures: Vec<egui::TextureHandle>,
    loading: Option<mpsc::Receiver<LoadResult>>,
    print_worker: Option<mpsc::Receiver<Result<printing::PrintOutput, String>>>,
    print_form: Option<printing::PrintForm>,
    print_options: polished_map_core::PrintOptions,
    print_request: Option<printing::PrintJob>,
    print_success: Option<String>,
    open_form: Option<OpenForm>,
    resize_form: Option<ResizeForm>,
    overworld_form: Option<overworld::OverworldForm>,
    overworld_map_size: u16,
    overworld_warning: Option<String>,
    block_form: Option<BlockForm>,
    tileset_form: Option<tileset_editor::TilesetForm>,
    palette_form: Option<palette_editor::PaletteForm>,
    roof_choice_form: Option<roof_workflows::RoofChoiceForm>,
    confirm_roof_change: bool,
    tileset_choice_form: Option<tileset_workflows::TilesetChoiceForm>,
    confirm_tileset_change: bool,
    pending_tileset_reload: bool,
    unloaded_palettes: polished_map_project::PaletteDocument,
    monochrome: bool,
    allow_256: bool,
    pending_palette_load: Option<PathBuf>,
    drag_and_drop: bool,
    pending_roof_colors: Option<PathBuf>,
    roof_color_success: Option<String>,
    roof_palette_layout: polished_map_project::RoofPaletteLayout,
    auto_special_palettes: bool,
    auto_roof_colors: bool,
    allow_priority: bool,
    show_priority: bool,
    preferences_writable: bool,
    recent_maps: recent_maps::RecentMaps,
    pending_recent: Option<PathBuf>,
    #[cfg(windows)]
    native_color_request: Option<polished_map_core::ColorSlot>,
    #[cfg(windows)]
    native_color_picker: polished_map_platform::ColorPicker,
    block_count_form: Option<u16>,
    block_count_windows_buttons: bool,
    block_clipboard: Option<block_workflows::BlockClipboard>,
    paste_shortcut_handled: bool,
    sidebar_drag: Option<u8>,
    sidebar_press: Option<(egui::PointerButton, u8)>,
    block_hotkeys: block_hotkeys::BlockHotkeys,
    sidebar_reveal: Option<u8>,
    reset_sidebar_scroll: bool,
    resize_anchor: ResizeAnchor,
    reset_canvas_scroll: bool,
    canvas_zoom: bool,
    canvas_scroll: Vec2,
    sidebar_scroll: f32,
    canvas_pan: workspace_pan::WorkspacePan,
    sidebar_pan: workspace_pan::WorkspacePan,
    roof_choices_available: bool,
    error: Option<String>,
    selected: u8,
    stroke: Vec<Point>,
    zoom: bool,
    rulers: bool,
    ids: bool,
    hex: bool,
    ruler_scroll: Vec2,
    ruler_cursor: Option<geometry::EventCursor>,
    grid: bool,
    dark: bool,
    transparency: transparency::Transparency,
    confirm_discard: bool,
    pending_exit: bool,
    pending_close: bool,
    pending_new: bool,
    discard_windows_buttons: bool,
    exit_approved: bool,
    status: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            last_window_title: None,
            native_fonts: None,
            about: None,
            help_window: Default::default(),
            event_ui: event_workflows::EventUi::default(),
            loaded: None,
            pending_open: None,
            pending_map_warning: None,
            pending_palette_warnings: Vec::new(),
            open_palette_warning: None,
            textures: Vec::new(),
            loading: None,
            print_worker: None,
            print_form: None,
            print_options: Default::default(),
            print_request: None,
            print_success: None,
            open_form: None,
            resize_form: None,
            overworld_form: None,
            overworld_map_size: 1300,
            overworld_warning: None,
            block_form: None,
            tileset_form: None,
            palette_form: None,
            roof_choice_form: None,
            confirm_roof_change: false,
            tileset_choice_form: None,
            confirm_tileset_change: false,
            pending_tileset_reload: false,
            unloaded_palettes: polished_map_project::PaletteDocument::default(),
            monochrome: false,
            allow_256: false,
            drag_and_drop: true,
            pending_palette_load: None,
            pending_roof_colors: None,
            roof_color_success: None,
            roof_palette_layout: polished_map_project::RoofPaletteLayout::default(),
            auto_special_palettes: true,
            auto_roof_colors: true,
            allow_priority: false,
            show_priority: true,
            preferences_writable: true,
            recent_maps: recent_maps::RecentMaps::default(),
            pending_recent: None,
            #[cfg(windows)]
            native_color_request: None,
            #[cfg(windows)]
            native_color_picker: polished_map_platform::ColorPicker::default(),
            block_count_form: None,
            block_count_windows_buttons: cfg!(target_os = "windows"),
            block_clipboard: None,
            paste_shortcut_handled: false,
            sidebar_drag: None,
            sidebar_press: None,
            block_hotkeys: Default::default(),
            sidebar_reveal: None,
            reset_sidebar_scroll: false,
            resize_anchor: ResizeAnchor::default(),
            reset_canvas_scroll: false,
            canvas_zoom: false,
            canvas_scroll: Vec2::ZERO,
            canvas_pan: Default::default(),
            sidebar_pan: Default::default(),
            sidebar_scroll: 0.0,
            roof_choices_available: false,
            error: None,
            selected: 0,
            stroke: Vec::new(),
            zoom: false,
            rulers: false,
            ids: false,
            hex: false,
            ruler_scroll: Vec2::ZERO,
            ruler_cursor: None,
            grid: true,
            dark: false,
            transparency: Default::default(),
            confirm_discard: false,
            pending_exit: false,
            pending_close: false,
            pending_new: false,
            discard_windows_buttons: cfg!(target_os = "windows"),
            exit_approved: false,
            status: "Open a map to begin. Development build: master parity is incomplete.".into(),
        }
    }
}

impl Editor {
    /// Native application startup: choose the reference's installed font roles.
    pub fn new_native(cc: &eframe::CreationContext<'_>) -> Self {
        let mut editor = Self::new(cc);
        editor.native_fonts = Some(fonts::install_native(&cc.egui_ctx));
        editor
    }

    /// Bundled, deterministic fonts for embedding and reproducible headless tests.
    /// The desktop executable uses `new_native` instead.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::install_bundled(&cc.egui_ctx);
        let mut editor = Self::default();
        editor.restore_preferences(cc.storage);
        cc.egui_ctx.set_theme(if editor.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        cc.egui_ctx.all_styles_mut(|s| {
            s.spacing.item_spacing = Vec2::new(4.0, 2.0);
            s.spacing.button_padding = Vec2::new(4.0, 2.0);
            s.spacing.scroll.bar_width = 16.0;
            // update_layout reserves exactly Fl::scrollbar_size(), without a gap.
            s.spacing.scroll.bar_inner_margin = 0.0;
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
                || l.events
                    .as_ref()
                    .is_some_and(|e| e.script().events().is_modified())
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
        self.pending_recent = None;
        self.pending_close = false;
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
        self.pending_recent = None;
        self.pending_close = false;
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
                    allow_256: self.allow_256,
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
            allow_256: self.allow_256,
            ..Default::default()
        });
    }

    fn start_load(&mut self, form: OpenForm, ctx: &egui::Context) {
        self.pending_map_warning = None;
        self.pending_palette_warnings.clear();
        self.allow_256 = form.allow_256;
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        let dark_theme = self.dark;
        let palette_session = self.palette_document().clone();
        let auto_events = self.event_ui.auto_load;
        let roof_layout = self.roof_palette_layout;
        let special_palettes = self.auto_special_palettes;
        let roof_colors = self.auto_roof_colors;
        let allow_priority = self.allow_priority;
        let monochrome = self.monochrome;
        std::thread::spawn(move || {
            let result = (|| {
                let root = PathBuf::from(&form.root);
                let report = if form.new_map {
                    MapDocument::new_map(&root, form.width, form.height).map(|document| {
                        polished_map_project::MapOpenReport {
                            document,
                            trailing_blocks: 0,
                        }
                    })
                } else {
                    MapDocument::open_in_project_with_report(
                        &form.path,
                        &OpenOptions {
                            width: (form.width != 0).then_some(form.width),
                            height: (form.height != 0).then_some(form.height),
                        },
                        Some(&root),
                    )
                }
                .map_err(|e| e.to_string())?;
                let document = report.document;
                let tileset = if form.tileset.is_empty() {
                    document
                        .metadata()
                        .tileset
                        .as_deref()
                        .ok_or("Choose a tileset")?
                } else {
                    &form.tileset
                };
                let (source, palette_plan) = MasterTilesetDocument::begin_for_map(
                    &document,
                    tileset,
                    &MasterGraphicsOptions {
                        allow_256: form.allow_256,
                        monochrome,
                        allow_priority,
                        dark_theme,
                        roof_layout,
                        special_palettes,
                        roof_colors,
                        roof: (!form.roof.is_empty()).then_some(form.roof),
                        ..Default::default()
                    },
                    &palette_session,
                )
                .map_err(|e| e.to_string())?;
                let blockset =
                    BlocksetDocument::open_in_project(&root, tileset).map_err(|e| e.to_string())?;
                let graphics = source
                    .graphics(blockset.blockset())
                    .map_err(|e| e.to_string())?;
                let (events, event_warning) =
                    event_workflows::auto_load_events(&document, auto_events);
                Ok(Loaded {
                    recent_path: (!form.new_map).then_some(form.path),
                    trailing_blocks: report.trailing_blocks,
                    events,
                    event_warning,
                    palette_plan: Some(palette_plan),
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
                    self.pending_open = Some(loaded);
                    self.advance_open_palettes(ctx);
                }
                Err(error) => {
                    self.status = "Open failed".into();
                    self.error = Some(error);
                }
            }
        }
        self.poll_print(ctx);
    }

    fn apply(&mut self, command: Command) {
        if let Some(loaded) = &mut self.loaded {
            if matches!(command, Command::Resize { .. }) {
                match event_workflows::resize_with_events(loaded, &command) {
                    Err(error) => self.error = Some(error),
                    Ok(true) => self.reset_canvas_scroll = true,
                    Ok(false) => {}
                }
                return;
            }
            if let Err(error) = loaded.document.map_mut().apply(&command) {
                self.error = Some(error.to_string());
            }
        }
    }

    fn save(&mut self) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let tiles = loaded
            .tileset
            .as_ref()
            .is_some_and(|t| t.tileset().is_modified());
        let roof = loaded
            .tileset
            .as_ref()
            .and_then(|t| t.roof())
            .is_some_and(|r| r.roof().tileset().is_modified());
        let blocks = loaded
            .blockset
            .as_ref()
            .is_some_and(|b| b.blockset().is_modified());
        let events = loaded
            .events
            .as_ref()
            .is_some_and(|e| e.script().events().is_modified());
        // Main_Window::save_cb: independent domains first; a clean map need not
        // be written when only another domain changed. Stop on failure/cancel.
        if tiles {
            self.save_tileset();
        }
        if self.error.is_some() {
            return;
        }
        if roof {
            self.save_roof();
        }
        if self.error.is_some() {
            return;
        }
        if blocks {
            self.save_blockset();
        }
        if self.error.is_some()
            || self
                .loaded
                .as_ref()
                .and_then(|l| l.blockset.as_ref())
                .is_some_and(|b| b.blockset().is_modified())
        {
            return;
        }
        if events {
            self.save_events();
        }
        if self.error.is_some() {
            return;
        }
        let Some(loaded) = &self.loaded else {
            return;
        };
        if (tiles || roof || blocks || events) && !loaded.document.map().is_modified() {
            return;
        }
        self.save_map();
    }

    fn save_map(&mut self) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        if !loaded.document.has_backing_file()
            || loaded
                .document
                .path()
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("map"))
        {
            self.save_as();
            return;
        }
        if let Some(loaded) = &mut self.loaded {
            let modified = loaded.document.map().is_modified();
            // master save_map(false): acknowledging an oversized input does
            // not dirty the map. A clean Save Map must not truncate its source.
            if !modified {
                self.status = "Map saved".into();
                return;
            }
            match loaded.document.save() {
                Ok(()) => {
                    self.status = "Map saved".into();
                    self.check_overworld_buffer();
                }
                Err(e) => self.error = Some(e.to_string()),
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
        self.save_as_path(path);
    }

    fn save_as_path(&mut self, path: PathBuf) {
        if self.loaded.is_none() {
            return;
        }
        match prepare_save_as(&path)
            .and_then(|target| self.loaded.as_mut().unwrap().document.save_as(&target))
        {
            Ok(()) => {
                let mut recent_path = path.clone();
                if recent_path.extension().is_none() {
                    recent_path.set_extension("blk");
                }
                self.loaded.as_mut().unwrap().recent_path = Some(recent_path.clone());
                self.recent_maps.remember(recent_path);
                self.status = format!("Saved {}", path.display());
                self.check_overworld_buffer();
            }
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
        if let Some(message) = self.open_palette_warning.clone() {
            egui::Modal::new(egui::Id::unique("open-palette-warning")).show(ctx, |ui| {
                ui.heading("Roof color warning");
                ui.label(message);
                if ui.button("OK").clicked() {
                    self.open_palette_warning = None;
                }
            });
            return;
        }
        if let Some(mut about) = self.about.take() {
            if !about.show(ctx) {
                self.about = Some(about);
            }
            return;
        }
        if self.print_form.is_some() || self.print_success.is_some() {
            self.print_dialog(ctx);
            return;
        }
        if self.overworld_warning.is_some() || self.overworld_form.is_some() {
            self.overworld_dialog(ctx);
            return;
        }
        if self.confirm_discard {
            self.discard_dialog(ctx);
            return;
        }
        if self.pending_roof_colors.is_some() || self.roof_color_success.is_some() {
            self.roof_color_dialog(ctx);
            return;
        }
        if self.event_ui.change.is_some() || self.event_ui.form.is_some() {
            self.event_dialog(ctx);
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
        if self.confirm_roof_change || self.roof_choice_form.is_some() {
            self.roof_choice_dialog(ctx);
            return;
        }
        if self.confirm_tileset_change || self.tileset_choice_form.is_some() {
            self.tileset_choice_dialog(ctx);
            return;
        }
        if self.pending_palette_load.is_some() {
            self.palette_load_dialog(ctx);
            return;
        }
        if self.pending_open.is_some() {
            if self.pending_map_warning.is_some() {
                self.map_length_dialog(ctx);
            } else {
                self.open_palette_dialog(ctx);
            }
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
                ui.label("Map constants are not updated. Loaded events move with the map.");
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
            self.pending_exit = false;
            self.exit_approved = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if self.pending_close {
            self.close_map();
        } else if self.pending_new {
            self.start_new_options();
        } else if let Some(path) = self.pending_recent.take() {
            self.prepare_open(path);
        } else if let Some(path) = self.event_ui.warp_path.take() {
            self.open_warp(path, ctx);
        } else if let Some(path) = rfd::FileDialog::new()
            .add_filter("Map blocks", &["blk", "map"])
            .pick_file()
        {
            self.prepare_open(path);
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        use icons::{Button, Icon};
        ui.spacing_mut().item_spacing.x = 0.0;
        if ui
            .add(Button::new(Icon::New, "New"))
            .on_hover_text("New… (Ctrl/Cmd+N)")
            .clicked()
        {
            self.choose_new();
        }
        if ui
            .add(Button::new(Icon::Open, "Open…"))
            .on_hover_text("Open… (Ctrl/Cmd+O)")
            .clicked()
        {
            self.choose_open();
        }
        if ui
            .add_enabled(
                self.loaded.is_some(),
                Button::new(Icon::LoadEvents, "Load events"),
            )
            .on_hover_text("Load Event Script… (Ctrl/Cmd+A)")
            .clicked()
        {
            self.choose_events();
        }
        if ui
            .add_enabled(
                self.loaded.as_ref().is_some_and(|l| l.events.is_some()),
                Button::new(Icon::Reload, "Reload events"),
            )
            .on_hover_text("Reload Event Script (Ctrl/Cmd+R)")
            .clicked()
        {
            self.request_event_change(event_workflows::EventChange::Reload);
        }
        if ui
            .add_enabled(self.loaded.is_some(), Button::new(Icon::Save, "Save"))
            .on_hover_text("Save (Ctrl/Cmd+S)")
            .clicked()
        {
            self.save();
        }
        icons::separator(ui);
        if ui
            .add_enabled(
                self.loaded.is_some()
                    && self.print_worker.is_none()
                    && self.print_request.is_none(),
                Button::new(Icon::Print, "Print…"),
            )
            .on_hover_text("Print (Ctrl/Cmd+P)")
            .clicked()
        {
            self.prepare_print();
        }
        icons::separator(ui);
        let undo = self
            .loaded
            .as_ref()
            .is_some_and(|l| l.document.map().can_undo());
        if ui
            .add_enabled(undo, Button::placeholder("U", "Undo"))
            .on_hover_text("Undo (Ctrl/Cmd+Z)")
            .clicked()
            && let Some(loaded) = &mut self.loaded
        {
            loaded.document.map_mut().undo();
        }
        let redo = self
            .loaded
            .as_ref()
            .is_some_and(|l| l.document.map().can_redo());
        if ui
            .add_enabled(redo, Button::placeholder("R", "Redo"))
            .on_hover_text("Redo (Ctrl/Cmd+Y)")
            .clicked()
            && let Some(loaded) = &mut self.loaded
        {
            loaded.document.map_mut().redo();
        }
        icons::separator(ui);
        for (icon, label, help, value) in [
            (Icon::Grid, "Grid", "Grid (Ctrl/Cmd+G)", &mut self.grid),
            (
                Icon::Rulers,
                "Rulers",
                "Rulers (Ctrl/Cmd+Shift+R)",
                &mut self.rulers,
            ),
            (Icon::Zoom, "Zoom", "Zoom (Ctrl/Cmd+=)", &mut self.zoom),
            (Icon::Ids, "IDs", "Block IDs (Ctrl/Cmd+I)", &mut self.ids),
            (
                Icon::Hex,
                "Hex",
                "Hexadecimal (Ctrl/Cmd+Shift+4)",
                &mut self.hex,
            ),
        ] {
            if ui
                .add(Button::new(icon, label).selected(*value))
                .on_hover_text(help)
                .clicked()
            {
                *value = !*value;
            }
        }
        if ui
            .add_enabled(
                self.allow_priority,
                Button::new(Icon::Priority, "Show Priority").selected(self.show_priority),
            )
            .on_hover_text("Show Priority (Ctrl/Cmd+Shift+P)")
            .clicked()
        {
            self.show_priority = !self.show_priority;
        }
        for (text, label, help, value) in [
            (
                "GB",
                "Game Boy screen",
                "Game Boy Screen (Ctrl/Cmd+Shift+M)",
                &mut self.event_ui.game_boy,
            ),
            (
                "SE",
                "Show events",
                "Show Events (Ctrl/Cmd+Shift+V)",
                &mut self.event_ui.show,
            ),
            (
                "WI",
                "Show warp IDs",
                "Show Warp IDs (Ctrl/Cmd+Shift+3)",
                &mut self.event_ui.warp_ids,
            ),
        ] {
            if ui
                .add(Button::placeholder(text, label).selected(*value))
                .on_hover_text(help)
                .clicked()
            {
                *value = !*value;
            }
        }
        icons::separator(ui);
        if ui
            .add(Button::placeholder("B", "Blocks mode").selected(!self.event_ui.mode))
            .on_hover_text("Blocks Mode (Ctrl/Cmd+Shift+B)")
            .clicked()
        {
            self.set_event_mode(false);
        }
        if ui
            .add(Button::placeholder("E", "Events mode").selected(self.event_ui.mode))
            .on_hover_text("Events Mode (Ctrl/Cmd+Shift+E)")
            .clicked()
        {
            self.set_event_mode(true);
        }
        icons::separator(ui);
        self.palette_set_toolbar(ui);
        if ui
            .add(Button::new(Icon::LoadPalettes, "Load palettes"))
            .on_hover_text("Load Palettes… (Ctrl/Cmd+L)")
            .clicked()
        {
            self.choose_palettes(ui.ctx());
        }
        if ui
            .add(Button::new(Icon::Palettes, "Edit current palettes"))
            .on_hover_text("Edit Current Palettes… (Ctrl/Cmd+Shift+L)")
            .clicked()
        {
            self.prepare_palettes();
        }
        icons::separator(ui);
        if ui
            .add_enabled(
                self.loaded.as_ref().is_some_and(|l| l.blockset.is_some()),
                Button::new(Icon::ResizeBlockset, "Resize blockset"),
            )
            .on_hover_text("Resize Blockset… (Ctrl/Cmd+B)")
            .clicked()
        {
            self.prepare_block_count();
        }
        if ui
            .add_enabled(
                self.loaded.is_some(),
                Button::new(Icon::Resize, "Resize map"),
            )
            .on_hover_text("Resize Map… (Ctrl/Cmd+E)")
            .clicked()
        {
            self.prepare_resize();
        }
        icons::separator(ui);
        let change = self
            .loaded
            .as_ref()
            .is_some_and(|l| l.tileset.is_some() && l.blockset.is_some());
        if ui
            .add_enabled(change, Button::placeholder("CT", "Change tileset"))
            .on_hover_text("Change Tileset… (Ctrl/Cmd+H)")
            .clicked()
        {
            self.request_tileset_change();
        }
        if ui
            .add_enabled(
                self.loaded.as_ref().is_some_and(|l| l.tileset.is_some()),
                Button::new(Icon::Tileset, "Tileset"),
            )
            .on_hover_text("Edit Tileset… (Ctrl/Cmd+T)")
            .clicked()
        {
            self.prepare_tileset();
        }
        icons::separator(ui);
        if ui
            .add_enabled(
                self.can_change_roof(),
                Button::placeholder("CR", "Change roof"),
            )
            .on_hover_text("Change Roof… (Ctrl/Cmd+Shift+H)")
            .clicked()
        {
            self.request_roof_change();
        }
        let roof = self
            .loaded
            .as_ref()
            .and_then(|l| l.tileset.as_ref())
            .and_then(|t| t.roof())
            .is_some();
        if ui
            .add_enabled(roof, Button::placeholder("RF", "Roof"))
            .on_hover_text("Edit Roof… (Ctrl/Cmd+F)")
            .clicked()
        {
            self.prepare_roof();
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui, block_size: f32) {
        if self.reset_sidebar_scroll || self.sidebar_reveal.is_some() {
            self.sidebar_pan.reset();
        }
        let pan = self.sidebar_pan.update(
            ui,
            Vec2::new(0.0, self.sidebar_scroll),
            Vec2::new(
                block_size * 4.0,
                block_size * self.textures.len().div_ceil(4) as f32,
            ),
            true,
        );
        if pan.handled {
            self.sidebar_press = None;
            self.sidebar_drag = None;
        }
        let priority = self
            .show_priority
            .then(|| priority::Textures::get(ui.ctx()));
        let mut scroll = egui::ScrollArea::vertical()
            .id_salt("blocks")
            .auto_shrink([false, false])
            .scroll_source(egui::scroll_area::ScrollSource {
                drag: egui::scroll_area::DragScroll::Never,
                ..Default::default()
            })
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);
        if let Some(offset) = pan.offset {
            scroll = scroll
                .scroll_offset(offset)
                .scroll_source(egui::scroll_area::ScrollSource {
                    mouse_wheel: true,
                    ..egui::scroll_area::ScrollSource::NONE
                });
        }
        if self.reset_sidebar_scroll {
            scroll = scroll.scroll_offset(Vec2::ZERO);
            self.reset_sidebar_scroll = false;
            self.sidebar_reveal = None;
        } else if let Some(id) = self.sidebar_reveal.take() {
            let offset = block_hotkeys::reveal_offset(
                id,
                block_size,
                ui.available_height(),
                self.sidebar_scroll,
            );
            scroll = scroll.scroll_offset(Vec2::new(0.0, offset));
            ui.spacing_mut().scroll.kinetic.rubber_band = false;
            ui.spacing_mut().scroll.kinetic.decay_time = 0.0;
        }
        let output = scroll.show(ui, |ui| {
            let rows = self.textures.len().div_ceil(4);
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(block_size * 4.0, block_size * rows as f32),
                Sense::click_and_drag(),
            );
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
            if !self.drag_and_drop || !ui.is_enabled() {
                self.sidebar_drag = None;
            }
            if !pan.handled
                && self.drag_and_drop
                && response.drag_started_by(egui::PointerButton::Primary)
            {
                self.sidebar_drag = ui.input(|i| i.pointer.press_origin()).and_then(hit);
            }
            let drop = if response.drag_stopped_by(egui::PointerButton::Primary) {
                let from = self.sidebar_drag.take();
                from.zip(response.interact_pointer_pos().and_then(hit))
            } else {
                None
            };
            // Metatile_Button sends its release callback to the pressed button;
            // select_metatile_cb acts only if the pointer is still over it.
            // Keep that cell identity even though egui uses one sidebar region.
            let mut released = None;
            if ui.is_enabled() && !pan.handled {
                for event in ui.input(|i| i.events.clone()) {
                    match event {
                        egui::Event::PointerButton {
                            pos,
                            button,
                            pressed: true,
                            ..
                        } if button != egui::PointerButton::Middle
                            && (response.contains_pointer()
                                || response.is_pointer_button_down_on()) =>
                        {
                            self.sidebar_press = hit(pos).map(|id| (button, id));
                        }
                        egui::Event::PointerButton {
                            pos,
                            button,
                            pressed: false,
                            ..
                        } => {
                            if let Some((held, id)) = self.sidebar_press
                                && held == button
                            {
                                self.sidebar_press = None;
                                if response.contains_pointer() && hit(pos) == Some(id) {
                                    released = Some((id, button));
                                }
                            }
                        }
                        egui::Event::PointerGone => self.sidebar_press = None,
                        _ => {}
                    }
                }
            }
            if !ui.is_enabled() || !ui.input(|i| i.pointer.any_down()) {
                self.sidebar_press = None;
            }
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
                    if let (Some(priority), Some(loaded)) = (&priority, &self.loaded) {
                        priority.block(
                            ui.painter(),
                            cell,
                            loaded.graphics.block_priorities(id as u8),
                            self.zoom,
                        );
                    }
                    let outlined = id == usize::from(self.selected)
                        || ui.is_enabled()
                            && self.sidebar_drag.is_some_and(|from| {
                                id == usize::from(from) || ui.rect_contains_pointer(cell)
                            });
                    map_buttons::decorate(
                        ui.painter(),
                        cell,
                        self.zoom,
                        self.grid,
                        outlined,
                        ui.visuals().dark_mode,
                    );
                    if self.ids {
                        map_labels::draw_id(
                            ui.painter(),
                            cell,
                            id as u8,
                            self.hex,
                            self.zoom,
                            false,
                        );
                    }
                    if let Some(digit) = self.block_hotkeys.digit(id as u8) {
                        block_hotkeys::draw(ui.painter(), cell, digit, id as u8, self);
                    }
                }
            }
            if let Some((id, button)) = released {
                self.selected = id;
                if button == egui::PointerButton::Secondary {
                    self.prepare_block(ui.ctx());
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
        self.sidebar_scroll = output.state.clamped_offset().y;
    }

    fn canvas(&mut self, ui: &mut egui::Ui, layout: &geometry::Layout) {
        if self.reset_canvas_scroll || self.canvas_zoom != self.zoom {
            self.canvas_pan.reset();
        }
        let content = self.loaded.as_ref().map_or(Vec2::ZERO, |loaded| {
            let map = loaded.document.map();
            Vec2::new(f32::from(map.width()) + 6.0, f32::from(map.height()) + 6.0)
                * layout.block_size
        });
        let pan = self
            .canvas_pan
            .update(ui, self.canvas_scroll, content, false);
        if pan.handled {
            self.stroke.clear();
            self.event_ui.drag = None;
        }
        let priority = self
            .show_priority
            .then(|| priority::Textures::get(ui.ctx()));
        self.ruler_cursor = None;
        self.ruler_scroll = Vec2::ZERO;
        self.event_ui.cursor = None;
        self.event_ui.screen = None;
        self.event_ui.hovered_event = None;
        if self.event_ui.mode {
            self.stroke.clear();
        } else {
            self.event_ui.drag = None;
        }
        let Some(loaded) = &self.loaded else {
            self.canvas_zoom = self.zoom;
            self.canvas_scroll = Vec2::ZERO;
            ui.centered_and_justified(|ui| {
                ui.label("Open a map to edit blocks.");
            });
            return;
        };
        let map = loaded.document.map();
        let size = layout.block_size;
        let mut command = None;
        let mut event_action = None;
        let mut scroll = egui::ScrollArea::both()
            .id_salt("map")
            .auto_shrink([false, false])
            // At the source minimum window size, zoom/rulers leave less than
            // egui's default 64-point viewport. Keep the bars inside that area.
            .min_scrolled_width(0.0)
            .min_scrolled_height(0.0)
            .scroll_source(egui::scroll_area::ScrollSource {
                drag: egui::scroll_area::DragScroll::Never,
                ..Default::default()
            });
        let mut retained_scroll = self.canvas_scroll;
        if let Some(offset) = pan.offset {
            retained_scroll = offset;
            scroll = scroll
                .scroll_offset(offset)
                .scroll_source(egui::scroll_area::ScrollSource {
                    mouse_wheel: true,
                    ..egui::scroll_area::ScrollSource::NONE
                });
        }
        if self.reset_canvas_scroll || self.canvas_zoom != self.zoom {
            // egui starts physics against the previous frame's bounds. A source
            // zoom/reset is immediate, even when it exceeds those old bounds.
            ui.spacing_mut().scroll.kinetic.rubber_band = false;
            ui.spacing_mut().scroll.kinetic.decay_time = 0.0;
        }
        if self.reset_canvas_scroll {
            retained_scroll = Vec2::ZERO;
            scroll = scroll.scroll_offset(Vec2::ZERO);
            self.reset_canvas_scroll = false;
        } else if self.canvas_zoom != self.zoom {
            retained_scroll = Vec2::new(
                geometry::zoom_scroll(self.canvas_scroll.x, self.zoom),
                geometry::zoom_scroll(self.canvas_scroll.y, self.zoom),
            );
            scroll = scroll.scroll_offset(retained_scroll);
        }
        let viewport_origin = ui.max_rect().min;
        let output = scroll.show_viewport(ui, |ui, viewport| {
            let extent = Vec2::new(
                (f32::from(map.width()) + 6.0) * size,
                (f32::from(map.height()) + 6.0) * size,
            )
            // FLTK 1.4.5 Fl_Slider::scrollvalue extends the range to include
            // pos + page size. Zooming out can therefore leave blank space at
            // the far edge. Retain only the last committed offset, not rubber
            // band overscroll, and let the range contract as the user returns.
            .max(retained_scroll + viewport.size());
            let (rect, response) = ui.allocate_exact_size(extent, Sense::click_and_drag());
            self.ruler_scroll = viewport_origin - rect.min;
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
                    if let Some(priority) = &priority {
                        priority.block(
                            ui.painter(),
                            cell,
                            loaded.graphics.block_priorities(id),
                            self.zoom,
                        );
                    }
                    let hovered = !self.event_ui.mode
                        && !pan.handled
                        && ui.is_enabled()
                        && ui.rect_contains_pointer(cell.intersect(clip));
                    map_buttons::decorate(
                        ui.painter(),
                        cell,
                        self.zoom,
                        self.grid,
                        hovered,
                        ui.visuals().dark_mode,
                    );
                    if self.ids {
                        map_labels::draw_id(ui.painter(), cell, id, self.hex, self.zoom, hovered);
                    }
                }
            }
            if let Some(events) = &loaded.events {
                event_action = event_workflows::canvas_events(
                    ui,
                    events,
                    origin,
                    size / 2.0,
                    (map.width(), map.height()),
                    &mut self.event_ui,
                    !pan.handled,
                );
            }
            if ui.is_enabled()
                && !pan.handled
                && ui.rect_contains_pointer(clip)
                && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
            {
                let event = self.event_ui.hovered_event.and_then(|index| {
                    loaded
                        .events
                        .as_ref()?
                        .script()
                        .events()
                        .entries()
                        .get(index)
                        .map(|event| (event.definition().x, event.definition().y))
                });
                self.ruler_cursor = geometry::EventCursor::at(
                    pos.x - origin.x,
                    pos.y - origin.y,
                    size,
                    map.width(),
                    map.height(),
                    event,
                );
                if self.event_ui.mode {
                    self.event_ui.cursor = self.ruler_cursor;
                    event_workflows::draw_event_cursor(ui, origin, size / 2.0, &mut self.event_ui);
                }
            }
            if !self.event_ui.mode
                && !pan.handled
                && let Some(pos) = response
                    .interact_pointer_pos()
                    .or_else(|| response.hover_pos())
            {
                let relative = pos - rect.min;
                if let Some((x, y)) = layout.hit(relative.x, relative.y, map.width(), map.height())
                {
                    let point = Point { x, y };
                    let old =
                        map.blocks()[usize::from(y) * usize::from(map.width()) + usize::from(x)];
                    // Block::handle calls change_block_cb on FL_PUSH, not release.
                    if ui.is_enabled()
                        && response.contains_pointer()
                        && clip.contains(pos)
                        && ui.input(|i| i.pointer.secondary_pressed())
                        && usize::from(old) < self.textures.len()
                    {
                        self.selected = old;
                        self.sidebar_reveal = Some(old);
                        ui.ctx().request_repaint();
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
        self.canvas_zoom = self.zoom;
        self.canvas_scroll = output.state.clamped_offset();
        if let Some(command) = command {
            self.apply(command);
        }
        if let Some(action) = event_action {
            self.event_action(action, ui.ctx());
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.poll_workers(&ctx);
        self.poll_event_scripts();
        if ctx.input(|i| i.viewport().close_requested())
            && (self.about.is_some()
                || self.print_form.is_some()
                || self.print_success.is_some()
                || self.print_request.is_some()
                || self.print_worker.is_some())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        } else if ctx.input(|i| i.viewport().close_requested())
            && self.is_modified()
            && !self.exit_approved
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.request_exit(&ctx);
        }
        let blocked = self.about.is_some()
            || self.print_form.is_some()
            || self.print_success.is_some()
            || self.print_worker.is_some()
            || self.print_request.is_some()
            || self.open_form.is_some()
            || self.overworld_form.is_some()
            || self.overworld_warning.is_some()
            || self.event_ui.change.is_some()
            || self.event_ui.form.is_some()
            || self.resize_form.is_some()
            || self.block_form.is_some()
            || self.tileset_form.is_some()
            || self.palette_form.is_some()
            || self.roof_choice_form.is_some()
            || self.confirm_roof_change
            || self.tileset_choice_form.is_some()
            || self.confirm_tileset_change
            || self.pending_palette_load.is_some()
            || self.pending_roof_colors.is_some()
            || self.roof_color_success.is_some()
            || self.block_count_form.is_some()
            || self.confirm_discard
            || self.error.is_some()
            || self.loading.is_some()
            || self.pending_open.is_some()
            || self.open_palette_warning.is_some();
        if !blocked && !ctx.egui_wants_keyboard_input() {
            self.block_hotkey_input(&ctx);
            if ctx.input_mut(|i| {
                i.modifiers.matches_exact(egui::Modifiers::COMMAND)
                    && i.consume_key(egui::Modifiers::COMMAND, egui::Key::Slash)
            }) {
                self.about = Some(Default::default());
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F1)) {
                self.help_window.open();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
                fullscreen::toggle(&ctx);
            }
            if ctx.input_mut(|i| {
                i.modifiers.matches_exact(egui::Modifiers::NONE)
                    && i.consume_key(egui::Modifiers::NONE, egui::Key::F10)
            }) {
                self.transparency.toggle();
            }
            if ctx.input_mut(|i| {
                // egui's logical match ignores extra Shift/Alt modifiers.
                // Command+Shift+P belongs to Show Priority, even when disabled.
                i.modifiers.matches_exact(egui::Modifiers::COMMAND)
                    && i.consume_key(egui::Modifiers::COMMAND, egui::Key::P)
            }) {
                self.prepare_print();
            }
            self.recent_shortcuts(&ctx);
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Plus,
                ) || i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Equals,
                ) || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Equals)
                    || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Plus)
            }) {
                self.zoom = !self.zoom;
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::G)) {
                self.grid = !self.grid;
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::I)) {
                self.ids = !self.ids;
            }
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Num4,
                )
            }) {
                self.hex = !self.hex;
            }
            if self.allow_priority
                && ctx.input_mut(|i| {
                    i.consume_key(
                        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                        egui::Key::P,
                    )
                })
            {
                self.show_priority = !self.show_priority;
            }
            self.event_shortcuts(&ctx);
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::W)) {
                self.request_close_map();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::F4)) {
                self.request_exit(&ctx);
            }
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
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::H,
                )
            }) {
                self.request_roof_change();
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::H)) {
                self.request_tileset_change();
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
        ui.painter()
            .rect_filled(bounds, 0.0, ui.visuals().panel_fill);
        ui.add_enabled_ui(!blocked, |ui| {
            let menu =
                Rect::from_min_size(bounds.min, Vec2::new(bounds.width(), geometry::MENU_HEIGHT));
            ui.scope_builder(egui::UiBuilder::new().max_rect(menu), |ui| {
                self.menu_bar(ui);
            });
            let toolbar = Rect::from_min_size(
                bounds.min + Vec2::new(0.0, geometry::MENU_HEIGHT),
                Vec2::new(bounds.width(), geometry::TOOLBAR_HEIGHT),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(toolbar), |ui| {
                ui.set_clip_rect(toolbar.intersect(ui.clip_rect()));
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(toolbar.shrink2(Vec2::new(0.0, 1.0))),
                    |ui| {
                        ui.horizontal(|ui| self.toolbar(ui));
                    },
                );
            });
            // Controls can change zoom/rulers during this frame. Size content
            // before applying the scaled offset, so the old extent cannot clamp it.
            let layout = geometry::Layout::new(
                bounds.width(),
                bounds.height(),
                self.zoom,
                self.rulers,
                16.0,
            );
            if self.canvas_zoom != self.zoom {
                // update_layout repositions sidebar children at its origin;
                // FLTK derives a zero scroll position from those child bounds.
                self.reset_sidebar_scroll = true;
            }
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
            self.draw_rulers(ui, &layout, bounds.min);
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect(layout.status)), |ui| {
                self.draw_status_bar(ui);
            });
        });
        if ctx.input(|i| i.pointer.any_released()) && !self.stroke.is_empty() {
            let points = std::mem::take(&mut self.stroke);
            self.apply(Command::Paint {
                points,
                block: self.selected,
            });
        }
        self.help_window.show(&ctx, !blocked);
        self.dialogs(&ctx);
        self.sync_window_title(&ctx);
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
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.store_preferences(storage);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        // Main_Window::exit_cb persists preferences only on exit.
        std::time::Duration::MAX
    }

    fn persist_egui_memory(&self) -> bool {
        // Document scroll, widget focus and transient modal state are not settings.
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
        #[cfg(not(test))]
        {
            self.dispatch_event_script(ui.ctx(), polished_map_platform::ScriptOpenRequest::open);
            self.dispatch_print(ui.ctx(), printing::choose_export, printing::run);
            self.dispatch_transparency(|opacity| {
                polished_map_platform::set_window_opacity(_frame, opacity)
            });
        }
        #[cfg(windows)]
        self.choose_palette_color(_frame);
    }
}
