//! Full-width dropdown rows; labels retain padding inside the interactive area.
use eframe::egui;

pub(crate) fn combo<'a>(id: &str) -> egui::ComboBox<'a> {
    egui::ComboBox::from_id_salt(id).popup_style(
        (|style: &mut egui::Style| {
            // Move horizontal frame padding inside the selectable rows. The
            // text keeps its inset, but the strip beside it is now clickable.
            style.spacing.button_padding.x += f32::from(style.spacing.menu_margin.left);
            style.spacing.menu_margin.left = 0;
            style.spacing.menu_margin.right = 0;
        })
        .into(),
    )
}

pub(crate) fn label<'a>(
    ui: &mut egui::Ui,
    selected: bool,
    text: impl egui::IntoAtoms<'a>,
) -> egui::Response {
    ui.add(egui::Button::selectable(selected, text).min_size(egui::vec2(ui.available_width(), 0.0)))
}

pub(crate) fn value<'a, T: PartialEq>(
    ui: &mut egui::Ui,
    current: &mut T,
    choice: T,
    text: impl egui::IntoAtoms<'a>,
) -> egui::Response {
    let mut response = label(ui, *current == choice, text);
    if response.clicked() && *current != choice {
        *current = choice;
        response.mark_changed();
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn dropdown_rows_include_both_edges_and_preserve_disabled_and_keyboard_behavior() {
        for scale in [1.0, 1.5, 2.0] {
            for right_edge in [false, true] {
                let mut h = Harness::builder()
                    .with_size(egui::vec2(320.0, 220.0))
                    .with_pixels_per_point(scale)
                    .with_context_setup(|ctx| {
                        // Popup areas use the context style, as configured by
                        // Editor startup, rather than inheriting the parent Ui.
                        ctx.all_styles_mut(|s| s.spacing.scroll.floating = false);
                    })
                    .build_ui_state(
                        |ui, selected: &mut usize| {
                            combo("choices")
                                .width(230.0)
                                .height(100.0)
                                .selected_text("Choices")
                                .show_ui(ui, |ui| {
                                    value(ui, selected, 0, "Short");
                                    value(ui, selected, 1, "A longer label");
                                    ui.add_enabled_ui(false, |ui| {
                                        value(ui, selected, 2, "Disabled");
                                    });
                                    for i in 3..40 {
                                        value(ui, selected, i, format!("Choice {i}"));
                                    }
                                })
                                .response
                                .widget_info(|| {
                                    egui::WidgetInfo::labeled(egui::Role::ComboBox, true, "Choices")
                                });
                        },
                        1,
                    );
                h.get_by_label("Choices").click();
                h.run();
                let short = h.get_by_label("Short").rect();
                let long = h.get_by_label("A longer label").rect();
                assert_eq!(short.x_range(), long.x_range());
                assert!(short.width() > 190.0);
                let point = egui::pos2(
                    if right_edge {
                        short.right() - 1.0
                    } else {
                        short.left() + 1.0
                    },
                    short.center().y,
                );
                h.hover_at(point);
                h.run();
                for pressed in [true, false] {
                    h.event(egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                    h.run();
                }
                assert_eq!(
                    *h.state(),
                    0,
                    "scale={scale} right={right_edge} row={short:?} point={point:?}"
                );
                h.get_by_label("Choices").click();
                h.run();
                h.get_by_label("Disabled").click();
                h.run();
                assert_eq!(*h.state(), 0);
                if h.query_by_label("A longer label").is_none() {
                    h.get_by_label("Choices").click();
                    h.run();
                }
                h.get_by_label("A longer label").focus();
                h.run();
                h.key_press(egui::Key::Enter);
                h.run();
                assert_eq!(*h.state(), 1);
            }
        }
    }
}
