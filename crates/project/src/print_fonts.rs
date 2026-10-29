//! CPU font adapter for master's Courier Bold 14 / Helvetica Bold 14 requests.
//! Native fonts remain installed on the host; no OS font data is redistributed.
//! Skrifa/Vello coverage does not claim identical GDI/Xft/Quartz output.
use polished_map_core::{PrintError, PrintFont, PrintTextMask};
use skrifa::{
    FontRef, MetadataProvider,
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
};
use vello_cpu::{color, kurbo};

const SIZE: f32 = 14.0;
const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;

/// An owned CPU font face. Callers supplying data are responsible for choosing
/// the intended bold face and respecting that font's license.
pub struct PrintFontFace {
    bytes: Vec<u8>,
    index: u32,
    name: String,
}

impl PrintFontFace {
    /// Accept TTF/OTF data or a selected face of a collection. No filesystem or
    /// display is accessed. Oversized/invalid data and unsupported metrics fail.
    pub fn from_bytes(bytes: &[u8], collection_index: u32) -> Result<Self, PrintError> {
        if bytes.is_empty() || bytes.len() > MAX_FONT_BYTES {
            return Err(PrintError::Text("font data must be 1..=32 MiB".into()));
        }
        let font = FontRef::from_index(bytes, collection_index).map_err(font_error)?;
        let name = font
            .localized_strings(skrifa::string::StringId::FULL_NAME)
            .english_or_first()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unnamed font".into());
        line_metrics(&font)?;
        Ok(Self {
            bytes: bytes.to_vec(),
            index: collection_index,
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn rasterize(&self, text: &str) -> Result<PrintTextMask, PrintError> {
        let font = FontRef::from_index(&self.bytes, self.index).map_err(font_error)?;
        let (baseline, line_height) = line_metrics(&font)?;
        let charmap = font.charmap();
        let metrics = font.glyph_metrics(Size::new(SIZE), LocationRef::default());
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
                .draw(
                    DrawSettings::unhinted(Size::new(SIZE), LocationRef::default()),
                    &mut pen,
                )
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

fn line_metrics(font: &FontRef<'_>) -> Result<(i32, i32), PrintError> {
    let metrics = font.metrics(Size::new(SIZE), LocationRef::default());
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

    /// Discover installed bold faces. Ordered fallbacks are explicit; Fontique
    /// enumerates fonts through DirectWrite, Core Text or Fontconfig.
    /// Failure is reported instead of silently printing missing-glyph boxes.
    pub fn system() -> Result<Self, PrintError> {
        let mut db = fontique::Collection::new(fontique::CollectionOptions::default());
        #[cfg(target_os = "windows")]
        let (mono, sans) = (
            [
                "Courier New",
                "Courier",
                "Liberation Mono",
                "DejaVu Sans Mono",
            ],
            ["Arial", "Helvetica", "Liberation Sans", "DejaVu Sans"],
        );
        #[cfg(not(target_os = "windows"))]
        let (mono, sans) = (
            [
                "Courier",
                "Courier New",
                "Liberation Mono",
                "DejaVu Sans Mono",
            ],
            ["Helvetica", "Arial", "Liberation Sans", "DejaVu Sans"],
        );
        Ok(Self::from_faces(
            select(&mut db, &mono)?,
            select(&mut db, &sans)?,
        ))
    }

    pub fn face_name(&self, requested: PrintFont) -> &str {
        self.face(requested).name()
    }

    fn face(&self, requested: PrintFont) -> &PrintFontFace {
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

fn select(db: &mut fontique::Collection, names: &[&str]) -> Result<PrintFontFace, PrintError> {
    for name in names {
        let Some(family) = db.family_by_name(name) else {
            continue;
        };
        let Some(face) = family.match_font(
            fontique::FontWidth::NORMAL,
            fontique::FontStyle::Normal,
            fontique::FontWeight::BOLD,
            false,
        ) else {
            continue;
        };
        if face.weight().value() < 600.0 || face.style() != fontique::FontStyle::Normal {
            continue;
        }
        if let Some(data) = face.load(None)
            && let Ok(font) = PrintFontFace::from_bytes(data.as_ref(), face.index())
        {
            return Ok(font);
        }
    }
    Err(PrintError::Text(format!(
        "no usable installed bold font: {}; install one or provide explicit font data",
        names.join(", ")
    )))
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
        assert!(
            select(
                &mut fontique::Collection::new(fontique::CollectionOptions {
                    system_fonts: false,
                    ..Default::default()
                }),
                &["Courier"]
            )
            .is_err()
        );
    }

    #[test]
    #[ignore = "uses installed system fonts; run explicitly on each supported host"]
    fn installed_bold_faces_render_every_required_glyph() {
        let fonts = PrintFonts::system().unwrap();
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
