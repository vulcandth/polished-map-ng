//! Editor session state and feature module registry. LGPL-3.0-only.
//!
//! `startup` constructs sessions; `frame` polls work and routes input/drawing.
//! `documents` adapts storage models; workflow modules prepare/commit candidates.
//! Forms own drafts and `dialog_host` owns native viewport lifetimes. Keep model
//! validation and filesystem writes in core/project, outside this UI coordinator.
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
use widgets::default_value;
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
use crate::geometry;
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
use widgets::nice_slider;
use widgets::numeric_input;
mod attributed_assets;
#[cfg(test)]
mod open_scroll_tests;
mod overworld;
mod palette_editor;
#[cfg(test)]
mod palette_editor_tests;
mod palette_session;
#[cfg(test)]
mod palette_session_tests;
mod preferences;
pub use preferences::native_startup::NativeStartup;
#[cfg(feature = "native-tests")]
pub use preferences::native_startup::native_probe::run as run_native_preferences_probe;
mod frame;
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
mod startup;
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
mod widgets;
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

/// A completely prepared document candidate or the active published document.
/// Attributed storage uses `attributed_assets`; classic storage uses `blockset`
/// and `tileset`. Open/Change Tileset publish these together with graphics.
struct Loaded {
    roof_layout: Option<polished_map_project::RoofPaletteLayout>,
    // Retain chosen spelling for history/title independently of canonical I/O paths.
    recent_path: Option<PathBuf>,
    trailing_blocks: usize,
    events: Option<documents::EventDocument>,
    event_warning: Option<String>,
    palette_plan: Option<documents::PalettePlan>,
    attributed_assets: Option<attributed_assets::AttributedAssets>,
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
    profile: documents::Storage,
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

/// One editor session: document adapters, drafts, view state and pending work.
///
/// Use [`Self::new_unified`] for automatic project detection. `Default` and
/// [`Self::new`] retain the fixed classic launcher behavior for existing callers.
/// Persistent edits go through core/project commands; this type coordinates UI.
/// Session internals are private so callers cannot bypass draft/save guards.
pub struct Editor {
    application: Application,
    inactive_palettes: Option<documents::PaletteDocument>,
    allow_512: bool,
    bank_order: polished_map_core::attributed::BankOrder,
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
    startup_window_state: Option<preferences::native_startup::StartupWindowState>,
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
    occurrence_brush_attributes: [bool; 3],
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

impl Editor {
    fn is_modified(&self) -> bool {
        self.loaded.as_ref().is_some_and(|l| {
            l.attributed_assets
                .as_ref()
                .is_some_and(|a| a.is_modified())
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
}
