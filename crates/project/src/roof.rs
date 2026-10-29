//! master Config::roof_png_path and Main_Window::save_roof; separate dirty state.
use crate::{
    ProjectError, SaveTarget, TilesetOutput,
    document::{optional_hash, persist_bytes, prepare_target},
    graphics::{encode_grayscale_png, graphics_path, read_tiles, validate_tileset},
};
use polished_map_core::Roof;
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
pub struct RoofDocument {
    png_platform: crate::PngPlatform,
    name: String,
    model: Roof,
    source: SaveTarget,
    output: SaveTarget,
}
impl RoofDocument {
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
    pub fn open_in_project_with_png_platform(
        root: &Path,
        name: &str,
        attributes: [u8; 9],
        png_platform: crate::PngPlatform,
    ) -> Result<Self, ProjectError> {
        validate_tileset(root, name)?;
        let directory = root.join("gfx/tilesets/roofs");
        let path = graphics_path(&directory, name);
        let source = prepare_target(&path)?;
        let tiles = read_tiles(&path, png_platform)?;
        let tiles = tiles.try_into().map_err(|_| ProjectError::Format {
            path: path.clone(),
            message: "roof graphics must contain exactly nine tiles".into(),
        })?;
        if optional_hash(&path)? != source.hash {
            return Err(ProjectError::Conflict(path));
        }
        Ok(Self {
            png_platform,
            name: name.into(),
            model: Roof::new(tiles, attributes)?,
            source,
            output: prepare_target(&directory.join(format!("{name}.png")))?,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn png_platform(&self) -> crate::PngPlatform {
        self.png_platform
    }
    pub fn roof(&self) -> &Roof {
        &self.model
    }
    pub fn roof_mut(&mut self) -> &mut Roof {
        &mut self.model
    }
    pub fn source(&self) -> &SaveTarget {
        &self.source
    }
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
