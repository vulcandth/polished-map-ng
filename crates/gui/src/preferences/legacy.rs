//! First-launch migration of master preference keys; original stores are read-only.
use super::*;
use polished_map_project::{
    LegacyPreferenceError, LegacyPreferences, LegacyUserLocation, find_legacy_preferences,
};
use std::path::Path;

#[derive(Clone, Copy, Default)]
pub(crate) struct StartupWindowState {
    fullscreen: bool,
    maximized: bool,
}

struct Imported {
    settings: Preferences,
    size: [f32; 2],
    position: Option<[f32; 2]>,
    state: StartupWindowState,
    source: PathBuf,
    theme_fallback: bool,
}

/// Prepared before eframe creates its native window. Existing NG storage always
/// wins, including unreadable/future records. No legacy source is ever written.
#[derive(Default)]
pub struct NativeStartup {
    imported: Option<Imported>,
    error: Option<String>,
}

impl NativeStartup {
    pub fn discover() -> Self {
        let user = if cfg!(windows) {
            LegacyUserLocation::Windows {
                roaming_appdata: polished_map_platform::legacy_roaming_appdata(),
            }
        } else if cfg!(target_os = "macos") {
            LegacyUserLocation::Mac {
                home: polished_map_platform::legacy_macos_home(),
            }
        } else {
            LegacyUserLocation::Unix {
                home: std::env::home_dir(),
                xdg_config_home: std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            }
        };
        let result = std::env::current_exe()
            .map_err(LegacyPreferenceError::from)
            .and_then(|exe| {
                Self::load(
                    exe.parent().unwrap_or(Path::new(".")),
                    &user,
                    eframe::storage_dir("polished-map-ng")
                        .map(|p| p.join("app.ron"))
                        .as_deref(),
                )
            });
        Self::from_result(result)
    }

    fn from_result(result: Result<Option<Imported>, LegacyPreferenceError>) -> Self {
        match result {
            Ok(imported) => Self {
                imported,
                error: None,
            },
            Err(error) => Self {
                imported: None,
                error: Some(error.to_string()),
            },
        }
    }

    fn load(
        executable_dir: &Path,
        user: &LegacyUserLocation,
        ng_file: Option<&Path>,
    ) -> Result<Option<Imported>, LegacyPreferenceError> {
        if let Some(path) = ng_file
            && path.try_exists()?
        {
            return Ok(None);
        }
        let Some(source) = find_legacy_preferences(executable_dir, user)? else {
            return Ok(None);
        };
        Imported::read(source).map(Some)
    }

    /// Initial normal geometry precedes fullscreen/maximization. This lets
    /// eframe capture a usable restore size before the first native transition.
    pub fn viewport(&self) -> egui::ViewportBuilder {
        let mut viewport = egui::ViewportBuilder::default()
            .with_app_id("polished-map-ng")
            .with_icon(crate::application_icon())
            .with_inner_size(self.imported.as_ref().map_or([800.0, 600.0], |i| i.size))
            .with_min_inner_size([335.0, 262.0]);
        if let Some(position) = self.imported.as_ref().and_then(|i| i.position) {
            viewport = viewport.with_position(position);
        }
        viewport
    }

    pub fn create_editor(self, cc: &eframe::CreationContext<'_>) -> Editor {
        let mut editor = Editor::new_native(cc);
        self.apply(&mut editor, cc.storage, &cc.egui_ctx);
        editor
    }

    fn apply(
        self,
        editor: &mut Editor,
        storage: Option<&dyn eframe::Storage>,
        ctx: &egui::Context,
    ) {
        // A native storage file could appear between preflight and construction.
        if storage.and_then(|s| s.get_string(KEY)).is_some() || !editor.preferences_writable {
            return;
        }
        if let Some(error) = self.error {
            editor.preferences_writable = false;
            editor.status = format!(
                "Legacy preferences could not be imported; using defaults and preserving settings: {error}"
            );
            eprintln!("{}", editor.status);
            return;
        }
        let Some(imported) = self.imported else {
            return;
        };
        imported.settings.apply(editor);
        ctx.set_theme(if editor.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        if storage.and_then(|s| s.get_string("window")).is_none() {
            editor.startup_window_state = Some(imported.state);
        }
        editor.status = format!(
            "Imported settings from {}.{}",
            imported.source.display(),
            if imported.theme_fallback {
                " Unsupported legacy theme uses Greybird."
            } else {
                ""
            }
        );
        eprintln!("{}", editor.status);
    }
}

impl Imported {
    fn read(source: PathBuf) -> Result<Self, LegacyPreferenceError> {
        let legacy = LegacyPreferences::read(&source)?;
        Self::from_legacy(&legacy, source)
    }

    fn from_legacy(
        legacy: &LegacyPreferences,
        source: PathBuf,
    ) -> Result<Self, LegacyPreferenceError> {
        use LegacyPreferenceError::Invalid;
        let boolean = |key, default| legacy.integer(key, i32::from(default)).map(|v| v != 0);
        let theme = legacy.integer("theme", 4)?;
        let mode = legacy.integer("mode", 0)?;
        let palette = legacy.integer("palettes", 1)?;
        let roof = legacy.integer("roof-palettes", 1)?;
        let anchor = legacy.integer("resize-anchor", 4)?;
        let capacity = legacy.integer("overworld-map", 1300)?;
        if !(0..12).contains(&theme)
            || !(0..2).contains(&mode)
            || !(0..6).contains(&palette)
            || !(0..5).contains(&roof)
            || !(0..9).contains(&anchor)
            || !(1..=4096).contains(&capacity)
        {
            return Err(Invalid("setting outside supported source range"));
        }
        let roofs = [
            RoofPaletteLayout::Custom,
            RoofPaletteLayout::DayNite,
            RoofPaletteLayout::MornDayNite,
            RoofPaletteLayout::DayNiteCustom,
            RoofPaletteLayout::MornDayNiteCustom,
        ];
        let mut recent_maps = recent_maps::RecentMaps::default();
        let mut recent = Vec::new();
        for i in 0..10 {
            if let Some(path) = legacy.path(&format!("recent{i}"))?
                && !recent.contains(&path)
            {
                recent.push(path);
            }
        }
        for path in recent.into_iter().rev() {
            recent_maps.remember(path);
        }
        let settings = Preferences {
            version: 1,
            dark: theme == 9,
            transparent: boolean("transparent", false)?,
            event_mode: mode == 1,
            grid: boolean("grid", true)?,
            rulers: boolean("rulers", false)?,
            zoom: boolean("zoom", false)?,
            ids: boolean("ids", false)?,
            hex: boolean("hex", false)?,
            game_boy: boolean("gameboy", false)?,
            show_events: boolean("event", true)?,
            show_warp_ids: boolean("warp-ids", true)?,
            palette_set: PaletteSet::ALL[palette as usize],
            roof_layout: roofs[roof as usize],
            auto_events: boolean("events", true)?,
            auto_special_palettes: boolean("special", true)?,
            auto_roof_colors: boolean("roofs", true)?,
            allow_priority: boolean("prioritize", false)?,
            monochrome: boolean("monochrome", false)?,
            allow_256: boolean("all256", false)?,
            drag_and_drop: boolean("drag", true)?,
            overworld_map_size: capacity as u16,
            meta128: boolean("meta128", false)?,
            show_priority: boolean("priority", true)?,
            resize_anchor: ResizeAnchor::ALL[anchor as usize],
            recent_maps,
            print_options: polished_map_core::PrintOptions {
                grid: boolean("print-grid", false)?,
                ids: boolean("print-ids", false)?,
                priority: boolean("print-priority", false)?,
                events: boolean("print-events", false)?,
                warp_ids: boolean("print-warp-ids", false)?,
            },
        };
        let (w, h) = (legacy.integer("w", 800)?, legacy.integer("h", 600)?);
        if !(1..=16384).contains(&w) || !(1..=16384).contains(&h) {
            return Err(Invalid("invalid window dimensions"));
        }
        let (x, y) = (legacy.integer("x", 48)?, legacy.integer("y", 48)?);
        let position = ((-1_000_000..=1_000_000).contains(&x)
            && (-1_000_000..=1_000_000).contains(&y))
        .then_some([x as f32, y as f32]);
        Ok(Self {
            settings,
            source,
            theme_fallback: !matches!(theme, 4 | 9),
            size: [w.max(335) as f32, h.max(262) as f32],
            position,
            state: StartupWindowState {
                fullscreen: boolean("fullscreen", false)?,
                maximized: boolean("maximized", false)?,
            },
        })
    }
}

impl Editor {
    pub(crate) fn apply_startup_window_state(&mut self, ctx: &egui::Context) {
        let Some(state) = self.startup_window_state else {
            return;
        };
        // The first eframe UI pass precedes showing the native window. The egui
        // `visible()` helper only describes minimization/occlusion, not mapping.
        let hidden =
            egui_winit::modal_windows::with_window(ctx, egui::ViewportId::ROOT, |window| {
                window.is_visible() == Some(false)
            })
            .unwrap_or(false);
        if ctx.cumulative_frame_nr() == 0
            || hidden
            || ctx.input(|i| i.viewport().minimized == Some(true))
        {
            ctx.request_repaint();
            return;
        }
        self.startup_window_state = None;
        // If the old display was removed, keep the titlebar on a current monitor.
        // Winit reports no absolute placement on Wayland; the compositor owns it.
        egui_winit::modal_windows::with_window(ctx, egui::ViewportId::ROOT, |window| {
            if let Ok(position) = window.outer_position() {
                let monitors: Vec<_> = window.available_monitors().collect();
                let mut point = egui::pos2(position.x as f32, position.y as f32);
                if cfg!(target_os = "macos") {
                    point = point / window.scale_factor() as f32;
                }
                let contains = monitors.iter().any(|monitor| {
                    let p = monitor.position();
                    let s = monitor.size();
                    let rect = Rect::from_min_size(
                        egui::pos2(p.x as f32, p.y as f32),
                        Vec2::new(s.width as f32, s.height as f32),
                    );
                    let rect = if cfg!(target_os = "macos") {
                        rect / monitor.scale_factor() as f32
                    } else {
                        rect
                    };
                    rect.contains(point)
                });
                if !contains
                    && let Some(monitor) = window
                        .current_monitor()
                        .or_else(|| monitors.first().cloned())
                {
                    let p = monitor.position();
                    let scale = monitor.scale_factor() as f32;
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                        egui::pos2(p.x as f32 / scale + 48.0, p.y as f32 / scale + 48.0)
                            / ctx.zoom_factor(),
                    ));
                }
            }
        });
        if state.fullscreen {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        } else if state.maximized {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
    }
}

#[cfg(test)]
mod tests;
