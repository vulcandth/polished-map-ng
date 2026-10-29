//! Installed-font policy from pinned master's themes.cpp::use_native_fonts.
//! LGPL-3.0-only. Fonts are read locally, never copied into project outputs.
//! FLTK 1.4.5 backend defaults are used after the application's ordered lists.
use fontique::{Collection, FontStyle, FontWeight, FontWidth, GenericFamily};
use skrifa::{FontRef, MetadataProvider};
use std::sync::Arc;

/// The reference uses the same preference lists on Linux and macOS, but their
/// underlying FLTK default families differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontPlatform {
    Windows,
    Mac,
    Unix,
}

impl FontPlatform {
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Mac
        } else {
            Self::Unix
        }
    }
}

/// Validated installed face, including its collection index and variable-font
/// coordinates. Retain these together when passing it to a renderer.
pub struct NativeFontFace {
    data: Arc<dyn AsRef<[u8]> + Send + Sync>,
    index: u32,
    family: String,
    variations: Vec<([u8; 4], f32)>,
    substituted_for: Option<&'static str>,
}

impl NativeFontFace {
    pub fn data(&self) -> Arc<dyn AsRef<[u8]> + Send + Sync> {
        Arc::clone(&self.data)
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn variations(&self) -> &[([u8; 4], f32)] {
        &self.variations
    }

    /// Reference family whose synthetic bold cannot yet be rendered. `None`
    /// means no explicit substitution; it does not certify rasterizer parity.
    pub fn substituted_for(&self) -> Option<&str> {
        self.substituted_for
    }
}

/// Missing roles are explicit: callers may keep bundled fallback faces, but
/// should report that native matching was unavailable. No display is required.
pub struct NativeFonts {
    platform: FontPlatform,
    pub ui: Option<NativeFontFace>,
    pub mono: Option<NativeFontFace>,
    pub mono_bold: Option<NativeFontFace>,
    pub sans_bold: Option<NativeFontFace>,
}

impl NativeFonts {
    /// Discover once at startup, outside painting. `message_font` is Windows'
    /// NONCLIENTMETRICS message-box family; it takes precedence only on Windows.
    pub fn discover(platform: FontPlatform, message_font: Option<&str>) -> Self {
        let mut db = Collection::new(Default::default());
        Self::from_collection(&mut db, platform, message_font)
    }

    fn from_collection(
        db: &mut Collection,
        platform: FontPlatform,
        message_font: Option<&str>,
    ) -> Self {
        let (ui, mono) = preferences(platform);
        let system_ui = (platform == FontPlatform::Windows)
            .then_some(message_font)
            .flatten()
            .and_then(|name| select(db, &[name], false));
        Self {
            platform,
            ui: system_ui
                .or_else(|| select(db, ui, false))
                .or_else(|| default_face(db, platform, false, false)),
            mono: select(db, mono, false).or_else(|| default_face(db, platform, true, false)),
            mono_bold: select(db, mono, true).or_else(|| default_face(db, platform, true, true)),
            // Unlike Courier Bold, FL_HELVETICA_BOLD is never remapped by themes.cpp.
            sans_bold: default_face(db, platform, false, true).or_else(|| {
                if platform != FontPlatform::Windows {
                    return None;
                }
                // GDI synthesizes this source face. Until the shared renderer
                // can do that, retain a real, explicitly reported bold fallback.
                let mut face = select(db, &["Arial"], true)?;
                face.substituted_for = Some("Microsoft Sans Serif Bold");
                Some(face)
            }),
        }
    }

    pub fn is_consolas(&self) -> bool {
        self.platform == FontPlatform::Windows
            && self
                .mono
                .as_ref()
                .is_some_and(|face| face.family == "Consolas")
    }
}

fn preferences(platform: FontPlatform) -> (&'static [&'static str], &'static [&'static str]) {
    match platform {
        FontPlatform::Windows => (
            &["Segoe UI", "Tahoma", "MS Sans Serif"],
            &["Consolas", "Lucida Console", "Courier New"],
        ),
        FontPlatform::Mac | FontPlatform::Unix => (
            &["Ubuntu", "Noto Sans", "Droid Sans", "DejaVu Sans"],
            &[
                "Ubuntu Mono",
                "Noto Sans Mono",
                "Droid Sans Mono",
                "DejaVu Sans Mono",
            ],
        ),
    }
}

fn default_face(
    db: &mut Collection,
    platform: FontPlatform,
    mono: bool,
    bold: bool,
) -> Option<NativeFontFace> {
    match platform {
        FontPlatform::Windows => select(
            db,
            &[if mono {
                "Courier New"
            } else {
                "Microsoft Sans Serif"
            }],
            bold,
        ),
        FontPlatform::Mac => select(db, &[if mono { "Courier" } else { "Arial" }], bold),
        FontPlatform::Unix => {
            let generic = if mono {
                GenericFamily::Monospace
            } else {
                GenericFamily::SansSerif
            };
            let ids: Vec<_> = db.generic_families(generic).collect();
            ids.into_iter().find_map(|id| {
                let family = db.family(id)?;
                load_face(&family, bold)
            })
        }
    }
}

fn select(db: &mut Collection, names: &[&str], bold: bool) -> Option<NativeFontFace> {
    names.iter().find_map(|name| {
        let family = db.family_by_name(name)?;
        // The original's use_font compares names exactly, not approximately.
        (family.name() == *name).then_some(())?;
        load_face(&family, bold)
    })
}

fn load_face(family: &fontique::FamilyInfo, bold: bool) -> Option<NativeFontFace> {
    let weight = if bold {
        FontWeight::BOLD
    } else {
        FontWeight::NORMAL
    };
    let face = family.match_font(FontWidth::NORMAL, FontStyle::Normal, weight, false)?;
    let synthesis = face.synthesis(FontWidth::NORMAL, FontStyle::Normal, weight);
    // egui cannot apply GDI-style synthetic bold/oblique. Do not silently label
    // a regular face as bold. This limitation is reported as a missing role.
    if synthesis.embolden() || synthesis.skew().is_some() || face.style() != FontStyle::Normal {
        return None;
    }
    // CSS-style nearest matching must not silently substitute a semibold,
    // light or condensed static face for the named normal/bold source face.
    if (!face.has_weight_axis() && face.weight() != weight)
        || (!face.has_width_axis() && face.width() != FontWidth::NORMAL)
    {
        return None;
    }
    let variations: Vec<_> = synthesis
        .variation_settings()
        .iter()
        .map(|(tag, value)| (tag.to_be_bytes(), *value))
        .collect();
    if variations.iter().any(|(tag, value)| {
        !value.is_finite()
            || !face
                .axes()
                .iter()
                .any(|axis| axis.tag.to_be_bytes() == *tag && (axis.min..=axis.max).contains(value))
    }) {
        return None;
    }
    let data = face.load(None)?;
    if data.as_ref().is_empty() || data.as_ref().len() > 32 * 1024 * 1024 {
        return None;
    }
    let font = FontRef::from_index(data.as_ref(), face.index()).ok()?;
    let metrics = font.metrics(
        skrifa::instance::Size::new(14.0),
        skrifa::instance::LocationRef::default(),
    );
    if !metrics.ascent.is_finite()
        || !metrics.descent.is_finite()
        || !(1.0..=64.0).contains(&(metrics.ascent - metrics.descent))
        || font
            .charmap()
            .map('A')
            .and_then(|id| font.outline_glyphs().get(id))
            .is_none()
    {
        return None;
    }
    Some(NativeFontFace {
        data: Arc::new(data),
        index: face.index(),
        family: family.name().to_owned(),
        variations,
        substituted_for: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collection() -> Collection {
        Collection::new(fontique::CollectionOptions {
            system_fonts: false,
            ..Default::default()
        })
    }

    fn add(db: &mut Collection, name: &str, bold: bool) -> fontique::FamilyId {
        // Metadata aliases let us test source ordering without installing or
        // distributing proprietary faces. These do not test real bold outlines.
        db.register_fonts(
            fontique::Blob::new(Arc::new(epaint_default_fonts::HACK_REGULAR)),
            Some(fontique::FontInfoOverride {
                family_name: Some(name),
                weight: Some(if bold {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                }),
                ..Default::default()
            }),
        )[0]
        .0
    }

    #[test]
    fn windows_message_family_precedes_fallbacks_and_mono_roles_are_independent() {
        let mut db = collection();
        for name in [
            "Configured UI",
            "Segoe UI",
            "Tahoma",
            "Consolas",
            "Lucida Console",
            "Courier New",
        ] {
            add(&mut db, name, false);
        }
        add(&mut db, "Courier New", true);
        let selected =
            NativeFonts::from_collection(&mut db, FontPlatform::Windows, Some("Configured UI"));
        assert_eq!(selected.ui.as_ref().unwrap().family(), "Configured UI");
        assert_eq!(selected.mono.as_ref().unwrap().family(), "Consolas");
        assert!(selected.is_consolas());
        assert_eq!(selected.mono_bold.unwrap().family(), "Courier New");
        let selected =
            NativeFonts::from_collection(&mut db, FontPlatform::Windows, Some("Missing"));
        assert_eq!(selected.ui.unwrap().family(), "Segoe UI");
    }

    #[test]
    fn unix_preferences_are_shared_with_mac_and_names_are_exact() {
        for platform in [FontPlatform::Mac, FontPlatform::Unix] {
            let mut db = collection();
            for name in [
                "Droid Sans",
                "Noto Sans",
                "DejaVu Sans",
                "Ubuntu",
                "Noto Sans Mono",
                "Ubuntu Mono",
            ] {
                add(&mut db, name, false);
            }
            add(&mut db, "Noto Sans Mono", true);
            let selected = NativeFonts::from_collection(&mut db, platform, Some("Droid Sans"));
            assert_eq!(selected.ui.unwrap().family(), "Ubuntu");
            assert_eq!(selected.mono.unwrap().family(), "Ubuntu Mono");
            assert_eq!(selected.mono_bold.unwrap().family(), "Noto Sans Mono");
            assert!(select(&mut db, &["ubuntu"], false).is_none());
        }
    }

    #[test]
    fn fltk_backend_defaults_and_unavailable_roles_are_explicit() {
        for platform in [FontPlatform::Windows, FontPlatform::Mac, FontPlatform::Unix] {
            let mut db = collection();
            let empty = NativeFonts::from_collection(&mut db, platform, None);
            assert!(crate::PrintFonts::from_native(&empty).is_err());
            assert!(
                empty.ui.is_none()
                    && empty.mono.is_none()
                    && empty.mono_bold.is_none()
                    && empty.sans_bold.is_none()
            );
            let (sans, mono) = match platform {
                FontPlatform::Windows => ("Microsoft Sans Serif", "Courier New"),
                FontPlatform::Mac => ("Arial", "Courier"),
                FontPlatform::Unix => ("Test sans", "Test mono"),
            };
            let sans_id = add(&mut db, sans, false);
            let mono_id = add(&mut db, mono, false);
            add(&mut db, mono, true);
            db.set_generic_families(GenericFamily::SansSerif, [sans_id].into_iter());
            db.set_generic_families(GenericFamily::Monospace, [mono_id].into_iter());
            let selected = NativeFonts::from_collection(&mut db, platform, None);
            assert_eq!(selected.ui.unwrap().family(), sans);
            assert_eq!(selected.mono.unwrap().family(), mono);
            assert_eq!(selected.mono_bold.unwrap().family(), mono);
            // A family with only a regular face cannot masquerade as bold.
            assert!(selected.sans_bold.is_none());
        }
    }

    #[test]
    fn unix_generic_consolas_does_not_enable_windows_size_adjustment() {
        let mut db = collection();
        let id = add(&mut db, "Consolas", false);
        db.set_generic_families(GenericFamily::Monospace, [id].into_iter());
        let selected = NativeFonts::from_collection(&mut db, FontPlatform::Unix, None);
        assert_eq!(selected.mono.as_ref().unwrap().family(), "Consolas");
        assert!(!selected.is_consolas());
    }

    #[test]
    fn synthetic_bold_substitution_is_explicit_and_shared_with_print() {
        let mut db = collection();
        add(&mut db, "Microsoft Sans Serif", false);
        add(&mut db, "Arial", true);
        add(&mut db, "Consolas", true);
        let fonts = NativeFonts::from_collection(&mut db, FontPlatform::Windows, None);
        let sans = fonts.sans_bold.as_ref().unwrap();
        assert_eq!(sans.family(), "Arial");
        assert_eq!(sans.substituted_for(), Some("Microsoft Sans Serif Bold"));
        let print = crate::PrintFonts::from_native(&fonts).unwrap();
        for (role, selected) in [
            (
                polished_map_core::PrintFont::CourierBold14,
                fonts.mono_bold.as_ref().unwrap(),
            ),
            (polished_map_core::PrintFont::HelveticaBold14, sans),
        ] {
            assert_eq!(print.face(role).collection_index(), selected.index());
            assert_eq!(print.face(role).variations(), selected.variations());
            assert_eq!(
                print.face(role).substituted_for(),
                selected.substituted_for()
            );
        }
        // If the exact reference face becomes available it always wins.
        add(&mut db, "Microsoft Sans Serif", true);
        let fonts = NativeFonts::from_collection(&mut db, FontPlatform::Windows, None);
        let sans = fonts.sans_bold.as_ref().unwrap();
        assert_eq!(sans.family(), "Microsoft Sans Serif");
        assert!(sans.substituted_for().is_none());
    }

    #[test]
    fn variable_native_bold_instance_survives_print_handoff() {
        let mut db = collection();
        let bytes = crate::print_fonts::variable_fixture::variable_hack();
        db.register_fonts(
            fontique::Blob::new(Arc::new(bytes)),
            Some(fontique::FontInfoOverride {
                family_name: Some("Consolas"),
                ..Default::default()
            }),
        );
        let selected = NativeFonts::from_collection(&mut db, FontPlatform::Windows, None);
        let face = selected.mono_bold.as_ref().unwrap();
        assert_eq!(face.variations(), &[(*b"wght", 700.0)]);
        let bold = crate::PrintFontFace::from_native(face).unwrap();
        let unused =
            crate::PrintFontFace::from_bytes(epaint_default_fonts::HACK_REGULAR, 0).unwrap();
        let print = crate::PrintFonts::from_faces(bold, unused);
        assert_eq!(
            print
                .rasterize(polished_map_core::PrintFont::CourierBold14, "A")
                .unwrap()
                .layout_size()[0],
            12
        );
    }

    #[test]
    fn nearest_static_weight_or_width_is_not_an_exact_face_match() {
        for (name, weight, width) in [
            ("Semibold only", FontWeight::new(600.0), FontWidth::NORMAL),
            ("Light only", FontWeight::new(300.0), FontWidth::NORMAL),
            ("Condensed only", FontWeight::NORMAL, FontWidth::CONDENSED),
        ] {
            let mut db = collection();
            db.register_fonts(
                fontique::Blob::new(Arc::new(epaint_default_fonts::HACK_REGULAR)),
                Some(fontique::FontInfoOverride {
                    family_name: Some(name),
                    weight: Some(weight),
                    width: Some(width),
                    ..Default::default()
                }),
            );
            assert!(select(&mut db, &[name], false).is_none());
            assert!(select(&mut db, &[name], true).is_none());
        }
    }

    #[test]
    #[ignore = "reads host-installed fonts; run in the native OS CI matrix"]
    fn installed_native_font_policy_loads_valid_host_faces() {
        let fonts = NativeFonts::discover(FontPlatform::current(), None);
        for (role, face) in [
            ("UI", &fonts.ui),
            ("mono", &fonts.mono),
            ("mono bold", &fonts.mono_bold),
        ] {
            let face = face
                .as_ref()
                .unwrap_or_else(|| panic!("missing {role} on native CI host"));
            eprintln!(
                "{role}: {} index={} variations={:?}",
                face.family(),
                face.index(),
                face.variations()
            );
            let data = face.data();
            let font = FontRef::from_index(data.as_ref().as_ref(), face.index()).unwrap();
            for ch in "0123456789ABCDEF".chars() {
                let glyph = font.charmap().map(ch).unwrap();
                assert!(font.outline_glyphs().get(glyph).is_some());
            }
        }
    }
}
