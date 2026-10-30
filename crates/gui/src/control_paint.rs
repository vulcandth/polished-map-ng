//! Source control boxes from master themes.cpp (Greybird/Dark).
//! FLTK color-ramp/inactive conventions: Copyright 1998-2024 Bill Spitzak and
//! others; LICENSES/FLTK.txt and NOTICE.md. Rust adaptation LGPL-3.0-only.
use super::*;

#[derive(Default)]
pub(super) struct Drawing(pub Vec<(Color32, [i32; 4])>);

struct Boxes {
    drawing: Drawing,
    dark: bool,
    enabled: bool,
}

pub(super) fn background(dark: bool) -> u8 {
    if dark { 83 } else { 206 }
}

fn average(a: u8, b: u8, weight: f32) -> u8 {
    (f32::from(a) * weight + f32::from(b) * (1.0 - weight)) as u8
}

impl Boxes {
    fn active(&self, gray: u8) -> u8 {
        if self.enabled {
            gray
        } else {
            average(gray, background(self.dark), 0.33)
        }
    }

    fn rect(&mut self, gray: u8, r: [i32; 4]) {
        self.drawing.0.push((Color32::from_gray(gray), r));
    }

    fn h(&mut self, gray: u8, x: i32, y: i32, end: i32) {
        self.rect(self.active(gray), [x.min(end), y, (end - x).abs() + 1, 1]);
    }

    fn v(&mut self, gray: u8, x: i32, y: i32, end: i32) {
        self.rect(self.active(gray), [x, y.min(end), 1, (end - y).abs() + 1]);
    }

    fn gradient(&mut self, r: [i32; 4], first: u8, last: u8, horizontal: bool) {
        let [x, y, w, h] = r;
        let (first, last) = (self.active(first), self.active(last));
        let count = if horizontal { w } else { h };
        for i in 0..count {
            // themes.cpp applies fl_inactive again inside the gradient helper.
            let gray = average(first, last, 1.0 - i as f32 / (count - 1).max(1) as f32);
            if horizontal {
                self.v(gray, x + i, y, y + h - 1);
            } else {
                self.h(gray, x, y + i, x + w - 1);
            }
        }
    }

    fn button(&mut self, w: i32, h: i32) {
        let (top, side, bottom, inner, upper_corner, lower_corner) = if self.dark {
            self.gradient([1, 2, w - 2, h - 2], 0x75, 0x62, w < h);
            (0x30, 0x2c, 0x27, 0x91, 0x47, 0x47)
        } else {
            self.gradient([2, 2, w - 4, h - 3], 0xdb, 0xcc, w < h);
            (0xa6, 0x96, 0x87, 0xee, 0xb8, 0xa0)
        };
        self.h(top, 2, 0, w - 3);
        self.v(side, 0, 2, h - 3);
        self.v(side, w - 1, 2, h - 3);
        self.h(bottom, 2, h - 1, w - 3);
        self.h(inner, 2, 1, w - 3);
        if !self.dark {
            self.v(0xe4, 1, 2, h - 3);
            self.v(0xe4, w - 2, 2, h - 3);
        }
        self.h(upper_corner, 0, 1, 1);
        self.v(upper_corner, 1, 1, 0);
        // Preserve the original draw order, including shared corner pixels.
        if self.dark {
            self.h(lower_corner, 0, h - 2, 1);
            self.v(lower_corner, 1, h - 2, h - 1);
        }
        self.v(upper_corner, w - 2, 0, 1);
        self.h(upper_corner, w - 2, 1, w - 1);
        if !self.dark {
            self.h(lower_corner, 0, h - 2, 1);
            self.v(lower_corner, 1, h - 2, h - 1);
        }
        self.v(lower_corner, w - 2, h - 1, h - 2);
        self.h(lower_corner, w - 2, h - 2, w - 1);
    }

    fn track(&mut self, w: i32, h: i32) {
        // FL_DARK2=45, FL_GRAY_RAMP=32, FL_GRAY=49, FL_NUM_GRAY=24.
        let exponent = (f64::from(background(self.dark)) / 255.0).ln() / (17.0f64 / 23.0).ln();
        let gray = ((13.0f64 / 23.0).powf(exponent) * 255.0 + 0.5) as u8;
        // OS_Slider passes fl_inactive(FL_DARK2) to a box which activates again.
        self.rect(self.active(self.active(gray)), [1, 1, w - 2, h - 2]);
        let (upper, lower) = if self.dark {
            (0x38, 0x74)
        } else {
            (0xba, 0xda)
        };
        self.v(upper, 0, h - 2, 0);
        self.h(upper, 0, 0, w - 2);
        self.h(lower, 0, h - 1, w - 1);
        self.v(lower, w - 1, h - 1, 0);
    }
}

pub(super) fn button(dark: bool, enabled: bool, w: i32, h: i32) -> Drawing {
    let mut boxes = Boxes {
        drawing: Drawing::default(),
        dark,
        enabled,
    };
    boxes.button(w, h);
    boxes.drawing
}

pub(super) fn track(dark: bool, enabled: bool, w: i32) -> Drawing {
    let mut boxes = Boxes {
        drawing: Drawing::default(),
        dark,
        enabled,
    };
    boxes.track(w, 4);
    boxes.drawing
}

pub(super) fn paint(painter: &egui::Painter, origin: Pos2, drawing: Drawing) {
    for (color, [x, y, w, h]) in drawing.0 {
        if w > 0 && h > 0 {
            painter.rect_filled(
                Rect::from_min_size(
                    origin + Vec2::new(x as f32, y as f32),
                    Vec2::new(w as f32, h as f32),
                ),
                0.0,
                color,
            );
        }
    }
}
