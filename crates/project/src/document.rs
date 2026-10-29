use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use polished_map_core::{BlocksetError, Map, MapError, RasterError};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::codecs::{CodecError, parse_asm};
use crate::{MapMetadata, discover_map, find_project_root};

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {message}")]
    Format { path: PathBuf, message: String },
    #[error("{0} changed on disk since it was read; reload or preview again before saving")]
    Conflict(PathBuf),
    #[error(
        "paired save failed and could not fully roll back; recovery files are in {path}: {message}"
    )]
    RecoveryRequired { path: PathBuf, message: String },
    #[error("map dimensions are unknown; supply width/height or use Name.WxH.tileset.blk")]
    DimensionsRequired,
    #[error("this map has no backing file; use Save As")]
    DestinationRequired,
    #[error(transparent)]
    Map(#[from] MapError),
    #[error(transparent)]
    Raster(#[from] RasterError),
    #[error(transparent)]
    Blockset(#[from] BlocksetError),
    #[error(transparent)]
    Tileset(#[from] polished_map_core::TilesetError),
    #[error(transparent)]
    Palette(#[from] polished_map_core::PaletteError),
    #[error(transparent)]
    Event(#[from] polished_map_core::EventError),
}

impl ProjectError {
    pub(crate) fn io(path: &Path, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
    pub(crate) fn codec(path: &Path, error: CodecError) -> Self {
        Self::Format {
            path: path.into(),
            message: error.to_string(),
        }
    }
    pub fn code(&self) -> &'static str {
        match self {
            Self::Io { .. } => "io",
            Self::Format { .. } => "format",
            Self::Conflict(_) => "conflict",
            Self::RecoveryRequired { .. } => "recovery_required",
            Self::DimensionsRequired => "dimensions_required",
            Self::DestinationRequired => "destination_required",
            Self::Map(_) => "invalid_map",
            Self::Raster(_) => "invalid_graphics",
            Self::Blockset(_) => "invalid_blockset",
            Self::Tileset(_) => "invalid_tileset",
            Self::Palette(_) => "invalid_palette",
            Self::Event(_) => "invalid_event",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct OpenOptions {
    pub width: Option<u16>,
    pub height: Option<u16>,
}

#[derive(Debug)]
pub struct MapDocument {
    path: PathBuf,
    hash: Option<[u8; 32]>,
    map: Map,
    metadata: MapMetadata,
}

/// Result of the original editor's prefix-loading behavior. No bytes are written.
/// `trailing_blocks` counts decoded map IDs beyond the chosen dimensions, even
/// for ASM input. The document's conflict hash covers the entire source file.
#[must_use]
#[derive(Debug)]
pub struct MapOpenReport {
    pub document: MapDocument,
    pub trailing_blocks: usize,
}

/// A destination observed before an overwrite decision. Saving rechecks its content.
/// Construct with `prepare_save_as`; fields are private so callers cannot skip observation.
#[derive(Clone, Debug)]
pub struct SaveTarget {
    pub(crate) path: PathBuf,
    pub(crate) hash: Option<[u8; 32]>,
}

impl SaveTarget {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn exists(&self) -> bool {
        self.hash.is_some()
    }
    pub fn content_hash(&self) -> Option<String> {
        self.hash.map(hex_hash)
    }
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

pub(crate) fn prepare_target(path: &Path) -> Result<SaveTarget, ProjectError> {
    let filename = path.file_name().ok_or_else(|| ProjectError::Format {
        path: path.into(),
        message: "destination needs a file name".into(),
    })?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|e| ProjectError::io(parent, e))?;
    let path = parent.join(filename);
    let hash = optional_hash(&path)?;
    Ok(SaveTarget { path, hash })
}

pub(crate) fn optional_hash(path: &Path) -> Result<Option<[u8; 32]>, ProjectError> {
    // Graphics imports allow eight MiB; format parsers retain their own tighter
    // bounds. Observation must accept every file the corresponding parser does.
    match read_bounded(path, 8 * 1024 * 1024) {
        Ok(bytes) => Ok(Some(Sha256::digest(&bytes).into())),
        Err(ProjectError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn hex_hash(hash: [u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Debug, Serialize)]
pub struct MapSummary<'a> {
    pub path: Option<&'a Path>,
    pub width: u8,
    pub height: u8,
    pub block_count: usize,
    pub overworld_buffer_bytes: u32,
    pub modified: bool,
    pub content_hash: Option<String>,
    pub metadata: &'a MapMetadata,
}

impl MapDocument {
    /// Create an unsaved zero-filled map. This never creates or overwrites a file.
    pub fn new_map(root: &Path, width: u16, height: u16) -> Result<Self, ProjectError> {
        let map = Map::blank(width, height)?;
        let root = fs::canonicalize(root).map_err(|e| ProjectError::io(root, e))?;
        if !root.is_dir() {
            return Err(ProjectError::Format {
                path: root,
                message: "project root must be a directory".into(),
            });
        }
        Ok(Self {
            path: root.join("NewMap.blk"),
            hash: None,
            map,
            metadata: MapMetadata {
                root: Some(root),
                ..Default::default()
            },
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
        Ok(Self::read_in_project(path, options, root, false)?.document)
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
        Self::read_in_project(path, options, root, true)
    }

    fn read_in_project(
        path: &Path,
        options: &OpenOptions,
        root: Option<&Path>,
        allow_trailing: bool,
    ) -> Result<MapOpenReport, ProjectError> {
        let path = fs::canonicalize(path).map_err(|e| ProjectError::io(path, e))?;
        let bytes = read_bounded(&path, 4 * 1024 * 1024)?;
        let metadata = discover_map(&path, root)?;
        let dimensions = metadata.dimensions;
        let width = options
            .width
            .or(dimensions.map(|d| d.0))
            .ok_or(ProjectError::DimensionsRequired)?;
        let height = options
            .height
            .or(dimensions.map(|d| d.1))
            .ok_or(ProjectError::DimensionsRequired)?;
        let mut blocks = if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("map"))
        {
            let text = std::str::from_utf8(&bytes).map_err(|e| ProjectError::Format {
                path: path.clone(),
                message: e.to_string(),
            })?;
            parse_asm(text).map_err(|e| ProjectError::codec(&path, e))?
        } else {
            bytes.clone()
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
    pub fn has_backing_file(&self) -> bool {
        self.hash.is_some()
    }
    pub fn map(&self) -> &Map {
        &self.map
    }
    pub fn metadata(&self) -> &MapMetadata {
        &self.metadata
    }
    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.map
    }
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
    pub fn content_hash(&self) -> Option<String> {
        self.hash.map(hex_hash)
    }

    /// Save to a reviewed target, updating this document only after replacement succeeds.
    /// Existing targets require the caller's overwrite decision after preparation.
    pub fn save_as(&mut self, target: &SaveTarget) -> Result<(), ProjectError> {
        if !target
            .path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("blk"))
        {
            return Err(ProjectError::Format {
                path: target.path.clone(),
                message: "binary maps require a .blk destination".into(),
            });
        }
        persist_bytes(&target.path, target.hash, self.map.blocks())?;
        self.path.clone_from(&target.path);
        self.hash = Some(Sha256::digest(self.map.blocks()).into());
        // Keep loaded map attributes/tileset; moving to another project changes the root.
        self.metadata.root = find_project_root(&self.path).or_else(|| self.metadata.root.clone());
        self.map.mark_saved();
        Ok(())
    }

    /// Replace one file after a content conflict check. This is not a multi-file transaction.
    pub fn save(&mut self) -> Result<(), ProjectError> {
        let expected = self.hash.ok_or(ProjectError::DestinationRequired)?;
        // ASM import is allowed, but silently writing binary over its source is unsafe.
        if self
            .path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("map"))
        {
            return Err(ProjectError::Format {
                path: self.path.clone(),
                message: "ASM map import requires a .blk Save As destination".into(),
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

pub(crate) fn persist_bytes(
    path: &Path,
    expected: Option<[u8; 32]>,
    blocks: &[u8],
) -> Result<(), ProjectError> {
    if optional_hash(path)? != expected {
        return Err(ProjectError::Conflict(path.into()));
    }
    let parent = path.parent().ok_or_else(|| ProjectError::Format {
        path: path.into(),
        message: "no parent directory".into(),
    })?;
    let mut staged =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| ProjectError::io(parent, e))?;
    if expected.is_some() {
        staged
            .as_file()
            .set_permissions(
                fs::metadata(path)
                    .map_err(|e| ProjectError::io(path, e))?
                    .permissions(),
            )
            .map_err(|e| ProjectError::io(path, e))?;
    }
    staged
        .write_all(blocks)
        .map_err(|e| ProjectError::io(path, e))?;
    staged
        .as_file()
        .sync_all()
        .map_err(|e| ProjectError::io(path, e))?;
    // Recheck immediately before replacement; external writers do not participate in a lock protocol.
    if optional_hash(path)? != expected {
        return Err(ProjectError::Conflict(path.into()));
    }
    if expected.is_some() {
        staged
            .persist(path)
            .map_err(|e| ProjectError::io(path, e.error))?;
    } else {
        staged.persist_noclobber(path).map_err(|e| {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                ProjectError::Conflict(path.into())
            } else {
                ProjectError::io(path, e.error)
            }
        })?;
    }
    Ok(())
}

pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, ProjectError> {
    let file = fs::File::open(path).map_err(|e| ProjectError::io(path, e))?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| ProjectError::io(path, e))?;
    if bytes.len() > limit {
        return Err(ProjectError::Format {
            path: path.into(),
            message: format!("file exceeds {limit} bytes"),
        });
    }
    Ok(bytes)
}
