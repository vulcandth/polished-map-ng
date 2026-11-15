//! Application identity is independent of the active document's storage.
//! Parity launchers remain fixed; the unified launcher has no mode selector.
use crate::editor::{
    Editor,
    documents::{PaletteDocument, Storage},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Application {
    Parity(Storage),
    Unified,
}

impl Default for Application {
    fn default() -> Self {
        Self::Parity(Storage::Classic)
    }
}

impl Application {
    pub fn list_roofs(
        self,
        profile: Storage,
        root: &std::path::Path,
    ) -> Result<Vec<String>, polished_map_project::ProjectError> {
        if self.is_unified() {
            polished_map_project::list_roofs(root)
        } else {
            profile.list_roofs(root)
        }
    }
    pub fn is_unified(self) -> bool {
        self == Self::Unified
    }
    pub fn initial_profile(self) -> Storage {
        match self {
            Self::Parity(profile) => profile,
            Self::Unified => Storage::Classic,
        }
    }
    pub fn accepts(self, profile: Storage) -> bool {
        self.is_unified() || self.initial_profile() == profile
    }
    pub fn app_id(self) -> &'static str {
        match self {
            Self::Parity(Storage::Classic) => "polished-map-ng",
            Self::Parity(Storage::Attributed) => "polished-map-ng-plusplus",
            Self::Unified => "polished-map-ng-unified",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Parity(Storage::Classic) => crate::editor::WINDOW_TITLE,
            Self::Parity(Storage::Attributed) => "Polished Map NG++ — development",
            Self::Unified => "Polished Map NG Unified — development",
        }
    }
}

impl Editor {
    /// Automatic fallback is document context, not a request to force the next
    /// project into monochrome. The fixed master launcher keeps its sticky flag.
    pub(super) fn monochrome_preference(&self) -> bool {
        if self.application.is_unified() {
            self.monochrome_override
        } else {
            self.monochrome
        }
    }

    pub(super) fn set_monochrome_preference(&mut self, enabled: bool) {
        self.monochrome_override = enabled;
        self.monochrome = enabled;
    }

    /// Derive capabilities from the document, never from a pending open request.
    pub(super) fn storage(&self) -> Storage {
        match self.application {
            Application::Parity(profile) => profile,
            Application::Unified => self.loaded.as_ref().map_or_else(
                || self.unloaded_palettes.storage(),
                |loaded| loaded.document.storage(),
            ),
        }
    }

    pub(super) fn palettes_for_open(&self, profile: Storage) -> PaletteDocument {
        if self.storage() == profile {
            self.palette_document().to_owned()
        } else if let Some(palettes) = &self.inactive_palettes {
            debug_assert!(palettes.storage() == profile);
            palettes.clone()
        } else {
            let mut palettes = match profile {
                Storage::Classic => PaletteDocument::default(),
                Storage::Attributed => {
                    polished_map_project::attributed::PaletteDocument::default().into()
                }
            };
            palettes.select(self.palette_document().selected());
            palettes
        }
    }
}
