//! Explicit plusplus blockset loading and persistence. Adapted from pinned
//! plusplus config.cpp/metatileset.cpp; LGPL-3.0-only. Uses the shared collision
//! codec and guarded multi-file persistence, without master format detection.

use crate::{
    ProjectError, SaveTarget,
    collisions::{CollisionDocument, validate_destination},
    document::{optional_hash, prepare_target},
    read_bounded,
    transaction::{Change, persist_pair},
};
use polished_map_core::plusplus::AttributeBlockset;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

mod roof;
mod tileset;
pub use roof::RoofDocument;
pub use tileset::TilesetDocument;

fn graphics_path(directory: &Path, name: &str) -> PathBuf {
    ["png", "2bpp", "2bpp.lz"]
        .into_iter()
        .map(|ext| directory.join(format!("{name}.{ext}")))
        .find(|p| p.is_file())
        .unwrap_or_else(|| directory.join(format!("{name}.2bpp.lz")))
}

#[derive(Debug)]
pub struct BlocksetDocument {
    offsets: SaveTarget,
    attributes: SaveTarget,
    collisions: Option<CollisionDocument>,
    blockset: AttributeBlockset,
}

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
    pub fn open_in_project(root: &Path, tileset: &str) -> Result<Self, ProjectError> {
        crate::graphics::validate_tileset(root, tileset)?;
        let offsets = binary_path(root, tileset, "metatiles");
        let attributes = binary_path(root, tileset, "attributes");
        let collision = collision_path(root, tileset);
        Self::open(&offsets, &attributes, collision.as_deref())
    }
    pub fn path(&self) -> &Path {
        &self.offsets.path
    }
    pub fn attribute_path(&self) -> &Path {
        &self.attributes.path
    }
    pub fn collision_path(&self) -> Option<&Path> {
        self.collisions.as_ref().map(|c| c.path.as_path())
    }
    pub fn blockset(&self) -> &AttributeBlockset {
        &self.blockset
    }
    pub fn blockset_mut(&mut self) -> &mut AttributeBlockset {
        &mut self.blockset
    }
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
                    .ok_or(polished_map_core::BlocksetError::CollisionLayout)?;
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
            _ => return Err(polished_map_core::BlocksetError::CollisionLayout.into()),
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

fn binary_path(root: &Path, tileset: &str, kind: &str) -> PathBuf {
    let base = tileset.split('.').next().unwrap_or(tileset);
    let candidates: Vec<_> = [tileset, base]
        .into_iter()
        .flat_map(|name| {
            [
                root.join(format!("data/tilesets/{name}_{kind}.bin")),
                root.join(format!("tilesets/{name}_{kind}.bin")),
            ]
        })
        .collect();
    candidates
        .iter()
        .find(|p| p.is_file())
        .unwrap_or(&candidates[3])
        .clone()
}

fn collision_path(root: &Path, tileset: &str) -> Option<PathBuf> {
    let base = tileset.split('.').next().unwrap_or(tileset);
    [tileset, base]
        .into_iter()
        .flat_map(|name| {
            [
                root.join(format!("data/tilesets/{name}_collision.asm")),
                root.join(format!("data/tilesets/{name}_collision.inc")),
                root.join(format!("tilesets/{name}_collision.asm")),
                root.join(format!("tilesets/{name}_collision.bin")),
            ]
        })
        .find(|p| p.is_file())
}

/// Read plusplus PNG/2bpp/LZ graphics without accepting master's CHR path.
/// The same bounded PNG/desaturation and LZ implementations serve both formats.
pub fn read_tiles(
    path: &Path,
    platform: crate::PngPlatform,
) -> Result<Vec<polished_map_core::Tile>, ProjectError> {
    let bytes = read_bounded(path, 8 * 1024 * 1024)?;
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "2bpp" => {
            crate::codecs::decode_plusplus_2bpp(&bytes).map_err(|e| ProjectError::codec(path, e))
        }
        "lz" => crate::codecs::decode_plusplus_lz_tiles(&bytes)
            .map_err(|e| ProjectError::codec(path, e)),
        "png" => crate::png_import::read_png_tiles_bounded(path, &bytes, platform, 512),
        _ => Err(ProjectError::Format {
            path: path.into(),
            message: "plusplus graphics require PNG, 2bpp or LZ".into(),
        }),
    }
}
