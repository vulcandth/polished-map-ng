//! Session construction, defaults and initial egui configuration. LGPL-3.0-only.
use super::*;

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
            occurrence_brush_attributes: [false; 3],
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
        Self::new_for_storage(cc, documents::Storage::Classic)
    }

    /// Separate plusplus editor with its own settings and eight-palette session.
    /// The profile is fixed before reading preferences or opening a project.
    pub fn new_plusplus(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_for_storage(cc, documents::Storage::Attributed)
    }

    /// Construct an automatically detected editor using independent NG settings.
    /// A document selects its storage model only after Open/New succeeds.
    pub fn new_ng(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_for_application(cc, Application::Ng)
    }

    pub(super) fn new_for_storage(
        cc: &eframe::CreationContext<'_>,
        storage: documents::Storage,
    ) -> Self {
        Self::new_for_application(cc, Application::Parity(storage))
    }

    pub(super) fn new_for_application(
        cc: &eframe::CreationContext<'_>,
        application: Application,
    ) -> Self {
        let storage = application.initial_storage();
        fonts::install_bundled(&cc.egui_ctx);
        let mut editor = Self {
            application,
            ..Self::default()
        };
        editor.help_window.application = application;
        editor.help_window.storage = storage;
        if storage == documents::Storage::Attributed {
            editor.unloaded_palettes =
                polished_map_project::attributed::PaletteDocument::default().into();
            editor.status =
                "Open a map to begin. Development build: plusplus parity is incomplete.".into();
        }
        if application.is_ng() {
            editor.status = "Open a map to begin. Polished Map NG development build.".into();
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
}
