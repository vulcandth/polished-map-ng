//! Pinned plusplus roof-window.cpp and Config roof paths. LGPL-3.0-only.
//! Reuse the nine-tile pixel editor, PNG export and guarded roof persistence;
//! plusplus exposes hue/definition snapshots, never master palette attributes.
use crate::{
    PngPlatform, ProjectError, SaveTarget, TilesetOutput,
    file_io::{optional_hash, prepare_target},
    graphics::validate_tileset,
};
use crate::{
    Tile, TileCommand, TileSnapshot, TilesetError,
    attributed::{Error, RawTileCommand, RawTileSnapshot},
};
use std::path::Path;

/// Pinned option-dialogs.cpp uses case-sensitive suffixes for roof choices.
pub fn list_roofs(root: &Path) -> Result<Vec<String>, ProjectError> {
    let mut names = crate::asset_names::graphics_stems_with_case(
        &root.join("gfx/tilesets/roofs"),
        &[".2bpp.lz", ".2bpp", ".png"],
        false,
        true,
    )?;
    let mut seen = std::collections::HashSet::new();
    names.retain(|name| seen.insert(name.clone()));
    Ok(names)
}

#[derive(Clone, Debug)]
/// Nine editable roof tiles with a separately observed source and PNG destination.
///
/// Saving roof pixels is independent of base tiles, maps, and palette exports.
pub struct RoofDocument {
    inner: crate::RoofDocument,
}

impl RoofDocument {
    /// Union source syntax; roof pixels remain raw and occurrence attributes stay
    /// in the blockset. Exactly nine tiles are required by both source models.
    pub fn open_unified_in_project(
        root: &Path,
        name: &str,
        platform: PngPlatform,
    ) -> Result<Self, ProjectError> {
        Ok(Self {
            inner: crate::RoofDocument::open_unified_in_project(root, name, [0; 9], platform)?,
        })
    }
    /// Discover and load exactly nine roof tiles with fixed storage rules.
    ///
    /// # Errors
    /// Rejects invalid names, read/decode failures, incorrect tile counts, and
    /// files that change during observation.
    pub fn open_in_project(
        root: &Path,
        name: &str,
        platform: PngPlatform,
    ) -> Result<Self, ProjectError> {
        validate_tileset(root, name)?;
        let directory = root.join("gfx/tilesets/roofs");
        let path = super::graphics_path(&directory, name);
        let source = prepare_target(&path)?;
        let tiles = super::read_tiles(&path, platform)?
            .try_into()
            .map_err(|_| ProjectError::Format {
                path: path.clone(),
                message: "roof graphics must contain exactly nine tiles".into(),
            })?;
        if optional_hash(&path)? != source.hash {
            return Err(ProjectError::Conflict(path));
        }
        Ok(Self {
            inner: crate::RoofDocument::from_source(
                name,
                platform,
                tiles,
                [0; 9],
                source,
                prepare_target(&directory.join(format!("{name}.png")))?,
            )?,
        })
    }
    /// Selected roof asset name without its graphics extension.
    pub fn name(&self) -> &str {
        self.inner.name()
    }
    /// Observed input path and hash; after saving this becomes the PNG output.
    pub fn source(&self) -> &SaveTarget {
        self.inner.source()
    }
    /// Observed PNG destination used by [`Self::save`].
    pub fn output(&self) -> &SaveTarget {
        self.inner.output()
    }
    /// Fingerprint observed input/output and PNG policy, excluding unsaved pixels.
    pub fn content_hash(&self) -> String {
        self.inner.content_hash()
    }
    /// Whether roof pixels have unsaved edits.
    pub fn is_modified(&self) -> bool {
        self.inner.roof().tileset().is_modified()
    }
    /// Copy all nine tiles in file order.
    pub fn tiles(&self) -> [Tile; 9] {
        std::array::from_fn(|i| self.inner.roof().tileset().tiles()[i].clone())
    }
    /// Copy a defined roof tile at index 0..=8; other indices return an error.
    pub fn snapshot(&self, tile: u16) -> Result<RawTileSnapshot, ProjectError> {
        let i = index(tile)?;
        Ok(RawTileSnapshot {
            pixels: self.inner.roof().tileset().snapshot(i)?.pixels,
            defined: true,
        })
    }
    /// Render an 8×8 grayscale tile at index 0..=8; other indices return an error.
    pub fn render_tile(&self, tile: u16) -> Result<crate::RgbImage, ProjectError> {
        Ok(self
            .inner
            .roof()
            .tileset()
            .render_tile(index(tile)?, &[[[255; 3], [85; 3], [170; 3], [0; 3]]; 9])?)
    }
    /// Render a 24×24 grayscale sheet, three tiles per row in file order.
    pub fn export(&self) -> crate::RgbImage {
        self.inner.roof().export()
    }
    /// Apply raw pixel commands atomically to the nine roof tiles.
    ///
    /// # Errors
    /// Rejects deletion, undefined snapshots, indices outside 0..=8, and invalid
    /// pixel/image data. Image pastes clip to the selected tile.
    pub fn apply_batch(&mut self, commands: &[RawTileCommand]) -> Result<(), ProjectError> {
        let commands = commands
            .iter()
            .map(convert)
            .collect::<Result<Vec<_>, _>>()?;
        self.inner.roof_mut().apply_batch(&commands)?;
        Ok(())
    }
    /// Describe a pending PNG write without changing files or dirty state.
    ///
    /// Returns `None` when clean. Changed source/destination hashes, read
    /// failures, or PNG encoding errors reject the preview.
    pub fn preview_save(&self) -> Result<Option<TilesetOutput>, ProjectError> {
        self.inner.preview_save()
    }
    /// Guard and publish changed roof pixels to PNG, then clear dirty state.
    ///
    /// # Errors
    /// Source/destination conflicts, encoding errors, and I/O failures retain
    /// the in-memory edits. Clean saves still check observations.
    pub fn save(&mut self) -> Result<(), ProjectError> {
        self.inner.save()
    }
}

fn index(tile: u16) -> Result<u8, ProjectError> {
    if tile < 9 {
        Ok(tile as u8)
    } else {
        Err(Error::TileIndex(tile).into())
    }
}
fn convert(command: &RawTileCommand) -> Result<TileCommand, ProjectError> {
    Ok(match command {
        RawTileCommand::Set { tile, value } => {
            if !value.defined {
                return Err(TilesetError::RoofCommand.into());
            }
            TileCommand::Set {
                tile: index(*tile)?,
                value: TileSnapshot {
                    pixels: value.pixels.clone(),
                    palette: 0,
                },
            }
        }
        RawTileCommand::Paint { tile, points, hue } => TileCommand::Paint {
            tile: index(*tile)?,
            points: points.clone(),
            hue: *hue,
        },
        RawTileCommand::Fill { tile, at, hue } => TileCommand::Fill {
            tile: index(*tile)?,
            at: *at,
            hue: *hue,
        },
        RawTileCommand::Replace { tile, from, to } => TileCommand::Replace {
            tile: index(*tile)?,
            from: *from,
            to: *to,
        },
        RawTileCommand::SwapHues { tile, a, b } => TileCommand::SwapHues {
            tile: index(*tile)?,
            a: *a,
            b: *b,
        },
        RawTileCommand::Copy { from, to } => TileCommand::Copy {
            from: index(*from)?,
            to: index(*to)?,
        },
        RawTileCommand::Swap { a, b } => TileCommand::Swap {
            a: index(*a)?,
            b: index(*b)?,
        },
        RawTileCommand::Delete { .. } => return Err(TilesetError::RoofCommand.into()),
        RawTileCommand::ImportRgb {
            tile,
            width,
            height,
            pixels,
            ..
        } => TileCommand::ImportRgb {
            tile: index(*tile)?,
            width: *width,
            height: *height,
            pixels: pixels.clone(),
            allow_256: true,
        },
    })
}
