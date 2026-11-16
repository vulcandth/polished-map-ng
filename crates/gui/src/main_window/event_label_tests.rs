use super::*;

#[test]
fn warp_labels_cover_digit_patterns_and_both_display_modes() {
    let mut h = egui_kittest::Harness::builder()
        .with_size(Vec2::new(800.0, 310.0))
        .with_theme(egui::Theme::Light)
        .build_ui(|ui| {
            ui.painter().rect_filled(
                Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 310.0)),
                0.0,
                Color32::WHITE,
            );
            let ids = [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 42, 99, 100, 255, 256, 999,
                1000, 4096,
            ];
            for (group, (hex, zoom, title)) in [
                (false, false, "Decimal · normal"),
                (true, false, "Hex · normal"),
                (false, true, "Decimal · zoomed"),
                (true, true, "Hex · zoomed"),
            ]
            .into_iter()
            .enumerate()
            {
                let origin = Pos2::new(
                    10.0 + (group % 2) as f32 * 400.0,
                    10.0 + (group / 2) as f32 * 155.0,
                );
                ui.painter().text(
                    origin,
                    egui::Align2::LEFT_TOP,
                    title,
                    egui::FontId::proportional(14.0),
                    Color32::BLACK,
                );
                for (index, id) in ids.into_iter().enumerate() {
                    let pos = origin
                        + Vec2::new(
                            (index % 12) as f32 * 32.0,
                            26.0 + (index / 12) as f32 * 55.0,
                        );
                    let area =
                        Rect::from_min_size(pos, Vec2::splat(if zoom { 32.0 } else { 16.0 }));
                    ui.painter()
                        .rect_filled(area, 0.0, Color32::from_rgb(160, 0, 160));
                    event_workflows::draw_warp_label(
                        ui.painter(),
                        area,
                        polished_map_core::WarpIdLabel::new(id, hex).unwrap(),
                        zoom,
                    );
                    ui.painter().text(
                        Pos2::new(pos.x, pos.y + 34.0),
                        egui::Align2::LEFT_TOP,
                        id.to_string(),
                        egui::FontId::monospace(9.0),
                        Color32::BLACK,
                    );
                }
            }
        });
    h.run();
    let image = h.render().unwrap();
    // Independent source glyph for ID 1, including its opaque white border.
    // This catches a painter/tessellator dropping the one-pixel black strokes.
    for (y, row) in [
        ".....", ".##..", "..#..", "..#..", "..#..", ".###.", ".....",
    ]
    .into_iter()
    .enumerate()
    {
        for (x, pixel) in row.bytes().enumerate() {
            let c = if pixel == b'#' { 0 } else { 255 };
            assert_eq!(
                image.get_pixel(20 + x as u32, 44 + y as u32).0,
                [c, c, c, 255]
            );
        }
    }
    h.snapshot("warp-label-atlas");
}
