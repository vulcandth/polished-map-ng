use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use polished_map_core::{Map, MapError, RasterError};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::codecs::{CodecError, parse_asm};

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
    #[error("{0} changed on disk since it was opened; reload before saving")]
    Conflict(PathBuf),
    #[error("map dimensions are unknown; supply width/height or use Name.WxH.tileset.blk")]
    DimensionsRequired,
    #[error(transparent)]
    Map(#[from] MapError),
    #[error(transparent)]
    Raster(#[from] RasterError),
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
            Self::DimensionsRequired => "dimensions_required",
            Self::Map(_) => "invalid_map",
            Self::Raster(_) => "invalid_graphics",
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
    hash: [u8; 32],
    map: Map,
}

#[derive(Debug, Serialize)]
pub struct MapSummary<'a> {
    pub path: &'a Path,
    pub width: u8,
    pub height: u8,
    pub block_count: usize,
    pub modified: bool,
    pub content_hash: String,
}

impl MapDocument {
    pub fn open(path: &Path, options: &OpenOptions) -> Result<Self, ProjectError> {
        let path = fs::canonicalize(path).map_err(|e| ProjectError::io(path, e))?;
        let bytes = read_bounded(&path, 4 * 1024 * 1024)?;
        let dimensions = filename_dimensions(&path);
        let width = options
            .width
            .or(dimensions.map(|d| d.0))
            .ok_or(ProjectError::DimensionsRequired)?;
        let height = options
            .height
            .or(dimensions.map(|d| d.1))
            .ok_or(ProjectError::DimensionsRequired)?;
        let blocks = if path
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
        Ok(Self {
            hash: Sha256::digest(&bytes).into(),
            map: Map::from_bytes(width, height, blocks)?,
            path,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn map(&self) -> &Map {
        &self.map
    }
    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.map
    }
    pub fn summary(&self) -> MapSummary<'_> {
        MapSummary {
            path: &self.path,
            width: self.map.width(),
            height: self.map.height(),
            block_count: self.map.blocks().len(),
            modified: self.map.is_modified(),
            content_hash: self.content_hash(),
        }
    }
    pub fn content_hash(&self) -> String {
        self.hash.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Replace one file after a content conflict check. This is not a multi-file transaction.
    pub fn save(&mut self) -> Result<(), ProjectError> {
        // ASM import is allowed, but silently writing binary over its source is unsafe.
        if self
            .path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("map"))
        {
            return Err(ProjectError::Format {
                path: self.path.clone(),
                message: "ASM map import requires a .blk Save As destination (not implemented yet)"
                    .into(),
            });
        }
        let current = read_bounded(&self.path, 4 * 1024 * 1024)?;
        let hash: [u8; 32] = Sha256::digest(&current).into();
        if hash != self.hash {
            return Err(ProjectError::Conflict(self.path.clone()));
        }
        if !self.map.is_modified() {
            return Ok(());
        }
        let parent = self.path.parent().ok_or_else(|| ProjectError::Format {
            path: self.path.clone(),
            message: "no parent directory".into(),
        })?;
        let mut staged =
            tempfile::NamedTempFile::new_in(parent).map_err(|e| ProjectError::io(parent, e))?;
        staged
            .as_file()
            .set_permissions(
                fs::metadata(&self.path)
                    .map_err(|e| ProjectError::io(&self.path, e))?
                    .permissions(),
            )
            .map_err(|e| ProjectError::io(&self.path, e))?;
        staged
            .write_all(self.map.blocks())
            .map_err(|e| ProjectError::io(&self.path, e))?;
        staged
            .as_file()
            .sync_all()
            .map_err(|e| ProjectError::io(&self.path, e))?;
        // Recheck immediately before replacement; external writers do not participate in a lock protocol.
        let current = read_bounded(&self.path, 4 * 1024 * 1024)?;
        if <[u8; 32]>::from(Sha256::digest(&current)) != self.hash {
            return Err(ProjectError::Conflict(self.path.clone()));
        }
        staged
            .persist(&self.path)
            .map_err(|e| ProjectError::io(&self.path, e.error))?;
        self.hash = Sha256::digest(self.map.blocks()).into();
        self.map.mark_saved();
        Ok(())
    }
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

fn filename_dimensions(path: &Path) -> Option<(u16, u16)> {
    let name = path.file_name()?.to_str()?;
    for part in name.split('.') {
        let Some((w, h)) = part.split_once('x') else {
            continue;
        };
        if let (Ok(w), Ok(h)) = (w.parse(), h.parse()) {
            return Some((w, h));
        }
    }
    None
}
