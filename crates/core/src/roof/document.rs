//! master Config::roof_png_path and Main_Window::save_roof; separate dirty state.
use crate::Roof;
use crate::image::encode_grayscale_png;
use crate::{
    ProjectError, SaveTarget, TilesetOutput,
    file_io::{optional_hash, persist_bytes, prepare_target},
    graphics::{graphics_path, read_tiles, validate_tileset},
};
use sha2::{Digest, Sha256};
use std::path::Path;

/// master Map_Options_Dialog roof discovery. Case-insensitive suffix matching;
/// duplicate stems collapse while original filename spelling is retained.
pub fn list_roofs(root: &Path) -> Result<Vec<String>, ProjectError> {
    let dir = root.join("gfx/tilesets/roofs");
    let mut names =
        crate::asset_names::graphics_stems(&dir, &[".2bpp.lz", ".2bpp", ".chr", ".png"], false)?;
    let mut seen = std::collections::HashSet::new();
    names.retain(|name| seen.insert(name.clone()));
    Ok(names)
}

#[derive(Clone, Debug)]
/// Nine editable roof tiles with a separately observed source and PNG destination.
///
/// Saving roof pixels is independent of base tiles, maps, and palette exports.
pub struct RoofDocument {
    png_platform: crate::PngPlatform,
    name: String,
    model: Roof,
    source: SaveTarget,
    output: SaveTarget,
}
impl RoofDocument {
    /// Discover and load exactly nine roof tiles with fixed storage rules.
    ///
    /// # Errors
    /// Rejects invalid names, read/decode failures, incorrect tile counts, and
    /// files that change during observation.
    pub fn open_in_project(
        root: &Path,
        name: &str,
        attributes: [u8; 9],
    ) -> Result<Self, ProjectError> {
        Self::open_in_project_with_png_platform(
            root,
            name,
            attributes,
            crate::PngPlatform::default(),
        )
    }
    /// Load a roof using explicit PNG-alpha semantics and nine palette attributes.
    ///
    /// # Errors
    /// Returns name, read, decode, attribute, tile-count, or observation errors.
    pub fn open_in_project_with_png_platform(
        root: &Path,
        name: &str,
        attributes: [u8; 9],
        png_platform: crate::PngPlatform,
    ) -> Result<Self, ProjectError> {
        Self::open_with_sources(root, name, attributes, png_platform, false)
    }
    /// Master's roof source syntax with checked unified asset resolution.
    pub fn open_unified_in_project(
        root: &Path,
        name: &str,
        attributes: [u8; 9],
        png_platform: crate::PngPlatform,
    ) -> Result<Self, ProjectError> {
        Self::open_with_sources(root, name, attributes, png_platform, true)
    }
    fn open_with_sources(
        root: &Path,
        name: &str,
        attributes: [u8; 9],
        png_platform: crate::PngPlatform,
        unified: bool,
    ) -> Result<Self, ProjectError> {
        validate_tileset(root, name)?;
        let directory = root.join("gfx/tilesets/roofs");
        let path = if unified {
            crate::graphics::unified_graphics_path(&directory, name)?
        } else {
            graphics_path(&directory, name)
        };
        let source = prepare_target(&path)?;
        let tiles = read_tiles(&path, png_platform)?;
        let tiles = tiles.try_into().map_err(|_| ProjectError::Format {
            path: path.clone(),
            message: "roof graphics must contain exactly nine tiles".into(),
        })?;
        if optional_hash(&path)? != source.hash {
            return Err(ProjectError::Conflict(path));
        }
        Self::from_source(
            name,
            png_platform,
            tiles,
            attributes,
            source,
            prepare_target(&directory.join(format!("{name}.png")))?,
        )
    }
    /// Shared storage/export lifecycle; profile adapters own discovery and decoding.
    pub(crate) fn from_source(
        name: &str,
        png_platform: crate::PngPlatform,
        tiles: [crate::Tile; 9],
        attributes: [u8; 9],
        source: SaveTarget,
        output: SaveTarget,
    ) -> Result<Self, ProjectError> {
        Ok(Self {
            png_platform,
            name: name.into(),
            model: Roof::new(tiles, attributes)?,
            source,
            output,
        })
    }
    /// Selected roof asset name without its graphics extension.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// PNG alpha/desaturation policy selected when opening.
    pub fn png_platform(&self) -> crate::PngPlatform {
        self.png_platform
    }
    /// Read the nine-tile model and dirty state.
    pub fn roof(&self) -> &Roof {
        &self.model
    }
    /// Edit through the roof model’s validated batch API.
    pub fn roof_mut(&mut self) -> &mut Roof {
        &mut self.model
    }
    /// Observed input path and hash; after saving this becomes the PNG output.
    pub fn source(&self) -> &SaveTarget {
        &self.source
    }
    /// Observed PNG destination used by [`Self::save`].
    pub fn output(&self) -> &SaveTarget {
        &self.output
    }
    /// Observed local source/destination state, independent of the unsaved draft.
    pub fn content_hash(&self) -> String {
        let mut hash = Sha256::new();
        hash.update([self.png_platform as u8]);
        for file in [&self.source, &self.output] {
            let path = file.path.as_os_str().as_encoded_bytes();
            hash.update((path.len() as u64).to_le_bytes());
            hash.update(path);
            hash.update([u8::from(file.hash.is_some())]);
            if let Some(observed) = file.hash {
                hash.update(observed);
            }
        }
        hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }
    fn prepared_bytes(&self) -> Result<Option<Vec<u8>>, ProjectError> {
        for file in [&self.source, &self.output] {
            if optional_hash(&file.path)? != file.hash {
                return Err(ProjectError::Conflict(file.path.clone()));
            }
        }
        if !self.model.tileset().is_modified() {
            return Ok(None);
        }
        encode_grayscale_png(&self.output.path, &self.model.export()).map(Some)
    }
    /// Describe a pending PNG write without changing files or dirty state.
    ///
    /// Returns `None` when clean. Changed source/destination hashes, read
    /// failures, or PNG encoding errors reject the preview.
    pub fn preview_save(&self) -> Result<Option<TilesetOutput>, ProjectError> {
        Ok(self.prepared_bytes()?.map(|bytes| TilesetOutput {
            path: self.output.path.clone(),
            previous_hash: self.output.content_hash(),
            content_hash: Sha256::digest(&bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
            bytes: bytes.len(),
        }))
    }
    /// Guard and publish changed roof pixels to PNG, then clear dirty state.
    ///
    /// # Errors
    /// Source/destination conflicts, encoding errors, and I/O failures retain
    /// the in-memory edits. Clean saves still check observations.
    pub fn save(&mut self) -> Result<(), ProjectError> {
        let Some(bytes) = self.prepared_bytes()? else {
            return Ok(());
        };
        if optional_hash(&self.source.path)? != self.source.hash {
            return Err(ProjectError::Conflict(self.source.path.clone()));
        }
        persist_bytes(&self.output.path, self.output.hash, &bytes)?;
        self.output.hash = Some(Sha256::digest(&bytes).into());
        self.source = self.output.clone();
        self.model.mark_saved();
        Ok(())
    }
}
