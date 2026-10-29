use super::*;

fn recorded(drawing: Drawing) -> String {
    use std::fmt::Write;
    let mut result = String::new();
    for (color, primitive) in drawing.0 {
        let (kind, coords): (&str, Vec<i32>) = match primitive {
            Primitive::Rect(r) => ("rect", r.into()),
            Primitive::Pie(r) => ("pie", r.into()),
            Primitive::Arc(r, angles) => ("arc", r.into_iter().chain(angles).collect()),
            Primitive::Polygon(p) => ("polygon", p.into_iter().flatten().collect()),
        };
        write!(result, "{kind},{},{},{}", color.r(), color.g(), color.b()).unwrap();
        for coord in coords {
            write!(result, ",{coord}").unwrap();
        }
        result.push(';');
    }
    result
}

#[test]
#[ignore = "requires tools/menu_marks_oracle.py; runs in visual CI"]
fn reference_menu_marks_match_original_drawing_commands() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.scratch/reference-menu-marks.tsv");
    let data = std::fs::read_to_string(path).unwrap();
    let mut drawings = 0;
    let mut colors = 0;
    for line in data.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields[0] == "color" {
            let actual = foreground(fields[1] == "1", fields[2] == "1", fields[3] == "1");
            let rgb =
                u32::from(actual.r()) << 16 | u32::from(actual.g()) << 8 | u32::from(actual.b());
            assert_eq!(rgb, fields[4].parse::<u32>().unwrap(), "{line}");
            colors += 1;
        } else {
            assert_eq!(
                recorded(drawing(
                    fields[0] == "1",
                    fields[2] == "1",
                    fields[3] == "1",
                    fields[1].parse().unwrap()
                )),
                fields[4],
                "case {:?}",
                &fields[..4]
            );
            drawings += 1;
        }
    }
    assert_eq!((drawings, colors), (72, 8));
}

#[test]
fn disabled_and_selected_colors_keep_source_contrast_rules() {
    assert_eq!(foreground(false, false, false), Color32::from_gray(157));
    assert_eq!(foreground(true, false, false), Color32::from_gray(139));
    assert_eq!(foreground(false, true, true), Color32::WHITE);
    assert_eq!(foreground(true, true, true), Color32::BLACK);
    for dark in [false, true] {
        assert_eq!(
            foreground(dark, false, true),
            foreground(dark, false, false)
        );
    }
    // Explicit 12x12 box, independent FLTK example: a six-vertex filled tick.
    assert_eq!(
        check_vertices([3, 3, 12, 12]),
        [[4, 7], [7, 10], [13, 4], [13, 7], [7, 13], [4, 10]]
    );
}

#[test]
fn menu_rows_preserve_disabled_colors_marks_and_interaction_at_both_scales() {
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    let specs = [
        Spec::new("Check on", Mark::Check, "C+G", false),
        Spec::new("Check disabled", Mark::Check, "C+P", false),
        Spec::new("Check off", Mark::Check, "", false),
        Spec::new("Radio on", Mark::Radio, "", false),
        Spec::new("Radio disabled", Mark::Radio, "", false),
        Spec::new("Radio off", Mark::Radio, "", false),
    ];
    for dark in [false, true] {
        for dpi in [1.0, 2.0] {
            let mut h = Harness::builder()
                .with_size(Vec2::new(280.0, 150.0))
                .with_theme(if dark {
                    egui::Theme::Dark
                } else {
                    egui::Theme::Light
                })
                .with_pixels_per_point(dpi)
                .build_ui(|ui| {
                    test_font_size(ui.ctx(), 13.0);
                    ui.painter().rect_filled(
                        ui.max_rect(),
                        0.0,
                        Color32::from_gray(if dark { 83 } else { 206 }),
                    );
                    scope(ui, "", &specs, |ui| {
                        for (index, spec) in specs.iter().enumerate() {
                            row_at(
                                ui,
                                !spec.name.contains("disabled"),
                                index,
                                Some(!spec.name.contains("off")),
                            );
                        }
                    });
                });
            h.remove_cursor();
            h.run();
            // A fresh painter must not brighten/dim the computed FLTK colors.
            for shape in &h.output().shapes {
                if let egui::Shape::Text(text) = &shape.shape
                    && text.galley.text().contains("disabled")
                {
                    assert_eq!(
                        text.override_text_color,
                        Some(foreground(dark, false, false))
                    );
                }
            }
            for name in ["Check disabled", "Radio disabled"] {
                assert!(h.get_by_label(name).accesskit_node().is_disabled());
            }
            let meshes: Vec<_> = h
                .output()
                .shapes
                .iter()
                .filter_map(|shape| {
                    if let egui::Shape::Mesh(mesh) = &shape.shape {
                        Some(mesh)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(meshes.len(), 2, "both on-checkboxes must use filled ticks");
            for mesh in meshes {
                assert_eq!(mesh.vertices.len(), 6);
                assert!(mesh.vertices.iter().all(|vertex| vertex.color
                    == if dark {
                        Color32::from_gray(214)
                    } else {
                        Color32::from_rgb(61, 123, 187)
                    }));
            }
            #[cfg(feature = "visual-tests")]
            {
                let image = h.render().unwrap();
                for name in ["Check on", "Check disabled"] {
                    let rect = h.get_by_label(name).rect();
                    let inset = 1 + (rect.height() as i32 - 4 - 14 + 1) / 2;
                    let x = ((rect.left() + 4.0) * dpi) as u32;
                    let y = ((rect.top() + inset as f32) * dpi) as u32;
                    let gray = if dark { 48 } else { 128 };
                    assert_eq!(image.get_pixel(x, y).0, [gray, gray, gray, 255]);
                }
                snapshots.add(h.try_snapshot(format!(
                    "menu-marks-{}-{dpi}x",
                    if dark { "dark" } else { "light" }
                )));
            }
            // Hover changes label contrast, not the selected indicator's color.
            h.get_by_label("Check on").focus();
            h.run();
            for shape in &h.output().shapes {
                if let egui::Shape::Text(text) = &shape.shape
                    && text.galley.text() == " Check on         "
                {
                    assert_eq!(text.override_text_color, Some(foreground(dark, true, true)));
                }
            }
        }
    }
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}
