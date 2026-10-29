//! master Main_Window::open_map/load_palettes/load_roof_colors. LGPL-3.0-only.
//! Explicit session transitions let GUI and headless callers make the same import decisions.
use crate::{MapDocument, MasterGraphicsOptions, PaletteDocument, ProjectError, RoofPaletteLayout};
use polished_map_core::PaletteSet;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
enum Step {
    File {
        path: PathBuf,
        missing_ok: bool,
    },
    Select {
        automatic: PaletteSet,
        explicit: Option<PaletteSet>,
    },
    Roof {
        path: PathBuf,
        group: usize,
        layout: RoofPaletteLayout,
    },
}

/// Ordered default/special imports, contextual selection, then roof colors.
/// Inspect `next_import_path` before each step to request consent for edited colors.
/// Skipping an import preserves its edited flag; accepting a palette file clears
/// it even if the default file is absent. Roof overlays never clear that flag.
#[derive(Clone, Debug)]
pub struct MapPalettePlan {
    steps: VecDeque<Step>,
}

impl MapPalettePlan {
    pub fn for_map(
        doc: &MapDocument,
        tileset: &str,
        options: &MasterGraphicsOptions,
    ) -> Result<Self, ProjectError> {
        crate::graphics::validate_tileset(doc.path(), tileset)?;
        let metadata = doc.metadata();
        let root = metadata
            .root
            .as_deref()
            .ok_or_else(|| ProjectError::Format {
                path: doc.path().into(),
                message: "project root is required for palettes".into(),
            })?;
        let default_paths = [
            root.join("gfx/tilesets/bg_tiles.pal"),
            root.join("tilesets/bg.pal"),
        ];
        let path = default_paths
            .iter()
            .find(|p| p.is_file())
            .unwrap_or(&default_paths[1])
            .clone();
        let mut steps = VecDeque::from([Step::File {
            path,
            missing_ok: true,
        }]);
        if options.special_palettes {
            let gfx = root.join(if root.join("MAPDATA").is_dir() {
                "MAPDATA"
            } else {
                "gfx/tilesets"
            });
            let base = tileset.split('.').next().unwrap_or(tileset);
            let mut paths = Vec::new();
            if doc.has_backing_file() {
                paths.push(doc.path().with_extension("pal"));
            }
            if !metadata.landmark.is_empty() && !metadata.landmark.contains(['/', '\\']) {
                paths.push(gfx.join(format!("{}.pal", metadata.landmark)));
            }
            paths.extend([
                gfx.join(format!("{tileset}.pal")),
                gfx.join(format!("{base}.pal")),
            ]);
            if let Some(path) = paths.into_iter().find(|p| p.is_file()) {
                steps.push_back(Step::File {
                    path,
                    missing_ok: false,
                });
            }
        }
        let automatic = if matches!(
            metadata.palette.as_str(),
            "PALETTE_NITE" | "PALETTE_DARK" | "2" | "4"
        ) {
            PaletteSet::Nite
        } else if matches!(metadata.environment.as_str(), "INDOOR" | "GATE" | "3" | "6") {
            PaletteSet::Indoor
        } else if matches!(metadata.palette.as_str(), "PALETTE_MORN" | "3") {
            PaletteSet::Morn
        } else if options.dark_theme && !options.monochrome {
            PaletteSet::Nite
        } else {
            PaletteSet::Day
        };
        steps.push_back(Step::Select {
            automatic,
            explicit: options.palette,
        });
        if options.roof_colors
            && metadata.group != 0
            && !options.monochrome
            && matches!(metadata.environment.as_str(), "TOWN" | "ROUTE" | "1" | "2")
        {
            steps.push_back(Step::Roof {
                path: crate::roof_palette_path(root),
                group: metadata.group,
                layout: options.roof_layout,
            });
        }
        Ok(Self { steps })
    }
    pub fn is_finished(&self) -> bool {
        self.steps.is_empty()
    }
    /// GUI opens warn and continue when optional roof colors cannot be read.
    /// Other callers can keep the error or explicitly skip this step.
    pub fn next_is_roof_colors(&self) -> bool {
        matches!(self.steps.front(), Some(Step::Roof { .. }))
    }
    /// None indicates either contextual selection (no prompt) or the end of the plan.
    pub fn next_import_path(&self) -> Option<&Path> {
        match self.steps.front() {
            Some(Step::File { path, .. } | Step::Roof { path, .. }) => Some(path),
            _ => None,
        }
    }
    /// Advance one step. `load=false` skips only file/roof imports; automatic
    /// selection always runs. Failure changes neither the document nor the plan.
    pub fn advance(
        &mut self,
        palettes: &mut PaletteDocument,
        load: bool,
    ) -> Result<(), ProjectError> {
        let Some(step) = self.steps.front() else {
            return Ok(());
        };
        let mut draft = palettes.clone();
        match step {
            Step::File { path, missing_ok } if load => {
                if *missing_ok && !path.try_exists().map_err(|e| ProjectError::io(path, e))? {
                    draft.accept_missing_import();
                } else {
                    draft.load(path)?;
                }
            }
            Step::Select {
                automatic,
                explicit,
            } => {
                draft.select(explicit.unwrap_or_else(|| {
                    if draft.selected() == PaletteSet::Custom {
                        PaletteSet::Custom
                    } else {
                        *automatic
                    }
                }));
            }
            Step::Roof {
                path,
                group,
                layout,
            } if load => draft.load_roof_colors(path, *group, *layout)?,
            _ => {}
        }
        *palettes = draft;
        self.steps.pop_front();
        Ok(())
    }
}
