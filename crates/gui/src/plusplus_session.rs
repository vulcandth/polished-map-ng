//! Pinned plusplus Main_Window asset/palette ownership and open/save sequencing.
//! LGPL-3.0-only. Keeps raw tiles, attribute blocks, roofs and palettes separate;
//! shared editor controls call the existing project documents for persistence.
use super::*;
use polished_map_core::plusplus::BankOrder;
use polished_map_project::{ProjectError, plusplus};

#[cfg(test)]
mod switching_tests;
#[cfg(test)]
pub(crate) mod tests;

pub(super) struct PlusPlusAssets {
    pub blocks: plusplus::BlocksetDocument,
    pub tiles: plusplus::TilesetDocument,
    pub roof: Option<plusplus::RoofDocument>,
    pub palettes: plusplus::PaletteDocument,
    pub order: BankOrder,
    pub allow_512: bool,
    unified: bool,
}
impl PlusPlusAssets {
    pub fn open(
        root: &std::path::Path,
        name: &str,
        roof: Option<&str>,
        palettes: plusplus::PaletteDocument,
        order: BankOrder,
        allow_512: bool,
        unified: bool,
    ) -> Result<Self, ProjectError> {
        let open_blocks = if unified {
            plusplus::BlocksetDocument::open_unified_in_project
        } else {
            plusplus::BlocksetDocument::open_in_project
        };
        let open_tiles = if unified {
            plusplus::TilesetDocument::open_unified_in_project
        } else {
            plusplus::TilesetDocument::open_in_project
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
            plusplus::BlocksetDocument::open_unified_in_project
        } else {
            plusplus::BlocksetDocument::open_in_project
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
    pub fn open_roof(&self, name: &str) -> Result<plusplus::RoofDocument, ProjectError> {
        let open = if self.unified {
            plusplus::RoofDocument::open_unified_in_project
        } else {
            plusplus::RoofDocument::open_in_project
        };
        open(self.tiles.root(), name, self.tiles.png_platform())
    }
    pub fn graphics(&self, palettes: &plusplus::PaletteDocument) -> MapGraphics {
        self.graphics_for_blocks(self.blocks.blockset(), palettes)
    }
    pub fn graphics_for_blocks(
        &self,
        blocks: &polished_map_core::plusplus::AttributeBlockset,
        palettes: &plusplus::PaletteDocument,
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
        if let Some(a) = &self.plus_assets {
            Some((a.tiles.root(), a.tiles.name()))
        } else {
            self.tileset.as_ref().map(|t| (t.root(), t.name()))
        }
    }
    pub(super) fn roof_name(&self) -> Option<&str> {
        if let Some(a) = &self.plus_assets {
            a.roof.as_ref().map(|r| r.name())
        } else {
            self.tileset
                .as_ref()
                .and_then(|t| t.roof())
                .map(|r| r.name())
        }
    }
    pub(super) fn roof_modified(&self) -> bool {
        if let Some(a) = &self.plus_assets {
            a.roof.as_ref().is_some_and(|r| r.is_modified())
        } else {
            self.tileset
                .as_ref()
                .and_then(|t| t.roof())
                .is_some_and(|r| r.roof().tileset().is_modified())
        }
    }
    pub(super) fn palette_view(&self) -> Option<documents::PaletteRef<'_>> {
        if let Some(assets) = &self.plus_assets {
            Some(documents::PaletteRef::PlusPlus(&assets.palettes))
        } else {
            self.tileset
                .as_ref()
                .map(|t| documents::PaletteRef::Master(t.palettes()))
        }
    }
    pub(super) fn advance_palette(&mut self, load: bool) -> Result<(), ProjectError> {
        match (
            &mut self.palette_plan,
            &mut self.plus_assets,
            &mut self.tileset,
        ) {
            (Some(documents::PalettePlan::Master(plan)), None, Some(tiles)) => {
                plan.advance(tiles.palettes_mut(), load)
            }
            (Some(documents::PalettePlan::PlusPlus(plan)), Some(assets), None) => {
                plan.advance(&mut assets.palettes, load)
            }
            _ => Err(ProjectError::Format {
                path: self.document.path().into(),
                message: "Palette import and asset profiles do not match".into(),
            }),
        }
    }
    pub(super) fn refresh_graphics(&self) -> Result<MapGraphics, String> {
        if let Some(assets) = &self.plus_assets {
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
    pub(super) fn open_plusplus(
        form: OpenForm,
        palettes: plusplus::PaletteDocument,
        options: plusplus::PaletteOptions,
        order: BankOrder,
        auto_events: bool,
    ) -> LoadResult {
        let open = || -> Result<Self, ProjectError> {
            let root = PathBuf::from(&form.root);
            let report = if form.new_map {
                plusplus::MapOpenReport {
                    document: if form.unified {
                        plusplus::MapDocument::new_unified_map(&root, form.width, form.height)?
                    } else {
                        plusplus::MapDocument::new_map(&root, form.width, form.height)?
                    },
                    trailing_blocks: 0,
                }
            } else {
                let read = if form.unified {
                    plusplus::MapDocument::open_unified_in_project_with_report
                } else {
                    plusplus::MapDocument::open_in_project_with_report
                };
                read(&form.path, &form.dimensions(), Some(&root))?
            };
            let name = form.tileset_name(report.document.metadata())?;
            let palette_plan = if form.unified {
                plusplus::MapPalettePlan::for_unified_map
            } else {
                plusplus::MapPalettePlan::for_map
            };
            let plan = palette_plan(&report.document, name, &options)?;
            let assets = PlusPlusAssets::open(
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
                palette_plan: Some(documents::PalettePlan::PlusPlus(plan)),
                plus_assets: Some(assets),
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
        if self.profile() != documents::Profile::PlusPlus {
            return Err("Bank ordering requires plusplus".into());
        }
        if let Some(loaded) = &mut self.loaded {
            let assets = loaded.plus_assets.as_mut().ok_or("No plusplus assets")?;
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
    pub(super) fn save_plusplus_tiles(&mut self) -> bool {
        let Some(assets) = self.loaded.as_mut().and_then(|l| l.plus_assets.as_mut()) else {
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
    pub(super) fn save_plusplus_roof(&mut self) -> bool {
        let Some(assets) = self.loaded.as_mut().and_then(|l| l.plus_assets.as_mut()) else {
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
    pub(super) fn save_plusplus_blocks(&mut self) -> bool {
        let Some(assets) = self.loaded.as_mut().and_then(|l| l.plus_assets.as_mut()) else {
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
