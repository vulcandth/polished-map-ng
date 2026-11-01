//! GUI document boundary for the two pinned Main_Window/map-events implementations.
//! LGPL-3.0-only. The owning map selects event parsing and persistence rules;
//! neither filename guessing nor a mutable global profile can change them.
use polished_map_core::{EventCommand, Map};
use polished_map_project::{EventScript, MapMetadata, ProjectError, SaveTarget, plusplus};
use std::path::{Path, PathBuf};

mod graphics;
pub(super) use graphics::MapGraphics;
mod palettes;
pub(super) use palettes::{PaletteDocument, PaletteRef};

pub(super) fn block_textures(
    ctx: &crate::egui::Context,
    graphics: &MapGraphics,
) -> Result<Vec<crate::egui::TextureHandle>, String> {
    (0..graphics.block_count())
        .map(|id| {
            let rgb = graphics.render_block(id as u8).map_err(|e| e.to_string())?;
            Ok(ctx.load_texture(
                format!("block-{id}"),
                crate::egui::ColorImage::from_rgb([32, 32], &rgb.pixels),
                crate::egui::TextureOptions::NEAREST,
            ))
        })
        .collect()
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Profile {
    #[default]
    Master,
    PlusPlus,
}
impl Profile {
    pub fn app_id(self) -> &'static str {
        match self {
            Self::Master => "polished-map-ng",
            Self::PlusPlus => "polished-map-ng-plusplus",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Master => crate::WINDOW_TITLE,
            Self::PlusPlus => "Polished Map NG++ — development",
        }
    }
    pub fn discover_map(
        self,
        path: &Path,
        root: Option<&Path>,
    ) -> Result<MapMetadata, ProjectError> {
        match self {
            Self::Master => polished_map_project::discover_map(path, root),
            Self::PlusPlus => plusplus::discover_map(path, root),
        }
    }
    pub fn tileset_options(
        self,
        root: &Path,
        hint: Option<&str>,
    ) -> Result<polished_map_project::TilesetOptions, ProjectError> {
        match self {
            Self::Master => polished_map_project::tileset_options(root, hint),
            Self::PlusPlus => plusplus::tileset_options(root, hint),
        }
    }
    pub fn list_roofs(self, root: &Path) -> Result<Vec<String>, ProjectError> {
        match self {
            Self::Master => polished_map_project::list_roofs(root),
            Self::PlusPlus => plusplus::list_roofs(root),
        }
    }
    pub fn inspect_map_sizes(
        self,
        path: &Path,
        preferred: Option<(u16, u16)>,
    ) -> Result<polished_map_project::MapSizeOptions, ProjectError> {
        match self {
            Self::Master => polished_map_project::inspect_map_sizes(path, preferred),
            Self::PlusPlus => plusplus::inspect_map_sizes(path, preferred),
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::Master => "blk",
            Self::PlusPlus => "ablk",
        }
    }
}

pub(super) enum PalettePlan {
    Master(polished_map_project::MapPalettePlan),
    PlusPlus(plusplus::MapPalettePlan),
}
impl PalettePlan {
    pub fn is_finished(&self) -> bool {
        match self {
            Self::Master(p) => p.is_finished(),
            Self::PlusPlus(p) => p.is_finished(),
        }
    }
    pub fn next_is_roof_colors(&self) -> bool {
        match self {
            Self::Master(p) => p.next_is_roof_colors(),
            Self::PlusPlus(p) => p.next_is_roof_colors(),
        }
    }
    pub fn next_import_path(&self) -> Option<&Path> {
        match self {
            Self::Master(p) => p.next_import_path(),
            Self::PlusPlus(p) => p.next_import_path(),
        }
    }
}

#[derive(Debug)]
pub(super) enum MapDocument {
    Master(polished_map_project::MapDocument),
    PlusPlus(plusplus::MapDocument),
}

impl From<polished_map_project::MapDocument> for MapDocument {
    fn from(document: polished_map_project::MapDocument) -> Self {
        Self::Master(document)
    }
}
impl From<plusplus::MapDocument> for MapDocument {
    fn from(document: plusplus::MapDocument) -> Self {
        Self::PlusPlus(document)
    }
}

impl MapDocument {
    pub fn profile(&self) -> Profile {
        match self {
            Self::Master(_) => Profile::Master,
            Self::PlusPlus(_) => Profile::PlusPlus,
        }
    }
    pub fn path(&self) -> &Path {
        match self {
            Self::Master(d) => d.path(),
            Self::PlusPlus(d) => d.path(),
        }
    }
    pub fn has_backing_file(&self) -> bool {
        match self {
            Self::Master(d) => d.has_backing_file(),
            Self::PlusPlus(d) => d.has_backing_file(),
        }
    }
    pub fn metadata(&self) -> &MapMetadata {
        match self {
            Self::Master(d) => d.metadata(),
            Self::PlusPlus(d) => d.metadata(),
        }
    }
    pub fn map(&self) -> &Map {
        match self {
            Self::Master(d) => d.map(),
            Self::PlusPlus(d) => d.map(),
        }
    }
    pub fn map_mut(&mut self) -> &mut Map {
        match self {
            Self::Master(d) => d.map_mut(),
            Self::PlusPlus(d) => d.map_mut(),
        }
    }
    pub fn save(&mut self) -> Result<(), ProjectError> {
        match self {
            Self::Master(d) => d.save(),
            Self::PlusPlus(d) => d.save(),
        }
    }
    pub fn save_as(&mut self, path: &Path) -> Result<(), ProjectError> {
        match self {
            Self::Master(d) => d.save_as(&polished_map_project::prepare_save_as(path)?),
            Self::PlusPlus(d) => d.save_as(&plusplus::prepare_save_as(path)?),
        }
    }
    pub fn save_extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Master(_) => &["blk"],
            Self::PlusPlus(_) => &["ablk", "blk"],
        }
    }
    pub fn default_extension(&self) -> &'static str {
        self.save_extensions()[0]
    }
    pub fn open_events(&self, path: &Path) -> Result<EventDocument, ProjectError> {
        match self {
            Self::Master(_) => polished_map_project::EventDocument::open(path).map(Into::into),
            Self::PlusPlus(_) => plusplus::EventDocument::open(path).map(Into::into),
        }
    }
    pub fn auto_load_events(&self) -> Result<Option<EventDocument>, ProjectError> {
        if !self.has_backing_file() {
            return Ok(None);
        }
        match self {
            Self::PlusPlus(d) => {
                plusplus::EventDocument::open_for_map(d).map(|d| d.map(Into::into))
            }
            Self::Master(d) => {
                let discovered = d
                    .metadata()
                    .root
                    .as_deref()
                    .map(|root| polished_map_project::event_script_path(root, &d.metadata().name))
                    .transpose()?;
                let path = discovered
                    .filter(|p| p.is_file())
                    .unwrap_or_else(|| d.path().with_extension("asm"));
                match self.open_events(&path) {
                    Ok(d) => Ok(Some(d)),
                    Err(ProjectError::Io { source, .. })
                        if source.kind() == std::io::ErrorKind::NotFound =>
                    {
                        Ok(None)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
    pub fn warp_path(&self, destination: &str) -> Result<PathBuf, String> {
        // Both branches confine macro-derived navigation to the map directory.
        if destination.is_empty()
            || !destination
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err("Invalid warp map name".into());
        }
        match self {
            Self::PlusPlus(d) => {
                plusplus::warp_map_path(d.path(), destination).map_err(|e| e.to_string())
            }
            Self::Master(d) => {
                let extension = d
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("blk");
                Ok(d.path()
                    .with_file_name(format!("{destination}.{extension}")))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum EventDocument {
    Master(polished_map_project::EventDocument),
    PlusPlus(plusplus::EventDocument),
}

impl From<polished_map_project::EventDocument> for EventDocument {
    fn from(document: polished_map_project::EventDocument) -> Self {
        Self::Master(document)
    }
}
impl From<plusplus::EventDocument> for EventDocument {
    fn from(document: plusplus::EventDocument) -> Self {
        Self::PlusPlus(document)
    }
}
impl EventDocument {
    pub fn source(&self) -> &SaveTarget {
        match self {
            Self::Master(d) => d.source(),
            Self::PlusPlus(d) => d.source(),
        }
    }
    pub fn script(&self) -> &EventScript {
        match self {
            Self::Master(d) => d.script(),
            Self::PlusPlus(d) => d.script(),
        }
    }
    pub fn apply(&mut self, commands: &[EventCommand]) -> Result<(), ProjectError> {
        match self {
            Self::Master(d) => d.apply(commands),
            Self::PlusPlus(d) => d.apply(commands),
        }
    }
    pub fn save(&mut self) -> Result<(), ProjectError> {
        match self {
            Self::Master(d) => d.save(),
            Self::PlusPlus(d) => d.save(),
        }
    }
}

#[cfg(test)]
mod tests;
