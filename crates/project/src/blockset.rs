//! Master config.cpp/metatileset.cpp blockset paths and byte I/O.
use crate::{
    ProjectError, SaveTarget, codecs,
    collisions::{CollisionDocument, collision_path, validate_destination},
    document::{optional_hash, persist_bytes, prepare_target},
    read_bounded,
    transaction::{Change, persist_pair},
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
    collisions: Option<CollisionDocument>,
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
            collisions: None,
        })
    }
    pub fn open_in_project(root: &Path, tileset: &str) -> Result<Self, ProjectError> {
        crate::graphics::validate_tileset(root, tileset)?;
        Self::open_with_collisions(
            &blockset_path(root, tileset),
            collision_path(root, tileset).as_deref(),
        )
    }
    pub fn open_with_collisions(
        path: &Path,
        collisions: Option<&Path>,
    ) -> Result<Self, ProjectError> {
        let mut doc = Self::open(path)?;
        if let Some(path) = collisions {
            let collision = CollisionDocument::open(path, doc.blockset.blocks().len())?;
            doc.blockset = doc.blockset.with_collisions(collision.rows())?;
            doc.collisions = Some(collision);
        }
        Ok(doc)
    }
    pub fn collision_path(&self) -> Option<&Path> {
        self.collisions.as_ref().map(|c| c.path.as_path())
    }
    pub fn collision_hash(&self) -> Option<String> {
        self.collisions
            .as_ref()
            .map(|c| c.hash.iter().map(|b| format!("{b:02x}")).collect())
    }
    pub fn binary_collisions(&self) -> bool {
        self.collisions.as_ref().is_some_and(|c| c.is_binary())
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
    pub fn verify_unchanged(&self) -> Result<(), ProjectError> {
        if optional_hash(&self.path)? != Some(self.hash) {
            return Err(ProjectError::Conflict(self.path.clone()));
        }
        if let Some(collision) = &self.collisions
            && optional_hash(&collision.path)? != Some(collision.hash)
        {
            return Err(ProjectError::Conflict(collision.path.clone()));
        }
        Ok(())
    }
    pub fn save(&mut self) -> Result<(), ProjectError> {
        if self.needs_save_as() {
            return Err(ProjectError::Format {
                path: self.path.clone(),
                message: "ASM blockset import requires a binary Save As destination".into(),
            });
        }
        if !self.blockset.is_modified() {
            return self.verify_unchanged();
        }
        let collision_target = self.collisions.as_ref().map(|c| SaveTarget {
            path: c.path.clone(),
            hash: Some(c.hash),
        });
        self.save_to(
            &SaveTarget {
                path: self.path.clone(),
                hash: Some(self.hash),
            },
            collision_target.as_ref(),
        )
    }
    pub fn save_as(&mut self, target: &SaveTarget) -> Result<(), ProjectError> {
        if self.collisions.is_some() {
            return Err(ProjectError::Format{path:target.path.clone(),message:"this blockset has collisions; choose a paired collision destination with save_as_pair".into()});
        }
        self.save_to(target, None)
    }
    pub fn save_as_pair(
        &mut self,
        target: &SaveTarget,
        collision_target: &SaveTarget,
    ) -> Result<(), ProjectError> {
        self.save_to(target, Some(collision_target))
    }
    fn save_to(
        &mut self,
        target: &SaveTarget,
        collision_target: Option<&SaveTarget>,
    ) -> Result<(), ProjectError> {
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
        let saved_collision = match (&self.collisions, collision_target) {
            (Some(collision), Some(collision_target)) => {
                validate_destination(&collision_target.path, collision.is_binary())?;
                let rows = self
                    .blockset
                    .collisions()
                    .ok_or(polished_map_core::BlocksetError::CollisionLayout)?;
                let encoded = collision.encode(rows)?;
                let saved = CollisionDocument::from_bytes(
                    collision_target.path.clone(),
                    encoded.clone(),
                    rows.len(),
                )?;
                persist_pair(&[
                    Change {
                        path: target.path.clone(),
                        expected: target.hash,
                        bytes: bytes.clone(),
                    },
                    Change {
                        path: collision_target.path.clone(),
                        expected: collision_target.hash,
                        bytes: encoded,
                    },
                ])?;
                Some(saved)
            }
            (None, None) => {
                persist_bytes(&target.path, target.hash, &bytes)?;
                None
            }
            _ => return Err(polished_map_core::BlocksetError::CollisionLayout.into()),
        };
        self.path.clone_from(&target.path);
        self.hash = Sha256::digest(&bytes).into();
        self.blockset.mark_saved();
        self.collisions = saved_collision;
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
