//! Native font installation and source font-role mapping. LGPL-3.0-only.
use super::*;
use polished_map_project::{FontPlatform, NativeFontFace, NativeFonts};
use std::sync::Arc;

const MONO_BOLD: &str = "Reference Courier Bold";
const SANS_BOLD: &str = "Reference Helvetica Bold";

#[derive(Clone, Copy, Default)]
struct Installed {
    consolas: bool,
    mono_bold: bool,
    sans_bold: bool,
}

fn key() -> egui::Id {
    egui::Id::unique("native-font-roles")
}

pub(super) fn install_bundled(ctx: &egui::Context) {
    ctx.set_fonts(help::font_definitions());
    ctx.data_mut(|data| data.remove::<Installed>(key()));
}

pub(super) fn install_native(ctx: &egui::Context) -> Arc<NativeFonts> {
    let faces = Arc::new(NativeFonts::discover(
        FontPlatform::current(),
        polished_map_platform::message_font_family().as_deref(),
    ));
    install(ctx, &faces);
    faces
}

fn install(ctx: &egui::Context, faces: &NativeFonts) {
    let mut definitions = help::font_definitions();
    // Keep an explicit bundled stack even when a native bold role is missing;
    // otherwise it would inherit the newly installed regular UI/mono face.
    for (name, fallback) in [
        (MONO_BOLD, egui::FontFamily::Monospace),
        (SANS_BOLD, egui::FontFamily::Proportional),
    ] {
        definitions.families.insert(
            egui::FontFamily::Name(name.into()),
            definitions.families[&fallback].clone(),
        );
    }
    let installed = Installed {
        consolas: faces.is_consolas(),
        mono_bold: true,
        sans_bold: true,
    };
    for (role, family, face) in [
        ("UI", egui::FontFamily::Proportional, faces.ui.as_ref()),
        (
            "Monospace",
            egui::FontFamily::Monospace,
            faces.mono.as_ref(),
        ),
        (
            MONO_BOLD,
            egui::FontFamily::Name(MONO_BOLD.into()),
            faces.mono_bold.as_ref(),
        ),
        (
            SANS_BOLD,
            egui::FontFamily::Name(SANS_BOLD.into()),
            faces.sans_bold.as_ref(),
        ),
    ] {
        if let Some(face) = face {
            prepend(&mut definitions, role, family, face);
            eprintln!(
                "Native font {role}: {} (face {})",
                face.family(),
                face.index()
            );
            if let Some(reference) = face.substituted_for() {
                eprintln!(
                    "Native font {role}: substitution for {reference}; GDI synthetic bold parity remains incomplete"
                );
            }
        } else {
            eprintln!(
                "Native font {role}: unavailable; retaining bundled fallback (native font parity incomplete)"
            );
        }
    }
    ctx.set_fonts(definitions);
    ctx.data_mut(|data| data.insert_temp(key(), installed));
    let size = if cfg!(windows) { 12.0 } else { 13.0 };
    ctx.all_styles_mut(|style| {
        for text_style in [egui::TextStyle::Body, egui::TextStyle::Button] {
            style
                .text_styles
                .insert(text_style, egui::FontId::proportional(size));
        }
    });
}

fn prepend(
    definitions: &mut egui::FontDefinitions,
    role: &str,
    family: egui::FontFamily,
    face: &NativeFontFace,
) {
    let mut data = egui::FontData::from_blob(face.data(), face.index());
    data.tweak.coords = egui::epaint::text::VariationCoords::new(
        face.variations().iter().map(|(tag, value)| (tag, *value)),
    );
    let name = format!("Native {role}");
    definitions.font_data.insert(name.clone(), Arc::new(data));
    let fallback = if role == MONO_BOLD {
        egui::FontFamily::Monospace
    } else {
        egui::FontFamily::Proportional
    };
    let fallback = definitions.families[&fallback].clone();
    let stack = definitions.families.entry(family).or_insert(fallback);
    stack.insert(0, name);
}

fn installed(ctx: &egui::Context) -> Installed {
    ctx.data(|data| data.get_temp(key())).unwrap_or_default()
}

pub(super) fn mono_bold(ctx: &egui::Context, size: f32) -> egui::FontId {
    if installed(ctx).mono_bold {
        egui::FontId::new(size, egui::FontFamily::Name(MONO_BOLD.into()))
    } else {
        egui::FontId::monospace(size)
    }
}

pub(super) fn sans_bold(ctx: &egui::Context, size: f32) -> egui::FontId {
    if installed(ctx).sans_bold {
        egui::FontId::new(size, egui::FontFamily::Name(SANS_BOLD.into()))
    } else {
        egui::FontId::proportional(size)
    }
}

pub(super) fn mono_size(ctx: &egui::Context, normal: f32) -> f32 {
    normal + if installed(ctx).consolas { 1.0 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_sizes_and_bold_roles_do_not_leak_between_contexts() {
        let native = egui::Context::default();
        native.data_mut(|data| {
            data.insert_temp(
                key(),
                Installed {
                    consolas: true,
                    mono_bold: true,
                    sans_bold: true,
                },
            )
        });
        assert_eq!(mono_size(&native, 9.0), 10.0);
        assert_eq!(mono_size(&native, 11.0), 12.0);
        assert_eq!(
            mono_bold(&native, 14.0),
            egui::FontId::new(14.0, egui::FontFamily::Name(MONO_BOLD.into()))
        );
        assert_eq!(
            sans_bold(&native, 24.0),
            egui::FontId::new(24.0, egui::FontFamily::Name(SANS_BOLD.into()))
        );
        let fixture = egui::Context::default();
        assert_eq!(mono_size(&fixture, 9.0), 9.0);
        assert_eq!(mono_size(&fixture, 11.0), 11.0);
        assert_eq!(mono_bold(&fixture, 14.0), egui::FontId::monospace(14.0));
        assert_eq!(sans_bold(&fixture, 24.0), egui::FontId::proportional(24.0));
        install_bundled(&native);
        assert_eq!(mono_size(&native, 9.0), 9.0);
        assert_eq!(mono_bold(&native, 14.0), egui::FontId::monospace(14.0));
    }

    #[test]
    #[ignore = "reads host-installed fonts; run in the native OS CI matrix"]
    fn installed_native_fonts_reach_egui_and_preserve_symbol_fallback() {
        let mut h = egui_kittest::Harness::builder().build_eframe(|cc| Editor::new_native(cc));
        h.run();
        assert!(
            installed(&h.ctx).mono_bold,
            "desktop startup must install the dedicated label family"
        );
        let size = if cfg!(windows) { 12.0 } else { 13.0 };
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            assert_eq!(
                h.ctx.style_of(theme).text_styles[&egui::TextStyle::Body].size,
                size
            );
        }
        h.ctx.fonts(|fonts| {
            let definitions = fonts.definitions();
            for (family, name) in [
                (egui::FontFamily::Proportional, "Native UI"),
                (egui::FontFamily::Monospace, "Native Monospace"),
                (
                    egui::FontFamily::Name(MONO_BOLD.into()),
                    "Native Reference Courier Bold",
                ),
            ] {
                assert_eq!(definitions.families[&family][0], name);
                assert!(definitions.font_data.contains_key(name));
            }
        });
        for (font, text) in [
            (egui::FontId::proportional(size), "Map: 999 x 999"),
            (egui::FontId::monospace(mono_size(&h.ctx, 9.0)), "254"),
            (mono_bold(&h.ctx, 14.0), "FF"),
            (sans_bold(&h.ctx, 24.0), "W"),
            (
                egui::FontId::new(13.0, egui::FontFamily::Name("Menu Symbols".into())),
                "⌘⌥⇧",
            ),
        ] {
            let galley = h.ctx.fonts_mut(|fonts| {
                assert!(fonts.has_glyphs(&font, text), "missing glyphs: {text}");
                fonts.layout_no_wrap(text.into(), font, Color32::WHITE)
            });
            assert!(galley.size().x.is_finite() && galley.size().x > 0.0);
            assert!(galley.size().y.is_finite() && galley.size().y > 0.0);
        }
    }
}
