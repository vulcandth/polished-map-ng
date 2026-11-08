//! Asset-model changes preserve the byte map and event histories. Prepare and
//! render everything before changing active storage or palette caches.
use super::*;

impl Editor {
    pub(super) fn change_storage_tileset(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        profile: documents::Profile,
    ) -> Result<(), String> {
        let loaded = self.loaded.as_ref().ok_or("No map is loaded")?;
        let (root, _) = loaded.tileset_location().ok_or("No tileset is loaded")?;
        let root = root.to_owned();
        let roof = loaded.roof_name().map(str::to_owned);
        let document = loaded
            .document
            .with_storage(profile, name)
            .map_err(|e| e.to_string())?;
        let palettes = self.palettes_for_open(profile);
        // First use of this model imports project colors. Returning to a cached
        // model retains its full session, including edits, as Change Tileset does.
        let initialize_palettes = self.inactive_palettes.is_none();
        let options = MasterGraphicsOptions {
            monochrome: self.monochrome_preference(),
            allow_256: self.allow_256,
            allow_priority: self.allow_priority,
            roof,
            roof_layout: self.roof_palette_layout,
            dark_theme: self.dark,
            special_palettes: self.auto_special_palettes,
            roof_colors: self.auto_roof_colors,
            ..Default::default()
        };
        let mut warnings = Vec::new();
        let prepared = || -> Result<_, polished_map_project::ProjectError> {
            match (&document, palettes) {
                (
                    documents::MapDocument::Master(doc),
                    documents::PaletteDocument::Master(palettes),
                ) => {
                    let (mut tiles, mut plan) = MasterTilesetDocument::begin_unified_for_map(
                        doc, name, &options, &palettes,
                    )?;
                    if initialize_palettes {
                        while !plan.is_finished() {
                            palette_session::advance_step(
                                plan.next_is_roof_colors(),
                                true,
                                &mut warnings,
                                |load| plan.advance(tiles.palettes_mut(), load),
                            )?;
                        }
                    }
                    let blocks = BlocksetDocument::open_in_project(&root, name)?;
                    let graphics = tiles.graphics(blocks.blockset())?.into();
                    Ok((graphics, Some(tiles), Some(blocks), None))
                }
                (
                    documents::MapDocument::PlusPlus(doc),
                    documents::PaletteDocument::PlusPlus(mut palettes),
                ) => {
                    if initialize_palettes {
                        let mut plan =
                            polished_map_project::plusplus::MapPalettePlan::for_unified_map(
                                doc,
                                name,
                                &polished_map_project::plusplus::PaletteOptions {
                                    dark_theme: options.dark_theme,
                                    special_palettes: options.special_palettes,
                                    roof_colors: options.roof_colors,
                                    roof_layout: options.roof_layout,
                                    ..Default::default()
                                },
                            )?;
                        while !plan.is_finished() {
                            palette_session::advance_step(
                                plan.next_is_roof_colors(),
                                true,
                                &mut warnings,
                                |load| plan.advance(&mut palettes, load),
                            )?;
                        }
                    }
                    let assets = plusplus_session::PlusPlusAssets::open(
                        &root,
                        name,
                        options.roof.as_deref(),
                        palettes,
                        self.bank_order,
                        self.allow_512,
                        true,
                    )?;
                    Ok((assets.graphics(&assets.palettes), None, None, Some(assets)))
                }
                _ => Err(polished_map_project::ProjectError::Format {
                    path: root.clone(),
                    message: "Map and palette storage do not match".into(),
                }),
            }
        };
        let (graphics, tileset, blockset, plus_assets) = prepared().map_err(|e| e.to_string())?;
        let textures = documents::block_textures(ctx, &graphics)?;
        self.inactive_palettes = Some(self.palette_document().to_owned());
        if let Some(tiles) = &tileset {
            self.monochrome = tiles.monochrome();
            self.allow_priority = tiles.allow_priority();
        }
        if let Some(assets) = &plus_assets {
            self.allow_512 = assets.allow_512;
        }
        let loaded = self.loaded.as_mut().expect("validated active document");
        loaded.allow_256 = tileset.as_ref().is_some_and(|t| t.allow_256());
        loaded.document = document;
        loaded.graphics = graphics;
        loaded.tileset = tileset;
        loaded.blockset = blockset;
        loaded.plus_assets = plus_assets;
        loaded.palette_plan = None;
        self.textures = textures;
        self.help_window.profile = profile;
        self.block_clipboard = None;
        self.block_form = None;
        self.tileset_form = None;
        self.palette_form = None;
        self.sidebar_drag = None;
        self.sidebar_drag_press = None;
        self.sidebar_press = None;
        self.stroke.clear();
        if !warnings.is_empty() {
            self.open_palette_warning = Some(warnings.join("\n\n"));
        }
        Ok(())
    }
}
