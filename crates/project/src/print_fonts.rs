//! CPU font adapter for master's Courier Bold 14 / Helvetica Bold 14 requests.
//! Native fonts remain installed on the host; no OS font data is redistributed.
//! Skrifa/Vello coverage does not claim identical GDI/Xft/Quartz output.
use crate::{FontPlatform, NativeFontFace, NativeFonts};
use polished_map_core::{PrintError, PrintFont, PrintTextMask};
use skrifa::{
    FontRef, MetadataProvider,
    instance::{Location, LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
};
use std::sync::Arc;
use vello_cpu::{color, kurbo};

const SIZE: f32 = 14.0;
const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;

#[cfg(test)]
pub(crate) mod variable_fixture;

/// An owned CPU font face. Callers supplying data are responsible for choosing
/// the intended bold face and respecting that font's license.
pub struct PrintFontFace {
    bytes: Arc<dyn AsRef<[u8]> + Send + Sync>,
    index: u32,
    name: String,
    variations: Vec<([u8; 4], f32)>,
    location: Location,
    substituted_for: Option<String>,
}

impl PrintFontFace {
    /// Accept TTF/OTF data or a selected face of a collection. No filesystem or
    /// display is accessed. Oversized/invalid data and unsupported metrics fail.
    pub fn from_bytes(bytes: &[u8], collection_index: u32) -> Result<Self, PrintError> {
        Self::from_bytes_with_variations(bytes, collection_index, &[])
    }

    /// Explicit variable-font instance. Reject unknown, duplicate, non-finite or
    /// out-of-range settings instead of silently clamping/ignoring them. At most
    /// 32 settings are accepted; omitted axes retain their default coordinates.
    pub fn from_bytes_with_variations(
        bytes: &[u8],
        collection_index: u32,
        variations: &[([u8; 4], f32)],
    ) -> Result<Self, PrintError> {
        if bytes.is_empty() || bytes.len() > MAX_FONT_BYTES {
            return Err(PrintError::Text("font data must be 1..=32 MiB".into()));
        }
        Self::from_data(Arc::new(bytes.to_vec()), collection_index, variations, None)
    }

    /// Use exactly the selected GUI face, preserving collection index, variable
    /// coordinates and substitution diagnostics without duplicating its bytes.
    pub fn from_native(face: &NativeFontFace) -> Result<Self, PrintError> {
        Self::from_data(
            face.data(),
            face.index(),
            face.variations(),
            face.substituted_for(),
        )
    }

    fn from_data(
        bytes: Arc<dyn AsRef<[u8]> + Send + Sync>,
        collection_index: u32,
        variations: &[([u8; 4], f32)],
        substituted_for: Option<&str>,
    ) -> Result<Self, PrintError> {
        let font =
            FontRef::from_index(bytes.as_ref().as_ref(), collection_index).map_err(font_error)?;
        if variations.len() > 32 {
            return Err(PrintError::Text(
                "at most 32 font variation settings are supported".into(),
            ));
        }
        let axes = font.axes();
        for (i, (tag, value)) in variations.iter().enumerate() {
            let axis = axes
                .get_by_tag(skrifa::Tag::new(tag))
                .ok_or_else(|| PrintError::Text("unknown font variation axis".into()))?;
            if !value.is_finite()
                || !(axis.min_value()..=axis.max_value()).contains(value)
                || variations[..i].iter().any(|(previous, _)| previous == tag)
            {
                return Err(PrintError::Text(
                    "invalid or duplicate font variation setting".into(),
                ));
            }
        }
        let location = axes.location(
            variations
                .iter()
                .map(|(tag, value)| (skrifa::Tag::new(tag), *value)),
        );
        let name = font
            .localized_strings(skrifa::string::StringId::FULL_NAME)
            .english_or_first()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unnamed font".into());
        line_metrics(&font, location.coords().into())?;
        Ok(Self {
            bytes,
            index: collection_index,
            name,
            variations: variations.to_vec(),
            location,
            substituted_for: substituted_for.map(str::to_owned),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variations(&self) -> &[([u8; 4], f32)] {
        &self.variations
    }

    pub fn collection_index(&self) -> u32 {
        self.index
    }

    pub fn substituted_for(&self) -> Option<&str> {
        self.substituted_for.as_deref()
    }

    fn rasterize(&self, text: &str) -> Result<PrintTextMask, PrintError> {
        let font =
            FontRef::from_index(self.bytes.as_ref().as_ref(), self.index).map_err(font_error)?;
        let location = self.location.coords();
        let (baseline, line_height) = line_metrics(&font, location.into())?;
        let charmap = font.charmap();
        let metrics = font.glyph_metrics(Size::new(SIZE), location);
        let outlines = font.outline_glyphs();
        let mut pen = TextPen {
            path: kurbo::BezPath::new(),
            advance: 0.0,
            baseline: f64::from(baseline),
            invalid: false,
        };
        for ch in text.chars() {
            let id = charmap
                .map(ch)
                .ok_or_else(|| PrintError::Text(format!("{} has no glyph for {ch}", self.name)))?;
            let advance = metrics
                .advance_width(id)
                .ok_or_else(|| PrintError::Text("font has no glyph advance".into()))?;
            if !advance.is_finite() || !(1.0..=128.0).contains(&advance) {
                return Err(PrintError::Text(
                    "font glyph metrics exceed Print limits".into(),
                ));
            }
            let outline = outlines
                .get(id)
                .ok_or_else(|| PrintError::Text("font has no glyph outline".into()))?;
            outline
                .draw(DrawSettings::unhinted(Size::new(SIZE), location), &mut pen)
                .map_err(font_error)?;
            // Whole-pixel label placement; native hinting/advance equivalence
            // remains a platform verification gate, not an implied guarantee.
            pen.advance += f64::from(advance.round());
        }
        if pen.invalid || pen.path.is_empty() || !(1.0..=128.0).contains(&pen.advance) {
            return Err(PrintError::Text("font outline exceeds Print limits".into()));
        }
        let bounds = pen.path.control_box().expand();
        let (width, height) = (bounds.width(), bounds.height());
        if !(1.0..=128.0).contains(&width)
            || !(1.0..=64.0).contains(&height)
            || !(-128.0..=128.0).contains(&bounds.x0)
            || !(-64.0..=64.0).contains(&bounds.y0)
        {
            return Err(PrintError::Text(
                "font ink bounds exceed Print limits".into(),
            ));
        }
        let mut raster = vello_cpu::RenderContext::new(width as u16, height as u16);
        raster.set_transform(kurbo::Affine::translate((-bounds.x0, -bounds.y0)));
        raster.set_paint(color::OpaqueColor::<color::Srgb>::WHITE);
        raster.fill_path(&pen.path);
        let mut pixels = vello_cpu::Pixmap::new(width as u16, height as u16);
        raster.render(&mut pixels, &mut vello_cpu::Resources::new());
        let coverage = pixels
            .data_as_u8_slice()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|rgba| rgba[3])
            .collect();
        PrintTextMask::with_layout(
            width as u16,
            height as u16,
            coverage,
            [pen.advance as u16, line_height as u16],
            [bounds.x0 as i16, bounds.y0 as i16],
        )
    }
}

fn font_error(error: impl std::fmt::Display) -> PrintError {
    PrintError::Text(error.to_string())
}

fn line_metrics(font: &FontRef<'_>, location: LocationRef<'_>) -> Result<(i32, i32), PrintError> {
    let metrics = font.metrics(Size::new(SIZE), location);
    let ascent = metrics.ascent.ceil();
    let height = ascent - metrics.descent.floor();
    if !ascent.is_finite()
        || !height.is_finite()
        || !(1.0..=64.0).contains(&height)
        || !(0.0..=64.0).contains(&ascent)
    {
        return Err(PrintError::Text(
            "font line metrics exceed Print limits".into(),
        ));
    }
    Ok((ascent as i32, height as i32))
}

// Translate outlines from a baseline with Y-up to an image with Y-down. Keep
// negative bearings; reject excessive geometry before raster allocation.
struct TextPen {
    path: kurbo::BezPath,
    advance: f64,
    baseline: f64,
    invalid: bool,
}
impl TextPen {
    fn point(&mut self, x: f32, y: f32) -> kurbo::Point {
        if !x.is_finite()
            || !y.is_finite()
            || !(-128.0..=128.0).contains(&x)
            || !(-64.0..=64.0).contains(&y)
        {
            self.invalid = true;
            return kurbo::Point::ZERO;
        }
        kurbo::Point::new(f64::from(x) + self.advance, self.baseline - f64::from(y))
    }
    fn push(&mut self, element: kurbo::PathEl) {
        if self.path.elements().len() >= 4096 {
            self.invalid = true;
        }
        if !self.invalid {
            self.path.push(element);
        }
    }
}
impl OutlinePen for TextPen {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.push(kurbo::PathEl::MoveTo(p));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.push(kurbo::PathEl::LineTo(p));
    }
    fn quad_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        let a = self.point(x0, y0);
        let b = self.point(x1, y1);
        self.push(kurbo::PathEl::QuadTo(a, b));
    }
    fn curve_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        let a = self.point(x0, y0);
        let b = self.point(x1, y1);
        let c = self.point(x2, y2);
        self.push(kurbo::PathEl::CurveTo(a, b, c));
    }
    fn close(&mut self) {
        self.push(kurbo::PathEl::ClosePath);
    }
}

/// Reusable faces for both GUI and autonomous/headless Print. No window,
/// graphics API or display server is required; discovery can run on a worker.
pub struct PrintFonts {
    courier: PrintFontFace,
    helvetica: PrintFontFace,
}

impl PrintFonts {
    pub fn from_faces(courier: PrintFontFace, helvetica: PrintFontFace) -> Self {
        Self { courier, helvetica }
    }

    /// Discover the same bold faces as desktop startup. Ordered preferences and
    /// explicit substitutions are owned by NativeFonts, not duplicated here.
    /// Failure is reported instead of silently printing missing-glyph boxes.
    pub fn system() -> Result<Self, PrintError> {
        Self::from_native(&NativeFonts::discover(FontPlatform::current(), None))
    }

    /// Supply an already selected policy result (e.g. the editor's startup
    /// selection). Both required roles must be available before output begins.
    pub fn from_native(fonts: &NativeFonts) -> Result<Self, PrintError> {
        let mono = fonts.mono_bold.as_ref().ok_or_else(|| {
            PrintError::Text(
                "no usable installed bold monospace font; supply explicit font data".into(),
            )
        })?;
        let sans = fonts.sans_bold.as_ref().ok_or_else(|| {
            PrintError::Text("no usable installed bold sans font; supply explicit font data".into())
        })?;
        Ok(Self::from_faces(
            PrintFontFace::from_native(mono)?,
            PrintFontFace::from_native(sans)?,
        ))
    }

    pub fn face_name(&self, requested: PrintFont) -> &str {
        self.face(requested).name()
    }

    pub fn face(&self, requested: PrintFont) -> &PrintFontFace {
        match requested {
            PrintFont::CourierBold14 => &self.courier,
            PrintFont::HelveticaBold14 => &self.helvetica,
        }
    }

    /// Only source Print labels are accepted: 1..=3 hexadecimal ASCII digits or
    /// a single uppercase event symbol. Reject arbitrary/unbounded shaping work.
    pub fn rasterize(&self, requested: PrintFont, text: &str) -> Result<PrintTextMask, PrintError> {
        let valid = match requested {
            PrintFont::CourierBold14 => {
                (1..=3).contains(&text.len())
                    && text
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b))
            }
            PrintFont::HelveticaBold14 => {
                text.len() == 1 && text.as_bytes()[0].is_ascii_uppercase()
            }
        };
        if !valid {
            return Err(PrintError::Text("unsupported Print label".into()));
        }
        self.face(requested).rasterize(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fonts() -> PrintFonts {
        // Existing dependency test faces, not substitutions shipped as bold fonts.
        PrintFonts::from_faces(
            PrintFontFace::from_bytes(epaint_default_fonts::HACK_REGULAR, 0).unwrap(),
            PrintFontFace::from_bytes(epaint_default_fonts::UBUNTU_LIGHT, 0).unwrap(),
        )
    }

    #[test]
    fn all_source_labels_render_without_a_display_and_retain_line_alignment() {
        let fonts = fonts();
        for id in 0..=255u8 {
            for text in [id.to_string(), format!("{id:02X}")] {
                let mask = fonts.rasterize(PrintFont::CourierBold14, &text).unwrap();
                assert!(mask.coverage().iter().any(|&a| a > 0));
                assert!(mask.coverage().iter().any(|&a| a > 0 && a < 255));
                assert_eq!(mask.layout_size()[0], 8 * text.len() as u16);
                assert_eq!(mask.layout_size()[1], 17);
            }
        }
        for kind in polished_map_core::EventKind::ALL {
            let mask = fonts
                .rasterize(PrintFont::HelveticaBold14, &kind.symbol().to_string())
                .unwrap();
            assert!(mask.coverage().iter().any(|&a| a > 0));
            assert_eq!(mask.layout_size()[1], 17);
            assert!(mask.offset()[1] >= 0);
        }
    }

    #[test]
    fn malformed_faces_and_unsupported_labels_return_errors() {
        for bytes in [&[][..], b"not a font", &[0; 512]] {
            assert!(PrintFontFace::from_bytes(bytes, 0).is_err());
        }
        assert!(PrintFontFace::from_bytes(epaint_default_fonts::HACK_REGULAR, u32::MAX).is_err());
        let fonts = fonts();
        for text in ["", "1234", "fff", "12\n", "W", "💾"] {
            assert!(fonts.rasterize(PrintFont::CourierBold14, text).is_err());
        }
        for text in ["", "WW", "w", "1", "💾"] {
            assert!(fonts.rasterize(PrintFont::HelveticaBold14, text).is_err());
        }
    }

    #[test]
    fn variable_instance_changes_both_ink_and_advance_and_rejects_bad_settings() {
        let data = variable_fixture::variable_hack();
        let normal = PrintFontFace::from_bytes(&data, 0).unwrap();
        let bold =
            PrintFontFace::from_bytes_with_variations(&data, 0, &[(*b"wght", 700.0)]).unwrap();
        let normal_a = normal.rasterize("A").unwrap();
        let bold_a = bold.rasterize("A").unwrap();
        // Independent expectations from the fixture's +500-unit advance and
        // +100-unit outline translation at its 2048-unit em and size 14.
        assert_eq!(normal_a.layout_size()[0], 8);
        assert_eq!(bold_a.layout_size()[0], 12);
        assert_ne!(normal_a.coverage(), bold_a.coverage());
        assert_eq!(bold.variations(), &[(*b"wght", 700.0)]);
        assert_eq!(bold.location.coords()[0].to_f32(), 1.0);
        assert_eq!(normal.rasterize("B").unwrap(), bold.rasterize("B").unwrap());
        for settings in [
            vec![(*b"wght", 399.0)],
            vec![(*b"wght", 701.0)],
            vec![(*b"wght", f32::NAN)],
            vec![(*b"wght", f32::INFINITY)],
            vec![(*b"wdth", 100.0)],
            vec![(*b"wght", 400.0), (*b"wght", 700.0)],
            vec![(*b"wght", 400.0); 33],
        ] {
            assert!(PrintFontFace::from_bytes_with_variations(&data, 0, &settings).is_err());
        }
        assert!(
            PrintFontFace::from_bytes_with_variations(
                epaint_default_fonts::HACK_REGULAR,
                0,
                &[(*b"wght", 700.0)]
            )
            .is_err()
        );
    }

    #[test]
    #[ignore = "uses installed system fonts; run explicitly on each supported host"]
    fn installed_bold_faces_render_every_required_glyph() {
        let fonts = PrintFonts::system().unwrap();
        let selected = NativeFonts::discover(FontPlatform::current(), None);
        let captured = PrintFonts::from_native(&selected).unwrap();
        for (role, source) in [
            (
                PrintFont::CourierBold14,
                selected.mono_bold.as_ref().unwrap(),
            ),
            (
                PrintFont::HelveticaBold14,
                selected.sans_bold.as_ref().unwrap(),
            ),
        ] {
            assert!(Arc::ptr_eq(&captured.face(role).bytes, &source.data()));
            assert_eq!(captured.face(role).collection_index(), source.index());
            assert_eq!(captured.face(role).variations(), source.variations());
            assert_eq!(fonts.face_name(role), captured.face_name(role));
            assert_eq!(fonts.face(role).substituted_for(), source.substituted_for());
        }
        eprintln!(
            "Print fonts: {}, {}",
            fonts.face_name(PrintFont::CourierBold14),
            fonts.face_name(PrintFont::HelveticaBold14)
        );
        for text in ["012", "345", "678", "9AB", "CDE", "FF"] {
            fonts.rasterize(PrintFont::CourierBold14, text).unwrap();
        }
        for kind in polished_map_core::EventKind::ALL {
            fonts
                .rasterize(PrintFont::HelveticaBold14, &kind.symbol().to_string())
                .unwrap();
        }
    }
}
