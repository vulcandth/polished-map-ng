//! Shared drawing utility from pinned utils.cpp::draw_outlined_text. LGPL-3.0-only.
//! File/path/ASM policies stay in their project owners rather than a catch-all module.
use eframe::egui::{Align2, Color32, FontId, Painter, Pos2, Vec2};

/// Paint four diagonal outline offsets before the foreground text.
pub(super) fn draw_outlined_text(
    painter: &Painter,
    position: Pos2,
    align: Align2,
    text: &str,
    font: FontId,
    foreground: Color32,
    outline: Color32,
) {
    for delta in [
        Vec2::new(-1.0, -1.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, 1.0),
    ] {
        painter.text(position + delta, align, text, font.clone(), outline);
    }
    painter.text(position, align, text, font, foreground);
}
