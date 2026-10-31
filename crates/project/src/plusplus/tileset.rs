//! Pinned plusplus Tileset::read_graphics/write_graphics and Config PNG paths.
//! LGPL-3.0-only. Shared codecs, PNG encoding and guarded staging supply I/O.
use crate::{
    PngPlatform, ProjectError, SaveTarget, TilesetOutput,
    document::{optional_hash, prepare_target},
    graphics::{encode_grayscale_png, validate_tileset},
    transaction::{Change, persist_pair},
};
use polished_map_core::{
    Palette, Tile,
    plusplus::{AttributeBlockset, BankOrder, Graphics, RawTileCommand, RawTileset},
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct Part {
    source: Option<SaveTarget>,
    output: SaveTarget,
    offset: usize,
    count: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct TilesetDocument {
    root: PathBuf,
    name: String,
    platform: PngPlatform,
    model: RawTileset,
    loaded_tiles: usize,
    parts: Vec<Part>,
}

impl TilesetDocument {
    pub fn open_in_project(
        root: &Path,
        name: &str,
        platform: PngPlatform,
    ) -> Result<Self, ProjectError> {
        validate_tileset(root, name)?;
        let root = std::fs::canonicalize(root).map_err(|e| ProjectError::io(root, e))?;
        let directory = root.join("gfx/tilesets");
        let (before_name, after_name) = name
            .split_once('.')
            .map(|(_, s)| s.split_once(',').unwrap_or((s, "")))
            .unwrap_or(("", ""));
        let load = |component: &str,
                    optional: bool|
         -> Result<(Vec<Tile>, Option<SaveTarget>), ProjectError> {
            if component.is_empty() {
                return Ok((Vec::new(), None));
            }
            validate_tileset(&directory, component)?;
            let path = super::graphics_path(&directory, component);
            if optional && !path.is_file() {
                return Ok((Vec::new(), None));
            }
            let source = prepare_target(&path)?;
            let tiles = super::read_tiles(&path, platform)?;
            if optional_hash(&path)? != source.hash {
                return Err(ProjectError::Conflict(path));
            }
            Ok((tiles, Some(source)))
        };
        let (main, main_source) = load(name, false)?;
        let (before, before_source) = load(before_name, true)?;
        let (after, after_source) = load(after_name, true)?;
        let before_count = before.len().div_ceil(16) * 16;
        let main_count = if after.is_empty() {
            main.len()
        } else {
            main.len().div_ceil(16) * 16
        };
        let after_offset = before_count + main_count;
        let loaded_tiles = after_offset + after.len();
        if loaded_tiles > 512 {
            return Err(ProjectError::Format {
                path: directory.join(name),
                message: "compound plusplus graphics exceed 512 tiles including row padding".into(),
            });
        }
        let mut tiles = vec![Tile::new([0; 64])?; 512];
        let mut defined = vec![false; 512];
        for (offset, component) in [(0, &before), (before_count, &main), (after_offset, &after)] {
            for (i, tile) in component.iter().enumerate() {
                tiles[offset + i] = tile.clone();
                defined[offset + i] = true;
            }
        }
        let model = RawTileset::new(tiles, defined)?;
        let mut parts = Vec::new();
        if !before_name.is_empty() {
            parts.push(Part {
                source: before_source,
                output: prepare_target(&directory.join(format!("{before_name}.png")))?,
                offset: 0,
                count: (before_count != 0).then_some(before_count),
            });
        }
        parts.push(Part {
            source: main_source,
            output: prepare_target(&directory.join(format!("{name}.png")))?,
            offset: before_count,
            count: (!after_name.is_empty() && main_count != 0).then_some(main_count),
        });
        if !after_name.is_empty() {
            parts.push(Part {
                source: after_source,
                output: prepare_target(&directory.join(format!("{after_name}.png")))?,
                offset: after_offset,
                count: None,
            });
        }
        Ok(Self {
            root,
            name: name.into(),
            platform,
            model,
            loaded_tiles,
            parts,
        })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn png_platform(&self) -> PngPlatform {
        self.platform
    }
    pub fn loaded_tile_count(&self) -> usize {
        self.loaded_tiles
    }
    pub fn enables_512_tiles(&self) -> bool {
        self.loaded_tiles > 256
    }
    pub fn model(&self) -> &RawTileset {
        &self.model
    }
    pub fn apply_batch(&mut self, commands: &[RawTileCommand]) -> Result<(), ProjectError> {
        self.model.apply_batch(commands)?;
        Ok(())
    }
    pub fn outputs(&self) -> Vec<&SaveTarget> {
        self.parts.iter().map(|p| &p.output).collect()
    }
    pub fn sources(&self) -> Vec<&SaveTarget> {
        self.parts
            .iter()
            .filter_map(|p| p.source.as_ref())
            .collect()
    }
    pub fn graphics(
        &self,
        blocks: &AttributeBlockset,
        palettes: [Palette; 8],
        order: BankOrder,
    ) -> Graphics {
        Graphics::new(self.model.clone(), blocks.clone(), palettes, order)
    }
    pub fn verify_unchanged(&self) -> Result<(), ProjectError> {
        for target in self
            .parts
            .iter()
            .flat_map(|p| p.source.iter().chain(std::iter::once(&p.output)))
        {
            if optional_hash(&target.path)? != target.hash {
                return Err(ProjectError::Conflict(target.path.clone()));
            }
        }
        Ok(())
    }
    pub fn content_hash(&self) -> String {
        let mut hash = Sha256::new();
        hash.update([self.platform as u8]);
        for target in self
            .parts
            .iter()
            .flat_map(|p| p.source.iter().chain(std::iter::once(&p.output)))
        {
            let path = target.path.as_os_str().as_encoded_bytes();
            hash.update((path.len() as u64).to_le_bytes());
            hash.update(path);
            hash.update([u8::from(target.hash.is_some())]);
            if let Some(h) = target.hash {
                hash.update(h);
            }
        }
        hex(&hash.finalize())
    }
    fn prepare_save(
        &self,
        blocks: &AttributeBlockset,
        order: BankOrder,
    ) -> Result<(RawTileset, Vec<Change>), ProjectError> {
        self.verify_unchanged()?;
        let mut draft = self.model.clone();
        let mut changes = Vec::new();
        if draft.is_modified() {
            draft.trim_unused(blocks, order);
            let mut destinations = std::collections::BTreeSet::new();
            let mut output_tiles = 0;
            for part in &self.parts {
                if !destinations.insert(&part.output.path) {
                    return Err(ProjectError::Format {
                        path: part.output.path.clone(),
                        message: "compound outputs resolve to the same destination".into(),
                    });
                }
                let raster = draft.export_region(part.offset, part.count)?;
                output_tiles += raster.width as usize * raster.height as usize / 64;
                if output_tiles > 512 {
                    return Err(ProjectError::Format {
                        path: part.output.path.clone(),
                        message: "saved compound graphics would exceed 512 tiles when reopened"
                            .into(),
                    });
                }
                changes.push(Change {
                    path: part.output.path.clone(),
                    expected: part.output.hash,
                    bytes: encode_grayscale_png(&part.output.path, &raster)?,
                });
            }
            self.verify_unchanged()?;
        }
        Ok((draft, changes))
    }
    pub fn preview_save(
        &self,
        blocks: &AttributeBlockset,
        order: BankOrder,
    ) -> Result<Vec<TilesetOutput>, ProjectError> {
        Ok(self
            .prepare_save(blocks, order)?
            .1
            .iter()
            .map(|c| TilesetOutput {
                path: c.path.clone(),
                previous_hash: c.expected.map(|h| hex(&h)),
                content_hash: hex(&Sha256::digest(&c.bytes)),
                bytes: c.bytes.len(),
            })
            .collect())
    }
    /// Preserve source PNG destinations, including optional compound names.
    /// All outputs encode before any replacement; imported binary sources stay intact.
    pub fn save(
        &mut self,
        blocks: &AttributeBlockset,
        order: BankOrder,
    ) -> Result<(), ProjectError> {
        let (mut draft, changes) = self.prepare_save(blocks, order)?;
        if changes.is_empty() {
            return Ok(());
        }
        persist_pair(&changes)?;
        for part in &mut self.parts {
            // The staging layer rejected duplicate destinations before writing.
            if let Some(c) = changes.iter().find(|c| c.path == part.output.path) {
                part.output.hash = Some(Sha256::digest(&c.bytes).into());
                part.source = Some(part.output.clone());
            }
        }
        draft.mark_saved();
        self.model = draft;
        Ok(())
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
