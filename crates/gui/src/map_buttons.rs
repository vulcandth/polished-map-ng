//! Pixel geometry adapted from pinned master map-buttons.cpp::draw_map_button
//! and draw_selection_border. Copyright Rangi/Polished Map contributors.
//! LGPL-3.0-only. Explicit quads retain opaque one-point FLTK edges in egui.
use super::*;

pub(super) fn grid_color(dark: bool) -> Color32 {
    // themes.cpp: Greybird (the upstream Unix default) and Dark inactive color.
    // Complete native theme selection remains separate from this renderer.
    Color32::from_gray(if dark { 0x26 } else { 0x55 })
}

pub(super) fn grid(painter: &egui::Painter, area: Rect, color: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.add_colored_rect(
        Rect::from_min_max(area.left_bottom() - Vec2::Y, area.right_bottom()),
        color,
    );
    mesh.add_colored_rect(
        Rect::from_min_max(area.right_top() - Vec2::X, area.right_bottom()),
        color,
    );
    painter.add(egui::Shape::mesh(mesh));
}

pub(super) fn selection(painter: &egui::Painter, area: Rect, zoom: bool) {
    let mut mesh = egui::Mesh::default();
    let last = if zoom { 3 } else { 2 };
    for inset in 0..=last {
        let area = area.shrink(inset as f32);
        let color = if inset == 0 || inset == last {
            Color32::BLACK
        } else {
            Color32::WHITE
        };
        for edge in [
            Rect::from_min_size(area.min, Vec2::new(area.width(), 1.0)),
            Rect::from_min_max(area.left_bottom() - Vec2::Y, area.right_bottom()),
            Rect::from_min_size(area.min + Vec2::Y, Vec2::new(1.0, area.height() - 2.0)),
            Rect::from_min_max(
                area.right_top() + Vec2::new(-1.0, 1.0),
                area.right_bottom() - Vec2::Y,
            ),
        ] {
            mesh.add_colored_rect(edge, color);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

pub(super) fn decorate(
    painter: &egui::Painter,
    area: Rect,
    zoom: bool,
    show_grid: bool,
    selected: bool,
    dark: bool,
) {
    if show_grid {
        grid(painter, area, grid_color(dark));
    }
    if selected {
        let selection_area =
            Rect::from_min_max(area.min, area.max - Vec2::splat(f32::from(show_grid)));
        selection(painter, selection_area, zoom);
    }
}

#[cfg(all(test, feature = "visual-tests"))]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn source_grid_and_selection_pixels_at_normal_and_double_dpi() {
        let mut cases = Vec::new();
        for zoom in [false, true] {
            for dark in [false, true] {
                for grid in [false, true] {
                    for selected in [false, true] {
                        cases.push((if zoom { 64 } else { 32 }, zoom, grid, selected, dark));
                    }
                }
            }
        }
        // Event half-cells, tile/chip editors and palette swatches share the rings.
        cases.extend([
            (16, false, false, true, false),
            (24, false, false, true, false),
            (32, true, false, true, false),
            (64, false, false, true, false),
            (20, false, false, true, false),
        ]);
        let extent = Vec2::new(340.0, 20.0 + cases.len().div_ceil(4) as f32 * 80.0);
        for scale in [1, 2] {
            let painted_cases = cases.clone();
            let mut h = egui_kittest::Harness::builder()
                .with_size(extent)
                .with_pixels_per_point(scale as f32)
                .build_ui(move |ui| {
                    ui.painter().rect_filled(
                        Rect::from_min_size(Pos2::ZERO, extent),
                        0.0,
                        Color32::WHITE,
                    );
                    for (index, &(size, zoom, grid, selected, dark)) in
                        painted_cases.iter().enumerate()
                    {
                        let area = Rect::from_min_size(
                            Pos2::new(
                                10.0 + (index % 4) as f32 * 80.0,
                                10.0 + (index / 4) as f32 * 80.0,
                            ),
                            Vec2::splat(size as f32),
                        );
                        ui.painter()
                            .rect_filled(area, 0.0, Color32::from_rgb(128, 64, 160));
                        decorate(ui.painter(), area, zoom, grid, selected, dark);
                    }
                });
            h.run();
            let image = h.render().unwrap();
            for (index, &(size, zoom, grid, selected, dark)) in cases.iter().enumerate() {
                for y in 0..size * scale {
                    for x in 0..size * scale {
                        let (lx, ly) = (x / scale, y / scale);
                        let expected = if grid && (lx == size - 1 || ly == size - 1) {
                            let c = if dark { 38 } else { 85 };
                            [c, c, c, 255]
                        } else if selected {
                            let end = size - 1 - u32::from(grid);
                            let depth = lx.min(ly).min(end - lx).min(end - ly);
                            match (depth, zoom) {
                                (0, _) | (2, false) | (3, true) => [0, 0, 0, 255],
                                (1, _) | (2, true) => [255, 255, 255, 255],
                                _ => [128, 64, 160, 255],
                            }
                        } else {
                            [128, 64, 160, 255]
                        };
                        let px = (10 + (index as u32 % 4) * 80) * scale + x;
                        let py = (10 + (index as u32 / 4) * 80) * scale + y;
                        assert_eq!(
                            image.get_pixel(px, py).0,
                            expected,
                            "case {index}, scale {scale}, ({x}, {y})"
                        );
                    }
                }
            }
            if scale == 1 {
                h.snapshot("map-border-atlas");
            }
        }
    }

    #[test]
    fn map_hover_grid_mode_and_sidebar_drag_use_source_borders() {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        h.remove_cursor();
        h.run();
        let baseline = h.render().unwrap();
        let map_fill = baseline.get_pixel(240, 143).0;
        assert_ne!(map_fill, [85, 85, 85, 255]);
        assert_eq!(baseline.get_pixel(271, 174).0, [85, 85, 85, 255]);
        assert_eq!(baseline.get_pixel(0, 47).0, [0, 0, 0, 255]);
        assert_eq!(baseline.get_pixel(1, 48).0, [255, 255, 255, 255]);
        assert_eq!(baseline.get_pixel(2, 49).0, [0, 0, 0, 255]);
        h.hover_at(Pos2::new(264.0, 167.0));
        h.run();
        let hovered = h.render().unwrap();
        assert_eq!(hovered.get_pixel(240, 143).0, [0, 0, 0, 255]);
        assert_eq!(hovered.get_pixel(241, 144).0, [255, 255, 255, 255]);
        assert_eq!(hovered.get_pixel(242, 145).0, [0, 0, 0, 255]);
        // Sample away from the rendered mouse pointer at the lower right.
        assert_eq!(hovered.get_pixel(240, 174).0, [85, 85, 85, 255]);
        h.snapshot("map-hover-border");
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::G);
        h.run();
        assert!(!h.state().grid);
        assert_eq!(h.render().unwrap().get_pixel(240, 174).0, [0, 0, 0, 255]);
        h.hover_at(Pos2::new(150.0, 100.0));
        h.run();
        assert_eq!(h.render().unwrap().get_pixel(240, 143).0, map_fill);
        h.key_press(egui::Key::Tab);
        h.hover_at(Pos2::new(264.0, 167.0));
        h.run();
        let event = h.render().unwrap();
        assert_eq!(event.get_pixel(240, 143).0, map_fill);
        assert_eq!(event.get_pixel(256, 159).0, [0, 0, 0, 255]);
        h.state_mut().prepare_resize();
        h.run();
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::G);
        h.run();
        assert!(!h.state().grid);
        h.get_by_label("Cancel").click();
        h.run();
        assert!(!h.state().is_modified());
        // Drag source and target both gain outlines while selection remains.
        h.hover_at(Pos2::new(16.0, 63.0));
        h.drag_at(Pos2::new(16.0, 63.0));
        h.run();
        h.hover_at(Pos2::new(48.0, 63.0));
        h.run();
        assert_eq!(h.state().sidebar_drag, Some(0));
        let dragged = h.render().unwrap();
        assert_eq!(dragged.get_pixel(0, 47).0, [0, 0, 0, 255]);
        assert_eq!(dragged.get_pixel(32, 47).0, [0, 0, 0, 255]);
        assert_eq!(dragged.get_pixel(33, 48).0, [255, 255, 255, 255]);
        h.snapshot("sidebar-drag-borders");
        // Release outside the sidebar must discard the drag, not edit either domain.
        h.event(egui::Event::PointerButton {
            pos: Pos2::new(150.0, 100.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        });
        h.run();
        assert!(h.state().sidebar_drag.is_none());
        assert!(!h.state().is_modified());
    }
}
