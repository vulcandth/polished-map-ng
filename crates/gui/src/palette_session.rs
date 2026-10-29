//! master Main_Window::open_map imports into a shared palette session.
//! LGPL-3.0-only. Keep the previous map until all import decisions and previews succeed.
use super::*;

fn advance_import(
    plan: &mut polished_map_project::MapPalettePlan,
    palettes: &mut polished_map_project::PaletteDocument,
    load: bool,
    warnings: &mut Vec<String>,
) -> Result<(), polished_map_project::ProjectError> {
    match plan.advance(palettes, load) {
        Err(e) if plan.next_is_roof_colors() => {
            warnings.push(format!(
                "The map opened, but roof colors were not loaded:\n{e}"
            ));
            plan.advance(palettes, false)
        }
        result => result,
    }
}

impl Editor {
    pub(super) fn advance_open_palettes(&mut self, ctx: &egui::Context) {
        let Some(mut loaded) = self.pending_open.take() else {
            return;
        };
        if let Some(plan) = &mut loaded.palette_plan {
            let Some(tileset) = &mut loaded.tileset else {
                self.error = Some("Palette import requires a tileset".into());
                return;
            };
            while !plan.is_finished() {
                if plan.next_import_path().is_some() && tileset.palettes().is_modified() {
                    self.pending_open = Some(loaded);
                    return;
                }
                if let Err(e) = advance_import(
                    plan,
                    tileset.palettes_mut(),
                    true,
                    &mut self.pending_palette_warnings,
                ) {
                    self.error = Some(e.to_string());
                    self.status = "Open failed".into();
                    return;
                }
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
        if loaded.palette_plan.take().is_some() {
            loaded.graphics = loaded
                .tileset
                .as_ref()
                .ok_or("No tileset")?
                .graphics(
                    loaded
                        .blockset
                        .as_ref()
                        .ok_or("No block definitions")?
                        .blockset(),
                )
                .map_err(|e| e.to_string())?;
        }
        let images = (0..loaded.graphics.block_count())
            .map(|id| loaded.graphics.render_block(id as u8))
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
        self.roof_choices_available = loaded.tileset.as_ref().is_some_and(|tileset| {
            polished_map_project::list_roofs(tileset.root()).is_ok_and(|names| !names.is_empty())
        });
        self.event_ui.drag = None;
        self.event_ui.form = None;
        self.loaded = Some(loaded);
        self.textures = textures;
        self.selected = 0;
        self.block_clipboard = None;
        self.sidebar_drag = None;
        self.sidebar_press = None;
        self.canvas_pan.reset();
        self.sidebar_pan.reset();
        self.block_hotkeys = Default::default();
        self.sidebar_reveal = None;
        self.reset_sidebar_scroll = true;
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
        let mut load = false;
        let mut skip = false;
        egui::Modal::new(egui::Id::unique("open-map-palettes")).show(ctx, |ui| {
            ui.heading("Edited palette colors");
            ui.label(format!(
                "The palettes have been edited!\n\nLoad {name} anyway?"
            ));
            ui.label("Skip keeps these colors and continues opening the map.");
            ui.horizontal(|ui| {
                load = ui.button("Load").clicked();
                skip = ui.button("Skip").clicked();
            });
            skip |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        });
        if load || skip {
            let pending = self.pending_open.as_mut().expect("dialog has pending map");
            let result = advance_import(
                pending
                    .palette_plan
                    .as_mut()
                    .expect("dialog has palette plan"),
                pending
                    .tileset
                    .as_mut()
                    .expect("palette plan has tileset")
                    .palettes_mut(),
                !skip,
                &mut self.pending_palette_warnings,
            );
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
