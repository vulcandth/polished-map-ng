//! Master Option_Dialog::refresh and Map_Options_Dialog::refresh_content.
//! LGPL-3.0-only. The caption/advanced section are outside the source client.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct Layout {
    pub size: Vec2,
    pub header: Rect,
    pub dimensions: Rect,
    pub height: Option<Rect>,
    pub bytes: Rect,
    pub tileset: Rect,
    pub roof: Rect,
    pub ok: Rect,
    pub cancel: Rect,
}

impl Layout {
    fn measured(
        has_sizes: bool,
        windows: bool,
        measure: impl Fn(&str) -> f32,
        tileset: f32,
        roof: f32,
    ) -> Self {
        let rect = |x, y, w| Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, 22.0));
        let offset = 10.0
            + (measure(if has_sizes { "Size:" } else { "Width:" }) + 4.0)
                .max(measure("Tileset:") + 4.0);
        let dimensions = rect(
            offset,
            36.0,
            measure(if has_sizes { "9999 x 9999" } else { "999" }) + 4.0 + 22.0,
        );
        let height = (!has_sizes).then(|| {
            rect(
                dimensions.right() + 10.0 + measure("Height:"),
                36.0,
                dimensions.width(),
            )
        });
        let end = height.unwrap_or(dimensions).right();
        let (ok, cancel) = if windows {
            (116.0, 210.0)
        } else {
            (210.0, 116.0)
        };
        Self {
            size: Vec2::new(300.0, 158.0),
            header: rect(10.0, 10.0, 280.0),
            dimensions,
            height,
            bytes: rect(end + 10.0, 36.0, 280.0 - end),
            tileset: rect(offset, 62.0, tileset + 12.0 + 22.0),
            roof: rect(offset, 88.0, roof + 12.0 + 22.0),
            ok: rect(ok, 126.0, 80.0),
            cancel: rect(cancel, 126.0, 80.0),
        }
    }
}

pub(super) fn label(ui: &mut egui::Ui, rect: Rect, text: &str) -> egui::Response {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.add(egui::Label::new(text).truncate())
}

fn combo(
    ui: &mut egui::Ui,
    rect: Rect,
    name: &str,
    selected: &str,
    enabled: bool,
    contents: impl FnOnce(&mut egui::Ui),
) {
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    child.spacing_mut().interact_size.y = 22.0;
    child.add_enabled_ui(enabled, |ui| {
        egui::ComboBox::from_id_salt(name)
            .width(rect.width())
            .truncate()
            .selected_text(selected)
            .show_ui(ui, contents)
            .response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::ComboBox, enabled, name));
    });
}

impl Editor {
    pub(super) fn map_options_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut form) = self.open_form.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        let choosing = egui::Popup::is_any_open(ctx);
        let windows = cfg!(windows);
        #[cfg(test)]
        let windows = ctx
            .data(|d| d.get_temp::<bool>(egui::Id::unique("test-map-options-windows")))
            .unwrap_or(windows);
        let font_size = if windows { 12.0 } else { 13.0 };
        dialog_host::client(ctx, egui::Id::unique("open-options"), |ui| {
            dialog_host::heading(ui, "Map Options");
            let font = egui::FontId::proportional(font_size);
            let painter = ui.painter().clone();
            let color = ui.visuals().text_color();
            let measure = |s: &str| {
                painter
                    .layout_no_wrap(s.into(), font.clone(), color)
                    .size()
                    .x
                    .ceil()
            };
            let has_sizes = form.sizes.as_ref().is_some_and(|s| !s.choices().is_empty());
            let layout = Layout::measured(
                has_sizes,
                windows,
                measure,
                form.tilesets
                    .iter()
                    .map(|c| measure(&c.label))
                    .fold(0.0, f32::max),
                form.roofs
                    .iter()
                    .map(|r| measure(r))
                    .fold(measure("(none)"), f32::max),
            );
            let bounds = ui.allocate_exact_size(layout.size, Sense::hover()).0;
            #[cfg(test)]
            {
                form.client_bounds = Some(bounds);
                form.client_layout = Some(layout);
            }
            let rect = |r: Rect| r.translate(bounds.min.to_vec2());
            let mut client = ui.new_child(egui::UiBuilder::new().max_rect(bounds));
            client.set_clip_rect(bounds.intersect(ui.clip_rect()));
            client.style_mut().override_font_id = Some(font.clone());
            client.spacing_mut().interact_size.y = 22.0;
            let header = if form.new_map {
                "New Map:".into()
            } else {
                format!(
                    "{}:",
                    form.path.file_name().unwrap_or_default().to_string_lossy()
                )
            };
            label(&mut client, rect(layout.header), &header)
                .on_hover_text(form.path.display().to_string());
            let outside = |control: Rect, text_width: f32| {
                Rect::from_min_size(
                    Pos2::new(control.left() - text_width - 3.0, control.top()),
                    Vec2::new(text_width, 22.0),
                )
            };
            // FLTK Fl_Group::draw_outside_label leaves three points before the control.
            for (control, name) in [
                (
                    layout.dimensions,
                    if has_sizes { "Size:" } else { "Width:" },
                ),
                (layout.tileset, "Tileset:"),
                (layout.roof, "Roof:"),
            ] {
                label(&mut client, rect(outside(control, measure(name))), name);
            }
            if let Some(height) = layout.height {
                label(
                    &mut client,
                    rect(outside(height, measure("Height:"))),
                    "Height:",
                );
                client
                    .put(
                        rect(layout.dimensions),
                        numeric_input::IntegerInput::new(&mut form.width, 1..=255),
                    )
                    .widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::SpinButton, true, "Width")
                    });
                client
                    .put(
                        rect(height),
                        numeric_input::IntegerInput::new(&mut form.height, 1..=255),
                    )
                    .widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::SpinButton, true, "Height")
                    });
            } else if let Some(sizes) = &form.sizes {
                let mut dimensions = (form.width, form.height);
                combo(
                    &mut client,
                    rect(layout.dimensions),
                    "Size",
                    &format!("{} x {}", form.width, form.height),
                    true,
                    |ui| {
                        for &(w, h) in sizes.choices() {
                            ui.selectable_value(&mut dimensions, (w, h), format!("{w} x {h}"));
                        }
                    },
                );
                (form.width, form.height) = dimensions;
            }
            if let Some(sizes) = &form.sizes {
                label(
                    &mut client,
                    rect(layout.bytes),
                    &format!("({} B)", sizes.block_count()),
                );
            }
            let current = form
                .tilesets
                .iter()
                .find(|c| c.name == form.tileset)
                .map_or("(none)", |c| c.label.as_str());
            combo(
                &mut client,
                rect(layout.tileset),
                "Tileset",
                current,
                !form.tilesets.is_empty(),
                |ui| {
                    for choice in &form.tilesets {
                        ui.selectable_value(&mut form.tileset, choice.name.clone(), &choice.label);
                    }
                },
            );
            let current = if form.roof.is_empty() {
                "(none)".to_owned()
            } else {
                form.roof.clone()
            };
            combo(
                &mut client,
                rect(layout.roof),
                "Roof",
                &current,
                !form.roofs.is_empty(),
                |ui| {
                    ui.selectable_value(&mut form.roof, String::new(), "(none)");
                    for name in &form.roofs {
                        ui.selectable_value(&mut form.roof, name.clone(), name);
                    }
                },
            );
            let enabled = form.root == form.choices_root
                && form.tilesets.iter().any(|c| c.name == form.tileset);
            client.add_enabled_ui(enabled, |ui| {
                let response = ui.put(rect(layout.ok), egui::Button::new("OK"));
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::Button,
                        enabled,
                        if form.new_map {
                            "Create map"
                        } else {
                            "Open map"
                        },
                    )
                });
                accept = response.on_hover_text("OK (Enter)").clicked();
            });
            cancel = client
                .put(rect(layout.cancel), egui::Button::new("Cancel"))
                .on_hover_text("Cancel (Esc)")
                .clicked();
            // Keep NG project overrides outside the measured source client.
            egui::CollapsingHeader::new("Project options")
                .open((form.root != form.choices_root).then_some(true))
                .show(ui, |ui| {
                    let label = ui.label("Project folder");
                    ui.text_edit_singleline(&mut form.root)
                        .labelled_by(label.id);
                    if form.root != form.choices_root
                        && ui.button("Reload project options").clicked()
                        && let Err(error) = form.refresh_choices()
                    {
                        self.error = Some(error.to_string());
                    }
                    ui.checkbox(&mut form.allow_256, "Allow all 256 tiles");
                });
            // Advanced controls run after the source client: revalidate after
            // editing the root so Enter cannot submit stale asset choices.
            let enabled = form.root == form.choices_root
                && form.tilesets.iter().any(|c| c.name == form.tileset);
            accept &= enabled;
            if !choosing && !egui::Popup::is_any_open(ctx) {
                accept |= enabled
                    && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            }
        });
        if cancel {
            return;
        }
        if accept {
            self.start_load(form, ctx);
        } else {
            self.open_form = Some(form);
        }
    }
}

#[cfg(test)]
mod tests;
