//! Optional Print overlays for headless render; defaults preserve plain export.
use super::*;
use polished_map_core::{Graphics, PrintFont, PrintOptions, PrintStyle, RgbImage};
use polished_map_project::{PrintFontFace, PrintFonts, event_script_path};

#[derive(Args, Default)]
pub(super) struct PrintArgs {
    #[arg(long)]
    print_grid: bool,
    #[arg(long)]
    print_ids: bool,
    #[arg(long)]
    print_priority: bool,
    #[arg(long)]
    print_events: bool,
    /// Has no effect without --print-events, matching upstream Print Options.
    #[arg(long)]
    print_warp_ids: bool,
    /// Hexadecimal block IDs; warp IDs retain each event's coordinate base.
    #[arg(long)]
    print_hex: bool,
    /// Use the source dark-theme grid color.
    #[arg(long)]
    print_dark: bool,
    /// Override automatic event-script discovery.
    #[arg(long, requires = "print_events")]
    events_file: Option<PathBuf>,
    /// Explicit bold mono font (TTF/OTF, first face); pair with --print-sans-font.
    #[arg(long, requires = "print_sans_font")]
    print_mono_font: Option<PathBuf>,
    /// Explicit bold sans font (TTF/OTF, first face); pair with --print-mono-font.
    #[arg(long, requires = "print_mono_font")]
    print_sans_font: Option<PathBuf>,
}

impl PrintArgs {
    pub(super) fn render(
        &self,
        graphics: &Graphics,
        doc: &MapDocument,
    ) -> Result<(RgbImage, Value), Failure> {
        let options = PrintOptions {
            grid: self.print_grid,
            ids: self.print_ids,
            priority: self.print_priority,
            events: self.print_events,
            warp_ids: self.print_warp_ids,
        };
        let events = if self.print_events {
            let path = if let Some(path) = &self.events_file {
                Some(path.clone())
            } else {
                let root = doc.metadata().root.as_deref().ok_or_else(|| {
                    Failure::new("root_required", "event discovery requires a project root")
                })?;
                let path = event_script_path(root, &doc.metadata().name)?;
                path.is_file().then_some(path)
            };
            path.map(|path| EventDocument::open(&path)).transpose()?
        } else {
            None
        };
        let needs_fonts = options.ids
            || events
                .as_ref()
                .is_some_and(|e| !e.script().events().entries().is_empty());
        let fonts = if needs_fonts {
            Some(match (&self.print_mono_font, &self.print_sans_font) {
                (Some(mono), Some(sans)) => PrintFonts::from_faces(
                    PrintFontFace::from_bytes(&read_bounded(mono, 32 * 1024 * 1024)?, 0)
                        .map_err(|e| Failure::new("print_font", e))?,
                    PrintFontFace::from_bytes(&read_bounded(sans, 32 * 1024 * 1024)?, 0)
                        .map_err(|e| Failure::new("print_font", e))?,
                ),
                _ => PrintFonts::system().map_err(|e| Failure::new("print_font", e))?,
            })
        } else {
            None
        };
        let style = PrintStyle {
            hexadecimal_ids: self.print_hex,
            grid_rgb: [if self.print_dark { 38 } else { 85 }; 3],
        };
        let pixels = graphics
            .render_print(
                doc.map(),
                events.as_ref().map(|e| e.script().events()),
                options,
                style,
                |font, text| {
                    fonts
                        .as_ref()
                        .ok_or_else(|| {
                            polished_map_core::PrintError::Text(
                                "font adapter not initialized".into(),
                            )
                        })?
                        .rasterize(font, text)
                },
            )
            .map_err(|e| Failure::new("print_render", e))?;
        let details = if options != PrintOptions::default() || self.print_hex || self.print_dark {
            json!({"options":options, "style":style, "event_script":events.as_ref().map(|e| e.source().path()), "fonts":fonts.as_ref().map(|f| json!({
                "courier":f.face_name(PrintFont::CourierBold14), "helvetica":f.face_name(PrintFont::HelveticaBold14), "rasterizer":"skrifa-vello-cpu"
            }))})
        } else {
            Value::Null
        };
        Ok((pixels, details))
    }
}
