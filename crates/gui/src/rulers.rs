//! Ruler layout/drawing adapted from Rangi42's pinned master `ruler.cpp`.
//! LGPL-3.0-only. Logical geometry matches integer FLTK positions; fractional
//! egui scrolling translates the same cells without rounding away alignment.
use super::*;

#[derive(Debug, PartialEq)]
struct Cell {
    start: f32,
    coordinate: i32,
    full_tick: bool,
    highlighted: bool,
}

fn cells(length: f32, scroll: f32, size: f32, hovered: Option<u16>, blocks: bool) -> Vec<Cell> {
    let first = (scroll / size).floor() as i32;
    let mut cells = Vec::new();
    // Pixel rounding may put an unscrolled content origin slightly *after*
    // the viewport origin. Floor division must pair with a positive remainder
    // in that case; Rust's signed `%` would mislabel the first cell by one.
    let mut start = first as f32 * size - scroll;
    let mut coordinate = first - 6;
    while start < length {
        cells.push(Cell {
            start,
            coordinate,
            full_tick: coordinate % 2 != 0,
            highlighted: hovered.is_some_and(|hovered| {
                let hovered = i32::from(hovered);
                coordinate == hovered || blocks && (coordinate & !1) == (hovered & !1)
            }),
        });
        start += size;
        coordinate += 1;
    }
    cells
}

fn label(coordinate: i32, hex: bool) -> String {
    if !hex {
        coordinate.to_string()
    } else if coordinate < 0 {
        // ruler.cpp adds a trailing newline, but FLTK 1.4.5 fl_draw.cxx consumes
        // it without adding an empty line. egui would render an extra line, so
        // omit it here to preserve the original visible text and baseline.
        format!("-{:02X}", -coordinate)
    } else {
        format!("{coordinate:02X}")
    }
}

struct Axis {
    rect: Rect,
    vertical: bool,
    scroll: f32,
    hovered: Option<u16>,
    maximum: i32,
}

fn maximum(dimension: Option<u8>) -> i32 {
    // master Map::max_event_x/y; the remaining ticks still span the viewport.
    dimension.map_or(i32::MAX, |n| (i32::from(n) * 2 + 5).min(249))
}

impl Editor {
    pub(super) fn draw_rulers(&self, ui: &egui::Ui, layout: &geometry::Layout, origin: Pos2) {
        let Some(rects) = layout.rulers else { return };
        let rect = |r: geometry::Rect| {
            Rect::from_min_size(origin + Vec2::new(r.x, r.y), Vec2::new(r.width, r.height))
        };
        let dimensions = self.loaded.as_ref().map(|l| {
            let map = l.document.map();
            (map.width(), map.height())
        });
        // Use the application's existing light/dark palette until full native
        // FLTK theme reproduction is implemented. Geometry is independent of it.
        let foreground = ui.visuals().text_color();
        let background = ui.visuals().panel_fill;
        let fill = ui.visuals().widgets.inactive.bg_fill;
        let edge = Color32::from_rgb(
            (f32::from(foreground.r()) * 0.4 + f32::from(background.r()) * 0.6) as u8,
            (f32::from(foreground.g()) * 0.4 + f32::from(background.g()) * 0.6) as u8,
            (f32::from(foreground.b()) * 0.4 + f32::from(background.b()) * 0.6) as u8,
        );
        let corner = rect(rects.corner);
        let mut mesh = egui::Mesh::default();
        mesh.add_colored_rect(corner, fill);
        mesh.add_colored_rect(
            Rect::from_min_max(corner.left_bottom() - Vec2::Y, corner.right_bottom()),
            edge,
        );
        mesh.add_colored_rect(
            Rect::from_min_max(corner.right_top() - Vec2::X, corner.right_bottom()),
            edge,
        );
        ui.painter().add(egui::Shape::mesh(mesh));
        for axis in [
            Axis {
                rect: rect(rects.horizontal),
                vertical: false,
                scroll: self.ruler_scroll.x,
                hovered: self.ruler_cursor.map(|c| c.event_x),
                maximum: maximum(dimensions.map(|d| d.0)),
            },
            Axis {
                rect: rect(rects.vertical),
                vertical: true,
                scroll: self.ruler_scroll.y,
                hovered: self.ruler_cursor.map(|c| c.event_y),
                maximum: maximum(dimensions.map(|d| d.1)),
            },
        ] {
            let size = layout.block_size / 2.0;
            let (length, thickness) = if axis.vertical {
                (axis.rect.height(), axis.rect.width())
            } else {
                (axis.rect.width(), axis.rect.height())
            };
            let transform = |along: f32, across: f32, width: f32, height: f32| {
                if axis.vertical {
                    Rect::from_min_size(
                        axis.rect.min + Vec2::new(across, along),
                        Vec2::new(height, width),
                    )
                } else {
                    Rect::from_min_size(
                        axis.rect.min + Vec2::new(along, across),
                        Vec2::new(width, height),
                    )
                }
            };
            let painter = ui
                .painter()
                .with_clip_rect(ui.clip_rect().intersect(axis.rect));
            let cells = cells(length, axis.scroll, size, axis.hovered, !self.event_ui.mode);
            let mut mesh = egui::Mesh::default();
            mesh.add_colored_rect(axis.rect, fill);
            mesh.add_colored_rect(transform(0.0, thickness - 1.0, length, 1.0), edge);
            for cell in &cells {
                let depth = if cell.full_tick { 0.0 } else { thickness / 2.0 };
                mesh.add_colored_rect(
                    transform(cell.start + size - 1.0, depth, 1.0, thickness - depth),
                    edge,
                );
                if cell.coordinate <= axis.maximum && cell.highlighted {
                    mesh.add_colored_rect(
                        transform(cell.start, 0.0, size - 1.0, thickness - 1.0),
                        foreground,
                    );
                }
            }
            painter.add(egui::Shape::mesh(mesh));
            for cell in cells.iter().filter(|c| c.coordinate <= axis.maximum) {
                // FL_ALIGN_BOTTOM_RIGHT | FL_ALIGN_INSIDE | FL_ALIGN_CLIP.
                let mut text_rect = transform(cell.start, 0.0, size, thickness);
                text_rect.max.x -= 2.0;
                painter
                    .with_clip_rect(painter.clip_rect().intersect(text_rect))
                    .text(
                        text_rect.right_bottom(),
                        egui::Align2::RIGHT_BOTTOM,
                        label(cell.coordinate, self.hex),
                        egui::FontId::monospace(9.0),
                        if cell.highlighted {
                            background
                        } else {
                            foreground
                        },
                    );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn ruler_menu_shortcut_hover_modes_zoom_scroll_and_modal_guard() {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        assert!(!h.state().rulers);
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::R,
        );
        h.run();
        assert!(h.state().rulers);
        // Insets move the map origin by exactly the 16-point ruler thickness.
        h.hover_at(Pos2::new(280.0, 183.0));
        h.run();
        assert_eq!(h.state().ruler_scroll, Vec2::ZERO);
        assert_eq!(
            h.state().ruler_cursor.map(|c| (c.event_x, c.event_y)),
            Some((1, 1))
        );
        assert!(h.state().event_ui.cursor.is_none());
        #[cfg(feature = "visual-tests")]
        h.snapshot("rulers-blocks");
        h.key_press(egui::Key::Tab);
        h.run();
        assert_eq!(h.state().event_ui.cursor, h.state().ruler_cursor);
        #[cfg(feature = "visual-tests")]
        h.snapshot("rulers-events");
        h.state_mut().zoom = true;
        h.hover_at(Pos2::new(528.0, 303.0));
        h.run();
        assert_eq!(
            h.state().ruler_cursor.map(|c| (c.event_x, c.event_y)),
            Some((1, 1))
        );
        #[cfg(feature = "visual-tests")]
        h.snapshot("rulers-zoomed");
        h.state_mut().prepare_resize();
        h.run();
        assert!(h.state().ruler_cursor.is_none());
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::R,
        );
        h.run();
        assert!(h.state().rulers);
        h.get_by_label("Cancel").click();
        h.run();
        assert!(!h.state().is_modified());
        h.state_mut().zoom = false;
        h.state_mut().apply(Command::Resize {
            width: 20,
            height: 20,
            anchor: ResizeAnchor::TopLeft,
        });
        h.hover_at(Pos2::new(280.0, 183.0));
        h.run();
        h.event(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            delta: Vec2::new(0.0, -19.0),
            modifiers: egui::Modifiers::NONE,
        });
        h.run();
        assert!((h.state().ruler_scroll.y - 19.0).abs() < 0.001);
        assert_eq!(
            h.state().ruler_cursor.map(|c| (c.event_x, c.event_y)),
            Some((1, 2))
        );
        #[cfg(feature = "visual-tests")]
        h.snapshot("rulers-scrolled");
        h.event(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            delta: Vec2::new(-37.5, 0.0),
            modifiers: egui::Modifiers::NONE,
        });
        h.run();
        // egui rounds the content origin (160 - 37.5) to 123 logical points.
        // Follow the actual 37-point displacement, not the raw wheel delta.
        assert_eq!(h.state().ruler_scroll.x, 37.0);
        assert_eq!(
            h.state().ruler_cursor.map(|c| (c.event_x, c.event_y)),
            Some((3, 2))
        );
        h.hover_at(Pos2::new(160.0, 50.0));
        h.run();
        assert!(h.state().ruler_cursor.is_none());
        h.get_by_label("View").click();
        h.run();
        h.get_by_label("Rulers").click();
        h.run();
        assert!(!h.state().rulers);
    }

    #[test]
    fn empty_map_rulers_keep_ticks_without_a_hover_highlight() {
        let mut h = egui_kittest::Harness::builder()
            .with_size(Vec2::new(800.0, 600.0))
            .with_theme(egui::Theme::Dark)
            .build_eframe(|cc| Editor::new(cc));
        h.state_mut().rulers = true;
        h.run();
        assert_eq!(h.state().ruler_scroll, Vec2::ZERO);
        assert!(h.state().ruler_cursor.is_none());
        #[cfg(feature = "visual-tests")]
        h.snapshot("rulers-empty-dark");
    }

    #[test]
    fn source_tick_vectors_and_highlight_pairing() {
        let initial = cells(49.0, 0.0, 16.0, None, false);
        assert_eq!(
            initial
                .iter()
                .map(|c| (c.start, c.coordinate, c.full_tick))
                .collect::<Vec<_>>(),
            [
                (0.0, -6, false),
                (16.0, -5, true),
                (32.0, -4, false),
                (48.0, -3, true)
            ]
        );
        let scrolled = cells(33.0, 19.0, 16.0, None, false);
        assert_eq!(
            scrolled
                .iter()
                .map(|c| (c.start, c.coordinate, c.full_tick))
                .collect::<Vec<_>>(),
            [(-3.0, -5, true), (13.0, -4, false), (29.0, -3, true)]
        );
        for size in [16.0, 32.0] {
            for blocks in [true, false] {
                let cells = cells(size * 4.0, size * 6.0 + 0.5, size, Some(1), blocks);
                assert_eq!(cells[0].start, -0.5);
                assert_eq!(
                    cells
                        .iter()
                        .filter(|c| c.highlighted)
                        .map(|c| c.coordinate)
                        .collect::<Vec<_>>(),
                    if blocks { vec![0, 1] } else { vec![1] }
                );
            }
        }
        assert!(cells(0.0, 0.0, 16.0, None, false).is_empty());
        let rounded_origin = cells(17.0, -0.5, 16.0, None, false);
        assert_eq!(
            rounded_origin
                .iter()
                .map(|c| (c.start, c.coordinate))
                .collect::<Vec<_>>(),
            [(-15.5, -7), (0.5, -6), (16.5, -5)]
        );
        assert_eq!(label(-6, false), "-6");
        assert_eq!(label(-6, true), "-06");
        assert_eq!(label(249, true), "F9");
        for (dimension, limit) in [(1, 7), (2, 9), (121, 247), (122, 249), (255, 249)] {
            assert_eq!(maximum(Some(dimension)), limit);
        }
        assert_eq!(maximum(None), i32::MAX);
    }
}
