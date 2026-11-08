//! master Main_Window::open_map imports into a shared palette session.
//! LGPL-3.0-only. Keep the previous map until all import decisions and previews succeed.
use super::*;

fn advance_import(
    loaded: &mut Loaded,
    load: bool,
    warnings: &mut Vec<String>,
) -> Result<(), polished_map_project::ProjectError> {
    let roof = loaded
        .palette_plan
        .as_ref()
        .is_some_and(|p| p.next_is_roof_colors());
    advance_step(roof, load, warnings, |load| loaded.advance_palette(load))
}

/// Optional roof colors warn without discarding an otherwise valid candidate.
/// Open and cross-storage asset changes share this recovery policy.
pub(super) fn advance_step(
    roof: bool,
    load: bool,
    warnings: &mut Vec<String>,
    mut advance: impl FnMut(bool) -> Result<(), polished_map_project::ProjectError>,
) -> Result<(), polished_map_project::ProjectError> {
    match advance(load) {
        Err(error) if roof => {
            warnings.push(format!("Roof colors were not loaded:\n{error}"));
            advance(false)
        }
        result => result,
    }
}

impl Editor {
    pub(super) fn advance_open_palettes(&mut self, ctx: &egui::Context) {
        let Some(mut loaded) = self.pending_open.take() else {
            return;
        };
        while loaded
            .palette_plan
            .as_ref()
            .is_some_and(|p| !p.is_finished())
        {
            if loaded
                .palette_plan
                .as_ref()
                .is_some_and(|p| p.next_import_path().is_some())
                && loaded.palette_view().is_some_and(|p| p.is_modified())
            {
                self.pending_open = Some(loaded);
                return;
            }
            if let Err(e) = advance_import(&mut loaded, true, &mut self.pending_palette_warnings) {
                self.error = Some(e.to_string());
                self.status = "Open failed".into();
                return;
            }
        }
        if loaded.trailing_blocks != 0 {
            self.pending_map_warning = Some(format!(
                "Warning: {}:\n\nThe .blk file is larger than the specified size.",
                loaded.display_name().to_string_lossy(),
            ));
            loaded.trailing_blocks = 0;
            self.pending_open = Some(loaded);
            return;
        }
        if let Err(e) = self.finish_open(loaded, ctx) {
            self.error = Some(e);
            self.status = "Open failed".into();
        }
    }

    fn finish_open(&mut self, mut loaded: Loaded, ctx: &egui::Context) -> Result<(), String> {
        let profile = loaded.document.profile();
        if !self.application.accepts(profile) || loaded.graphics.profile() != profile {
            return Err("Map and graphics must belong to this editor profile".into());
        }
        if loaded.palette_plan.take().is_some() {
            loaded.graphics = loaded.refresh_graphics()?;
        }
        if loaded.plus_assets.is_some()
            && (loaded.tileset.is_some()
                || loaded.blockset.is_some()
                || !matches!(loaded.document, documents::MapDocument::PlusPlus(_)))
        {
            return Err("Plusplus assets require a separate plusplus document".into());
        }
        let textures = documents::block_textures(ctx, &loaded.graphics)?;
        if self.profile() != profile {
            self.inactive_palettes = Some(self.palette_document().to_owned());
        }
        if let Some(assets) = &loaded.plus_assets {
            self.allow_512 = assets.allow_512;
        }
        self.help_window.profile = profile;
        self.status = if loaded.document.has_backing_file() {
            loaded.document.path().display().to_string()
        } else {
            "New map (unsaved)".into()
        };
        self.error = loaded.event_warning.take();
        if let Some(tileset) = &mut loaded.tileset {
            self.monochrome = tileset.monochrome();
            self.allow_256 = tileset.allow_256();
            self.allow_priority |= tileset.allow_priority();
            tileset.set_allow_priority(self.allow_priority);
        }
        if loaded.document.has_backing_file() {
            self.recent_maps.remember(
                loaded
                    .recent_path
                    .clone()
                    .unwrap_or_else(|| loaded.document.path().to_owned()),
            );
        }
        // Map_Options_Dialog inventories roofs when opening a project. Cache
        // availability once; never scan the filesystem while painting controls.
        self.roof_choices_available = loaded.tileset_location().is_some_and(|(root, _)| {
            self.application
                .list_roofs(profile, root)
                .is_ok_and(|names| !names.is_empty())
        });
        self.event_ui.drag = None;
        self.event_ui.form = None;
        self.loaded = Some(loaded);
        self.textures = textures;
        self.selected = 0;
        self.block_clipboard = None;
        self.sidebar_drag = None;
        self.sidebar_drag_press = None;
        self.sidebar_press = None;
        self.canvas_pan.reset();
        self.sidebar_pan.reset();
        self.block_hotkeys = Default::default();
        self.sidebar_reveal = None;
        self.reset_sidebar_scroll = true;
        self.reset_canvas_scroll = false;
        self.initial_canvas_scroll = true;
        self.stroke.clear();
        if !self.pending_palette_warnings.is_empty() {
            self.open_palette_warning =
                Some(std::mem::take(&mut self.pending_palette_warnings).join("\n\n"));
        }
        Ok(())
    }

    pub(super) fn open_palette_dialog(&mut self, ctx: &egui::Context) {
        let Some(path) = self
            .pending_open
            .as_ref()
            .and_then(|l| l.palette_plan.as_ref())
            .and_then(|p| p.next_import_path())
        else {
            return;
        };
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let message = format!("The palettes have been edited!\n\nLoad {name} anyway?");
        let (load, skip) = message_dialog::confirm(
            ctx,
            "open-map-palettes",
            &message,
            self.discard_windows_buttons,
            ["Load", "Skip"],
        );
        if load || skip {
            let pending = self.pending_open.as_mut().expect("dialog has pending map");
            let result = advance_import(pending, !skip, &mut self.pending_palette_warnings);
            if let Err(e) = result {
                self.pending_open = None;
                self.error = Some(e.to_string());
                self.status = "Open failed".into();
            } else {
                self.advance_open_palettes(ctx);
            }
        }
    }
}
