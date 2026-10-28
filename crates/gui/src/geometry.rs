//! Formulas from pinned upstream main-window.cpp::update_layout and event.h.

pub const MENU_HEIGHT: f32 = 21.0;
pub const TOOLBAR_HEIGHT: f32 = 26.0;
pub const STATUS_HEIGHT: f32 = 23.0;
pub const MAP_MARGIN: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub struct Layout {
    pub sidebar: Rect,
    pub canvas: Rect,
    pub status: Rect,
    pub block_size: f32,
    pub rulers: Option<RulerRects>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RulerRects {
    pub horizontal: Rect,
    pub vertical: Rect,
    pub corner: Rect,
}

impl Layout {
    pub fn new(width: f32, height: f32, zoom: bool, rulers: bool, scrollbar: f32) -> Self {
        let block_size = if zoom { 64.0 } else { 32.0 };
        let top = MENU_HEIGHT + TOOLBAR_HEIGHT;
        let body = (height - top - STATUS_HEIGHT).max(0.0);
        let sidebar_width = 4.0 * block_size + scrollbar;
        let ruler = if rulers { scrollbar } else { 0.0 };
        Self {
            sidebar: Rect {
                x: 0.0,
                y: top,
                width: sidebar_width,
                height: body,
            },
            canvas: Rect {
                x: sidebar_width + ruler,
                y: top + ruler,
                width: (width - sidebar_width - ruler).max(0.0),
                height: (body - ruler).max(0.0),
            },
            status: Rect {
                x: 0.0,
                y: height - STATUS_HEIGHT,
                width,
                height: STATUS_HEIGHT,
            },
            block_size,
            rulers: rulers.then_some(RulerRects {
                horizontal: Rect {
                    x: sidebar_width + ruler,
                    y: top,
                    width: (width - sidebar_width - ruler).max(0.0),
                    height: ruler,
                },
                vertical: Rect {
                    x: sidebar_width,
                    y: top + ruler,
                    width: ruler,
                    height: (body - ruler).max(0.0),
                },
                corner: Rect {
                    x: sidebar_width,
                    y: top,
                    width: ruler,
                    height: ruler,
                },
            }),
        }
    }

    /// Convert content coordinates, including the three-block border, to map coordinates.
    pub fn hit(&self, x: f32, y: f32, width: u8, height: u8) -> Option<(u16, u16)> {
        let x = (x / self.block_size - MAP_MARGIN).floor();
        let y = (y / self.block_size - MAP_MARGIN).floor();
        if x.is_finite()
            && y.is_finite()
            && x >= 0.0
            && y >= 0.0
            && x < f32::from(width)
            && y < f32::from(height)
        {
            Some((x as u16, y as u16))
        } else {
            None
        }
    }
}

pub fn snap_to_pixel(point: f32, pixels_per_point: f32) -> f32 {
    (point * pixels_per_point).round() / pixels_per_point
}

/// master Map::block_under, Block::right_half/bottom_half and
/// Main_Window::update_event_cursor. Coordinates are relative to the map origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventCursor {
    pub block_x: u8,
    pub block_y: u8,
    pub event_x: u16,
    pub event_y: u16,
}
impl EventCursor {
    pub fn at(
        x: f32,
        y: f32,
        block_size: f32,
        width: u8,
        height: u8,
        event: Option<(i16, i16)>,
    ) -> Option<Self> {
        if !x.is_finite() || !y.is_finite() || !block_size.is_finite() || block_size <= 0.0 {
            return None;
        }
        // C++ integer division truncates toward zero: an event at -1 maps to
        // block 0, while an event at -2 is outside. Preserve this boundary quirk.
        let (bx, by) = event.map_or_else(
            || ((x / block_size).floor(), (y / block_size).floor()),
            |(x, y)| (f32::from(x / 2), f32::from(y / 2)),
        );
        if bx < 0.0 || by < 0.0 || bx >= f32::from(width) || by >= f32::from(height) {
            return None;
        }
        let (block_x, block_y) = (bx as u8, by as u8);
        Some(Self {
            block_x,
            block_y,
            event_x: u16::from(block_x) * 2 + u16::from(x >= (bx + 0.5) * block_size),
            event_y: u16::from(block_y) * 2 + u16::from(y >= (by + 0.5) * block_size),
        })
    }

    /// master Main_Window::update_gameboy_screen, in map-relative logical points.
    pub fn game_boy_screen(self, cell: f32) -> Rect {
        Rect {
            x: (f32::from(self.event_x) - 4.0) * cell,
            y: (f32::from(self.event_y) - 4.0) * cell,
            width: 10.0 * cell,
            height: 9.0 * cell,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_cursor_halves_margin_quirks_and_screen_geometry() {
        for size in [32.0, 64.0] {
            for (x, y, event, expected) in [
                (0.0, 0.0, None, Some((0, 0, 0, 0))),
                (0.5, 0.5, None, Some((0, 0, 1, 1))),
                (1.999, 0.999, None, Some((1, 0, 3, 1))),
                (2.0, 0.0, None, None),
                (-0.1, 0.0, None, None),
                (-0.1, 0.0, Some((-1, 0)), Some((0, 0, 0, 0))),
                (-1.0, 0.0, Some((-2, 0)), None),
                (0.0, 0.0, Some((4, 0)), None),
                (f32::NAN, 0.0, None, None),
            ] {
                let cursor = EventCursor::at(x * size, y * size, size, 2, 1, event);
                assert_eq!(
                    cursor.map(|c| (c.block_x, c.block_y, c.event_x, c.event_y)),
                    expected
                );
            }
        }
        let cursor = EventCursor::at(48.0, 16.0, 32.0, 2, 1, None).unwrap();
        assert_eq!(
            cursor.game_boy_screen(16.0),
            Rect {
                x: -16.0,
                y: -48.0,
                width: 160.0,
                height: 144.0
            }
        );
        assert_eq!(
            cursor.game_boy_screen(32.0),
            Rect {
                x: -32.0,
                y: -96.0,
                width: 320.0,
                height: 288.0
            }
        );
    }
    #[test]
    fn independent_baseline_rectangles() {
        let l = Layout::new(800.0, 600.0, false, false, 16.0);
        assert_eq!(
            l.sidebar,
            Rect {
                x: 0.0,
                y: 47.0,
                width: 144.0,
                height: 530.0
            }
        );
        assert_eq!(
            l.canvas,
            Rect {
                x: 144.0,
                y: 47.0,
                width: 656.0,
                height: 530.0
            }
        );
        let l = Layout::new(800.0, 600.0, true, true, 16.0);
        assert_eq!(
            l.rulers,
            Some(RulerRects {
                horizontal: Rect {
                    x: 288.0,
                    y: 47.0,
                    width: 512.0,
                    height: 16.0
                },
                vertical: Rect {
                    x: 272.0,
                    y: 63.0,
                    width: 16.0,
                    height: 514.0
                },
                corner: Rect {
                    x: 272.0,
                    y: 47.0,
                    width: 16.0,
                    height: 16.0
                },
            })
        );
        assert_eq!(
            l.canvas,
            Rect {
                x: 288.0,
                y: 63.0,
                width: 512.0,
                height: 514.0
            }
        );
    }
    #[test]
    fn hits_respect_margins_and_exclusive_edges() {
        let l = Layout::new(800.0, 600.0, false, false, 16.0);
        assert_eq!(l.hit(96.0, 96.0, 2, 1), Some((0, 0)));
        assert_eq!(l.hit(159.9, 127.9, 2, 1), Some((1, 0)));
        assert_eq!(l.hit(160.0, 127.9, 2, 1), None);
        assert_eq!(l.hit(95.9, 96.0, 2, 1), None);
        assert_eq!(l.hit(f32::NAN, 96.0, 2, 1), None);
    }
    #[test]
    fn dpi_snapping_never_moves_more_than_half_a_physical_pixel() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            for i in 0..100 {
                let p = i as f32 / 3.0;
                let snapped = snap_to_pixel(p, scale);
                assert!(((snapped - p) * scale).abs() <= 0.5001);
                assert!((snapped * scale - (snapped * scale).round()).abs() < 0.0001);
            }
        }
    }
}
