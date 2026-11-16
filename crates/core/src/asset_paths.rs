//! Pinned master/plusplus Config::{metatileset,attributes,palette_map,collisions}_path.
//! Keep compound-name fallback and directory precedence in one place for loading
//! and automatic detection. LGPL-3.0-only; see upstream.lock.toml.
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{ProjectError, ProjectFormat};

#[derive(Clone, Copy)]
pub(crate) enum AssetKind {
    MasterBlocks,
    PlusPlusBlocks,
    Attributes,
    PaletteMap,
    Collisions(ProjectFormat),
}

impl AssetKind {
    fn candidates(self, root: &Path, tileset: &str) -> Vec<PathBuf> {
        let base = tileset.split('.').next().unwrap_or(tileset);
        [tileset, base]
            .into_iter()
            .flat_map(|name| match self {
                Self::MasterBlocks => vec![
                    root.join(format!("data/tilesets/{name}_metatiles.bin")),
                    root.join(format!("gfx/blocksets/{name}.bst")),
                    root.join(format!("MAPDATA/{name}.CEL")),
                    root.join(format!("tilesets/{name}_metatiles.bin")),
                ],
                Self::PlusPlusBlocks | Self::Attributes => {
                    let kind = if matches!(self, Self::Attributes) {
                        "attributes"
                    } else {
                        "metatiles"
                    };
                    vec![
                        root.join(format!("data/tilesets/{name}_{kind}.bin")),
                        root.join(format!("tilesets/{name}_{kind}.bin")),
                    ]
                }
                Self::PaletteMap => vec![
                    root.join(format!("gfx/tilesets/{name}_palette_map.asm")),
                    root.join(format!("gfx/tilesets/{name}_palette_map.inc")),
                    root.join(format!("color/tilesets/{name}.asm")),
                    root.join(format!("tilesets/{name}_palette_map.asm")),
                ],
                Self::Collisions(format) => {
                    let mut paths = vec![
                        root.join(format!("data/tilesets/{name}_collision.asm")),
                        root.join(format!("data/tilesets/{name}_collision.inc")),
                    ];
                    if format == ProjectFormat::Master {
                        paths.push(root.join(format!("data/tilesets/{name}_collision.bin")));
                    }
                    paths.extend([
                        root.join(format!("tilesets/{name}_collision.asm")),
                        root.join(format!("tilesets/{name}_collision.bin")),
                    ]);
                    paths
                }
            })
            .collect()
    }

    pub(crate) fn existing(
        self,
        root: &Path,
        tileset: &str,
    ) -> Result<Option<PathBuf>, ProjectError> {
        crate::graphics::validate_tileset(root, tileset)?;
        for path in self.candidates(root, tileset) {
            if is_file(&path)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    pub(crate) fn path(self, root: &Path, tileset: &str) -> Result<PathBuf, ProjectError> {
        Ok(self.existing(root, tileset)?.unwrap_or_else(|| {
            // Every kind has a nonempty, fixed set of candidates. Match Config's
            // last-resort output path even when none exists yet.
            self.candidates(root, tileset)
                .pop()
                .expect("asset candidates")
        }))
    }
}

pub(crate) fn is_file(path: &Path) -> Result<bool, ProjectError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // Windows reports PATH_NOT_FOUND when a *file* occupies a parent
            // directory. Do not silently fall back to a lower-priority asset.
            for parent in path
                .ancestors()
                .skip(1)
                .filter(|p| !p.as_os_str().is_empty())
            {
                match fs::metadata(parent) {
                    Ok(metadata) if metadata.is_dir() => break,
                    Ok(_) => {
                        return Err(ProjectError::io(
                            parent,
                            std::io::Error::new(
                                std::io::ErrorKind::NotADirectory,
                                "asset path parent is not a directory",
                            ),
                        ));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(ProjectError::io(parent, error)),
                }
            }
            Ok(false)
        }
        Err(error) => Err(ProjectError::io(path, error)),
    }
}
