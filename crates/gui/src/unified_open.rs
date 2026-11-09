//! Automatic Open/New context, separate from the two immutable parity launchers.
use super::*;
use polished_map_project::{FormatDetectionError, MapMetadata, ProjectError, detect_map_format};
use std::path::Path;

impl From<polished_map_project::ProjectFormat> for documents::Profile {
    fn from(format: polished_map_project::ProjectFormat) -> Self {
        match format {
            polished_map_project::ProjectFormat::Master => Self::Master,
            polished_map_project::ProjectFormat::PlusPlus => Self::PlusPlus,
        }
    }
}

impl Editor {
    pub(super) fn open_metadata(&self, path: &Path) -> Result<MapMetadata, ProjectError> {
        if self.application.is_unified() {
            Ok(MapMetadata {
                root: polished_map_project::find_unified_project_root(path)?,
                ..Default::default()
            })
        } else {
            self.profile().discover_map(path, None)
        }
    }
}

impl OpenForm {
    pub(super) fn ready(&self) -> bool {
        self.root == self.choices_root
            && self.detection_error.is_none()
            && self
                .tilesets
                .iter()
                .any(|choice| choice.name == self.tileset)
    }

    pub(super) fn refresh_unified_choices(&mut self) -> Result<(), ProjectError> {
        let root = Path::new(&self.root);
        let choices = polished_map_project::unified_tileset_choices(root)?;
        let metadata = if self.new_map {
            None
        } else {
            match detect_map_format(&self.path, Some(root), None) {
                Ok(detected) => Some(detected.metadata().clone()),
                Err(FormatDetectionError::Project(error)) => return Err(error),
                Err(_) => Some(polished_map_project::discover_unified_map(
                    &self.path,
                    Some(root),
                )?),
            }
        };
        // Existing shared maps need an actual selection, not a guessed first
        // tileset. A new map may start with the first visible creation choice.
        self.tileset = metadata
            .as_ref()
            .and_then(|m| m.tileset.clone())
            .or_else(|| {
                self.new_map
                    .then(|| choices.first().map(|c| c.name.clone()))
                    .flatten()
            })
            .unwrap_or_default();
        self.tilesets = choices;
        self.roofs = polished_map_project::list_roofs(root)?;
        self.roof.clear();
        self.sizes = metadata
            .map(|m| polished_map_project::inspect_map_sizes(&self.path, m.dimensions))
            .transpose()?;
        if let Some(sizes) = &self.sizes {
            (self.width, self.height) = sizes.dimensions();
        }
        self.choices_root = self.root.clone();
        match polished_map_project::infer_roof_palette_layout(root) {
            Ok(Some(layout)) => {
                self.roof_layout = Some(layout);
                self.roof_layout_note =
                    "Roof palettes detected from the project table; override if needed.".into();
            }
            Ok(None) => {
                self.roof_layout_note =
                    "Roof palette layout is not unambiguous; check the selected setting.".into()
            }
            Err(error) => self.roof_layout_note = format!("Could not infer roof palettes: {error}"),
        }
        self.resolve_profile();
        Ok(())
    }

    pub(super) fn resolve_profile(&mut self) {
        let path = if self.new_map {
            self.path.with_extension("")
        } else {
            self.path.clone()
        };
        match detect_map_format(
            &path,
            Some(Path::new(&self.root)),
            (!self.tileset.is_empty()).then_some(self.tileset.as_str()),
        ) {
            Ok(detected) => {
                self.profile = detected.format().into();
                self.detected_context = match self.profile {
                    documents::Profile::PlusPlus => {
                        "Per-block attributes · 512-tile capacity (Polished Crystal style)"
                    }
                    documents::Profile::Master
                        if detected.evidence().iter().any(|e| {
                            e.kind == polished_map_project::FormatEvidenceKind::PaletteMap
                        }) =>
                    {
                        "Tile palette map · 256-tile capacity (pokecrystal style)"
                    }
                    documents::Profile::Master => {
                        "Monochrome tiles · 256-tile capacity (pokered style)"
                    }
                }
                .into();
                self.detection_error = if self.tileset.is_empty() {
                    Some("Choose a tileset for this shared map.".into())
                } else {
                    None
                };
            }
            Err(error) => {
                self.detected_context.clear();
                self.detection_error = Some(error.to_string());
            }
        }
    }
}
