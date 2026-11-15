//! Pinned master Resize_Dialog::{refresh_content,anchor_button_cb} and
//! Option_Dialog::refresh. LGPL-3.0-only. Client dimensions exclude native chrome.
use super::*;

const RESIZE_HELP: &str = concat!(
    "Resize map (Enter). New cells use block 0; resizing clears undo history ",
    "and moves loaded events. Map constants are not updated.",
);

#[derive(Clone, Copy, Debug)]
struct Layout {
    size: Vec2,
    width: Rect,
    height: Rect,
    anchors: [Rect; 9],
    ok: Rect,
    cancel: Rect,
}

impl Layout {
    fn measured(windows: bool, measure: impl Fn(&str) -> f32) -> Self {
        let rect = |x, y, w, h| Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h));
        let offset = 10.0 + measure("Width:").max(measure("Height:")) + 4.0;
        let width = measure("999") + 4.0 + 22.0;
        let anchor_x = offset + width + 20.0;
        // Source content reports 74px; its three 24px rows + 2px gaps occupy 76px.
        let (ok, cancel) = if windows {
            (36.0, 130.0)
        } else {
            (130.0, 36.0)
        };
        Self {
            size: Vec2::new(220.0, 132.0),
            width: rect(offset, 10.0, width, 22.0),
            height: rect(offset, 36.0, width, 22.0),
            anchors: std::array::from_fn(|i| {
                rect(
                    anchor_x + (i % 3) as f32 * 26.0,
                    10.0 + (i / 3) as f32 * 26.0,
                    24.0,
                    24.0,
                )
            }),
            ok: rect(ok, 100.0, 80.0, 22.0),
            cancel: rect(cancel, 100.0, 80.0, 22.0),
        }
    }
}

impl Editor {
    pub(super) fn resize_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut form) = self.resize_form.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        dialog_host::client(ctx, egui::Id::unique("resize-map"), |ui| {
            dialog_host::heading(ui, "Resize Map");
            let font = egui::FontId::proportional(if form.windows_buttons { 12.0 } else { 13.0 });
            ui.style_mut().override_font_id = Some(font.clone());
            let layout = Layout::measured(form.windows_buttons, |s| {
                ui.painter()
                    .layout_no_wrap(s.into(), font.clone(), ui.visuals().text_color())
                    .size()
                    .x
                    .ceil()
            });
            let area = ui.allocate_exact_size(layout.size, Sense::hover()).0;
            #[cfg(test)]
            {
                form.bounds = area;
            }
            let rect = |r: Rect| r.translate(area.min.to_vec2());
            for (((name, value), control), default) in
                [("Width", &mut form.width), ("Height", &mut form.height)]
                    .into_iter()
                    .zip([layout.width, layout.height])
                    .zip(&mut form.defaults)
            {
                ui.painter().text(
                    rect(control).left_center() - Vec2::new(3.0, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("{name}:"),
                    font.clone(),
                    ui.visuals().text_color(),
                );
                let response = ui.put(
                    rect(control),
                    numeric_input::IntegerInput::new(&mut *value, 1..=255),
                );
                response
                    .widget_info(|| egui::WidgetInfo::labeled(egui::Role::SpinButton, true, name));
                default.restore(ui, &response, value);
            }
            let labels = [
                "Top left",
                "Top center",
                "Top right",
                "Middle left",
                "Center",
                "Middle right",
                "Bottom left",
                "Bottom center",
                "Bottom right",
            ];
            for (i, anchor) in ResizeAnchor::ALL.into_iter().enumerate() {
                let selected = form.anchor == anchor;
                let response = ui.put(
                    rect(layout.anchors[i]),
                    egui::Button::new("").selected(selected),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::RadioButton, true, labels[i])
                });
                let active = ResizeAnchor::ALL
                    .iter()
                    .position(|a| *a == form.anchor)
                    .unwrap_or(4);
                let direction = Vec2::new(
                    (i % 3) as f32 - (active % 3) as f32,
                    (i / 3) as f32 - (active / 3) as f32,
                );
                let color = ui.visuals().text_color();
                if selected {
                    ui.painter().rect_filled(
                        Rect::from_center_size(response.rect.center(), Vec2::splat(6.0)),
                        0.0,
                        color,
                    );
                } else if direction.x.abs() <= 1.0 && direction.y.abs() <= 1.0 {
                    let direction = direction.normalized() * 8.0;
                    ui.painter().arrow(
                        response.rect.center() - direction * 0.5,
                        direction,
                        egui::Stroke::new(1.0, color),
                    );
                }
                if response.on_hover_text(labels[i]).clicked() {
                    form.anchor = anchor;
                }
            }
            let ok = ui.put(rect(layout.ok), egui::Button::new("OK"));
            if !form.opened {
                ok.request_focus();
                form.opened = true;
            }
            ok.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Resize"));
            accept = ok.on_hover_text(RESIZE_HELP).clicked();
            cancel = ui
                .put(rect(layout.cancel), egui::Button::new("Cancel"))
                .on_hover_text("Cancel (Esc)")
                .clicked();
            accept |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        });
        if accept && !cancel {
            self.resize_anchor = form.anchor;
            self.stroke.clear();
            self.apply(Command::Resize {
                width: form.width,
                height: form.height,
                anchor: form.anchor,
            });
        } else if !cancel {
            self.resize_form = Some(form);
        }
    }
}

#[cfg(test)]
mod tests;
