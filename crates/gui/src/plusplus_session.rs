//! Pinned plusplus Main_Window asset/palette ownership and open/save sequencing.
//! LGPL-3.0-only. Keeps raw tiles, attribute blocks, roofs and palettes separate;
//! shared editor controls call the existing project documents for persistence.
use super::*;
use polished_map_core::plusplus::BankOrder;
use polished_map_project::{ProjectError, plusplus};

#[cfg(test)]
mod tests;

pub(super) struct PlusPlusAssets {
    pub blocks: plusplus::BlocksetDocument,
    pub tiles: plusplus::TilesetDocument,
    pub roof: Option<plusplus::RoofDocument>,
    pub palettes: plusplus::PaletteDocument,
    pub order: BankOrder,
    pub allow_512: bool,
}
impl PlusPlusAssets {
    pub fn graphics(&self, palettes: &plusplus::PaletteDocument) -> MapGraphics {
        let graphics = self.tiles.graphics(
            self.blocks.blockset(),
            palettes.colors(palettes.selected()),
            self.order,
        );
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
                    document: plusplus::MapDocument::new_map(&root, form.width, form.height)?,
                    trailing_blocks: 0,
                }
            } else {
                plusplus::MapDocument::open_in_project_with_report(
                    &form.path,
                    &OpenOptions {
                        width: (form.width != 0).then_some(form.width),
                        height: (form.height != 0).then_some(form.height),
                    },
                    Some(&root),
                )?
            };
            let name = if form.tileset.is_empty() {
                report
                    .document
                    .metadata()
                    .tileset
                    .as_deref()
                    .ok_or_else(|| ProjectError::Format {
                        path: form.path.clone(),
                        message: "Choose a tileset".into(),
                    })?
            } else {
                &form.tileset
            };
            let blocks = plusplus::BlocksetDocument::open_in_project(&root, name)?;
            let tiles =
                plusplus::TilesetDocument::open_in_project(&root, name, Default::default())?;
            let roof = (!form.roof.is_empty())
                .then(|| {
                    plusplus::RoofDocument::open_in_project(&root, &form.roof, Default::default())
                })
                .transpose()?;
            let allow_512 = form.allow_512
                || tiles.enables_512_tiles()
                || blocks.blockset().enables_512_tiles(order);
            let plan = plusplus::MapPalettePlan::for_map(&report.document, name, &options)?;
            let assets = PlusPlusAssets {
                blocks,
                tiles,
                roof,
                palettes,
                order,
                allow_512,
            };
            let graphics = assets.graphics(&assets.palettes);
            let document = report.document.into();
            let (events, event_warning) = event_workflows::auto_load_events(&document, auto_events);
            Ok(Self {
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
