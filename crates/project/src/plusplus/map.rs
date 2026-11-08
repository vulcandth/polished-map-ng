//! Pinned plusplus config.cpp, option-dialogs.cpp and Map::{read,write}_blocks.
//! Byte-map editing/history and guarded persistence are shared with master;
//! discovery and allowed file types stay explicit. LGPL-3.0-only.
use crate::{
    MapMetadata, MapSizeOptions, OpenOptions, ProjectError, SaveTarget, TilesetChoice,
    TilesetOptions,
    discovery::{self, Rules},
};
use polished_map_core::Map;
use std::path::{Path, PathBuf};

pub fn find_project_root(map: &Path) -> Option<PathBuf> {
    Rules::PlusPlus.find_root(map)
}

pub fn discover_map(map: &Path, root: Option<&Path>) -> Result<MapMetadata, ProjectError> {
    discovery::discover_with_rules(map, root, Rules::PlusPlus)
}

pub fn tileset_options(root: &Path, hint: Option<&str>) -> Result<TilesetOptions, ProjectError> {
    discovery::tileset_options_with_rules(root, hint, Rules::PlusPlus)
}

pub fn tileset_choices(root: &Path) -> Result<Vec<TilesetChoice>, ProjectError> {
    Ok(tileset_options(root, None)?.choices().to_vec())
}

pub fn list_tilesets(root: &Path) -> Result<Vec<String>, ProjectError> {
    Ok(tileset_choices(root)?.into_iter().map(|c| c.name).collect())
}

fn validate_map_path(path: &Path) -> Result<(), ProjectError> {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| Rules::PlusPlus.binary_extension(e))
    {
        Ok(())
    } else {
        Err(ProjectError::Format {
            path: path.into(),
            message: "plusplus maps require .ablk or .blk files".into(),
        })
    }
}

/// No extension defaults to .ablk. No ASM or other format conversion is inferred.
pub fn prepare_save_as(path: &Path) -> Result<SaveTarget, ProjectError> {
    let path = if path.extension().is_none() {
        path.with_extension("ablk")
    } else {
        path.into()
    };
    validate_map_path(&path)?;
    crate::document::prepare_target(&path)
}

pub fn inspect_map_sizes(
    path: &Path,
    preferred: Option<(u16, u16)>,
) -> Result<MapSizeOptions, ProjectError> {
    validate_map_path(path)?;
    crate::inspect_map_sizes(path, preferred)
}

#[derive(Debug)]
pub struct MapDocument {
    pub(super) inner: crate::MapDocument,
}

#[must_use]
#[derive(Debug)]
pub struct MapOpenReport {
    pub document: MapDocument,
    pub trailing_blocks: usize,
}

impl MapDocument {
    pub fn new_map(root: &Path, width: u16, height: u16) -> Result<Self, ProjectError> {
        Ok(Self {
            inner: crate::MapDocument::new_with_rules(root, width, height, Rules::PlusPlus)?,
        })
    }
    pub fn open(path: &Path, options: &OpenOptions) -> Result<Self, ProjectError> {
        Self::open_in_project(path, options, None)
    }
    pub fn open_in_project(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
    ) -> Result<Self, ProjectError> {
        validate_map_path(path)?;
        Ok(Self {
            inner: crate::MapDocument::read_in_project(
                path,
                options,
                root,
                false,
                Rules::PlusPlus,
            )?
            .document,
        })
    }
    /// Surface extra bytes to the caller before adopting the dimension-sized prefix.
    pub fn open_in_project_with_report(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
    ) -> Result<MapOpenReport, ProjectError> {
        validate_map_path(path)?;
        let report =
            crate::MapDocument::read_in_project(path, options, root, true, Rules::PlusPlus)?;
        Ok(MapOpenReport {
            document: Self {
                inner: report.document,
            },
            trailing_blocks: report.trailing_blocks,
        })
    }
    /// Union metadata and ASM input with the occurrence-attribute storage model.
    pub fn open_unified_in_project_with_report(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
    ) -> Result<MapOpenReport, ProjectError> {
        let report = crate::MapDocument::read_unified(path, options, root, Rules::PlusPlus)?;
        Ok(MapOpenReport {
            document: Self {
                inner: report.document,
            },
            trailing_blocks: report.trailing_blocks,
        })
    }
    /// Unified source import using master's bounded ASM decoder with plusplus
    /// storage. Parity Open remains binary-only. In-place source saves fail;
    /// callers must choose an explicit binary Save As destination.
    pub fn import_asm_with_report(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
    ) -> Result<MapOpenReport, ProjectError> {
        if !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("map"))
        {
            return Err(ProjectError::Format {
                path: path.into(),
                message: "ASM map imports require a .map file".into(),
            });
        }
        let report = crate::MapDocument::read_encoded(
            path,
            options,
            root,
            true,
            Rules::PlusPlus,
            crate::MapEncoding::Assembly,
        )?;
        Ok(MapOpenReport {
            document: Self {
                inner: report.document,
            },
            trailing_blocks: report.trailing_blocks,
        })
    }
    pub fn path(&self) -> &Path {
        self.inner.path()
    }
    pub fn has_backing_file(&self) -> bool {
        self.inner.has_backing_file()
    }
    pub fn map(&self) -> &Map {
        self.inner.map()
    }
    pub fn map_mut(&mut self) -> &mut Map {
        self.inner.map_mut()
    }
    pub fn metadata(&self) -> &MapMetadata {
        self.inner.metadata()
    }
    pub fn summary(&self) -> crate::document::MapSummary<'_> {
        self.inner.summary()
    }
    pub fn content_hash(&self) -> Option<String> {
        self.inner.content_hash()
    }
    pub fn save(&mut self) -> Result<(), ProjectError> {
        self.inner.save()
    }
    pub fn save_as(&mut self, target: &SaveTarget) -> Result<(), ProjectError> {
        self.inner.save_as(target)
    }
}
