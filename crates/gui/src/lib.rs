//! FLTK-derived logical geometry and egui adapter. LGPL-3.0-only.
mod about;
mod application;
use application::Application;
mod artwork;
mod asset_drag;
pub use artwork::application_icon;
mod block_editor;
mod block_hotkeys;
mod block_workflows;
mod canvas;
mod choice_preview;
mod default_value;
mod dialog_host;
mod documents;
#[cfg(test)]
mod drag_tests;
mod editor_dialogs;
#[cfg(all(test, feature = "visual-tests"))]
mod event_label_tests;
#[cfg(test)]
mod event_workflow_tests;
mod event_workflows;
mod external_drop;
mod file_workflows;
mod fonts;
mod fullscreen;
pub mod geometry;
mod graphics_setup;
mod help;
mod icons;
mod map_buttons;
mod map_labels;
mod map_loading;
mod map_open;
#[cfg(test)]
mod map_open_tests;
mod map_options;
mod map_save;
mod menu_popup;
mod menus;
mod message_dialog;
mod nice_slider;
mod numeric_input;
#[cfg(test)]
mod open_scroll_tests;
mod overworld;
mod palette_editor;
#[cfg(test)]
mod palette_editor_tests;
mod palette_session;
#[cfg(test)]
mod palette_session_tests;
mod plusplus_session;
mod preferences;
pub use preferences::legacy::NativeStartup;
#[cfg(feature = "native-tests")]
pub use preferences::legacy::native_probe::run as run_native_preferences_probe;
mod printing;
mod priority;
mod recent_maps;
mod resize_dialog;
mod roof_colors;
#[cfg(test)]
mod roof_workflow_tests;
mod roof_workflows;
mod rulers;
mod save_feedback;
mod shortcuts;
mod sidebar;
mod startup_theme;
mod status_bar;
mod tileset_editor;
#[cfg(test)]
mod tileset_workflow_tests;
mod tileset_workflows;
mod toolbar;
#[cfg(test)]
mod toolbar_tests;
mod transparency;
mod unified_open;
#[cfg(test)]
mod unified_tests;
mod unified_tileset;
mod window_title;
mod workspace_pan;
#[cfg(test)]
mod zoom_tests;
use block_editor::BlockForm;
#[cfg(test)]
use canvas::append_line;

#[cfg(test)]
mod block_workflow_tests;
#[cfg(test)]
mod roof_editor_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tileset_editor_tests;

use std::{path::PathBuf, sync::mpsc};

use documents::MapGraphics;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Vec2};
#[cfg(test)]
use polished_map_core::Graphics;
use polished_map_core::{Command, Point, ResizeAnchor};
use polished_map_project::{
    BlocksetDocument, MapDocument, MasterGraphicsOptions, MasterTilesetDocument, OpenOptions,
    prepare_blockset_save_as, prepare_collision_save_as,
};

/// Development application identity, also used before a map is open.
pub const WINDOW_TITLE: &str = "Polished Map NG — development";

struct Loaded {
    roof_layout: Option<polished_map_project::RoofPaletteLayout>,
    // Retain chosen spelling for history/title independently of canonical I/O paths.
    recent_path: Option<PathBuf>,
    trailing_blocks: usize,
    events: Option<documents::EventDocument>,
    event_warning: Option<String>,
    palette_plan: Option<documents::PalettePlan>,
    plus_assets: Option<plusplus_session::PlusPlusAssets>,
    document: documents::MapDocument,
    graphics: MapGraphics,
    blockset: Option<BlocksetDocument>,
    tileset: Option<MasterTilesetDocument>,
    allow_256: bool,
}
type LoadResult = Result<Loaded, String>;

#[derive(Default)]
struct OpenForm {
    roof_layout: Option<polished_map_project::RoofPaletteLayout>,
    detected_context: String,
    roof_layout_note: String,
    unified: bool,
    detection_error: Option<String>,
    profile: documents::Profile,
    allow_512: bool,
    new_map: bool,
    path: PathBuf,
    root: String,
    tileset: String,
    roof: String,
    width: u16,
    height: u16,
    allow_256: bool,
    sizes: Option<polished_map_project::MapSizeOptions>,
    tilesets: Vec<polished_map_project::TilesetChoice>,
    roofs: Vec<String>,
    choices_root: String,
    #[cfg(test)]
    client_bounds: Option<Rect>,
    #[cfg(test)]
    client_layout: Option<map_options::Layout>,
}

impl OpenForm {
    fn refresh_choices(&mut self) -> Result<(), polished_map_project::ProjectError> {
        if self.unified {
            return self.refresh_unified_choices();
        }
        let root = std::path::Path::new(&self.root);
        let metadata = if self.new_map {
            None
        } else {
            Some(self.profile.discover_map(&self.path, Some(root))?)
        };
        let choices = self.profile.tileset_options(
            root,
            metadata.as_ref().and_then(|m| m.tileset_hint.as_deref()),
        )?;
        let roofs = self.profile.list_roofs(root)?;
        let sizes = if let Some(metadata) = metadata {
            Some(
                self.profile
                    .inspect_map_sizes(&self.path, metadata.dimensions)?,
            )
        } else {
            None
        };
        self.tileset = choices
            .selected()
            .map(|i| choices.choices()[i].name.clone())
            .unwrap_or_default();
        self.tilesets = choices.choices().to_vec();
        self.roof.clear();
        self.roofs = roofs;
        if let Some(sizes) = &sizes {
            (self.width, self.height) = sizes.dimensions();
        }
        self.sizes = sizes;
        self.choices_root = self.root.clone();
        Ok(())
    }
}

struct ResizeForm {
    width: u16,
    height: u16,
    defaults: [default_value::DefaultValue<u16>; 2],
    anchor: ResizeAnchor,
    windows_buttons: bool,
    opened: bool,
    #[cfg(test)]
    bounds: Rect,
}

pub struct Editor {
    application: Application,
    inactive_palettes: Option<documents::PaletteDocument>,
    allow_512: bool,
    bank_order: polished_map_core::plusplus::BankOrder,
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
    print_clipboard: Option<u64>,
    print_form: Option<printing::PrintForm>,
    print_options: polished_map_core::PrintOptions,
    print_request: Option<printing::PrintJob>,
    print_success: Option<String>,
    open_form: Option<OpenForm>,
    resize_form: Option<ResizeForm>,
    overworld_form: Option<overworld::OverworldForm>,
    overworld_map_size: u16,
    overworld_warning: Option<String>,
    save_ui: save_feedback::SaveUi,
    meta128: bool,
    block_form: Option<BlockForm>,
    tileset_form: Option<tileset_editor::TilesetForm>,
    palette_form: Option<palette_editor::PaletteForm>,
    roof_choice_form: Option<roof_workflows::RoofChoiceForm>,
    confirm_roof_change: bool,
    tileset_choice_form: Option<tileset_workflows::TilesetChoiceForm>,
    confirm_tileset_change: bool,
    pending_tileset_reload: bool,
    unloaded_palettes: documents::PaletteDocument,
    monochrome: bool,
    monochrome_override: bool,
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
    startup_window_state: Option<preferences::legacy::StartupWindowState>,
    recent_maps: recent_maps::RecentMaps,
    pending_recent: Option<PathBuf>,
    #[cfg(windows)]
    native_color_request: Option<polished_map_core::ColorSlot>,
    #[cfg(windows)]
    native_color_picker: polished_map_platform::ColorPicker,

    startup_theme: Option<mpsc::Receiver<Option<bool>>>,
    block_count_form: Option<u16>,
    block_count_default: default_value::DefaultValue<u16>,
    block_count_windows_buttons: bool,
    block_clipboard: Option<block_workflows::BlockClipboard>,
    plus_block_attributes: [bool; 3],
    paste_shortcut_handled: bool,
    sidebar_drag: Option<u8>,
    sidebar_drag_press: Option<(u8, Pos2)>,
    sidebar_press: Option<(egui::PointerButton, u8)>,
    block_hotkeys: block_hotkeys::BlockHotkeys,
    sidebar_reveal: Option<u8>,
    reset_sidebar_scroll: bool,
    resize_anchor: ResizeAnchor,
    reset_canvas_scroll: bool,
    initial_canvas_scroll: bool,
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
            application: Default::default(),
            inactive_palettes: None,
            allow_512: false,
            bank_order: Default::default(),
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
            print_clipboard: None,
            print_form: None,
            print_options: Default::default(),
            print_request: None,
            print_success: None,
            open_form: None,
            resize_form: None,
            overworld_form: None,
            overworld_map_size: 1300,
            overworld_warning: None,
            save_ui: Default::default(),
            meta128: false,
            block_form: None,
            tileset_form: None,
            palette_form: None,
            roof_choice_form: None,
            confirm_roof_change: false,
            tileset_choice_form: None,
            confirm_tileset_change: false,
            pending_tileset_reload: false,
            unloaded_palettes: Default::default(),
            monochrome: false,
            monochrome_override: false,
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
            startup_window_state: None,
            recent_maps: recent_maps::RecentMaps::default(),
            pending_recent: None,
            #[cfg(windows)]
            native_color_request: None,
            #[cfg(windows)]
            native_color_picker: polished_map_platform::ColorPicker::default(),
            startup_theme: None,
            block_count_form: None,
            block_count_default: default_value::DefaultValue::new(1),
            block_count_windows_buttons: cfg!(target_os = "windows"),
            block_clipboard: None,
            plus_block_attributes: [false; 3],
            paste_shortcut_handled: false,
            sidebar_drag: None,
            sidebar_drag_press: None,
            sidebar_press: None,
            block_hotkeys: Default::default(),
            sidebar_reveal: None,
            reset_sidebar_scroll: false,
            resize_anchor: ResizeAnchor::default(),
            reset_canvas_scroll: false,
            initial_canvas_scroll: false,
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
    /// Desktop launchers use `NativeStartup` to add native fonts and migration.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_for_profile(cc, documents::Profile::Master)
    }

    /// Separate plusplus editor with its own settings and eight-palette session.
    /// The profile is fixed before reading preferences or opening a project.
    pub fn new_plusplus(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_for_profile(cc, documents::Profile::PlusPlus)
    }

    pub fn new_unified(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_for_application(cc, Application::Unified)
    }

    fn new_for_profile(cc: &eframe::CreationContext<'_>, profile: documents::Profile) -> Self {
        Self::new_for_application(cc, Application::Parity(profile))
    }

    fn new_for_application(cc: &eframe::CreationContext<'_>, application: Application) -> Self {
        let profile = application.initial_profile();
        fonts::install_bundled(&cc.egui_ctx);
        let mut editor = Self {
            application,
            ..Self::default()
        };
        editor.help_window.application = application;
        editor.help_window.profile = profile;
        if profile == documents::Profile::PlusPlus {
            editor.unloaded_palettes =
                polished_map_project::plusplus::PaletteDocument::default().into();
            editor.status =
                "Open a map to begin. Development build: plusplus parity is incomplete.".into();
        }
        if application.is_unified() {
            editor.status =
                "Open a map to begin. Development build: unified editing is incomplete.".into();
        }
        editor.restore_preferences(cc.storage);
        // Ctrl+= belongs to map zoom and Ctrl+0 to block assignment. Letting
        // egui consume the remaining UI-zoom shortcuts creates one-way scaling.
        // OS DPI scaling remains automatic.
        cc.egui_ctx
            .options_mut(|options| options.zoom_with_keyboard = false);
        // Recover a scale persisted by the old one-way shortcut.
        cc.egui_ctx.set_zoom_factor(1.0);
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
            s.spacing.scroll.foreground_color = false;
            s.interaction.selectable_labels = false;
            s.text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
            s.text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
        });
        editor
    }

    fn is_modified(&self) -> bool {
        self.loaded.as_ref().is_some_and(|l| {
            l.plus_assets.as_ref().is_some_and(|a| a.is_modified())
                || l.document.map().is_modified()
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

    fn apply(&mut self, command: Command) {
        if let Some(loaded) = &mut self.loaded {
            if matches!(command, Command::Resize { .. }) {
                match event_workflows::resize_with_events(loaded, &command) {
                    Err(error) => self.error = Some(error),
                    Ok(true) => {
                        self.reset_canvas_scroll = true;
                        self.initial_canvas_scroll = false;
                    }
                    Ok(false) => {}
                }
                return;
            }
            if let Err(error) = loaded.document.map_mut().apply(&command) {
                self.error = Some(error.to_string());
            }
        }
    }

    fn prepare_resize(&mut self) {
        if let Some(loaded) = &self.loaded {
            self.resize_form = Some(ResizeForm {
                width: u16::from(loaded.document.map().width()),
                height: u16::from(loaded.document.map().height()),
                defaults: [
                    loaded.document.map().width(),
                    loaded.document.map().height(),
                ]
                .map(|n| default_value::DefaultValue::new(u16::from(n))),
                anchor: self.resize_anchor,
                windows_buttons: cfg!(windows),
                opened: false,
                #[cfg(test)]
                bounds: Rect::NOTHING,
            });
        }
    }

    fn continue_after_discard(&mut self, ctx: &egui::Context) {
        if self.pending_exit {
            self.pending_exit = false;
            self.exit_approved = true;
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
        } else if self.pending_close {
            self.close_map();
        } else if self.pending_new {
            self.start_new_options();
        } else if let Some(path) = self.pending_recent.take() {
            self.prepare_open(path);
        } else if let Some(path) = self.event_ui.warp_path.take() {
            self.open_warp(path, ctx);
        } else {
            self.open_map_picker();
        }
    }

    fn interaction_blocked(&self) -> bool {
        self.save_ui.busy()
            || self.about.is_some()
            || self.print_form.is_some()
            || self.print_success.is_some()
            || self.print_worker.is_some()
            || self.print_clipboard.is_some()
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
            || self.open_palette_warning.is_some()
            || self.pending_map_warning.is_some()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.poll_workers(&ctx);
        self.poll_event_scripts();
        self.advance_save();
        if ctx.input(|i| i.viewport().close_requested()) && self.interaction_blocked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        } else if ctx.input(|i| i.viewport().close_requested())
            && self.is_modified()
            && !self.exit_approved
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.request_exit(&ctx);
        }
        self.external_file_drops(&ctx, ui.max_rect());
        let blocked = self.interaction_blocked();
        if !blocked {
            menu_popup::begin(&ctx);
        }
        if !blocked && !ctx.egui_wants_keyboard_input() {
            block_hotkeys::normalize_shifted_digits(&ctx);
            self.block_hotkey_input(&ctx);
            if ctx.input_mut(|i| {
                shortcuts::consume_exact(i, egui::Modifiers::COMMAND, egui::Key::Slash)
            }) {
                self.about = Some(Default::default());
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F1)) {
                self.help_window.open();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
                fullscreen::toggle(&ctx);
            }
            if ctx.input_mut(|i| shortcuts::consume_exact(i, egui::Modifiers::NONE, egui::Key::F9))
            {
                self.dark = !self.dark;
                self.apply_theme_choice(&ctx);
            }
            if ctx.input_mut(|i| shortcuts::consume_exact(i, egui::Modifiers::NONE, egui::Key::F10))
            {
                self.transparency.toggle();
            }
            if ctx.input_mut(|i| {
                // egui's logical match ignores extra Shift/Alt modifiers.
                // Command+Shift+P belongs to Show Priority, even when disabled.
                shortcuts::consume_exact(i, egui::Modifiers::COMMAND, egui::Key::P)
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
            if (self.allow_priority || self.profile() == documents::Profile::PlusPlus)
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
        self.poll_startup_theme(ui.ctx());
        artwork::install_window_icons(ui.ctx());
        self.apply_startup_window_state(ui.ctx());
        self.show(ui);
        #[cfg(not(test))]
        {
            self.dispatch_event_script(ui.ctx(), polished_map_platform::ScriptOpenRequest::open);
            self.dispatch_print(ui.ctx(), printing::choose_export, printing::run);
            self.dispatch_native_transparency(ui.ctx(), |opacity| {
                polished_map_platform::set_window_opacity(_frame, opacity)
            });
        }
        #[cfg(windows)]
        self.choose_palette_color(ui.ctx(), _frame);
    }
}
