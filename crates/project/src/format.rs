//! Automatic project context for the unified editor.
//!
//! Asset evidence follows both pinned config.cpp implementations (see
//! upstream.lock.toml). Root markers locate a project, not its format: both
//! project families can contain Makefile and main.asm. A .blk is also shared.
//! Detection never writes or falls back to another parser after a load error.
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

use crate::{
    MapMetadata, ProjectError,
    asset_paths::{AssetKind, is_file},
    discovery::{Rules, discover_with_rules},
};

/// The on-disk attribute model, independent of application identity/preferences.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectFormat {
    Master,
    PlusPlus,
}

/// Where the project stores palette, flip and priority information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeStorage {
    TilePaletteMap,
    BlockOccurrences,
}

/// Map byte decoding is independent of tileset attribute storage. In particular,
/// master's ASM import is useful in the unified application with either model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MapEncoding {
    Binary,
    Assembly,
}

impl ProjectFormat {
    pub fn attribute_storage(self) -> AttributeStorage {
        match self {
            Self::Master => AttributeStorage::TilePaletteMap,
            Self::PlusPlus => AttributeStorage::BlockOccurrences,
        }
    }

    /// Physical tile capacity; editing limits/settings can be narrower.
    pub fn tile_capacity(self) -> usize {
        match self {
            Self::Master => 256,
            Self::PlusPlus => 512,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatEvidenceKind {
    MapExtension,
    PaletteMap,
    OccurrenceAttributes,
    LegacyBlockset,
    /// Binary block definitions without an occurrence-attribute companion are
    /// also valid master monochrome data; a palette map is not required.
    UnattributedBlocks,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FormatEvidence {
    pub format: ProjectFormat,
    pub kind: FormatEvidenceKind,
    pub path: PathBuf,
}

/// Detection is context, not validation of the assets' contents. The chosen
/// loader must still validate all data, and must not retry another format.
#[derive(Debug)]
pub struct DetectedFormat {
    format: ProjectFormat,
    encoding: MapEncoding,
    metadata: MapMetadata,
    evidence: Vec<FormatEvidence>,
}

impl DetectedFormat {
    pub fn format(&self) -> ProjectFormat {
        self.format
    }
    pub fn encoding(&self) -> MapEncoding {
        self.encoding
    }
    pub fn metadata(&self) -> &MapMetadata {
        &self.metadata
    }
    pub fn evidence(&self) -> &[FormatEvidence] {
        &self.evidence
    }
}

#[derive(Debug, Error)]
pub enum FormatDetectionError {
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error("{path}: choose a project folder and tileset to identify this map's attribute storage")]
    ContextRequired { path: PathBuf },
    #[error(
        "{path}: metadata identifies different tilesets ({master} and {plusplus}); choose the intended tileset"
    )]
    ConflictingTilesets {
        path: PathBuf,
        master: String,
        plusplus: String,
    },
    #[error(
        "{path}: conflicting master and plusplus asset evidence; check the project folder and selected tileset"
    )]
    ConflictingEvidence {
        path: PathBuf,
        evidence: Vec<FormatEvidence>,
    },
}

/// Nearest ancestor containing any supported root marker. An explicit root in
/// `detect_map_format` bypasses this search. IO failures are never treated as
/// evidence that an ancestor or asset does not exist.
pub fn find_unified_project_root(map: &Path) -> Result<Option<PathBuf>, ProjectError> {
    let absolute = if map.is_absolute() {
        map.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| ProjectError::io(map, e))?
            .join(map)
    };
    for ancestor in absolute.ancestors().skip(1) {
        for marker in ["Makefile", "makefile", "main.asm", "layout.link"] {
            if is_file(&ancestor.join(marker))? {
                return Ok(Some(ancestor.into()));
            }
        }
    }
    Ok(None)
}

/// Inventory both graphics layouts without choosing storage from the first item.
pub fn unified_tileset_choices(root: &Path) -> Result<Vec<crate::TilesetChoice>, ProjectError> {
    let mut choices = crate::tileset_choices(root)?;
    for choice in crate::plusplus::tileset_choices(root)? {
        if !choices.iter().any(|existing| existing.name == choice.name) {
            choices.push(choice);
        }
    }
    Ok(choices)
}

/// Determine the storage model using the map's actual metadata or an explicitly
/// selected *tileset*, never a global mode or the first directory-list entry.
/// Missing/broken attributes do not trigger a fallback after choosing plusplus.
pub fn detect_map_format(
    map: &Path,
    root: Option<&Path>,
    tileset: Option<&str>,
) -> Result<DetectedFormat, FormatDetectionError> {
    let extension = map
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let extension_format = match extension.as_str() {
        "ablk" => Some(ProjectFormat::PlusPlus),
        "map" | "blk" | "" => None,
        _ => {
            return Err(ProjectError::Format {
                path: map.into(),
                message: "maps require .blk, .ablk or .map input".into(),
            }
            .into());
        }
    };
    let root = match root {
        Some(root) => Some(root.to_path_buf()),
        None => find_unified_project_root(map)?,
    };
    let master = discover_with_rules(map, root.as_deref(), Rules::Master)?;
    let plusplus = discover_with_rules(map, root.as_deref(), Rules::PlusPlus)?;
    let selected = if let Some(name) = tileset {
        Some(name.to_owned())
    } else {
        match (&master.tileset, &plusplus.tileset) {
            (Some(a), Some(b)) if a != b => {
                return Err(FormatDetectionError::ConflictingTilesets {
                    path: map.into(),
                    master: a.clone(),
                    plusplus: b.clone(),
                });
            }
            (a, b) => a.as_ref().or(b.as_ref()).cloned(),
        }
    };
    let mut evidence = Vec::new();
    if let Some(format) = extension_format {
        evidence.push(FormatEvidence {
            format,
            kind: FormatEvidenceKind::MapExtension,
            path: map.into(),
        });
    }
    if let (Some(root), Some(tileset)) = (root.as_deref(), selected.as_deref()) {
        collect_asset_evidence(root, tileset, &mut evidence)?;
    }
    let format = evidence
        .first()
        .map(|e| e.format)
        .ok_or_else(|| FormatDetectionError::ContextRequired { path: map.into() })?;
    if evidence.iter().any(|e| e.format != format) {
        return Err(FormatDetectionError::ConflictingEvidence {
            path: map.into(),
            evidence,
        });
    }
    let mut metadata = match format {
        ProjectFormat::Master => master,
        ProjectFormat::PlusPlus => plusplus,
    };
    if selected.is_some() {
        metadata.tileset = selected;
    }
    Ok(DetectedFormat {
        format,
        encoding: if extension == "map" {
            MapEncoding::Assembly
        } else {
            MapEncoding::Binary
        },
        metadata,
        evidence,
    })
}

fn collect_asset_evidence(
    root: &Path,
    tileset: &str,
    evidence: &mut Vec<FormatEvidence>,
) -> Result<(), ProjectError> {
    let attributes = AssetKind::Attributes.existing(root, tileset)?;
    let blocks = AssetKind::MasterBlocks.existing(root, tileset)?;
    for (path, format, kind) in [
        (
            attributes.clone(),
            ProjectFormat::PlusPlus,
            FormatEvidenceKind::OccurrenceAttributes,
        ),
        (
            AssetKind::PaletteMap.existing(root, tileset)?,
            ProjectFormat::Master,
            FormatEvidenceKind::PaletteMap,
        ),
    ] {
        if let Some(path) = path {
            evidence.push(FormatEvidence { format, kind, path });
        }
    }
    if let Some(path) = blocks {
        let legacy = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("bst") || ext.eq_ignore_ascii_case("cel"));
        if legacy || attributes.is_none() {
            evidence.push(FormatEvidence {
                format: ProjectFormat::Master,
                kind: if legacy {
                    FormatEvidenceKind::LegacyBlockset
                } else {
                    FormatEvidenceKind::UnattributedBlocks
                },
                path,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
