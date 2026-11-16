//! Map discovery, decoding, edit-state ownership and save lifecycle.
//! Adapted from pinned map.cpp/config.cpp; LGPL-3.0-only.
//! Shared filesystem services live in `file_io`; source dialect rules in `discovery`.
use crate::{
    MapMetadata, ProjectError, SaveTarget,
    codecs::parse_asm,
    config::{Rules, discover_with_rules},
    file_io::{hex_hash, persist_bytes, prepare_target},
    read_bounded,
};
use polished_map_core::Map;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default)]
/// Optional dimensions in map blocks; missing axes are resolved from metadata.
pub struct OpenOptions {
    /// Explicit width, validated in 1..=255 blocks.
    pub width: Option<u16>,
    /// Explicit height, validated in 1..=255 blocks.
    pub height: Option<u16>,
}

#[derive(Clone, Debug)]
/// A byte map with discovery context and guarded persistence.
///
/// The model owns edits/history; this document owns its path and observed
/// source hash. Opening never writes. Use the report-based open methods only
/// when the caller can surface ignored trailing data before adopting the map.
///
/// ```no_run
/// use std::path::Path;
/// use polished_map_core::{Command, Point};
/// use polished_map_project::{MapDocument, OpenOptions, prepare_save_as};
/// let mut doc = MapDocument::open(Path::new("maps/Route1.10x9.blk"), &OpenOptions::default())?;
/// doc.map_mut().apply(&Command::Paint { points: vec![Point { x: 0, y: 0 }], block: 2 })?;
/// let target = prepare_save_as(Path::new("maps/Route1-copy.10x9.blk"))?;
/// // Existing destinations require the caller's overwrite decision.
/// if !target.exists() {
///     doc.save_as(&target)?;
/// }
/// # Ok::<(), polished_map_project::ProjectError>(())
/// ```
pub struct MapDocument {
    path: PathBuf,
    hash: Option<[u8; 32]>,
    map: Map,
    metadata: MapMetadata,
    rules: Rules,
    unified: bool,
    require_new_destination: bool,
}

/// Result of the original editor's prefix-loading behavior. No bytes are written.
/// `trailing_blocks` counts decoded map IDs beyond the chosen dimensions, even
/// for ASM input. The document's conflict hash covers the entire source file.
#[must_use]
#[derive(Debug)]
pub struct MapOpenReport {
    /// Loaded dimension-sized prefix; do not adopt until the report is surfaced.
    pub document: MapDocument,
    /// Decoded block IDs beyond the selected width × height.
    pub trailing_blocks: usize,
}

/// Prepare a binary-map destination; no filesystem writes occur here.
/// An omitted extension gets `.blk`, as in upstream Save As.
pub fn prepare_save_as(path: &Path) -> Result<SaveTarget, ProjectError> {
    let mut path = path.to_path_buf();
    if path.extension().is_none() {
        path.set_extension("blk");
    }
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("blk"))
    {
        return Err(ProjectError::Format {
            path,
            message: "choose a .blk destination for binary map data".into(),
        });
    }
    prepare_target(&path)
}

pub(crate) fn decode_map_blocks(path: &Path, bytes: &[u8]) -> Result<Vec<u8>, ProjectError> {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("map"))
    {
        let text = std::str::from_utf8(bytes).map_err(|e| ProjectError::Format {
            path: path.into(),
            message: e.to_string(),
        })?;
        parse_asm(text).map_err(|e| ProjectError::codec(path, e))
    } else {
        Ok(bytes.to_vec())
    }
}

#[derive(Debug, Serialize)]
/// Read-only map status for headless inspection and JSON output.
pub struct MapSummary<'a> {
    /// Backing file path, or `None` before a first save.
    pub path: Option<&'a Path>,
    /// Width in 32-pixel map blocks.
    pub width: u8,
    /// Height in 32-pixel map blocks.
    pub height: u8,
    /// Number of block IDs, equal to width × height.
    pub block_count: usize,
    /// Game map buffer size including its six-block padding on both axes.
    pub overworld_buffer_bytes: u32,
    /// Whether the model has unsaved changes.
    pub modified: bool,
    /// Last observed file SHA-256; this is not a hash of current in-memory edits.
    pub content_hash: Option<String>,
    /// Borrowed discovery and selected asset context.
    pub metadata: &'a MapMetadata,
}

struct ReadPolicy {
    rules: Rules,
    encoding: crate::MapEncoding,
    unified: bool,
}

impl MapDocument {
    /// Create an unsaved zero-filled map. This never creates or overwrites a file.
    pub fn new_map(root: &Path, width: u16, height: u16) -> Result<Self, ProjectError> {
        Self::new_with_rules(root, width, height, Rules::Master)
    }

    /// Create an unsaved zero-filled map using unified project discovery.
    ///
    /// # Errors
    /// Invalid roots or dimensions outside 1..=255 blocks are rejected.
    pub fn new_unified_map(root: &Path, width: u16, height: u16) -> Result<Self, ProjectError> {
        Self::new_unified_with_rules(root, width, height, Rules::Master)
    }

    pub(crate) fn new_unified_with_rules(
        root: &Path,
        width: u16,
        height: u16,
        rules: Rules,
    ) -> Result<Self, ProjectError> {
        let mut document = Self::new_with_rules(root, width, height, rules)?;
        document.unified = true;
        Ok(document)
    }

    /// Preserve byte-map history while changing the selected asset model.
    /// Companion assets must be loaded/validated before the caller adopts this candidate.
    pub fn with_plusplus_tileset(
        &self,
        name: &str,
    ) -> Result<crate::attributed::MapDocument, ProjectError> {
        Ok(crate::attributed::MapDocument::from_inner(
            self.with_storage(Rules::PlusPlus, name)?,
        ))
    }

    pub(crate) fn with_storage(&self, rules: Rules, name: &str) -> Result<Self, ProjectError> {
        crate::graphics::validate_tileset(self.path(), name)?;
        let mut document = self.clone();
        document.require_new_destination |= self.rules != rules && self.has_backing_file();
        document.rules = rules;
        document.unified = true;
        document.select_tileset(name)?;
        Ok(document)
    }

    /// Update in-memory selected context only; never rewrite map headers.
    pub fn select_tileset(&mut self, name: &str) -> Result<(), ProjectError> {
        crate::graphics::validate_tileset(self.path(), name)?;
        self.metadata.tileset = Some(name.into());
        Ok(())
    }

    /// Whether an unsaved map, imported ASM, or storage change needs a destination.
    pub fn needs_save_as(&self) -> bool {
        self.require_new_destination
            || !self.has_backing_file()
            || self
                .path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("map"))
    }

    pub(crate) fn new_with_rules(
        root: &Path,
        width: u16,
        height: u16,
        rules: Rules,
    ) -> Result<Self, ProjectError> {
        let map = Map::blank(width, height)?;
        let root = fs::canonicalize(root).map_err(|e| ProjectError::io(root, e))?;
        if !root.is_dir() {
            return Err(ProjectError::Format {
                path: root,
                message: "project root must be a directory".into(),
            });
        }
        Ok(Self {
            path: root.join(format!("NewMap.{}", rules.default_extension())),
            rules,
            unified: false,
            require_new_destination: false,
            hash: None,
            map,
            metadata: MapMetadata {
                root: Some(root),
                ..Default::default()
            },
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
        Ok(Self::read_in_project(path, options, root, false, Rules::Master)?.document)
    }

    /// Load the dimension-sized prefix and report extra decoded blocks, matching
    /// master Map::read_blocks/read_asm_blocks. Callers must surface the report
    /// before publishing the document. Short input and invalid dimensions fail.
    /// Use `open`/`open_in_project` when an exact-length input is required.
    pub fn open_in_project_with_report(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
    ) -> Result<MapOpenReport, ProjectError> {
        Self::read_in_project(path, options, root, true, Rules::Master)
    }

    pub(crate) fn read_in_project(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
        allow_trailing: bool,
        rules: Rules,
    ) -> Result<MapOpenReport, ProjectError> {
        let encoding = if rules == Rules::Master
            && path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("map"))
        {
            crate::MapEncoding::Assembly
        } else {
            crate::MapEncoding::Binary
        };
        Self::read_encoded(path, options, root, allow_trailing, rules, encoding)
    }

    pub(crate) fn read_encoded(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
        allow_trailing: bool,
        rules: Rules,
        encoding: crate::MapEncoding,
    ) -> Result<MapOpenReport, ProjectError> {
        Self::read_source(
            path,
            options,
            root,
            allow_trailing,
            ReadPolicy {
                rules,
                encoding,
                unified: false,
            },
        )
    }

    /// Unified metadata syntax with classic storage. Attribute selection remains
    /// the caller's validated context; this does not convert any companion files.
    pub fn open_unified_in_project_with_report(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
    ) -> Result<MapOpenReport, ProjectError> {
        Self::read_unified(path, options, root, Rules::Master)
    }

    pub(crate) fn read_unified(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
        rules: Rules,
    ) -> Result<MapOpenReport, ProjectError> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let encoding = if extension.eq_ignore_ascii_case("map") {
            crate::MapEncoding::Assembly
        } else if rules.binary_extension(extension) {
            crate::MapEncoding::Binary
        } else {
            return Err(ProjectError::Format {
                path: path.into(),
                message: "map extension does not match the selected storage".into(),
            });
        };
        Self::read_source(
            path,
            options,
            root,
            true,
            ReadPolicy {
                rules,
                encoding,
                unified: true,
            },
        )
    }

    fn read_source(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
        allow_trailing: bool,
        policy: ReadPolicy,
    ) -> Result<MapOpenReport, ProjectError> {
        let ReadPolicy {
            rules,
            encoding,
            unified,
        } = policy;
        let path = fs::canonicalize(path).map_err(|e| ProjectError::io(path, e))?;
        let bytes = read_bounded(&path, 4 * 1024 * 1024)?;
        let metadata = if unified {
            crate::discover_unified_map(&path, root)?
        } else {
            discover_with_rules(&path, root, rules)?
        };
        let dimensions = metadata.dimensions;
        let width = options
            .width
            .or(dimensions.map(|d| d.0))
            .ok_or(ProjectError::DimensionsRequired)?;
        let height = options
            .height
            .or(dimensions.map(|d| d.1))
            .ok_or(ProjectError::DimensionsRequired)?;
        let mut blocks = match encoding {
            crate::MapEncoding::Assembly => decode_map_blocks(&path, &bytes)?,
            crate::MapEncoding::Binary => bytes.clone(),
        };
        let mut trailing_blocks = 0;
        if allow_trailing && (1..=255).contains(&width) && (1..=255).contains(&height) {
            let expected = usize::from(width) * usize::from(height);
            trailing_blocks = blocks.len().saturating_sub(expected);
            blocks.truncate(expected);
        }
        let document = Self {
            hash: Some(Sha256::digest(&bytes).into()),
            map: Map::from_bytes(width, height, blocks)?,
            path,
            metadata,
            rules,
            unified,
            require_new_destination: false,
        };
        Ok(MapOpenReport {
            document,
            trailing_blocks,
        })
    }
    /// Backing path, or a suggested NewMap.blk path if `has_backing_file` is false.
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn rules(&self) -> Rules {
        self.rules
    }
    /// Whether a source hash was observed; this does not recheck file existence.
    pub fn has_backing_file(&self) -> bool {
        self.hash.is_some()
    }
    /// Read the validated block grid and edit/history state.
    pub fn map(&self) -> &Map {
        &self.map
    }
    /// Discovered and selected context; it is not a live filesystem query.
    pub fn metadata(&self) -> &MapMetadata {
        &self.metadata
    }
    /// Edit the in-memory map through its validated command and resize APIs.
    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.map
    }
    /// Borrow document metadata and collect the serializable map status.
    pub fn summary(&self) -> MapSummary<'_> {
        MapSummary {
            path: self.has_backing_file().then_some(&self.path),
            width: self.map.width(),
            height: self.map.height(),
            block_count: self.map.blocks().len(),
            overworld_buffer_bytes: self.map.overworld_buffer_bytes(),
            modified: self.map.is_modified(),
            content_hash: self.content_hash(),
            metadata: &self.metadata,
        }
    }
    /// SHA-256 of the last observed backing file, or `None` for an unsaved map.
    pub fn content_hash(&self) -> Option<String> {
        self.hash.map(hex_hash)
    }

    /// Save to a reviewed target, updating this document only after replacement succeeds.
    /// Existing targets require the caller's overwrite decision after preparation.
    pub fn save_as(&mut self, target: &SaveTarget) -> Result<(), ProjectError> {
        if !target
            .path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| self.rules.binary_extension(e))
        {
            return Err(ProjectError::Format {
                path: target.path.clone(),
                message: match self.rules {
                    Rules::Master => "binary maps require a .blk destination",
                    Rules::PlusPlus => "plusplus binary maps require a .ablk or .blk destination",
                }
                .into(),
            });
        }
        let root = if self.unified {
            crate::find_unified_project_root(&target.path)?
        } else {
            self.rules.find_root(&target.path)
        }
        .or_else(|| self.metadata.root.clone());
        persist_bytes(&target.path, target.hash, self.map.blocks())?;
        self.path.clone_from(&target.path);
        self.hash = Some(Sha256::digest(self.map.blocks()).into());
        self.require_new_destination = false;
        // Keep loaded map attributes/tileset; moving to another project changes the root.
        self.metadata.root = root;
        self.map.mark_saved();
        Ok(())
    }

    /// Replace one file after a content conflict check. This is not a multi-file transaction.
    pub fn save(&mut self) -> Result<(), ProjectError> {
        if self.require_new_destination {
            return Err(ProjectError::DestinationRequired);
        }
        let expected = self.hash.ok_or(ProjectError::DestinationRequired)?;
        // ASM import is allowed, but silently writing binary over its source is unsafe.
        if self
            .path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("map"))
        {
            return Err(ProjectError::Format {
                path: self.path.clone(),
                message: format!(
                    "ASM map import requires a .{} Save As destination",
                    self.rules.default_extension()
                ),
            });
        }
        let current = read_bounded(&self.path, 4 * 1024 * 1024)?;
        let hash: [u8; 32] = Sha256::digest(&current).into();
        if hash != expected {
            return Err(ProjectError::Conflict(self.path.clone()));
        }
        if !self.map.is_modified() {
            return Ok(());
        }
        persist_bytes(&self.path, Some(expected), self.map.blocks())?;
        self.hash = Some(Sha256::digest(self.map.blocks()).into());
        self.map.mark_saved();
        Ok(())
    }
}
