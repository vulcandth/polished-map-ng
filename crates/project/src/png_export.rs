//! Observed PNG destinations for master's Print export workflow.
use crate::{ProjectError, SaveTarget};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Destination observed before an overwrite decision. Unlike map/asset imports,
/// screenshots may exceed eight MiB; hashes stream with a 256-MiB upper bound.
#[derive(Clone, Debug)]
pub struct PngExportTarget {
    pub(crate) observed: SaveTarget,
    extension_added: bool,
}

impl PngExportTarget {
    pub fn path(&self) -> &Path {
        self.observed.path()
    }
    pub fn exists(&self) -> bool {
        self.observed.exists()
    }
    pub fn content_hash(&self) -> Option<String> {
        self.observed.content_hash()
    }
    /// A chooser may have confirmed the spelling without `.png`, rather than
    /// the actual destination; callers must confirm that existing file too.
    pub fn extension_added(&self) -> bool {
        self.extension_added
    }
}

/// Add `.png` only if the basename contains no extension, as in add_dot_ext.
/// An existing suffix (including a leading dot) is preserved; the data is PNG.
/// This observes the destination and does not grant overwrite consent.
pub fn prepare_png_export(path: &Path) -> Result<PngExportTarget, ProjectError> {
    let name = path.file_name().ok_or_else(|| ProjectError::Format {
        path: path.into(),
        message: "destination needs a file name".into(),
    })?;
    let extension_added = !name.as_encoded_bytes().contains(&b'.');
    let mut filename = name.to_os_string();
    if extension_added {
        filename.push(".png");
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|e| ProjectError::io(parent, e))?;
    let path = parent.join(filename);
    let hash = png_hash(&path)?;
    Ok(PngExportTarget {
        observed: SaveTarget { path, hash },
        extension_added,
    })
}

pub(crate) fn png_hash(path: &Path) -> Result<Option<[u8; 32]>, ProjectError> {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ProjectError::io(path, e)),
    };
    let mut hash = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0; 8192];
    loop {
        let size = file
            .read(&mut buffer)
            .map_err(|e| ProjectError::io(path, e))?;
        if size == 0 {
            break;
        }
        total += size as u64;
        if total > 256 * 1024 * 1024 {
            return Err(ProjectError::Format {
                path: PathBuf::from(path),
                message: "PNG export destination exceeds 256 MiB".into(),
            });
        }
        hash.update(&buffer[..size]);
    }
    Ok(Some(hash.finalize().into()))
}
