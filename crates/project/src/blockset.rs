//! Master config.cpp/metatileset.cpp blockset paths and byte I/O.
use crate::{
    ProjectError, SaveTarget, codecs,
    collisions::{CollisionDocument, validate_destination},
    file_io::{optional_hash, persist_bytes, prepare_target},
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
/// Block definitions and optional collision source with guarded paired saving.
pub struct BlocksetDocument {
    path: PathBuf,
    hash: [u8; 32],
    blockset: Blockset,
    collisions: Option<CollisionDocument>,
}

/// Observe a binary block destination; default the extension to `.bin`.
///
/// # Errors
/// Rejects extensions other than `.bin`/`.bst`, invalid paths, or failed reads.
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
    /// Load 1..=256 blocks of 16 tile IDs from binary or `.cel` assembly.
    ///
    /// # Errors
    /// Rejects unreadable/oversized files, malformed assembly, and invalid block lengths.
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
    /// Discover and validate block definitions and optional collision data.
    ///
    /// # Errors
    /// Invalid asset names, unreadable sources, or inconsistent block/collision
    /// layouts return an error before a document is created.
    pub fn open_in_project(root: &Path, tileset: &str) -> Result<Self, ProjectError> {
        crate::graphics::validate_tileset(root, tileset)?;
        Self::open_with_collisions(
            &crate::asset_paths::AssetKind::MasterBlocks.path(root, tileset)?,
            crate::asset_paths::AssetKind::Collisions(crate::ProjectFormat::Master)
                .existing(root, tileset)?
                .as_deref(),
        )
    }
    /// Load blocks and optional collision rows of the same count.
    ///
    /// # Errors
    /// Read/parse failures and mismatched block/collision layouts are rejected.
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
    /// Observed collision source path, if loaded.
    pub fn collision_path(&self) -> Option<&Path> {
        self.collisions.as_ref().map(|c| c.path.as_path())
    }
    /// Last observed collision-file SHA-256, if collisions were loaded.
    pub fn collision_hash(&self) -> Option<String> {
        self.collisions
            .as_ref()
            .map(|c| c.hash.iter().map(|b| format!("{b:02x}")).collect())
    }
    /// Whether attached collisions use packed bytes rather than assembly fields.
    pub fn binary_collisions(&self) -> bool {
        self.collisions.as_ref().is_some_and(|c| c.is_binary())
    }
    /// Observed path of the tile-offset/block-definition file.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Read the validated blocks and any attached collision rows.
    pub fn blockset(&self) -> &Blockset {
        &self.blockset
    }
    /// Edit through the validated block commands; saves include attached collisions.
    pub fn blockset_mut(&mut self) -> &mut Blockset {
        &mut self.blockset
    }
    /// SHA-256 of the last observed block-definition source, excluding edits.
    pub fn content_hash(&self) -> String {
        self.hash.iter().map(|b| format!("{b:02x}")).collect()
    }
    /// Whether the imported `.cel` assembly requires an explicit binary destination.
    pub fn needs_save_as(&self) -> bool {
        self.path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("cel"))
    }
    /// Recheck observed block, attribute (when present), and collision files.
    ///
    /// # Errors
    /// Returns an I/O error or [`ProjectError::Conflict`] for changed content.
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
    /// Save dirty block data and attached companions with conflict checks.
    ///
    /// Clean saves still verify observed files. Failed multi-file publication
    /// attempts rollback; incomplete rollback returns recovery-file locations.
    ///
    /// # Errors
    /// Rejects changed sources, invalid output layouts, or filesystem failures.
    /// Dirty state clears only after successful publication.
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
    /// Save a collision-free blockset to an observed binary target.
    ///
    /// # Errors
    /// Rejects attached collisions (use [`Self::save_as_pair`]), unsupported
    /// extensions, changed target content, or I/O failures.
    pub fn save_as(&mut self, target: &SaveTarget) -> Result<(), ProjectError> {
        if self.collisions.is_some() {
            return Err(ProjectError::Format{path:target.path.clone(),message:"this blockset has collisions; choose a paired collision destination with save_as_pair".into()});
        }
        self.save_to(target, None)
    }
    /// Save blocks and attached collisions to explicitly observed destinations.
    ///
    /// # Errors
    /// Rejects mismatched formats/layouts, duplicate or changed destinations,
    /// and I/O failures. Incomplete rollback reports recovery-file locations.
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
