//! Master themes.cpp and FLTK 1.4.5 Fl_Menu.cxx/fl_draw.cxx/fl_contrast.cxx,
//! plus the background gray ramp from Fl_get_system_colors.cxx/Enumerations.H.
//! Based in part on the work of the FLTK project, Copyright 1998-2024 Bill
//! Spitzak and others. See NOTICE.md and LICENSES/FLTK.txt. LGPL-3.0 adaptation.
//! Drawing coordinates/colors are source-derived; native arc/polygon coverage
//! remains driver-specific and is not claimed identical to egui tessellation.
use super::*;

pub(super) fn foreground(dark: bool, enabled: bool, hovered: bool) -> Color32 {
    let normal = Color32::from_gray(if dark { 255 } else { 60 });
    if !enabled {
        average(
            normal,
            Color32::from_gray(if dark { 83 } else { 206 }),
            0.33,
        )
    } else if hovered {
        contrast(normal, selection(dark))
    } else {
        normal
    }
}

fn selection(dark: bool) -> Color32 {
    if dark {
        Color32::from_gray(214)
    } else {
        Color32::from_rgb(61, 123, 187)
    }
}

fn average(a: Color32, b: Color32, weight: f32) -> Color32 {
    let channel = |a: u8, b: u8| (f32::from(a) * weight + f32::from(b) * (1.0 - weight)) as u8;
    Color32::from_rgb(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
    )
}

fn contrast(fg: Color32, bg: Color32) -> Color32 {
    // main.cpp sets CIELAB contrast level 50 (not FLTK's default 39).
    let lightness = |c: Color32| {
        let y = 0.2126729 * (f64::from(c.r()) / 255.0).powf(2.4)
            + 0.7151522 * (f64::from(c.g()) / 255.0).powf(2.4)
            + 0.0721750 * (f64::from(c.b()) / 255.0).powf(2.4);
        if y <= 216.0 / 24389.0 {
            y * (24389.0 / 27.0)
        } else {
            y.powf(1.0 / 3.0) * 116.0 - 16.0
        }
    };
    let bg_lightness = lightness(bg);
    if (lightness(fg) - bg_lightness).abs() >= 50.0 {
        fg
    } else if bg_lightness > 50.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

#[derive(Debug, PartialEq)]
enum Primitive {
    Rect([i32; 4]),
    Arc([i32; 4], [i32; 2]),
    Pie([i32; 4]),
    Polygon([[i32; 2]; 6]),
}

#[derive(Default)]
struct Drawing(Vec<(Color32, Primitive)>);

impl Drawing {
    fn rect(&mut self, c: Color32, r: [i32; 4]) {
        self.0.push((c, Primitive::Rect(r)));
    }
    fn h(&mut self, c: Color32, x: i32, y: i32, end: i32) {
        self.rect(c, [x.min(end), y, (end - x).abs() + 1, 1]);
    }
    fn v(&mut self, c: Color32, x: i32, y: i32, end: i32) {
        self.rect(c, [x, y.min(end), 1, (end - y).abs() + 1]);
    }
    fn arc(&mut self, c: Color32, r: [i32; 4], angles: [i32; 2]) {
        self.0.push((c, Primitive::Arc(r, angles)));
    }
    fn pie(&mut self, c: Color32, r: [i32; 4]) {
        self.0.push((c, Primitive::Pie(r)));
    }
    fn gradient(&mut self, r: [i32; 4], top: u8, bottom: u8) {
        let [x, y, w, h] = r;
        for i in 0..h {
            let weight = 1.0 - i as f32 / (h - 1).max(1) as f32;
            self.h(
                average(Color32::from_gray(top), Color32::from_gray(bottom), weight),
                x,
                y + i,
                x + w - 1,
            );
        }
    }
}

fn drawing(dark: bool, radio: bool, selected: bool, row_height: i32) -> Drawing {
    let mut out = Drawing::default();
    let h = row_height - 4;
    let d = (h - 14 + 1) / 2;
    let w = h - 2 * d;
    let (x, y) = (2, 1 + d);
    let background = Color32::from_gray(if dark { 58 } else { 252 });
    let mark = contrast(selection(dark), background);
    let gray = Color32::from_gray;
    if radio {
        if dark {
            out.arc(gray(117), [x + 1, y + 1, w - 2, w - 2], [0, 180]);
            out.arc(gray(98), [x + 1, y + 1, w - 2, w - 2], [180, 360]);
            out.gradient([x + 2, y + 2, w - 4, w - 4], 116, 99);
        } else {
            out.pie(background, [x + 1, y + 1, w - 2, w - 2]);
        }
        out.arc(gray(if dark { 48 } else { 128 }), [x, y, w, w], [0, 360]);
        if selected {
            // The theme's box_dw changes the mark diameter: Dark 2, Greybird 6.
            let mut tw = (w - if dark { 2 } else { 6 }) / 2 + 1;
            if (w - tw) & 1 != 0 {
                tw += 1;
            }
            let td = (w - tw) / 2;
            let (rx, ry, diameter) = (td + 1, 1 + d + td - 1, tw + 2);
            out.pie(mark, [rx, ry, diameter, diameter]);
            out.pie(
                average(Color32::WHITE, mark, 0.2),
                [rx + 2, ry + 2, diameter - 4, diameter - 4],
            );
            out.arc(
                average(Color32::WHITE, mark, 0.5),
                [rx + 1, ry + 1, diameter - 1, diameter - 1],
                [60, 180],
            );
        }
    } else {
        if dark {
            out.gradient([x + 1, y + 2, w - 2, w - 2], 117, 98);
        } else {
            out.rect(background, [x + 1, y + 1, w - 2, w - 2]);
        }
        out.h(gray(if dark { 48 } else { 128 }), x + 2, y, x + w - 3);
        for xx in [x, x + w - 1] {
            out.v(gray(if dark { 44 } else { 137 }), xx, y + 2, y + w - 3);
        }
        out.h(
            gray(if dark { 39 } else { 144 }),
            x + 2,
            y + w - 1,
            x + w - 3,
        );
        if dark {
            out.h(gray(145), x + 2, y + 1, x + w - 3);
        }
        // fl_xyline / fl_yxline corners are two orthogonal segments, not diagonals.
        let upper = gray(if dark { 71 } else { 166 });
        let lower = gray(if dark { 71 } else { 176 });
        out.h(upper, x, y + 1, x + 1);
        out.v(upper, x + 1, y + 1, y);
        if dark {
            out.h(lower, x, y + w - 2, x + 1);
            out.v(lower, x + 1, y + w - 2, y + w - 1);
        }
        out.v(upper, x + w - 2, y, y + 1);
        out.h(upper, x + w - 2, y + 1, x + w - 1);
        if !dark {
            out.h(lower, x, y + w - 2, x + 1);
            out.v(lower, x + 1, y + w - 2, y + w - 1);
        }
        out.v(lower, x + w - 2, y + w - 1, y + w - 2);
        out.h(lower, x + w - 2, y + w - 2, x + w - 1);
        if selected {
            out.0.push((
                mark,
                Primitive::Polygon(check_vertices([3, y + 1, w - 2, w - 2])),
            ));
        }
    }
    out
}

fn check_vertices([x, y, w, h]: [i32; 4]) -> [[i32; 2]; 6] {
    let tw = w - if w > 10 { 2 } else { 0 };
    let th = h - if h > 10 { 2 } else { 0 };
    let mut lh = 3;
    let mut d1 = (tw / 3).min(6);
    let mut d2 = 2 * d1;
    if d2 + lh + 1 > th {
        d2 = th - lh - 1;
        d1 = (d2 + 1) / 2;
    }
    if d1 < 2 {
        d1 = 2;
        d2 = 4;
    }
    if d1 < 3 {
        lh = 2;
    }
    let tx = x + (w - (d1 + d2 + 1) + 1) / 2;
    let ty = y + (h - (d2 + lh + 1) + 1) / 2 + d2 - d1;
    [
        [tx, ty],
        [tx + d1, ty + d1],
        [tx + d1 + d2, ty + d1 - d2],
        [tx + d1 + d2, ty + d1 - d2 + lh],
        [tx + d1, ty + d1 + lh],
        [tx, ty + lh],
    ]
}

pub(super) fn paint(
    painter: &egui::Painter,
    origin: Pos2,
    row: f32,
    dark: bool,
    radio: bool,
    selected: bool,
) {
    paint_drawing(painter, origin, drawing(dark, radio, selected, row as i32));
}

fn panel(dark: bool, width: i32, height: i32) -> Drawing {
    // themes.cpp::{greybird,dark}_panel_thin_up_box/frame. Preserve inclusive
    // line endpoints and draw order, including the top-right/bottom-left corners.
    let (background, upper, lower) = if dark { (83, 106, 40) } else { (206, 218, 193) };
    let mut d = Drawing::default();
    d.rect(
        Color32::from_gray(background),
        [1, 1, width - 2, height - 2],
    );
    d.v(Color32::from_gray(upper), 0, height - 2, 0);
    d.h(Color32::from_gray(upper), 0, 0, width - 2);
    d.h(Color32::from_gray(lower), 0, height - 1, width - 1);
    d.v(Color32::from_gray(lower), width - 1, height - 1, 0);
    d
}

fn divider(dark: bool, width: i32, row: i32, index: i32) -> Drawing {
    // FL_DARK3/FL_LIGHT3 after Fl::background(206) or Fl::background(83), not
    // the theme's panel-frame colors. Verified against the unchanged gray-ramp
    // function in FLTK 1.4.5 Fl_get_system_colors.cxx and its color indices.
    let (upper, lower) = if dark { (3, 216) } else { (110, 247) };
    let mut d = Drawing::default();
    let y = (index + 1) * row - 1;
    // drawentry extends through x=width, clipped by the native menu window.
    d.h(Color32::from_gray(upper), 0, y, width);
    d.h(Color32::from_gray(lower), 0, y + 1, width);
    d
}

pub(super) fn paint_panel(painter: &egui::Painter, bounds: Rect, dark: bool) {
    paint_drawing(
        painter,
        bounds.min,
        panel(dark, bounds.width() as i32, bounds.height() as i32),
    );
}

pub(super) fn paint_divider(
    painter: &egui::Painter,
    bounds: Rect,
    dark: bool,
    row: f32,
    index: usize,
) {
    paint_drawing(
        &painter.with_clip_rect(painter.clip_rect().intersect(bounds)),
        bounds.min,
        divider(dark, bounds.width() as i32, row as i32, index as i32),
    );
}

fn paint_drawing(painter: &egui::Painter, origin: Pos2, drawing: Drawing) {
    let pos = |x: i32, y: i32| origin + Vec2::new(x as f32, y as f32);
    for (color, primitive) in drawing.0 {
        match primitive {
            Primitive::Rect([x, y, w, h]) => {
                painter.rect_filled(
                    Rect::from_min_size(pos(x, y), Vec2::new(w as f32, h as f32)),
                    0.0,
                    color,
                );
            }
            Primitive::Pie([x, y, w, h]) => {
                painter.circle_filled(
                    pos(x, y) + Vec2::new(w as f32, h as f32) / 2.0,
                    w as f32 / 2.0,
                    color,
                );
            }
            Primitive::Arc([x, y, w, h], [start, end]) => {
                let center = pos(x, y) + Vec2::new(w as f32, h as f32) / 2.0;
                let radius = (w as f32 - 1.0) / 2.0;
                let count = ((end - start) / 5).max(1);
                let points = (0..=count)
                    .map(|i| {
                        let angle = (start as f32 + (end - start) as f32 * i as f32 / count as f32)
                            .to_radians();
                        center + Vec2::new(angle.cos(), -angle.sin()) * radius
                    })
                    .collect();
                painter.add(egui::Shape::line(points, egui::Stroke::new(1.0, color)));
            }
            Primitive::Polygon(points) => {
                // This concave polygon is the union of two trapezoids.
                let mut mesh = egui::Mesh::default();
                for [x, y] in points {
                    mesh.colored_vertex(pos(x, y), color);
                }
                mesh.indices
                    .extend_from_slice(&[0, 1, 4, 0, 4, 5, 1, 2, 3, 1, 3, 4]);
                painter.add(egui::Shape::mesh(mesh));
            }
        }
    }
}

#[cfg(test)]
mod tests;
