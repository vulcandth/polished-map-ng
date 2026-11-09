//! Prepare a complete document off-thread before committing any editor state.
//! Shared Open/New orchestration retains the pinned storage adapters' semantics.
use super::*;
use polished_map_core::plusplus::BankOrder;
use polished_map_project::{MapMetadata, ProjectError};

pub(super) struct LoadSettings {
    pub graphics: MasterGraphicsOptions,
    pub bank_order: BankOrder,
    pub auto_events: bool,
}

impl OpenForm {
    pub(super) fn dimensions(&self) -> OpenOptions {
        OpenOptions {
            width: (self.width != 0).then_some(self.width),
            height: (self.height != 0).then_some(self.height),
        }
    }

    pub(super) fn tileset_name<'a>(
        &'a self,
        metadata: &'a MapMetadata,
    ) -> Result<&'a str, ProjectError> {
        if self.tileset.is_empty() {
            metadata
                .tileset
                .as_deref()
                .ok_or_else(|| ProjectError::Format {
                    path: self.path.clone(),
                    message: "Choose a tileset".into(),
                })
        } else {
            Ok(&self.tileset)
        }
    }
}

impl Loaded {
    pub(super) fn open(
        form: OpenForm,
        palettes: documents::PaletteDocument,
        settings: LoadSettings,
    ) -> LoadResult {
        let roof_layout = Some(settings.graphics.roof_layout);
        let selected_tileset = form.unified.then(|| form.tileset.clone());
        let mut loaded = match (form.profile, palettes) {
            (documents::Profile::Master, documents::PaletteDocument::Master(palettes)) => {
                Self::open_master(form, palettes, settings.graphics, settings.auto_events)
                    .map_err(|e| e.to_string())
            }
            (documents::Profile::PlusPlus, documents::PaletteDocument::PlusPlus(palettes)) => {
                let options = settings.graphics;
                Self::open_plusplus(
                    form,
                    palettes,
                    polished_map_project::plusplus::PaletteOptions {
                        dark_theme: options.dark_theme,
                        special_palettes: options.special_palettes,
                        roof_colors: options.roof_colors,
                        roof_layout: options.roof_layout,
                        ..Default::default()
                    },
                    settings.bank_order,
                    settings.auto_events,
                )
            }
            _ => Err("Map storage and palette session do not match".into()),
        }?;
        // Open/New may explicitly correct missing or stale metadata. Commit the
        // same selected context as the assets, including unsaved new documents.
        if let Some(name) = selected_tileset.filter(|name| !name.is_empty()) {
            loaded
                .document
                .select_tileset(&name)
                .map_err(|e| e.to_string())?;
        }
        loaded.roof_layout = roof_layout;
        Ok(loaded)
    }

    fn open_master(
        form: OpenForm,
        palettes: polished_map_project::PaletteDocument,
        mut options: MasterGraphicsOptions,
        auto_events: bool,
    ) -> Result<Self, ProjectError> {
        let root = PathBuf::from(&form.root);
        let report = if form.new_map {
            polished_map_project::MapOpenReport {
                document: if form.unified {
                    MapDocument::new_unified_map(&root, form.width, form.height)?
                } else {
                    MapDocument::new_map(&root, form.width, form.height)?
                },
                trailing_blocks: 0,
            }
        } else {
            let open = if form.unified {
                MapDocument::open_unified_in_project_with_report
            } else {
                MapDocument::open_in_project_with_report
            };
            open(&form.path, &form.dimensions(), Some(&root))?
        };
        let name = form.tileset_name(report.document.metadata())?;
        options.allow_256 = form.allow_256;
        options.roof = (!form.roof.is_empty()).then(|| form.roof.clone());
        let begin = if form.unified {
            MasterTilesetDocument::begin_unified_for_map
        } else {
            MasterTilesetDocument::begin_for_map
        };
        let (tileset, plan) = begin(&report.document, name, &options, &palettes)?;
        let blocks = BlocksetDocument::open_in_project(&root, name)?;
        let graphics = tileset.graphics(blocks.blockset())?;
        let document = report.document.into();
        let (events, event_warning) =
            event_workflows::auto_load_events(&document, auto_events, form.unified);
        Ok(Self {
            roof_layout: None,
            recent_path: (!form.new_map).then_some(form.path),
            trailing_blocks: report.trailing_blocks,
            document,
            graphics: graphics.into(),
            events,
            event_warning,
            palette_plan: Some(documents::PalettePlan::Master(plan)),
            plus_assets: None,
            blockset: Some(blocks),
            tileset: Some(tileset),
            allow_256: form.allow_256,
        })
    }
}
