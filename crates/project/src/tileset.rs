//! master Tileset::{read_graphics,write_graphics}, Main_Window::{save_tileset,change_tileset_cb}.
//! Base tiles and roof overlays remain separate; edits never bake a roof into
//! the base sheet. Multi-file output is staged before any destination is changed.
use crate::{
    BlocksetDocument, MapDocument, MasterGraphicsOptions, PaletteDocument, ProjectError,
    RoofDocument, SaveTarget, codecs,
    document::{optional_hash, prepare_target},
    graphics::{
        encode_grayscale_png, graphics_path, palette_map_path, read_tiles, tile_slot,
        validate_tileset,
    },
    palettes::map_palette_bank,
    read_bounded, tile_palettes,
    transaction::{Change, persist_pair},
};
use polished_map_core::{
    Blockset, Graphics, Palette, Tile, TileCommand, Tileset, UNDEFINED_COLORS,
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

struct Prepared {
    draft: Tileset,
    changes: Vec<Change>,
    palette_bytes: Option<Vec<u8>>,
    allow_priority: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct TilesetOutput {
    pub path: PathBuf,
    pub previous_hash: Option<String>,
    pub content_hash: String,
    pub bytes: usize,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Clone, Debug)]
struct Part {
    source: Option<SaveTarget>,
    output: SaveTarget,
    offset: usize,
    count: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct MasterTilesetDocument {
    root: PathBuf,
    name: String,
    model: Tileset,
    initial: Tileset,
    palettes: PaletteDocument,
    clear_slots: Vec<bool>,
    roof: Option<RoofDocument>,
    sources: Vec<SaveTarget>,
    parts: Vec<Part>,
    palette_source: Option<(SaveTarget, String)>,
    allow_256: bool,
    monochrome: bool,
    allow_priority: bool,
}

impl MasterTilesetDocument {
    /// Begin a GUI-style open from an explicit palette session. Resolve the
    /// returned plan against `palettes_mut` before publishing the final preview.
    /// Neither the supplied session nor any project files are modified.
    pub fn begin_for_map(
        doc: &MapDocument,
        name: &str,
        options: &MasterGraphicsOptions,
        session: &PaletteDocument,
    ) -> Result<(Self, crate::MapPalettePlan), ProjectError> {
        let root = doc
            .metadata()
            .root
            .as_deref()
            .ok_or_else(|| ProjectError::Format {
                path: doc.path().into(),
                message: "project root is required for graphics".into(),
            })?;
        let mut options = options.clone();
        options.monochrome |= !palette_map_path(root, name).is_file();
        let plan = crate::MapPalettePlan::for_map(doc, name, &options)?;
        let tileset = Self::open_colors(root, name, &options, session.clone())?;
        Ok((tileset, plan))
    }
    pub fn open_for_map(
        doc: &MapDocument,
        name: &str,
        options: &MasterGraphicsOptions,
    ) -> Result<Self, ProjectError> {
        let root = doc
            .metadata()
            .root
            .as_deref()
            .ok_or_else(|| ProjectError::Format {
                path: doc.path().into(),
                message: "project root is required for graphics".into(),
            })?;
        validate_tileset(root, name)?;
        let mut options = options.clone();
        options.monochrome |= !palette_map_path(root, name).is_file();
        let (bank, selected) = map_palette_bank(doc, name, &options)?;
        Self::open_colors(root, name, &options, PaletteDocument::new(bank, selected))
    }

    /// DAY defaults without map metadata; use open_for_map for contextual colors.
    pub fn open_in_project(
        root: &Path,
        name: &str,
        options: &MasterGraphicsOptions,
    ) -> Result<Self, ProjectError> {
        Self::open_colors(root, name, options, PaletteDocument::default())
    }

    fn open_colors(
        root: &Path,
        name: &str,
        options: &MasterGraphicsOptions,
        palettes: PaletteDocument,
    ) -> Result<Self, ProjectError> {
        validate_tileset(root, name)?;
        let gfx = root.join(if root.join("MAPDATA").is_dir() {
            "MAPDATA"
        } else {
            "gfx/tilesets"
        });
        let (before_name, after_name) = name
            .split_once('.')
            .map(|(_, suffix)| suffix.split_once(',').unwrap_or((suffix, "")))
            .unwrap_or(("", ""));
        let mut sources = Vec::new();
        let mut load =
            |name: &str, optional: bool| -> Result<(Vec<Tile>, Option<SaveTarget>), ProjectError> {
                if name.is_empty() {
                    return Ok((Vec::new(), None));
                }
                validate_tileset(&gfx, name)?;
                let path = graphics_path(&gfx, name);
                if optional && !path.is_file() {
                    return Ok((Vec::new(), None));
                }
                let observed = prepare_target(&path)?;
                let tiles = read_tiles(&path)?;
                if optional_hash(&path)? != observed.hash {
                    return Err(ProjectError::Conflict(path));
                }
                sources.push(observed.clone());
                Ok((tiles, Some(observed)))
            };
        let (main, main_source) = load(name, false)?;
        let (before, before_source) = load(before_name, true)?;
        let (after, after_source) = load(after_name, true)?;
        let before_end = before.len().div_ceil(16) * 16;
        let main_count = if after.is_empty() {
            main.len()
        } else {
            main.len().div_ceil(16) * 16
        };
        let middle_end = before_end + main_count;
        let path = palette_map_path(root, name);
        let monochrome = options.monochrome || !path.is_file();
        let (ids, palette_source) = if !monochrome {
            let observed = prepare_target(&path)?;
            let raw = String::from_utf8(read_bounded(&path, 1024 * 1024)?).map_err(|e| {
                ProjectError::Format {
                    path: path.clone(),
                    message: e.to_string(),
                }
            })?;
            if optional_hash(&path)? != observed.hash {
                return Err(ProjectError::Conflict(path));
            }
            let ids = codecs::parse_palette_map(&raw).map_err(|e| ProjectError::codec(&path, e))?;
            sources.push(observed.clone());
            (ids, Some((observed, raw)))
        } else {
            (vec![8; if options.allow_256 { 256 } else { 96 }], None)
        };
        let allow_priority = ids.iter().any(|&id| id & 0x80 != 0);
        let blank = Tile::new([0; 64])?;
        let mut tiles = vec![blank.clone(); 256];
        let mut attributes = vec![9; 256];
        let mut clear_slots = vec![true; 256];
        for i in 0..256 {
            let tile = if i < before.len() {
                Some(&before[i])
            } else if i >= before_end && i < before_end + main.len() {
                Some(&main[i - before_end])
            } else if i >= middle_end {
                Some(after.get(i - middle_end).unwrap_or(&blank))
            } else {
                None
            };
            if let Some(tile) = tile {
                let j = tile_slot(i, options.allow_256);
                tiles[j] = tile.clone();
                attributes[j] = ids.get(i).copied().unwrap_or(9);
                clear_slots[j] = false;
            }
        }
        let model = Tileset::new(tiles, attributes)?;
        let roof = options
            .roof
            .as_ref()
            .map(|name| {
                RoofDocument::open_in_project(
                    root,
                    name,
                    std::array::from_fn(|i| ids.get(i + 0x0a).copied().unwrap_or(9)),
                )
            })
            .transpose()?;
        let mut parts = Vec::new();
        if !before_name.is_empty() {
            parts.push(Part {
                source: before_source,
                output: prepare_target(&gfx.join(format!("{before_name}.png")))?,
                offset: 0,
                count: Some(before_end),
            });
        }
        parts.push(Part {
            source: main_source,
            output: prepare_target(&gfx.join(format!("{name}.png")))?,
            offset: before_end,
            count: (!after_name.is_empty()).then_some(main_count),
        });
        if !after_name.is_empty() {
            parts.push(Part {
                source: after_source,
                output: prepare_target(&gfx.join(format!("{after_name}.png")))?,
                offset: middle_end,
                count: None,
            });
        }
        Ok(Self {
            root: std::fs::canonicalize(root).map_err(|e| ProjectError::io(root, e))?,
            name: name.into(),
            initial: model.clone(),
            model,
            palettes,
            clear_slots,
            roof,
            sources,
            parts,
            palette_source,
            allow_256: options.allow_256,
            monochrome,
            allow_priority,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Prepare master's Change Tileset operation without modifying the caller.
    /// Retains session palettes, tile mode and roof name; reloads base/roof pixels,
    /// block definitions and collisions with clean dirty flags. The caller resolves
    /// unsaved asset edits before replacing its documents. Map bytes/history are
    /// deliberately outside this operation, including IDs beyond the new blockset.
    pub fn prepare_replacement(
        &self,
        name: &str,
    ) -> Result<(Self, BlocksetDocument), ProjectError> {
        let options = MasterGraphicsOptions {
            allow_256: self.allow_256,
            monochrome: self.monochrome,
            roof: self.roof.as_ref().map(|r| r.name().to_owned()),
            ..Default::default()
        };
        let mut tileset = Self::open_colors(&self.root, name, &options, self.palettes.clone())?;
        // Palette_Map only enables the global priority option; a plain new map
        // does not switch it off after an earlier priority-capable tileset.
        tileset.allow_priority |= self.allow_priority;
        let blocks = BlocksetDocument::open_in_project(&self.root, name)?;
        // Preflight definitions before returning a replacement to either adapter.
        tileset.graphics(blocks.blockset())?;
        Ok((tileset, blocks))
    }
    /// Replace only the roof overlay, preserving base pixels, palettes and edits.
    /// The caller must resolve any unsaved-roof warning before calling this.
    /// Failed reads leave the previous roof intact. A successful reload preserves
    /// upstream's sticky roof dirty flag; removing a roof clears that domain.
    pub fn change_roof(&mut self, name: Option<&str>) -> Result<(), ProjectError> {
        let modified = self
            .roof
            .as_ref()
            .is_some_and(|roof| roof.roof().tileset().is_modified());
        let roof = name
            .map(|name| {
                let mut roof = RoofDocument::open_in_project(
                    &self.root,
                    name,
                    std::array::from_fn(|i| self.model.attributes()[i + 0x0a]),
                )?;
                if modified {
                    let value = roof.roof().tileset().snapshot(0)?;
                    roof.roof_mut()
                        .apply_batch(&[TileCommand::Set { tile: 0, value }])?;
                }
                Ok::<_, ProjectError>(roof)
            })
            .transpose()?;
        self.roof = roof;
        Ok(())
    }
    pub fn tileset(&self) -> &Tileset {
        &self.model
    }
    pub fn roof(&self) -> Option<&RoofDocument> {
        self.roof.as_ref()
    }
    pub fn roof_mut(&mut self) -> Option<&mut RoofDocument> {
        self.roof.as_mut()
    }
    pub fn colors(&self) -> &[Palette; 9] {
        self.palettes.bank().colors(self.palettes.selected())
    }
    pub fn palettes(&self) -> &PaletteDocument {
        &self.palettes
    }
    pub fn palettes_mut(&mut self) -> &mut PaletteDocument {
        &mut self.palettes
    }
    pub fn allow_256(&self) -> bool {
        self.allow_256
    }
    pub fn monochrome(&self) -> bool {
        self.monochrome
    }
    pub fn allow_priority(&self) -> bool {
        self.allow_priority
    }
    pub fn outputs(&self) -> Vec<&SaveTarget> {
        self.parts
            .iter()
            .map(|p| &p.output)
            .chain(self.palette_source.iter().map(|(target, _)| target))
            .collect()
    }
    pub fn source_files(&self) -> &[SaveTarget] {
        &self.sources
    }
    /// Fingerprint all observed files, editing mode and block definitions used
    /// during trimming. Bound to these local paths; not a portable asset ID.
    pub fn content_hash(&self, blocks: &Blockset) -> String {
        let mut hash = Sha256::new();
        hash.update([u8::from(self.allow_256), u8::from(self.monochrome)]);
        hash.update(blocks.bytes());
        let observations: std::collections::BTreeMap<_, _> = self
            .sources
            .iter()
            .chain(self.outputs())
            .map(|s| (&s.path, s.hash))
            .collect();
        for (path, observed) in observations {
            let path = path.as_os_str().as_encoded_bytes();
            hash.update((path.len() as u64).to_le_bytes());
            hash.update(path);
            hash.update([u8::from(observed.is_some())]);
            if let Some(observed) = observed {
                hash.update(observed);
            }
        }
        hex(&hash.finalize())
    }
    pub fn preview_save(&self, blocks: &Blockset) -> Result<Vec<TilesetOutput>, ProjectError> {
        let prepared = self.prepare_save(blocks)?;
        Ok(prepared
            .changes
            .iter()
            .map(|c| TilesetOutput {
                path: c.path.clone(),
                previous_hash: c.expected.map(|h| hex(&h)),
                content_hash: hex(&Sha256::digest(&c.bytes)),
                bytes: c.bytes.len(),
            })
            .collect())
    }
    pub fn apply_batch(&mut self, commands: &[TileCommand]) -> Result<(), ProjectError> {
        self.model.apply_batch(commands)?;
        if !commands.is_empty()
            && let Some(roof) = &mut self.roof
        {
            roof.roof_mut()
                .synchronize_attributes(std::array::from_fn(|i| {
                    self.model.attributes()[i + 0x0a]
                }))?;
        }
        Ok(())
    }

    pub fn graphics(&self, blocks: &Blockset) -> Result<Graphics, ProjectError> {
        let mut tiles = self.model.tiles().to_vec();
        let mut palettes = Vec::with_capacity(256);
        for i in 0..256 {
            let attr = self.model.attributes()[i];
            let unchanged = self.model.tiles()[i] == self.initial.tiles()[i]
                && attr == self.initial.attributes()[i];
            palettes.push(if self.clear_slots[i] && unchanged {
                [[255; 3]; 4]
            } else if attr == 9 {
                UNDEFINED_COLORS
            } else {
                self.colors()[if self.monochrome {
                    8
                } else {
                    usize::from(attr & 15)
                }]
            });
        }
        if let Some(roof) = &self.roof {
            for (i, tile) in roof.roof().tileset().tiles().iter().enumerate() {
                let slot = i + 0x0a;
                tiles[slot] = tile.clone();
                let attr = roof.roof().tileset().attributes()[i];
                palettes[slot] = if attr == 9 {
                    UNDEFINED_COLORS
                } else {
                    self.colors()[if self.monochrome {
                        8
                    } else {
                        usize::from(attr & 15)
                    }]
                };
            }
        }
        Ok(Graphics::new(tiles, palettes, blocks.blocks().to_vec())?)
    }

    /// Save uses master's PNG destinations even for 2bpp/CHR/LZ imports. The
    /// original source remains intact. All observed sources and destinations are
    /// conflict-checked, and all output bytes are prepared before replacement.
    fn prepare_save(&self, blocks: &Blockset) -> Result<Prepared, ProjectError> {
        for source in &self.sources {
            if optional_hash(&source.path)? != source.hash {
                return Err(ProjectError::Conflict(source.path.clone()));
            }
        }
        let mut draft = self.model.clone();
        if !draft.is_modified() {
            return Ok(Prepared {
                draft,
                changes: Vec::new(),
                palette_bytes: None,
                allow_priority: self.allow_priority,
            });
        }
        draft.trim_unused(blocks);
        let mut changes = Vec::new();
        for part in &self.parts {
            let raster = draft.export_region(part.offset, part.count, self.allow_256)?;
            let bytes = encode_grayscale_png(&part.output.path, &raster)?;
            changes.push(Change {
                path: part.output.path.clone(),
                expected: part.output.hash,
                bytes,
            });
        }
        let allow_priority =
            self.allow_priority || draft.attributes().iter().any(|&a| a & 0x80 != 0);
        let palette_bytes = if let Some((target, raw)) = &self.palette_source {
            let canonical = draft.palette_map_asm(self.allow_256, allow_priority)?;
            let bytes = tile_palettes::encode(&target.path, raw, &canonical)?;
            changes.push(Change {
                path: target.path.clone(),
                expected: target.hash,
                bytes: bytes.clone(),
            });
            Some(bytes)
        } else {
            None
        };
        // Preflight sources once more after PNG encoding, before staging writes.
        for source in &self.sources {
            if optional_hash(&source.path)? != source.hash {
                return Err(ProjectError::Conflict(source.path.clone()));
            }
        }
        for change in &changes {
            if optional_hash(&change.path)? != change.expected {
                return Err(ProjectError::Conflict(change.path.clone()));
            }
        }
        let mut destinations = std::collections::BTreeSet::new();
        for change in &changes {
            if !destinations.insert(&change.path) {
                return Err(ProjectError::Format {
                    path: change.path.clone(),
                    message: "compound outputs resolve to the same destination".into(),
                });
            }
        }
        Ok(Prepared {
            draft,
            changes,
            palette_bytes,
            allow_priority,
        })
    }

    pub fn save(&mut self, blocks: &Blockset) -> Result<(), ProjectError> {
        let Prepared {
            mut draft,
            changes,
            palette_bytes,
            allow_priority,
        } = self.prepare_save(blocks)?;
        if changes.is_empty() {
            return Ok(());
        }
        persist_pair(&changes)?;
        let old_assets: Vec<_> = self
            .parts
            .iter()
            .filter_map(|p| p.source.as_ref().map(|s| s.path.clone()))
            .collect();
        // Update observations from the exact committed bytes, never by accepting
        // an unobserved external edit that happened immediately after our save.
        for change in &changes {
            let hash = Some(<[u8; 32]>::from(Sha256::digest(&change.bytes)));
            for source in &mut self.sources {
                if source.path == change.path {
                    source.hash = hash;
                }
            }
            for part in &mut self.parts {
                if part.output.path == change.path {
                    part.output.hash = hash;
                    if part.source.as_ref().is_none_or(|s| s.path != change.path) {
                        self.sources.push(part.output.clone());
                    }
                    part.source = Some(part.output.clone());
                }
            }
            if let Some((target, _)) = &mut self.palette_source
                && target.path == change.path
            {
                target.hash = hash;
            }
        }
        self.sources.retain(|s| {
            !old_assets.contains(&s.path)
                || self.parts.iter().any(|p| {
                    p.source
                        .as_ref()
                        .is_some_and(|current| current.path == s.path)
                })
        });
        if let Some(bytes) = palette_bytes
            && let Some((_, raw)) = &mut self.palette_source
        {
            *raw = String::from_utf8(bytes).expect("palette encoder emits UTF-8");
        }
        for (i, clear) in self.clear_slots.iter_mut().enumerate() {
            *clear &= draft.tiles()[i] == self.initial.tiles()[i]
                && draft.attributes()[i] == self.initial.attributes()[i];
        }
        draft.mark_saved();
        self.initial = draft.clone();
        self.model = draft;
        self.allow_priority = allow_priority;
        Ok(())
    }
}
