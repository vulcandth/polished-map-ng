//! Master config.cpp/metatileset.cpp blockset paths and byte I/O.
use crate::{
    ProjectError, SaveTarget, codecs,
    document::{persist_bytes, prepare_target},
    read_bounded,
};
use polished_map_core::Blockset;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct BlocksetDocument {
    path: PathBuf,
    hash: [u8; 32],
    blockset: Blockset,
}

pub fn prepare_blockset_save_as(path: &Path) -> Result<SaveTarget, ProjectError> {
    let path = if path.extension().is_none() {
        path.with_extension("bin")
    } else {
        path.to_path_buf()
    };
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("bin") || e.eq_ignore_ascii_case("bst"))
    {
        return Err(ProjectError::Format {
            path,
            message: "choose a .bin or .bst destination for binary block definitions".into(),
        });
    }
    prepare_target(&path)
}

impl BlocksetDocument {
    pub fn open(path: &Path) -> Result<Self, ProjectError> {
        let path = fs::canonicalize(path).map_err(|e| ProjectError::io(path, e))?;
        let raw = read_bounded(&path, 4 * 1024 * 1024)?;
        let hash = Sha256::digest(&raw).into();
        let bytes = if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("cel"))
        {
            let text = std::str::from_utf8(&raw).map_err(|e| ProjectError::Format {
                path: path.clone(),
                message: e.to_string(),
            })?;
            codecs::parse_asm(text).map_err(|e| ProjectError::codec(&path, e))?
        } else {
            raw
        };
        Ok(Self {
            path,
            hash,
            blockset: Blockset::from_bytes(&bytes)?,
        })
    }
    pub fn open_in_project(root: &Path, tileset: &str) -> Result<Self, ProjectError> {
        crate::graphics::validate_tileset(root, tileset)?;
        Self::open(&blockset_path(root, tileset))
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn blockset(&self) -> &Blockset {
        &self.blockset
    }
    pub fn blockset_mut(&mut self) -> &mut Blockset {
        &mut self.blockset
    }
    pub fn content_hash(&self) -> String {
        self.hash.iter().map(|b| format!("{b:02x}")).collect()
    }
    pub fn needs_save_as(&self) -> bool {
        self.path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("cel"))
    }
    pub fn save(&mut self) -> Result<(), ProjectError> {
        if self.needs_save_as() {
            return Err(ProjectError::Format {
                path: self.path.clone(),
                message: "ASM blockset import requires a binary Save As destination".into(),
            });
        }
        self.save_as(&SaveTarget {
            path: self.path.clone(),
            hash: Some(self.hash),
        })
    }
    pub fn save_as(&mut self, target: &SaveTarget) -> Result<(), ProjectError> {
        // SaveTarget can also originate from map Save As; reject a mismatched format.
        if !target
            .path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("bin") || e.eq_ignore_ascii_case("bst"))
        {
            return Err(ProjectError::Format {
                path: target.path.clone(),
                message: "binary blocksets require .bin or .bst".into(),
            });
        }
        let bytes = self.blockset.bytes();
        persist_bytes(&target.path, target.hash, &bytes)?;
        self.path.clone_from(&target.path);
        self.hash = Sha256::digest(&bytes).into();
        self.blockset.mark_saved();
        Ok(())
    }
}

fn blockset_path(root: &Path, tileset: &str) -> PathBuf {
    let base = tileset.split('.').next().unwrap_or(tileset);
    let mut paths = Vec::new();
    for name in [tileset, base] {
        paths.extend([
            root.join(format!("data/tilesets/{name}_metatiles.bin")),
            root.join(format!("gfx/blocksets/{name}.bst")),
            root.join(format!("MAPDATA/{name}.CEL")),
            root.join(format!("tilesets/{name}_metatiles.bin")),
        ]);
    }
    paths
        .iter()
        .find(|p| p.is_file())
        .unwrap_or(&paths[paths.len() - 1])
        .clone()
}
