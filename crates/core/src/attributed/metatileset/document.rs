//! Paired offsets, occurrence attributes and collision persistence.
//! Adapted from pinned plusplus metatileset.cpp; LGPL-3.0-only.
use crate::attributed::AttributeBlockset;
use crate::{
    ProjectError, SaveTarget,
    collisions::{CollisionDocument, validate_destination},
    file_io::{optional_hash, prepare_target},
    read_bounded,
    transaction::{Change, persist_pair},
};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Debug)]
/// Paired tile offsets/attributes and optional collisions with guarded group saving.
pub struct BlocksetDocument {
    offsets: SaveTarget,
    attributes: SaveTarget,
    collisions: Option<CollisionDocument>,
    blockset: AttributeBlockset,
}

/// Observe an offsets/attributes destination, defaulting to `.bin`.
///
/// # Errors
/// Rejects other extensions, invalid paths, and observation failures.
pub fn prepare_binary_save_as(path: &Path) -> Result<SaveTarget, ProjectError> {
    let path = if path.extension().is_none() {
        path.with_extension("bin")
    } else {
        path.to_path_buf()
    };
    validate_binary(&path)?;
    prepare_target(&path)
}

fn validate_binary(path: &Path) -> Result<(), ProjectError> {
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("bin"))
    {
        return Err(ProjectError::Format {
            path: path.into(),
            message: "plusplus block offsets and attributes require .bin files".into(),
        });
    }
    Ok(())
}

fn read_binary(path: &Path) -> Result<(SaveTarget, Vec<u8>), ProjectError> {
    validate_binary(path)?;
    let path = fs::canonicalize(path).map_err(|e| ProjectError::io(path, e))?;
    let bytes = read_bounded(&path, 4096)?;
    Ok((
        SaveTarget {
            path,
            hash: Some(Sha256::digest(&bytes).into()),
        },
        bytes,
    ))
}

impl BlocksetDocument {
    /// Load distinct offsets, attribute and optional collision files.
    ///
    /// # Errors
    /// Rejects invalid binary sizes, mismatched companion lengths, duplicate
    /// paths, and source read/parse errors.
    pub fn open(
        offsets: &Path,
        attributes: &Path,
        collisions: Option<&Path>,
    ) -> Result<Self, ProjectError> {
        let (offsets, bytes) = read_binary(offsets)?;
        let (attributes, attrs) = read_binary(attributes)?;
        let mut blockset = AttributeBlockset::from_bytes(&bytes, &attrs)?;
        let collisions = collisions
            .map(|p| CollisionDocument::open(p, blockset.offsets().blocks().len()))
            .transpose()?;
        if offsets.path == attributes.path
            || collisions
                .as_ref()
                .is_some_and(|c| c.path == offsets.path || c.path == attributes.path)
        {
            return Err(ProjectError::Format {
                path: offsets.path,
                message: "offsets, attributes and collisions must be distinct files".into(),
            });
        }
        if let Some(c) = &collisions {
            blockset = blockset.with_collisions(c.rows())?;
        }
        Ok(Self {
            offsets,
            attributes,
            collisions,
            blockset,
        })
    }
    /// Discover and validate block definitions and optional collision data.
    ///
    /// # Errors
    /// Invalid asset names, unreadable sources, or inconsistent block/collision
    /// layouts return an error before a document is created.
    pub fn open_in_project(root: &Path, tileset: &str) -> Result<Self, ProjectError> {
        Self::open_with_collisions_rule(root, tileset, crate::ProjectFormat::PlusPlus)
    }
    /// Union collision discovery; offsets/attributes retain their paired format.
    pub fn open_unified_in_project(root: &Path, tileset: &str) -> Result<Self, ProjectError> {
        Self::open_with_collisions_rule(root, tileset, crate::ProjectFormat::Master)
    }
    fn open_with_collisions_rule(
        root: &Path,
        tileset: &str,
        rule: crate::ProjectFormat,
    ) -> Result<Self, ProjectError> {
        crate::graphics::validate_tileset(root, tileset)?;
        let offsets = crate::asset_paths::AssetKind::PlusPlusBlocks.path(root, tileset)?;
        let attributes = crate::asset_paths::AssetKind::Attributes.path(root, tileset)?;
        let collision = crate::asset_paths::AssetKind::Collisions(rule).existing(root, tileset)?;
        Self::open(&offsets, &attributes, collision.as_deref())
    }
    /// Observed path of the tile-offset/block-definition file.
    pub fn path(&self) -> &Path {
        &self.offsets.path
    }
    /// Observed path of the per-occurrence attribute bytes.
    pub fn attribute_path(&self) -> &Path {
        &self.attributes.path
    }
    /// Observed collision source path, if loaded.
    pub fn collision_path(&self) -> Option<&Path> {
        self.collisions.as_ref().map(|c| c.path.as_path())
    }
    /// Read the validated blocks and any attached collision rows.
    pub fn blockset(&self) -> &AttributeBlockset {
        &self.blockset
    }
    /// Edit through the validated block commands; saves include attached collisions.
    pub fn blockset_mut(&mut self) -> &mut AttributeBlockset {
        &mut self.blockset
    }
    /// Recheck observed block, attribute (when present), and collision files.
    ///
    /// # Errors
    /// Returns an I/O error or [`ProjectError::Conflict`] for changed content.
    pub fn verify_unchanged(&self) -> Result<(), ProjectError> {
        for (path, hash) in [
            (&self.offsets.path, self.offsets.hash),
            (&self.attributes.path, self.attributes.hash),
        ]
        .into_iter()
        .chain(self.collisions.iter().map(|c| (&c.path, Some(c.hash))))
        {
            if optional_hash(path)? != hash {
                return Err(ProjectError::Conflict(path.clone()));
            }
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
        if !self.blockset.is_modified() {
            return self.verify_unchanged();
        }
        let collision = self.collisions.as_ref().map(|c| SaveTarget {
            path: c.path.clone(),
            hash: Some(c.hash),
        });
        self.save_as(
            &self.offsets.clone(),
            &self.attributes.clone(),
            collision.as_ref(),
        )
    }
    /// All related destinations are required. No format conversion or inferred
    /// overwrite of an unrelated attribute/collision file is performed.
    pub fn save_as(
        &mut self,
        offsets: &SaveTarget,
        attributes: &SaveTarget,
        collision: Option<&SaveTarget>,
    ) -> Result<(), ProjectError> {
        validate_binary(&offsets.path)?;
        validate_binary(&attributes.path)?;
        let bytes = self.blockset.offsets().bytes();
        let attrs = self.blockset.attribute_bytes();
        let mut changes = vec![
            Change {
                path: offsets.path.clone(),
                expected: offsets.hash,
                bytes: bytes.clone(),
            },
            Change {
                path: attributes.path.clone(),
                expected: attributes.hash,
                bytes: attrs.clone(),
            },
        ];
        let saved_collision = match (&self.collisions, collision) {
            (Some(source), Some(target)) => {
                validate_destination(&target.path, source.is_binary())?;
                let rows = self
                    .blockset
                    .offsets()
                    .collisions()
                    .ok_or(crate::BlocksetError::CollisionLayout)?;
                let encoded = source.encode(rows)?;
                let saved = CollisionDocument::from_bytes(
                    target.path.clone(),
                    encoded.clone(),
                    rows.len(),
                )?;
                changes.push(Change {
                    path: target.path.clone(),
                    expected: target.hash,
                    bytes: encoded,
                });
                Some(saved)
            }
            (None, None) => None,
            _ => return Err(crate::BlocksetError::CollisionLayout.into()),
        };
        persist_pair(&changes)?;
        self.offsets = SaveTarget {
            path: offsets.path.clone(),
            hash: Some(Sha256::digest(&bytes).into()),
        };
        self.attributes = SaveTarget {
            path: attributes.path.clone(),
            hash: Some(Sha256::digest(&attrs).into()),
        };
        self.collisions = saved_collision;
        self.blockset.mark_saved();
        Ok(())
    }
}
