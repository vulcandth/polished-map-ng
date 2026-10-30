use super::*;
use egui_kittest::kittest::Queryable;

#[test]
fn source_measurements_keep_fixed_client_and_platform_button_order() {
    // Independent text-width inputs, including fractional-font rounded widths.
    for (windows, label, digits) in [(true, 35.0, 21.0), (false, 39.0, 24.0)] {
        let layout = Layout::measured(windows, |s| if s == "999" { digits } else { label });
        assert_eq!(layout.size, Vec2::new(220.0, 132.0));
        assert_eq!(layout.width.min, Pos2::new(label + 14.0, 10.0));
        assert_eq!(layout.width.size(), Vec2::new(digits + 26.0, 22.0));
        assert_eq!(layout.height.min.y, 36.0);
        assert_eq!(layout.height.min.x, layout.width.min.x);
        for (i, anchor) in layout.anchors.iter().enumerate() {
            assert_eq!(anchor.size(), Vec2::splat(24.0));
            assert_eq!(
                anchor.min,
                Pos2::new(
                    label + digits + 60.0 + 26.0 * (i % 3) as f32,
                    10.0 + 26.0 * (i / 3) as f32
                )
            );
        }
        assert_eq!(
            layout.ok.min,
            Pos2::new(if windows { 36.0 } else { 130.0 }, 100.0)
        );
        assert_eq!(
            layout.cancel.min,
            Pos2::new(if windows { 130.0 } else { 36.0 }, 100.0)
        );
        assert_eq!(layout.ok.size(), Vec2::new(80.0, 22.0));
    }
}

#[test]
fn rendered_source_client_and_all_anchor_choices_at_both_scales() {
    for windows in [false, true] {
        for scale in [1.0, 2.0] {
            let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
            h.ctx.set_pixels_per_point(scale);
            h.set_size(Vec2::new(800.0, 600.0));
            h.state_mut().prepare_resize();
            h.state_mut().resize_form.as_mut().unwrap().windows_buttons = windows;
            h.run();
            let bounds = h.state().resize_form.as_ref().unwrap().bounds;
            assert_eq!(bounds.size(), Vec2::new(220.0, 132.0));
            let width = h.get_by_label("Width").rect();
            let height = h.get_by_label("Height").rect();
            assert_eq!(width.height(), 22.0);
            assert_eq!(height.min - width.min, Vec2::new(0.0, 26.0));
            let ok = h.get_by_label("Resize").rect();
            let cancel = h.get_by_label("Cancel").rect();
            assert_eq!(ok.size(), Vec2::new(80.0, 22.0));
            assert_eq!(cancel.size(), ok.size());
            assert_eq!(ok.top() - bounds.top(), 100.0);
            assert_eq!(ok.left() < cancel.left(), windows);
            let expected_font = if windows { 12.0 } else { 13.0 };
            for text in ["Width:", "Height:", "2", "1"] {
                let glyphs = h
                    .output()
                    .shapes
                    .iter()
                    .find_map(|s| match &s.shape {
                        egui::Shape::Text(t)
                            if t.galley.text() == text && bounds.contains(t.pos) =>
                        {
                            Some(t)
                        }
                        _ => None,
                    })
                    .expect("source-sized field text");
                assert!(
                    glyphs
                        .galley
                        .job
                        .sections
                        .iter()
                        .all(|s| s.format.font_id.size == expected_font)
                );
            }
            for (label, anchor) in [
                "Top left",
                "Top center",
                "Top right",
                "Middle left",
                "Center",
                "Middle right",
                "Bottom left",
                "Bottom center",
                "Bottom right",
            ]
            .into_iter()
            .zip(ResizeAnchor::ALL)
            {
                assert_eq!(h.get_by_label(label).rect().size(), Vec2::splat(24.0));
                h.get_by_label(label).click();
                h.run();
                assert_eq!(h.state().resize_form.as_ref().unwrap().anchor, anchor);
                assert!(!h.state().is_modified());
            }
            h.key_press(egui::Key::Escape);
            h.run();
            assert!(h.state().resize_form.is_none());
            assert!(!h.state().is_modified());
        }
    }
}
