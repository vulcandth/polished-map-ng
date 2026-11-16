//! Storage-specific GUI adapters over common map, palette, event and render APIs.
//! LGPL-3.0-only; adapted from the pinned Main_Window/map-events implementations.
//! The owning document fixes its parsing and persistence rules. Application
//! identity belongs to `application`; an uncommitted picker cannot change storage.
pub(super) use polished_map_core::EventDocument;
use polished_map_core::Map;
use polished_map_core::{MapMetadata, ProjectError, attributed};
use std::path::{Path, PathBuf};

mod graphics;
pub(super) use graphics::MapGraphics;
mod palettes;
pub(super) use palettes::{PaletteDocument, PaletteRef};

pub(super) fn block_textures(
    ctx: &crate::main_window::egui::Context,
    graphics: &MapGraphics,
) -> Result<Vec<crate::main_window::egui::TextureHandle>, String> {
    (0..graphics.block_count())
        .map(|id| {
            let rgb = graphics.render_block(id as u8).map_err(|e| e.to_string())?;
            Ok(ctx.load_texture(
                format!("block-{id}"),
                crate::main_window::egui::ColorImage::from_rgb([32, 32], &rgb.pixels),
                crate::main_window::egui::TextureOptions::NEAREST,
            ))
        })
        .collect()
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
/// Current document storage, independent of the application's launcher identity.
pub(super) enum Storage {
    #[default]
    Classic,
    Attributed,
}
impl Storage {
    pub fn discover_map(
        self,
        path: &Path,
        root: Option<&Path>,
    ) -> Result<MapMetadata, ProjectError> {
        match self {
            Self::Classic => polished_map_core::discover_map(path, root),
            Self::Attributed => attributed::discover_map(path, root),
        }
    }
    pub fn tileset_options(
        self,
        root: &Path,
        hint: Option<&str>,
    ) -> Result<polished_map_core::TilesetOptions, ProjectError> {
        match self {
            Self::Classic => polished_map_core::tileset_options(root, hint),
            Self::Attributed => attributed::tileset_options(root, hint),
        }
    }
    pub fn list_roofs(self, root: &Path) -> Result<Vec<String>, ProjectError> {
        match self {
            Self::Classic => polished_map_core::list_roofs(root),
            Self::Attributed => attributed::list_roofs(root),
        }
    }
    pub fn inspect_map_sizes(
        self,
        path: &Path,
        preferred: Option<(u16, u16)>,
    ) -> Result<polished_map_core::MapSizeOptions, ProjectError> {
        match self {
            Self::Classic => polished_map_core::inspect_map_sizes(path, preferred),
            Self::Attributed => attributed::inspect_map_sizes(path, preferred),
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::Classic => "blk",
            Self::Attributed => "ablk",
        }
    }
}

pub(super) enum PalettePlan {
    Classic(polished_map_core::MapPalettePlan),
    Attributed(attributed::MapPalettePlan),
}
impl PalettePlan {
    pub fn is_finished(&self) -> bool {
        match self {
            Self::Classic(p) => p.is_finished(),
            Self::Attributed(p) => p.is_finished(),
        }
    }
    pub fn next_is_roof_colors(&self) -> bool {
        match self {
            Self::Classic(p) => p.next_is_roof_colors(),
            Self::Attributed(p) => p.next_is_roof_colors(),
        }
    }
    pub fn next_import_path(&self) -> Option<&Path> {
        match self {
            Self::Classic(p) => p.next_import_path(),
            Self::Attributed(p) => p.next_import_path(),
        }
    }
}

#[derive(Debug)]
pub(super) enum MapDocument {
    Classic(polished_map_core::MapDocument),
    Attributed(attributed::MapDocument),
}

impl From<polished_map_core::MapDocument> for MapDocument {
    fn from(document: polished_map_core::MapDocument) -> Self {
        Self::Classic(document)
    }
}
impl From<attributed::MapDocument> for MapDocument {
    fn from(document: attributed::MapDocument) -> Self {
        Self::Attributed(document)
    }
}

impl MapDocument {
    // Keep the fixed launchers' save_as_cb naming. Unified context transitions
    // need current hints: changing only .blk to .ablk would retain stale assets.
    pub fn save_path_suggestion(&self, unified: bool) -> PathBuf {
        let source = self.path();
        let suggested = if matches!(self, Self::Attributed(_))
            && !self.needs_save_as()
            && self.save_extensions().iter().any(|extension| {
                source
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case(extension))
            }) {
            source.to_owned()
        } else {
            source.with_extension(self.default_extension())
        };
        if unified && self.needs_save_as() {
            polished_map_core::map_path_with_context(
                &suggested,
                (self.map().width(), self.map().height()),
                // A failed inventory read cannot justify a tileset hint.
                // Omitting the hint keeps Save As usable and Open explicit.
                self.metadata().filename_tileset_hint().ok().flatten(),
            )
            .unwrap_or(suggested)
        } else {
            suggested
        }
    }

    pub fn needs_save_as(&self) -> bool {
        match self {
            Self::Classic(d) => d.needs_save_as(),
            Self::Attributed(d) => d.needs_save_as(),
        }
    }
    pub fn select_tileset(&mut self, name: &str) -> Result<(), ProjectError> {
        match self {
            Self::Classic(d) => d.select_tileset(name),
            Self::Attributed(d) => d.select_tileset(name),
        }
    }
    pub fn with_storage(&self, storage: Storage, name: &str) -> Result<Self, ProjectError> {
        match (self, storage) {
            (Self::Classic(d), Storage::Attributed) => {
                d.with_plusplus_tileset(name).map(Self::Attributed)
            }
            (Self::Attributed(d), Storage::Classic) => {
                d.with_classic_tileset(name).map(Self::Classic)
            }
            (Self::Classic(d), Storage::Classic) => {
                let mut d = d.clone();
                d.select_tileset(name)?;
                Ok(Self::Classic(d))
            }
            (Self::Attributed(d), Storage::Attributed) => {
                let mut d = d.clone();
                d.select_tileset(name)?;
                Ok(Self::Attributed(d))
            }
        }
    }

    pub fn storage(&self) -> Storage {
        match self {
            Self::Classic(_) => Storage::Classic,
            Self::Attributed(_) => Storage::Attributed,
        }
    }
    pub fn path(&self) -> &Path {
        match self {
            Self::Classic(d) => d.path(),
            Self::Attributed(d) => d.path(),
        }
    }
    pub fn has_backing_file(&self) -> bool {
        match self {
            Self::Classic(d) => d.has_backing_file(),
            Self::Attributed(d) => d.has_backing_file(),
        }
    }
    pub fn metadata(&self) -> &MapMetadata {
        match self {
            Self::Classic(d) => d.metadata(),
            Self::Attributed(d) => d.metadata(),
        }
    }
    pub fn map(&self) -> &Map {
        match self {
            Self::Classic(d) => d.map(),
            Self::Attributed(d) => d.map(),
        }
    }
    pub fn map_mut(&mut self) -> &mut Map {
        match self {
            Self::Classic(d) => d.map_mut(),
            Self::Attributed(d) => d.map_mut(),
        }
    }
    pub fn save(&mut self) -> Result<(), ProjectError> {
        match self {
            Self::Classic(d) => d.save(),
            Self::Attributed(d) => d.save(),
        }
    }
    pub fn save_as(&mut self, path: &Path) -> Result<(), ProjectError> {
        match self {
            Self::Classic(d) => d.save_as(&polished_map_core::prepare_save_as(path)?),
            Self::Attributed(d) => d.save_as(&attributed::prepare_save_as(path)?),
        }
    }
    pub fn save_extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Classic(_) => &["blk"],
            Self::Attributed(_) => &["ablk", "blk"],
        }
    }
    pub fn default_extension(&self) -> &'static str {
        self.save_extensions()[0]
    }
    pub fn open_events(&self, path: &Path) -> Result<EventDocument, ProjectError> {
        match self {
            Self::Classic(_) => EventDocument::open(path),
            Self::Attributed(_) => attributed::EventDocument::open(path).map(Into::into),
        }
    }
    pub fn open_unified_events(&self, path: &Path) -> Result<EventDocument, ProjectError> {
        EventDocument::open_unified(path)
    }
    pub fn auto_load_events(&self) -> Result<Option<EventDocument>, ProjectError> {
        self.auto_load_event_catalog(false)
    }
    pub fn auto_load_unified_events(&self) -> Result<Option<EventDocument>, ProjectError> {
        self.auto_load_event_catalog(true)
    }
    fn auto_load_event_catalog(
        &self,
        unified: bool,
    ) -> Result<Option<EventDocument>, ProjectError> {
        if !self.has_backing_file() {
            return Ok(None);
        }
        if !unified && let Self::Attributed(d) = self {
            return attributed::EventDocument::open_for_map(d).map(|d| d.map(Into::into));
        }
        let discovered = self
            .metadata()
            .root
            .as_deref()
            .map(|root| polished_map_core::event_script_path(root, &self.metadata().name))
            .transpose()?;
        let path = discovered
            .filter(|p| p.is_file())
            .unwrap_or_else(|| self.path().with_extension("asm"));
        let opened = if unified {
            self.open_unified_events(&path)
        } else {
            self.open_events(&path)
        };
        match opened {
            Ok(document) => Ok(Some(document)),
            Err(ProjectError::Io { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
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
            Self::Attributed(d) => {
                attributed::warp_map_path(d.path(), destination).map_err(|e| e.to_string())
            }
            Self::Classic(d) => {
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

#[cfg(test)]
mod tests;
