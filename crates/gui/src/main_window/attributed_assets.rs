//! Pinned plusplus Main_Window asset/palette ownership and open/save sequencing.
//! LGPL-3.0-only. Keeps raw tiles, attribute blocks, roofs and palettes separate;
//! shared editor controls call the existing project documents for persistence.
use super::*;
use polished_map_core::attributed::BankOrder;
use polished_map_project::{ProjectError, attributed};

#[cfg(test)]
mod switching_tests;
#[cfg(test)]
pub(crate) mod tests;

pub(super) struct AttributedAssets {
    pub blocks: attributed::BlocksetDocument,
    pub tiles: attributed::TilesetDocument,
    pub roof: Option<attributed::RoofDocument>,
    pub palettes: attributed::PaletteDocument,
    pub order: BankOrder,
    pub allow_512: bool,
    unified: bool,
}
impl AttributedAssets {
    pub fn open(
        root: &std::path::Path,
        name: &str,
        roof: Option<&str>,
        palettes: attributed::PaletteDocument,
        order: BankOrder,
        allow_512: bool,
        unified: bool,
    ) -> Result<Self, ProjectError> {
        let open_blocks = if unified {
            attributed::BlocksetDocument::open_unified_in_project
        } else {
            attributed::BlocksetDocument::open_in_project
        };
        let open_tiles = if unified {
            attributed::TilesetDocument::open_unified_in_project
        } else {
            attributed::TilesetDocument::open_in_project
        };
        let blocks = open_blocks(root, name)?;
        let tiles = open_tiles(root, name, Default::default())?;
        let mut assets = Self {
            allow_512: allow_512
                || tiles.enables_512_tiles()
                || blocks.blockset().enables_512_tiles(order),
            blocks,
            tiles,
            roof: None,
            palettes,
            order,
            unified,
        };
        assets.roof = roof.map(|name| assets.open_roof(name)).transpose()?;
        Ok(assets)
    }

    pub fn replacement(
        &self,
        name: &str,
        allow_512: bool,
        order: BankOrder,
    ) -> Result<Self, ProjectError> {
        let root = self.tiles.root();
        let tiles = self.tiles.replacement(name)?;
        let open_blocks = if self.unified {
            attributed::BlocksetDocument::open_unified_in_project
        } else {
            attributed::BlocksetDocument::open_in_project
        };
        let blocks = open_blocks(root, name)?;
        let roof = self
            .roof
            .as_ref()
            .map(|r| self.open_roof(r.name()))
            .transpose()?;
        Ok(Self {
            allow_512: allow_512
                || tiles.enables_512_tiles()
                || blocks.blockset().enables_512_tiles(order),
            tiles,
            blocks,
            roof,
            order,
            palettes: self.palettes.clone(),
            unified: self.unified,
        })
    }
    pub fn open_roof(&self, name: &str) -> Result<attributed::RoofDocument, ProjectError> {
        let open = if self.unified {
            attributed::RoofDocument::open_unified_in_project
        } else {
            attributed::RoofDocument::open_in_project
        };
        open(self.tiles.root(), name, self.tiles.png_platform())
    }
    pub fn graphics(&self, palettes: &attributed::PaletteDocument) -> MapGraphics {
        self.graphics_for_blocks(self.blocks.blockset(), palettes)
    }
    pub fn graphics_for_blocks(
        &self,
        blocks: &polished_map_core::attributed::AttributeBlockset,
        palettes: &attributed::PaletteDocument,
    ) -> MapGraphics {
        let graphics =
            self.tiles
                .graphics(blocks, palettes.colors(palettes.selected()), self.order);
        match &self.roof {
            Some(roof) => graphics.with_roof(roof.tiles()).into(),
            None => graphics.into(),
        }
    }
    pub fn is_modified(&self) -> bool {
        self.blocks.blockset().is_modified()
            || self.tiles.model().is_modified()
            || self.roof.as_ref().is_some_and(|r| r.is_modified())
    }
}

impl Loaded {
    pub(super) fn tileset_location(&self) -> Option<(&std::path::Path, &str)> {
        if let Some(a) = &self.attributed_assets {
            Some((a.tiles.root(), a.tiles.name()))
        } else {
            self.tileset.as_ref().map(|t| (t.root(), t.name()))
        }
    }
    pub(super) fn roof_name(&self) -> Option<&str> {
        if let Some(a) = &self.attributed_assets {
            a.roof.as_ref().map(|r| r.name())
        } else {
            self.tileset
                .as_ref()
                .and_then(|t| t.roof())
                .map(|r| r.name())
        }
    }
    pub(super) fn roof_modified(&self) -> bool {
        if let Some(a) = &self.attributed_assets {
            a.roof.as_ref().is_some_and(|r| r.is_modified())
        } else {
            self.tileset
                .as_ref()
                .and_then(|t| t.roof())
                .is_some_and(|r| r.roof().tileset().is_modified())
        }
    }
    pub(super) fn palette_view(&self) -> Option<documents::PaletteRef<'_>> {
        if let Some(assets) = &self.attributed_assets {
            Some(documents::PaletteRef::Attributed(&assets.palettes))
        } else {
            self.tileset
                .as_ref()
                .map(|t| documents::PaletteRef::Classic(t.palettes()))
        }
    }
    pub(super) fn advance_palette(&mut self, load: bool) -> Result<(), ProjectError> {
        match (
            &mut self.palette_plan,
            &mut self.attributed_assets,
            &mut self.tileset,
        ) {
            (Some(documents::PalettePlan::Classic(plan)), None, Some(tiles)) => {
                plan.advance(tiles.palettes_mut(), load)
            }
            (Some(documents::PalettePlan::Attributed(plan)), Some(assets), None) => {
                plan.advance(&mut assets.palettes, load)
            }
            _ => Err(ProjectError::Format {
                path: self.document.path().into(),
                message: "Palette import and asset profiles do not match".into(),
            }),
        }
    }
    pub(super) fn refresh_graphics(&self) -> Result<MapGraphics, String> {
        if let Some(assets) = &self.attributed_assets {
            Ok(assets.graphics(&assets.palettes))
        } else {
            self.tileset
                .as_ref()
                .ok_or("No tileset")?
                .graphics(
                    self.blockset
                        .as_ref()
                        .ok_or("No block definitions")?
                        .blockset(),
                )
                .map(Into::into)
                .map_err(|e| e.to_string())
        }
    }
    pub(super) fn open_attributed(
        form: OpenForm,
        palettes: attributed::PaletteDocument,
        options: attributed::PaletteOptions,
        order: BankOrder,
        auto_events: bool,
    ) -> LoadResult {
        let open = || -> Result<Self, ProjectError> {
            let root = PathBuf::from(&form.root);
            let report = if form.new_map {
                attributed::MapOpenReport {
                    document: if form.unified {
                        attributed::MapDocument::new_unified_map(&root, form.width, form.height)?
                    } else {
                        attributed::MapDocument::new_map(&root, form.width, form.height)?
                    },
                    trailing_blocks: 0,
                }
            } else {
                let read = if form.unified {
                    attributed::MapDocument::open_unified_in_project_with_report
                } else {
                    attributed::MapDocument::open_in_project_with_report
                };
                read(&form.path, &form.dimensions(), Some(&root))?
            };
            let name = form.tileset_name(report.document.metadata())?;
            let palette_plan = if form.unified {
                attributed::MapPalettePlan::for_unified_map
            } else {
                attributed::MapPalettePlan::for_map
            };
            let plan = palette_plan(&report.document, name, &options)?;
            let assets = AttributedAssets::open(
                &root,
                name,
                (!form.roof.is_empty()).then_some(form.roof.as_str()),
                palettes,
                order,
                form.allow_512,
                form.unified,
            )?;
            let graphics = assets.graphics(&assets.palettes);
            let document = report.document.into();
            let (events, event_warning) =
                event_workflows::auto_load_events(&document, auto_events, form.unified);
            Ok(Self {
                roof_layout: None,
                recent_path: (!form.new_map).then_some(form.path),
                trailing_blocks: report.trailing_blocks,
                document,
                graphics,
                events,
                event_warning,
                palette_plan: Some(documents::PalettePlan::Attributed(plan)),
                attributed_assets: Some(assets),
                blockset: None,
                tileset: None,
                allow_256: false,
            })
        };
        open().map_err(|e| e.to_string())
    }
}

impl Editor {
    pub(super) fn change_bank_order(
        &mut self,
        ctx: &egui::Context,
        order: BankOrder,
    ) -> Result<(), String> {
        if self.storage() != documents::Storage::Attributed {
            return Err("Bank ordering requires plusplus".into());
        }
        if let Some(loaded) = &mut self.loaded {
            let assets = loaded
                .attributed_assets
                .as_mut()
                .ok_or("No plusplus assets")?;
            let graphics = assets.tiles.graphics(
                assets.blocks.blockset(),
                assets.palettes.colors(assets.palettes.selected()),
                order,
            );
            let graphics: MapGraphics = match &assets.roof {
                Some(r) => graphics.with_roof(r.tiles()).into(),
                None => graphics.into(),
            };
            let textures = documents::block_textures(ctx, &graphics)?;
            assets.order = order;
            loaded.graphics = graphics;
            self.textures = textures;
        }
        self.bank_order = order;
        Ok(())
    }
    pub(super) fn save_attributed_tiles(&mut self) -> bool {
        let Some(assets) = self
            .loaded
            .as_mut()
            .and_then(|l| l.attributed_assets.as_mut())
        else {
            return false;
        };
        let modified = assets.tiles.model().is_modified();
        if !modified {
            assets
                .tiles
                .trim_unused(assets.blocks.blockset(), assets.order);
        }
        let name = if modified {
            format!("{}.png", assets.tiles.name())
        } else {
            assets.tiles.name().to_owned()
        };
        let result = if modified {
            assets
                .blocks
                .verify_unchanged()
                .and_then(|()| assets.tiles.save(assets.blocks.blockset(), assets.order))
        } else {
            Ok(())
        };
        match result {
            Ok(()) => {
                self.status = "Tileset saved".into();
                self.saved_message(name);
            }
            Err(e) => self.error = Some(e.to_string()),
        }
        true
    }
    pub(super) fn save_attributed_roof(&mut self) -> bool {
        let Some(assets) = self
            .loaded
            .as_mut()
            .and_then(|l| l.attributed_assets.as_mut())
        else {
            return false;
        };
        let Some(roof) = assets.roof.as_mut() else {
            return true;
        };
        let modified = roof.is_modified();
        let name = if modified {
            save_feedback::basename(roof.output().path())
        } else {
            roof.name().to_owned()
        };
        let result = if modified { roof.save() } else { Ok(()) };
        match result {
            Ok(()) => {
                self.status = "Roof saved".into();
                self.saved_message(name);
            }
            Err(e) => self.error = Some(e.to_string()),
        }
        true
    }
    pub(super) fn save_attributed_blocks(&mut self) -> bool {
        let Some(assets) = self
            .loaded
            .as_mut()
            .and_then(|l| l.attributed_assets.as_mut())
        else {
            return false;
        };
        let doc = &mut assets.blocks;
        let name = save_feedback::basename(doc.path());
        let attributes = save_feedback::basename(doc.attribute_path());
        let message = if let Some(collision) = doc.collision_path() {
            format!(
                "{name}, {attributes},\nand {}",
                save_feedback::basename(collision)
            )
        } else {
            format!("{name}\nand {attributes}")
        };
        let result = if doc.blockset().is_modified() {
            doc.save()
        } else {
            Ok(())
        };
        match result {
            Ok(()) => {
                self.status = "Blockset saved".into();
                self.saved_message(message);
            }
            Err(e) => self.error = Some(e.to_string()),
        }
        true
    }
}
