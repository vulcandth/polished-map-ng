//! Bounded reads, observed destinations and guarded single-file replacement.
//! LGPL-3.0-only. No map or graphics semantics belong in this module.
//! Multi-file staging/recovery is coordinated by `transaction`.
use crate::ProjectError;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// A destination observed before an overwrite decision. Saving rechecks its content.
/// Construct with `prepare_save_as`; fields are private so callers cannot skip observation.
#[derive(Clone, Debug)]
pub struct SaveTarget {
    pub(crate) path: PathBuf,
    pub(crate) hash: Option<[u8; 32]>,
}

impl SaveTarget {
    /// Observed destination with a canonicalized parent directory.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Whether a file existed when this target was observed, not a fresh disk check.
    pub fn exists(&self) -> bool {
        self.hash.is_some()
    }
    /// Lowercase SHA-256 of the observed bytes, or `None` if absent.
    pub fn content_hash(&self) -> Option<String> {
        self.hash.map(hex_hash)
    }
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

pub(crate) fn hex_hash(hash: [u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
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

/// Read a file without accepting more than `limit` bytes.
///
/// Reads at most one byte beyond the limit to detect oversized input.
///
/// # Errors
/// Returns an I/O error for inaccessible files and a format error when
/// the byte limit is exceeded.
pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, ProjectError> {
    let file = fs::File::open(path).map_err(|e| ProjectError::io(path, e))?;
    let mut bytes = Vec::new();
    file.take((limit as u64).saturating_add(1))
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
