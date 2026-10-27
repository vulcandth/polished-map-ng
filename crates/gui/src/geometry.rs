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

#[cfg(test)]
mod tests {
    use super::*;
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
