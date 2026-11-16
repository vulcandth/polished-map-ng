//! Game Boy screen guide: geometry and painting, not an emulator display.
//!
//! Adapted from pinned master/plusplus `gameboy-screen.cpp` / `gameboy-screen.h`
//! (`Game_Boy_Screen::draw` / `resize`) and `Main_Window::update_gameboy_screen`.
//! Revisions are recorded in upstream.lock.toml; LGPL-3.0-only.
//! The event workflow owns visibility/cursor state; this module owns the guide.
use crate::geometry::{EventCursor, Rect};
use eframe::egui::{self, Color32, Pos2, Vec2};

/// Guide rectangle in map-relative logical points; event cells are 16 or 32 points.
pub(crate) fn bounds(cursor: EventCursor, cell: f32) -> Rect {
    Rect {
        x: (f32::from(cursor.event_x) - 4.0) * cell,
        y: (f32::from(cursor.event_y) - 4.0) * cell,
        width: 10.0 * cell,
        height: 9.0 * cell,
    }
}

/// Paint using the caller's viewport clip and return the uncut logical rectangle.
pub(crate) fn draw(
    painter: &egui::Painter,
    origin: Pos2,
    cursor: EventCursor,
    cell: f32,
) -> egui::Rect {
    let r = bounds(cursor, cell);
    let area =
        egui::Rect::from_min_size(origin + Vec2::new(r.x, r.y), Vec2::new(r.width, r.height));
    // The original 320x288 embedded PNG is uniformly RGBA(224,255,0,64).
    // Recreate its flat fill; no source bitmap is included.
    painter.rect_filled(area, 0.0, Color32::from_rgba_unmultiplied(224, 255, 0, 64));
    painter.rect_stroke(area, 0.0, (1.0, Color32::BLACK), egui::StrokeKind::Inside);
    painter.rect_stroke(
        area.shrink(1.0),
        0.0,
        (1.0, Color32::WHITE),
        egui::StrokeKind::Inside,
    );
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guide_geometry_and_compatibility_entry_point_match_both_zoom_levels() {
        let cursor = EventCursor {
            block_x: 1,
            block_y: 0,
            event_x: 3,
            event_y: 1,
        };
        for cell in [16.0, 32.0] {
            let expected = Rect {
                x: -cell,
                y: -3.0 * cell,
                width: 10.0 * cell,
                height: 9.0 * cell,
            };
            assert_eq!(bounds(cursor, cell), expected);
            assert_eq!(cursor.game_boy_screen(cell), expected);
        }
    }

    #[test]
    fn drawing_preserves_fill_outlines_and_caller_clipping() {
        let ctx = egui::Context::default();
        let cursor = EventCursor {
            block_x: 0,
            block_y: 0,
            event_x: 1,
            event_y: 1,
        };
        let clip = egui::Rect::from_min_max(Pos2::new(80.0, 80.0), Pos2::new(200.0, 200.0));
        let area = egui::Rect::from_min_size(Pos2::new(52.0, 52.0), Vec2::new(160.0, 144.0));
        ctx.begin_pass(Default::default());
        let painter = ctx
            .layer_painter(egui::LayerId::background())
            .with_clip_rect(clip);
        assert_eq!(draw(&painter, Pos2::new(100.0, 100.0), cursor, 16.0), area);
        let mut output = ctx.end_pass();
        // This headless shape test intentionally does not upload the font texture.
        output.textures_delta.clear();
        assert_eq!(output.shapes.len(), 3);
        let rects: Vec<_> = output
            .shapes
            .iter()
            .map(|shape| {
                assert_eq!(shape.clip_rect, clip);
                let egui::Shape::Rect(rect) = &shape.shape else {
                    panic!("guide must use rectangle shapes")
                };
                rect
            })
            .collect();
        assert_eq!(rects[0].rect, area);
        assert_eq!(
            rects[0].fill,
            Color32::from_rgba_unmultiplied(224, 255, 0, 64)
        );
        assert_eq!(rects[1].rect, area);
        assert_eq!(rects[1].stroke, egui::Stroke::new(1.0, Color32::BLACK));
        assert_eq!(rects[2].rect, area.shrink(1.0));
        assert_eq!(rects[2].stroke, egui::Stroke::new(1.0, Color32::WHITE));
        for rect in &rects[1..] {
            assert_eq!(rect.stroke_kind, egui::StrokeKind::Inside);
        }
    }
}
