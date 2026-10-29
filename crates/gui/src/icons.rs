//! Reviewed Polished Map icon pixels and source-sized controls.
//!
//! Artwork: Rangi/Polished Map adaptations (LGPL-3.0-only), including Fugue
//! Icons by Yusuke Kamiyamane (CC BY 3.0). See assets/icons/NOTICE.md. Layout
//! follows main-window.cpp and widgets.cpp; disabled arithmetic follows
//! Image::make_deimage and FLTK 1.4.5 Fl_Pixmap::{desaturate,color_average}.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum Icon {
    New,
    Reload,
    Save,
    Print,
    Grid,
    Rulers,
    Zoom,
    Ids,
    Hex,
    Priority,
    Palettes,
    Resize,
    Tileset,
    Copy,
    Paste,
    CollisionTopLeft,
    CollisionTopRight,
    CollisionBottomLeft,
    CollisionBottomRight,
}

impl Icon {
    pub(super) fn pixels(self, dark: bool) -> &'static [u8; 1024] {
        match (self, dark) {
            (Self::New, _) => include_bytes!("../assets/icons/new.rgba"),
            (Self::Reload, _) => include_bytes!("../assets/icons/reload.rgba"),
            (Self::Save, _) => include_bytes!("../assets/icons/save.rgba"),
            (Self::Print, _) => include_bytes!("../assets/icons/print.rgba"),
            (Self::Grid, false) => include_bytes!("../assets/icons/grid.rgba"),
            (Self::Grid, true) => include_bytes!("../assets/icons/grid-dark.rgba"),
            (Self::Rulers, _) => include_bytes!("../assets/icons/rulers.rgba"),
            (Self::Zoom, _) => include_bytes!("../assets/icons/zoom.rgba"),
            (Self::Ids, false) => include_bytes!("../assets/icons/ids.rgba"),
            (Self::Ids, true) => include_bytes!("../assets/icons/ids-dark.rgba"),
            (Self::Hex, false) => include_bytes!("../assets/icons/hex.rgba"),
            (Self::Hex, true) => include_bytes!("../assets/icons/hex-dark.rgba"),
            (Self::Priority, _) => include_bytes!("../assets/icons/priority.rgba"),
            (Self::Palettes, _) => include_bytes!("../assets/icons/palettes.rgba"),
            (Self::Resize, _) => include_bytes!("../assets/icons/resize.rgba"),
            (Self::Tileset, _) => include_bytes!("../assets/icons/tileset.rgba"),
            (Self::Copy, _) => include_bytes!("../assets/icons/copy.rgba"),
            (Self::Paste, _) => include_bytes!("../assets/icons/paste.rgba"),
            (Self::CollisionTopLeft, false) => include_bytes!("../assets/icons/coll-top-left.rgba"),
            (Self::CollisionTopLeft, true) => {
                include_bytes!("../assets/icons/coll-top-left-dark.rgba")
            }
            (Self::CollisionTopRight, false) => {
                include_bytes!("../assets/icons/coll-top-right.rgba")
            }
            (Self::CollisionTopRight, true) => {
                include_bytes!("../assets/icons/coll-top-right-dark.rgba")
            }
            (Self::CollisionBottomLeft, false) => {
                include_bytes!("../assets/icons/coll-bottom-left.rgba")
            }
            (Self::CollisionBottomLeft, true) => {
                include_bytes!("../assets/icons/coll-bottom-left-dark.rgba")
            }
            (Self::CollisionBottomRight, false) => {
                include_bytes!("../assets/icons/coll-bottom-right.rgba")
            }
            (Self::CollisionBottomRight, true) => {
                include_bytes!("../assets/icons/coll-bottom-right-dark.rgba")
            }
        }
    }
}

fn disabled(rgb: [u8; 3], background: u8) -> Color32 {
    let [r, g, b] = rgb.map(u32::from);
    let gray = (r * 31 + g * 61 + b * 8) / 100;
    Color32::from_gray(((gray + u32::from(background)) / 2) as u8)
}

/// Paint 16 logical pixels centered in the supplied bounds. Mesh quads
/// avoid antialiasing/filtering individual one-point source pixels.
pub(super) fn paint(ui: &egui::Ui, bounds: Rect, icon: Icon, enabled: bool) {
    if !ui.is_rect_visible(bounds) {
        return;
    }
    let dark = ui.visuals().dark_mode;
    // themes.cpp Greybird/Dark FL_BACKGROUND_COLOR (FL_GRAY). Complete native
    // theme rendering is separate from these source-derived icon colors.
    let background = if dark { 0x53 } else { 0xCE };
    let origin = bounds.center() - Vec2::splat(8.0);
    let scale = ui.ctx().pixels_per_point();
    let origin = Pos2::new(
        geometry::snap_to_pixel(origin.x, scale),
        geometry::snap_to_pixel(origin.y, scale),
    );
    let mut mesh = egui::Mesh::default();
    for (index, pixel) in icon.pixels(dark).as_chunks::<4>().0.iter().enumerate() {
        if pixel[3] == 0 {
            continue;
        }
        let color = if enabled {
            Color32::from_rgb(pixel[0], pixel[1], pixel[2])
        } else {
            disabled([pixel[0], pixel[1], pixel[2]], background)
        };
        mesh.add_colored_rect(
            Rect::from_min_size(
                origin + Vec2::new((index % 16) as f32, (index / 16) as f32),
                Vec2::splat(1.0),
            ),
            color,
        );
    }
    // The source-disabled pixels are already transformed. Avoid egui's second
    // disabled-Ui color/opacity operation, while retaining layer and clipping.
    let painter = egui::Painter::new(ui.ctx().clone(), ui.layer_id(), ui.clip_rect());
    painter.add(egui::Shape::mesh(mesh));
}

pub(super) struct Button<'a> {
    face: Face<'a>,
    label: &'a str,
    selected: Option<bool>,
    size: f32,
}

enum Face<'a> {
    Icon(Icon),
    Text(&'a str),
}

pub(super) fn separator(ui: &mut egui::Ui) {
    // Main_Window::initialize: two-point space + two-point Spacer + space.
    let (rect, _) = ui.allocate_exact_size(Vec2::new(6.0, 24.0), Sense::hover());
    ui.painter().vline(
        rect.left() + 2.5,
        rect.y_range(),
        ui.visuals().widgets.noninteractive.bg_stroke,
    );
}

impl<'a> Button<'a> {
    pub(super) fn new(icon: Icon, label: &'a str) -> Self {
        Self {
            face: Face::Icon(icon),
            label,
            selected: None,
            size: 24.0,
        }
    }

    /// Keep source control geometry while the original artwork is under review.
    pub(super) fn placeholder(text: &'a str, label: &'a str) -> Self {
        Self {
            face: Face::Text(text),
            label,
            selected: None,
            size: 24.0,
        }
    }

    pub(super) fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    pub(super) fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl egui::Widget for Button<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::click());
        response.widget_info(|| match self.selected {
            Some(selected) => egui::WidgetInfo::selected(
                egui::Role::Button,
                ui.is_enabled(),
                selected,
                self.label,
            ),
            None => egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), self.label),
        });
        if ui.is_rect_visible(rect) {
            let selected = self.selected.unwrap_or(false);
            if selected || response.hovered() || response.is_pointer_button_down_on() {
                let visuals = ui.style().interact_selectable(&response, selected);
                ui.painter().rect(
                    rect,
                    0.0,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            if response.has_focus() {
                ui.painter().rect_stroke(
                    rect.shrink(1.0),
                    0.0,
                    ui.visuals().selection.stroke,
                    egui::StrokeKind::Inside,
                );
            }
            match self.face {
                Face::Icon(icon) => paint(ui, rect, icon, ui.is_enabled()),
                Face::Text(text) => {
                    ui.painter()
                        .with_clip_rect(rect.intersect(ui.clip_rect()))
                        .text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::monospace(11.0),
                            ui.visuals().text_color(),
                        );
                }
            }
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    #[test]
    fn toolbar_icons_keep_names_bounds_actions_and_disabled_guards() {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        for name in [
            "New",
            "Reload events",
            "Save",
            "Print…",
            "Grid",
            "Rulers",
            "Zoom",
            "IDs",
            "Hex",
            "Show Priority",
            "Edit current palettes",
            "Resize map",
            "Tileset",
        ] {
            let rect = h.get_by_label(name).rect();
            assert_eq!(rect.size(), Vec2::splat(24.0), "{name}");
            assert_eq!((rect.top(), rect.bottom()), (22.0, 46.0), "{name}");
        }
        assert!(
            h.get_by_label("Reload events")
                .accesskit_node()
                .is_disabled()
        );
        h.get_by_label("Rulers").click();
        h.run();
        assert!(h.state().rulers);
        h.get_by_label("Resize map").click();
        h.run();
        assert!(h.state().resize_form.is_some());
        assert!(h.get_by_label("New").accesskit_node().is_disabled());
        h.get_by_label("Cancel").click();
        h.run();
        h.get_by_label("Edit current palettes").click();
        h.run();
        assert!(h.state().palette_form.is_some());
        h.get_by_label("Cancel").click();
        h.run();
        h.get_by_label("Tileset").click();
        h.run();
        assert!(h.state().tileset_form.is_some());
        for name in ["Copy tile image", "Paste tile image"] {
            assert_eq!(h.get_by_label(name).rect().size(), Vec2::splat(22.0));
        }
        h.get_by_label("Cancel").click();
        h.run();
        assert!(!h.state().is_modified());
        // A dirty document exercises New's keyboard dispatch without opening
        // the native folder chooser required by a synthetic rootless map.
        h.state_mut().apply(Command::Paint {
            points: vec![polished_map_core::Point { x: 0, y: 0 }],
            block: 1,
        });
        h.run();
        h.get_by_label("New").focus();
        h.key_press(egui::Key::Space);
        h.run();
        assert!(h.state().confirm_discard && h.state().pending_new);
        h.get_by_label("Cancel").click();
        h.run();
        assert!(h.state().is_modified());
        let empty = Harness::builder()
            .with_size(Vec2::new(800.0, 600.0))
            .build_eframe(|cc| Editor::new(cc));
        for name in ["Reload events", "Save", "Print…", "Resize map", "Tileset"] {
            assert!(
                empty.get_by_label(name).accesskit_node().is_disabled(),
                "{name}"
            );
        }
    }

    #[cfg(feature = "visual-tests")]
    #[test]
    fn every_icon_pixel_survives_theme_disabled_state_and_dpi() {
        let icons = [
            Icon::New,
            Icon::Reload,
            Icon::Save,
            Icon::Print,
            Icon::Grid,
            Icon::Rulers,
            Icon::Zoom,
            Icon::Ids,
            Icon::Hex,
            Icon::Priority,
            Icon::Palettes,
            Icon::Resize,
            Icon::Tileset,
            Icon::Copy,
            Icon::Paste,
            Icon::CollisionTopLeft,
            Icon::CollisionTopRight,
            Icon::CollisionBottomLeft,
            Icon::CollisionBottomRight,
        ];
        let mut rows = Vec::new();
        for size in [24_u32, 22] {
            for dark in [false, true] {
                for enabled in [true, false] {
                    rows.push((size, dark, enabled));
                }
            }
        }
        for scale in [1_u32, 2] {
            let mut h = Harness::builder()
                .with_size(Vec2::new(624.0, 272.0))
                .with_pixels_per_point(scale as f32)
                .build_ui(|ui| {
                    ui.painter()
                        .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(128, 64, 160));
                    for (row, &(size, dark, enabled)) in rows.iter().enumerate() {
                        for (col, &icon) in icons.iter().enumerate() {
                            let area = Rect::from_min_size(
                                Pos2::new(8.0 + col as f32 * 32.0, 8.0 + row as f32 * 32.0),
                                Vec2::splat(size as f32),
                            );
                            let mut child = ui.new_child(
                                egui::UiBuilder::new().max_rect(area).id_salt((row, col)),
                            );
                            child.visuals_mut().dark_mode = dark;
                            child.add_enabled(enabled, Button::new(icon, "Icon").size(size as f32));
                        }
                    }
                });
            h.run();
            let image = h.render().unwrap();
            for (row, &(size, dark, enabled)) in rows.iter().enumerate() {
                for (col, &icon) in icons.iter().enumerate() {
                    for y in 0..size * scale {
                        for x in 0..size * scale {
                            let (lx, ly, inset) = (x / scale, y / scale, (size - 16) / 2);
                            let mut expected = [128, 64, 160, 255];
                            if (inset..inset + 16).contains(&lx)
                                && (inset..inset + 16).contains(&ly)
                            {
                                let index = ((ly - inset) * 16 + lx - inset) as usize * 4;
                                let pixel = &icon.pixels(dark)[index..index + 4];
                                if pixel[3] != 0 {
                                    expected.copy_from_slice(pixel);
                                    if !enabled {
                                        // Independent source-derived expectation, not the renderer's helper.
                                        let luminance = (31 * u32::from(pixel[0])
                                            + 61 * u32::from(pixel[1])
                                            + 8 * u32::from(pixel[2]))
                                            / 100;
                                        let v = ((luminance * 128
                                            + if dark { 83 * 128 } else { 206 * 128 })
                                            >> 8)
                                            as u8;
                                        expected = [v, v, v, 255];
                                    }
                                }
                            }
                            assert_eq!(
                                image
                                    .get_pixel(
                                        (8 + col as u32 * 32) * scale + x,
                                        (8 + row as u32 * 32) * scale + y
                                    )
                                    .0,
                                expected,
                                "{icon:?} row {row} scale {scale} pixel ({x},{y})"
                            );
                        }
                    }
                }
            }
            if scale == 1 {
                h.snapshot("licensed-icon-atlas");
            }
        }
    }

    #[test]
    fn disabled_colors_follow_two_integer_truncations() {
        for (rgb, light, dark) in [
            ([0, 0, 0], 103, 41),
            ([255, 255, 255], 230, 169),
            ([255, 0, 0], 142, 81),
            ([0, 255, 0], 180, 119),
            ([0, 0, 255], 113, 51),
            ([1, 1, 1], 103, 42),
            ([17, 93, 201], 142, 80),
        ] {
            assert_eq!(disabled(rgb, 206), Color32::from_gray(light));
            assert_eq!(disabled(rgb, 83), Color32::from_gray(dark));
        }
    }
}
