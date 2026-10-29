use super::*;
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn source_fixed_field_widths_insets_and_separators_do_not_reflow() {
    for width in [200.0, 800.0, 1200.0] {
        let fields = field_rects(
            Rect::from_min_size(Pos2::new(7.0, 577.0), Vec2::new(width, 23.0)),
            [80.0, 100.0, 50.0, 100.0, 160.0],
        );
        for (index, (x, width)) in [
            (8.0, 96.0),
            (106.0, 116.0),
            (224.0, 66.0),
            (292.0, 116.0),
            (410.0, 176.0),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                fields[index],
                Rect::from_min_size(Pos2::new(x, 578.0), Vec2::new(width, 21.0))
            );
        }
    }
    let fields = field_rects(
        Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 23.0)),
        [0.25, 10.5, 10.0, 100.75, 0.0],
    );
    assert_eq!(
        fields.map(|rect| rect.width()),
        [17.0, 27.0, 26.0, 117.0, 16.0]
    );
}

#[test]
fn formatting_preserves_separate_fields_padding_and_event_mode() {
    for (id, x, y, ex, ey, decimal, hex) in [
        (
            0,
            0,
            0,
            0,
            1,
            ["ID: 0", "X/Y (0, 0)", "Event: X/Y (0, 1)"],
            ["ID: $00", "X/Y ($0, $0)", "Event: X/Y ($0, $1)"],
        ),
        (
            175,
            10,
            15,
            21,
            30,
            ["ID: 175", "X/Y (10, 15)", "Event: X/Y (21, 30)"],
            ["ID: $AF", "X/Y ($A, $F)", "Event: X/Y ($15, $1E)"],
        ),
        (
            255,
            254,
            254,
            509,
            508,
            ["ID: 255", "X/Y (254, 254)", "Event: X/Y (509, 508)"],
            ["ID: $FF", "X/Y ($FE, $FE)", "Event: X/Y ($1FD, $1FC)"],
        ),
    ] {
        let cursor = geometry::EventCursor {
            block_x: x,
            block_y: y,
            event_x: ex,
            event_y: ey,
        };
        for (base, expected) in [(false, decimal), (true, hex)] {
            assert_eq!(hover_fields(id, cursor, base, true), expected);
            assert_eq!(
                hover_fields(id, cursor, base, false),
                [expected[0], expected[1], ""]
            );
        }
    }
}

#[test]
fn fields_follow_hover_halves_bases_mode_leave_and_document_lifetime() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.remove_cursor();
    h.run();
    assert_eq!(
        h.state().status_fields(),
        ["Blocks: 2", "Map: 2 x 1", "", "", ""]
    );
    h.hover_at(Pos2::new(248.0, 151.0));
    h.run();
    h.get_by_label("ID: 0");
    h.get_by_label("X/Y (0, 0)");
    h.key_press(egui::Key::Tab);
    h.run();
    // Event coordinates are available even without an event script loaded.
    h.get_by_label("Event: X/Y (0, 0)");
    h.hover_at(Pos2::new(256.0, 159.0));
    h.run();
    h.get_by_label("Event: X/Y (1, 1)");
    h.key_press_modifiers(
        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        egui::Key::Num4,
    );
    h.run();
    assert_eq!(
        h.state().status_fields(),
        [
            "Blocks: 2",
            "Map: 2 x 1",
            "ID: $00",
            "X/Y ($0, $0)",
            "Event: X/Y ($1, $1)"
        ]
    );
    h.hover_at(Pos2::new(200.0, 100.0));
    h.run();
    assert_eq!(
        h.state().status_fields(),
        ["Blocks: 2", "Map: 2 x 1", "", "", ""]
    );
    assert!(!h.state().is_modified());
    assert!(!h.state().loaded.as_ref().unwrap().document.map().can_undo());
    let ctx = h.ctx.clone();
    assert!(h.state_mut().apply_block_commands(
        &ctx,
        &[polished_map_core::BlockCommand::Resize { count: 256 }]
    ));
    h.state_mut().apply(Command::Resize {
        width: 20,
        height: 10,
        anchor: ResizeAnchor::TopLeft,
    });
    h.run();
    // Count and dimensions remain decimal while the global hover base is hex.
    h.get_by_label("Blocks: 256");
    h.get_by_label("Map: 20 x 10");
    h.state_mut().close_map();
    h.run();
    assert_eq!(h.state().status_fields(), ["", "", "", "", ""]);
    assert!(h.query_by_label("Blocks: 256").is_none());
}

#[test]
fn hover_id_tracks_live_paint_then_release_and_undo() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().selected = 1;
    let point = Pos2::new(248.0, 151.0);
    h.hover_at(point);
    h.run();
    h.drag_at(point);
    h.run();
    assert!(!h.state().stroke.is_empty());
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        [0, 1]
    );
    h.get_by_label("ID: 1");
    h.drop_at(point);
    h.hover_at(point);
    h.run();
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        [1, 1]
    );
    h.get_by_label("ID: 1");
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
    h.run();
    h.get_by_label("ID: 0");
    let map = h.state().loaded.as_ref().unwrap().document.map();
    assert_eq!(map.blocks(), [0, 1]);
    assert!(map.can_redo());
    assert!(!map.can_undo());
}

#[test]
fn operation_feedback_remains_available_on_status_hover() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.ctx
        .all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
    h.hover_at(h.get_by_label("Blocks: 2").rect().center());
    h.run_steps(3);
    h.get_by_label("Saved\nSynthetic tileset");
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.run_steps(3);
    h.get_by_label("Modified\nSynthetic tileset");
}

#[test]
fn field_widgets_use_platform_font_metrics_and_keep_their_bounds_at_both_dpi() {
    for (os, size) in [
        (egui::os::OperatingSystem::Windows, 12.0),
        (egui::os::OperatingSystem::Nix, 13.0),
        (egui::os::OperatingSystem::Mac, 13.0),
    ] {
        for dpi in [1.0, 2.0] {
            let labels = [
                "Blocks: 256",
                "Map: 255 x 255",
                "ID: $FF",
                "X/Y ($FE, $FE)",
                "Event: X/Y ($1FD, $1FD)",
            ]
            .map(str::to_owned);
            let mut h = Harness::builder()
                .with_size(Vec2::new(800.0, 90.0))
                .with_os(os)
                .with_pixels_per_point(dpi)
                .build_ui(|ui| {
                    draw(
                        ui,
                        Rect::from_min_size(
                            Pos2::new(8.0, 8.0),
                            Vec2::new(ui.max_rect().width(), 23.0),
                        ),
                        &labels,
                        "Diagnostic",
                    );
                });
            h.run();
            let widths = h.ctx.fonts_mut(|fonts| {
                SAMPLES.map(|sample| {
                    fonts
                        .layout_no_wrap(
                            sample.into(),
                            egui::FontId::proportional(size),
                            Color32::WHITE,
                        )
                        .size()
                        .x
                        .ceil()
                        + 16.0
                })
            });
            let mut x = 9.0;
            for (label, width) in labels.iter().zip(widths) {
                let rect = h.get_by_label(label).rect();
                assert_eq!(
                    rect,
                    Rect::from_min_size(Pos2::new(x, 9.0), Vec2::new(width, 21.0))
                );
                x += width + 2.0;
            }
            let original = h.get_by_label("Blocks: 256").rect();
            h.set_size(Vec2::new(200.0, 90.0));
            h.run();
            assert_eq!(h.get_by_label("Blocks: 256").rect(), original);
        }
    }
}

#[cfg(feature = "visual-tests")]
#[test]
fn theme_frame_separator_pixels_and_narrow_clipping_match_source() {
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for (os, name) in [
        (egui::os::OperatingSystem::Windows, "windows"),
        (egui::os::OperatingSystem::Nix, "unix"),
    ] {
        for dark in [false, true] {
            for dpi in [1_u32, 2] {
                let labels = [
                    "Blocks: 256",
                    "Map: 255 x 255",
                    "ID: $FF",
                    "X/Y ($FE, $FE)",
                    "Event: X/Y ($1FD, $1FD)",
                ]
                .map(str::to_owned);
                let mut h = Harness::builder()
                    .with_size(Vec2::new(640.0, 64.0))
                    .with_os(os)
                    .with_pixels_per_point(dpi as f32)
                    .with_theme(if dark {
                        egui::Theme::Dark
                    } else {
                        egui::Theme::Light
                    })
                    .build_ui(|ui| {
                        ui.painter().rect_filled(
                            ui.max_rect(),
                            0.0,
                            Color32::from_rgb(127, 20, 160),
                        );
                        draw(
                            ui,
                            Rect::from_min_size(
                                Pos2::new(8.0, 8.0),
                                Vec2::new(ui.max_rect().width() - 16.0, 23.0),
                            ),
                            &labels,
                            "Diagnostic",
                        );
                    });
                h.run();
                let image = h.render().unwrap();
                let (background, upper, lower, sep_upper, sep_lower) = if dark {
                    (83, 106, 40, 56, 116)
                } else {
                    (206, 218, 193, 186, 218)
                };
                let x = h.get_by_label("Blocks: 256").rect().right() as u32;
                for (x, y, expected) in [
                    (8, 8, upper),
                    (8, 29, upper),
                    (615, 8, lower),
                    (8, 30, lower),
                    (9, 9, background),
                    (x, 9, sep_upper),
                    (x, 28, sep_upper),
                    (x, 29, sep_lower),
                    (x + 1, 9, sep_lower),
                ] {
                    assert_eq!(
                        image.get_pixel(x * dpi, y * dpi).0,
                        [expected, expected, expected, 255]
                    );
                }
                if dpi == 1 {
                    snapshots.add(h.try_snapshot(format!(
                        "status-bar-{name}-{}",
                        if dark { "dark" } else { "light" }
                    )));
                }
                h.set_size(Vec2::new(200.0, 64.0));
                h.run();
                let image = h.render().unwrap();
                // Truncated fields cannot overwrite the enclosing frame or
                // escape into the background beyond the 176-point right edge.
                for y in 8..31 {
                    assert_eq!(
                        image.get_pixel(175 * dpi, y * dpi).0,
                        [lower, lower, lower, 255]
                    );
                    assert_eq!(image.get_pixel(176 * dpi, y * dpi).0, [127, 20, 160, 255]);
                }
            }
        }
    }
    snapshots.unwrap();
}
