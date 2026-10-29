//! ID labels adapted from pinned master map-buttons.cpp and utils.cpp.
//! LGPL-3.0-only. Status fields are implemented in status_bar.rs.
//! The global display base deliberately does not change event-script formats.
use super::*;

fn id_text(id: u8, hex: bool) -> String {
    if hex {
        format!("{id:02X}")
    } else {
        id.to_string()
    }
}

pub(super) fn draw_id(
    painter: &egui::Painter,
    area: Rect,
    id: u8,
    hex: bool,
    zoom: bool,
    hovered: bool,
) {
    let z = if zoom { 2.0 } else { 1.0 };
    let position = area.min + Vec2::new(z + 2.0, z);
    let text = id_text(id, hex);
    // The source uses Courier Bold at 14 in both zoom modes.
    let font = fonts::mono_bold(painter.ctx(), 14.0);
    for delta in [
        Vec2::new(-1.0, -1.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, 1.0),
    ] {
        painter.text(
            position + delta,
            egui::Align2::LEFT_TOP,
            &text,
            font.clone(),
            Color32::BLACK,
        );
    }
    painter.text(
        position,
        egui::Align2::LEFT_TOP,
        text,
        font,
        if hovered {
            Color32::YELLOW
        } else {
            Color32::WHITE
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_workflows::{EventAction, EventChange};
    use egui_kittest::kittest::Queryable;

    #[test]
    fn display_controls_update_live_without_editing_map_or_event_bases() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let path = dir.path().join("Map.asm");
        let source = "\twarp_event $0, $0, OTHER_MAP, 1\n\twarp_event 2, 0, OTHER_MAP, 1\n";
        std::fs::write(&path, source).unwrap();
        h.state_mut()
            .request_event_change(EventChange::Load(path.clone()));
        h.state_mut().rulers = true;
        h.run();
        assert!(!h.state().ids && !h.state().hex);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::I);
        h.run();
        assert!(h.state().ids);
        h.hover_at(Pos2::new(280.0, 183.0));
        h.run();
        h.get_by_label("ID: 0");
        h.get_by_label("X/Y (0, 0)");
        #[cfg(feature = "visual-tests")]
        h.snapshot("display-decimal-blocks");
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::Num4,
        );
        h.run();
        assert!(h.state().hex);
        h.get_by_label("ID: $00");
        h.get_by_label("X/Y ($0, $0)");
        #[cfg(feature = "visual-tests")]
        h.snapshot("display-hex-blocks");
        h.key_press(egui::Key::Tab);
        h.run();
        h.get_by_label("ID: $00");
        h.get_by_label("X/Y ($0, $0)");
        h.get_by_label("Event: X/Y ($1, $1)");
        h.state_mut().zoom = true;
        h.hover_at(Pos2::new(528.0, 303.0));
        h.run();
        #[cfg(feature = "visual-tests")]
        h.snapshot("display-hex-events-zoomed");
        let ctx = h.ctx.clone();
        for index in 0..2 {
            h.state_mut()
                .event_action(EventAction::Options(index), &ctx);
            h.run();
            h.get_by_label(if index == 0 {
                "warp_event $1:"
            } else {
                "warp_event #2:"
            });
            h.get_by_label("Event X").click();
            h.run();
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::I);
            h.key_press_modifiers(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::Num4,
            );
            h.run();
            assert!(h.state().ids && h.state().hex);
            h.get_by_label("Cancel").click();
            h.run();
        }
        h.get_by_label("IDs").click();
        h.run();
        assert!(!h.state().ids && h.state().hex);
        h.get_by_label("Hex").click();
        h.run();
        assert!(!h.state().hex);
        h.get_by_label("View").click();
        h.run();
        h.get_by_label("Block IDs").click();
        h.run();
        h.get_by_label("View").click();
        h.run();
        h.get_by_label("Hexadecimal").click();
        h.run();
        assert!(h.state().ids && h.state().hex);
        assert!(!h.state().is_modified());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
        let loaded = h.state().loaded.as_ref().unwrap();
        assert_eq!(loaded.document.map().blocks(), [0, 1]);
        let events = loaded.events.as_ref().unwrap().script().events().entries();
        assert!(events[0].definition().hex_coordinates);
        assert!(!events[1].definition().hex_coordinates);
        assert_eq!(loaded.events.as_ref().unwrap().script().render(), source);
    }

    #[test]
    #[cfg(feature = "visual-tests")]
    fn id_label_atlas_covers_bases_sizes_and_hover_colors() {
        let mut h = egui_kittest::Harness::builder()
            .with_size(Vec2::new(800.0, 380.0))
            .with_theme(egui::Theme::Light)
            .build_ui(|ui| {
                ui.painter().rect_filled(
                    Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 380.0)),
                    0.0,
                    Color32::WHITE,
                );
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
                        10.0 + (group / 2) as f32 * 190.0,
                    );
                    ui.painter().text(
                        origin,
                        egui::Align2::LEFT_TOP,
                        title,
                        egui::FontId::proportional(14.0),
                        Color32::BLACK,
                    );
                    for (index, id) in [0, 9, 10, 99, 100, 175, 255].into_iter().enumerate() {
                        let pos = origin
                            + Vec2::new(
                                (index % 4) as f32 * 80.0,
                                26.0 + (index / 4) as f32 * 72.0,
                            );
                        let area =
                            Rect::from_min_size(pos, Vec2::splat(if zoom { 64.0 } else { 32.0 }));
                        ui.painter().rect_filled(area, 0.0, Color32::from_gray(110));
                        draw_id(ui.painter(), area, id, hex, zoom, index % 2 != 0);
                    }
                }
            });
        h.run();
        h.snapshot("block-id-atlas");
    }

    #[test]
    fn display_formats_match_source_without_changing_values() {
        for (id, decimal, hex) in [
            (0, "0", "00"),
            (9, "9", "09"),
            (10, "10", "0A"),
            (100, "100", "64"),
            (255, "255", "FF"),
        ] {
            assert_eq!(id_text(id, false), decimal);
            assert_eq!(id_text(id, true), hex);
        }
    }
}
