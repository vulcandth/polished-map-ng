//! Pinned plusplus roof-window.cpp and Config roof paths. LGPL-3.0-only.
//! Reuse the nine-tile pixel editor, PNG export and guarded roof persistence;
//! plusplus exposes hue/definition snapshots, never master palette attributes.
use crate::{
    PngPlatform, ProjectError, SaveTarget, TilesetOutput,
    document::{optional_hash, prepare_target},
    graphics::validate_tileset,
};
use polished_map_core::{
    Tile, TileCommand, TileSnapshot, TilesetError,
    plusplus::{Error, RawTileCommand, RawTileSnapshot},
};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct RoofDocument {
    inner: crate::RoofDocument,
}

impl RoofDocument {
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
    pub fn name(&self) -> &str {
        self.inner.name()
    }
    pub fn source(&self) -> &SaveTarget {
        self.inner.source()
    }
    pub fn output(&self) -> &SaveTarget {
        self.inner.output()
    }
    pub fn content_hash(&self) -> String {
        self.inner.content_hash()
    }
    pub fn is_modified(&self) -> bool {
        self.inner.roof().tileset().is_modified()
    }
    pub fn tiles(&self) -> [Tile; 9] {
        std::array::from_fn(|i| self.inner.roof().tileset().tiles()[i].clone())
    }
    pub fn snapshot(&self, tile: u16) -> Result<RawTileSnapshot, ProjectError> {
        let i = index(tile)?;
        Ok(RawTileSnapshot {
            pixels: self.inner.roof().tileset().snapshot(i)?.pixels,
            defined: true,
        })
    }
    pub fn render_tile(&self, tile: u16) -> Result<polished_map_core::RgbImage, ProjectError> {
        Ok(self
            .inner
            .roof()
            .tileset()
            .render_tile(index(tile)?, &[[[255; 3], [85; 3], [170; 3], [0; 3]]; 9])?)
    }
    pub fn export(&self) -> polished_map_core::RgbImage {
        self.inner.roof().export()
    }
    pub fn apply_batch(&mut self, commands: &[RawTileCommand]) -> Result<(), ProjectError> {
        let commands = commands
            .iter()
            .map(convert)
            .collect::<Result<Vec<_>, _>>()?;
        self.inner.roof_mut().apply_batch(&commands)?;
        Ok(())
    }
    pub fn preview_save(&self) -> Result<Option<TilesetOutput>, ProjectError> {
        self.inner.preview_save()
    }
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
