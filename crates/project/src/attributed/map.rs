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

/// Find the nearest ancestor containing `main.asm` or `layout.link`.
pub fn find_project_root(map: &Path) -> Option<PathBuf> {
    Rules::PlusPlus.find_root(map)
}

/// Inspect metadata using attributed discovery precedence without loading assets.
///
/// # Errors
/// Unreadable or oversized metadata returns a project error.
pub fn discover_map(map: &Path, root: Option<&Path>) -> Result<MapMetadata, ProjectError> {
    discovery::discover_with_rules(map, root, Rules::PlusPlus)
}

/// Inventory attributed tilesets and resolve an optional filename hint.
///
/// # Errors
/// Returns directory/metadata I/O and format errors.
pub fn tileset_options(root: &Path, hint: Option<&str>) -> Result<TilesetOptions, ProjectError> {
    discovery::tileset_options_with_rules(root, hint, Rules::PlusPlus)
}

/// Return attributed tileset names and display labels in discovery order.
///
/// # Errors
/// Returns directory/metadata I/O and format errors.
pub fn tileset_choices(root: &Path) -> Result<Vec<TilesetChoice>, ProjectError> {
    Ok(tileset_options(root, None)?.choices().to_vec())
}

/// Return attributed tileset names in discovery order.
///
/// # Errors
/// Returns directory/metadata I/O and format errors.
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
    crate::file_io::prepare_target(&path)
}

/// Suggest dimension pairs for an `.ablk` or `.blk` file.
///
/// # Errors
/// Rejects other extensions, invalid preferred dimensions, or bounded-read failures.
pub fn inspect_map_sizes(
    path: &Path,
    preferred: Option<(u16, u16)>,
) -> Result<MapSizeOptions, ProjectError> {
    validate_map_path(path)?;
    crate::inspect_map_sizes(path, preferred)
}

#[derive(Clone, Debug)]
/// A map document bound to per-occurrence attribute storage.
///
/// This facade delegates byte-map editing and saves to [`crate::MapDocument`]
/// while retaining attributed path validation and discovery precedence. It
/// does not duplicate the editing engine or convert companion assets.
pub struct MapDocument {
    pub(super) inner: crate::MapDocument,
}

#[must_use]
#[derive(Debug)]
/// Loaded map prefix and the trailing byte count that the caller must surface.
pub struct MapOpenReport {
    /// Loaded dimension-sized prefix; do not adopt until the report is surfaced.
    pub document: MapDocument,
    /// Decoded block IDs beyond the selected width × height.
    pub trailing_blocks: usize,
}

impl MapDocument {
    pub(crate) fn from_inner(inner: crate::MapDocument) -> Self {
        Self { inner }
    }
    /// Create an unsaved zero-filled map using unified project discovery.
    ///
    /// # Errors
    /// Invalid roots or dimensions outside 1..=255 blocks are rejected.
    pub fn new_unified_map(root: &Path, width: u16, height: u16) -> Result<Self, ProjectError> {
        Ok(Self {
            inner: crate::MapDocument::new_unified_with_rules(
                root,
                width,
                height,
                Rules::PlusPlus,
            )?,
        })
    }
    /// Keep byte-map edits/history and require an explicit destination after a
    /// storage-model change. This does not convert tiles, palettes or attributes.
    pub fn with_classic_tileset(&self, name: &str) -> Result<crate::MapDocument, ProjectError> {
        self.inner.with_storage(Rules::Master, name)
    }
    /// Validate and update selected in-memory context without rewriting headers.
    ///
    /// # Errors
    /// Rejects an invalid asset name.
    pub fn select_tileset(&mut self, name: &str) -> Result<(), ProjectError> {
        self.inner.select_tileset(name)
    }
    /// Whether a storage-context change requires an explicit destination.
    pub fn needs_save_as(&self) -> bool {
        self.inner.needs_save_as()
    }

    /// Create an unsaved map with attributed discovery rules.
    ///
    /// # Errors
    /// Rejects an invalid root or dimensions outside 1..=255 blocks.
    pub fn new_map(root: &Path, width: u16, height: u16) -> Result<Self, ProjectError> {
        Ok(Self {
            inner: crate::MapDocument::new_with_rules(root, width, height, Rules::PlusPlus)?,
        })
    }
    /// Open an exact-length map with automatically discovered project context.
    ///
    /// # Errors
    /// I/O, decoding, unknown/invalid dimensions, and a decoded block count
    /// that differs from width × height cause failure.
    pub fn open(path: &Path, options: &OpenOptions) -> Result<Self, ProjectError> {
        Self::open_in_project(path, options, None)
    }
    /// Open an exact-length map with an optional explicit project root.
    ///
    /// Explicit dimensions override metadata. This preserves the fixed storage
    /// discovery rules; use the unified report method for combined source layouts.
    ///
    /// # Errors
    /// Invalid paths, reads, decoded bytes, dimensions, or length are rejected.
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
    /// Current backing path, or the placeholder path for an unsaved map.
    pub fn path(&self) -> &Path {
        self.inner.path()
    }
    /// Whether the document has an observed backing file.
    pub fn has_backing_file(&self) -> bool {
        self.inner.has_backing_file()
    }
    /// Read the validated block grid and edit/history state.
    pub fn map(&self) -> &Map {
        self.inner.map()
    }
    /// Edit the in-memory map through its validated command and resize APIs.
    pub fn map_mut(&mut self) -> &mut Map {
        self.inner.map_mut()
    }
    /// Discovered and selected context; it is not a live filesystem query.
    pub fn metadata(&self) -> &MapMetadata {
        self.inner.metadata()
    }
    /// Borrow document metadata and collect the serializable map status.
    pub fn summary(&self) -> crate::MapSummary<'_> {
        self.inner.summary()
    }
    /// SHA-256 of the last observed backing file, or `None` for an unsaved map.
    pub fn content_hash(&self) -> Option<String> {
        self.inner.content_hash()
    }
    /// Persist edits through the shared guarded map writer.
    ///
    /// # Errors
    /// An absent/new destination, ASM input, changed source hash, or failed
    /// filesystem operation prevents saving; dirty state is retained on failure.
    pub fn save(&mut self) -> Result<(), ProjectError> {
        self.inner.save()
    }
    /// Write binary blocks to an observed destination and then adopt its context.
    ///
    /// # Errors
    /// Rejects unsupported extensions, changed destination content, failed
    /// discovery, or I/O. Use [`prepare_save_as`] to obtain the target.
    pub fn save_as(&mut self, target: &SaveTarget) -> Result<(), ProjectError> {
        self.inner.save_as(target)
    }
}
