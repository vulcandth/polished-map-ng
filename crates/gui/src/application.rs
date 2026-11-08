//! Application identity is independent of the active document's storage.
//! Parity launchers remain fixed; the unified launcher has no mode selector.
use crate::{
    Editor,
    documents::{PaletteDocument, Profile},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Application {
    Parity(Profile),
    Unified,
}

impl Default for Application {
    fn default() -> Self {
        Self::Parity(Profile::Master)
    }
}

impl Application {
    pub fn is_unified(self) -> bool {
        self == Self::Unified
    }
    pub fn initial_profile(self) -> Profile {
        match self {
            Self::Parity(profile) => profile,
            Self::Unified => Profile::Master,
        }
    }
    pub fn accepts(self, profile: Profile) -> bool {
        self.is_unified() || self.initial_profile() == profile
    }
    pub fn app_id(self) -> &'static str {
        match self {
            Self::Parity(profile) => profile.app_id(),
            Self::Unified => "polished-map-ng-unified",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Parity(profile) => profile.title(),
            Self::Unified => "Polished Map NG Unified — development",
        }
    }
}

impl Editor {
    /// Derive capabilities from the document, never from a pending open request.
    pub(super) fn profile(&self) -> Profile {
        match self.application {
            Application::Parity(profile) => profile,
            Application::Unified => self.loaded.as_ref().map_or_else(
                || self.unloaded_palettes.profile(),
                |loaded| loaded.document.profile(),
            ),
        }
    }

    pub(super) fn palettes_for_open(&self, profile: Profile) -> PaletteDocument {
        if self.profile() == profile {
            self.palette_document().to_owned()
        } else if let Some(palettes) = &self.inactive_palettes {
            debug_assert!(palettes.profile() == profile);
            palettes.clone()
        } else {
            let mut palettes = match profile {
                Profile::Master => PaletteDocument::default(),
                Profile::PlusPlus => {
                    polished_map_project::plusplus::PaletteDocument::default().into()
                }
            };
            palettes.select(self.palette_document().selected());
            palettes
        }
    }
}
