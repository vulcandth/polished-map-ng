//! Event-field form layout, text editing and draft acceptance. LGPL-3.0-only.
use super::*;

impl Editor {
    pub(in crate::editor) fn event_dialog(&mut self, ctx: &egui::Context) {
        if let Some(change) = &self.event_ui.change {
            let (action, path) = match change {
                EventChange::Load(path) => ("Load", Some(path.as_path())),
                EventChange::Reload | EventChange::Unload => (
                    if matches!(change, EventChange::Reload) {
                        "Reload"
                    } else {
                        "Unload"
                    },
                    self.loaded
                        .as_ref()
                        .and_then(|l| l.events.as_ref())
                        .map(|d| d.source().path()),
                ),
            };
            let name = path.map(save_feedback::basename).unwrap_or_default();
            let message = format!("The events have been edited!\n\n{action} {name} anyway?");
            let (accept, cancel) = message_dialog::confirm(
                ctx,
                "replace-events",
                &message,
                self.discard_windows_buttons,
                ["Discard Event Changes", "Cancel"],
            );
            if cancel {
                self.event_ui.change = None;
            } else if accept && let Some(change) = self.event_ui.change.take() {
                self.perform_event_change(change);
            }
            return;
        }
        let Some(mut form) = self.event_ui.form.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        dialog_host::client(ctx, egui::Id::unique("edit-event"), |ui| {
            dialog_host::heading(ui, "Event Options");
            let font = egui::FontId::proportional(if form.windows_buttons { 12.0 } else { 13.0 });
            ui.style_mut().override_font_id = Some(font.clone());
            let measure = |s: &str| {
                ui.painter()
                    .layout_no_wrap(s.into(), font.clone(), ui.visuals().text_color())
                    .size()
                    .x
                    .ceil()
            };
            let mut offset = measure("X:") + 4.0;
            if form.definition.prefix.is_some() {
                offset = offset.max(measure("Prefix:") + 4.0);
            }
            if form.definition.suffix.is_some() {
                offset = offset.max(measure("Suffix:") + 4.0);
            }
            offset += 10.0;
            let field_width = 290.0 - offset;
            let height = |s: &Option<String>| {
                s.as_ref().map_or(0.0, |s| {
                    if measured_field_lines(s, field_width, measure) <= 1 {
                        22.0
                    } else {
                        // Preserve the source's opening mode, but size wrapped
                        // text using the actual renderer and its inner width.
                        (ui.painter()
                            .layout(
                                s.clone(),
                                font.clone(),
                                ui.visuals().text_color(),
                                field_width - EVENT_FIELD_MARGIN.sum().x,
                            )
                            .size()
                            .y
                            .ceil()
                            + EVENT_FIELD_MARGIN.sum().y)
                            .max(22.0)
                    }
                })
            };
            // Option_Dialog::show refreshes once. Editing text must not resize
            // the dialog or change an input's mode underneath the cursor.
            let first_frame = form.field_heights.is_none();
            let [ph, sh] = *form.field_heights.get_or_insert_with(|| {
                [
                    height(&form.definition.prefix),
                    height(&form.definition.suffix),
                ]
            });
            let mut multiline_focused = false;
            let ch = 48.0
                + if ph > 0.0 { ph + 4.0 } else { 0.0 }
                + if sh > 0.0 { sh + 4.0 } else { 0.0 };
            let number_width = if form.definition.hex_coordinates {
                measure("AA").max(measure("FF")) + 26.0
            } else {
                measure("999") + 26.0
            };
            let y_offset = measure("Y:") + 14.0;
            let line = format!("Line: {}", form.definition.line);
            let line_width = measure(&line) + 12.0;
            let (area, _) = ui.allocate_exact_size(Vec2::new(300.0, ch + 58.0), Sense::hover());
            #[cfg(test)]
            {
                form.bounds = area;
            }
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            map_options::label(ui, rect(10.0, 10.0, 280.0 - line_width, 22.0), &form.title);
            map_options::label(ui, rect(290.0 - line_width, 10.0, line_width, 22.0), &line);
            let mut y = 36.0;
            if let Some(field) = &mut form.definition.prefix {
                outside_field_label(ui, rect(offset, y, field_width, ph), "Prefix:", &font);
                let response = event_field(ui, rect(offset, y, field_width, ph), field, "prefix");
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Event prefix")
                });
                multiline_focused |= ph > 22.0 && response.has_focus();
                accept |= ph <= 22.0
                    && response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter));
                y += ph + 4.0;
            }
            for ((label, value, x), default) in [
                ("X:", &mut form.definition.x, offset),
                (
                    "Y:",
                    &mut form.definition.y,
                    offset + number_width + y_offset,
                ),
            ]
            .into_iter()
            .zip(&mut form.defaults)
            {
                // FLTK places the coordinate label outside the control. Do not
                // wrap X into the slightly narrower measured Y-label width.
                ui.painter().text(
                    area.min + Vec2::new(x - 3.0, y + 11.0),
                    egui::Align2::RIGHT_CENTER,
                    label,
                    font.clone(),
                    ui.visuals().text_color(),
                );
                let mut spinner = numeric_input::IntegerInput::new(&mut *value, -6..=249);
                if form.definition.hex_coordinates {
                    spinner = spinner.hex();
                }
                let response = ui.put(rect(x, y, number_width, 22.0), spinner);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::SpinButton,
                        true,
                        if label == "X:" { "Event X" } else { "Event Y" },
                    )
                });
                default.restore(ui, &response, value);
            }
            y += 26.0;
            if let Some(field) = &mut form.definition.suffix {
                outside_field_label(ui, rect(offset, y, field_width, sh), "Suffix:", &font);
                let response = event_field(ui, rect(offset, y, field_width, sh), field, "suffix");
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Event suffix")
                });
                multiline_focused |= sh > 22.0 && response.has_focus();
                accept |= sh <= 22.0
                    && response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter));
            }
            let (ok, no) = if form.windows_buttons {
                (116.0, 210.0)
            } else {
                (210.0, 116.0)
            };
            let ok = ui.put(rect(ok, ch + 26.0, 80.0, 22.0), egui::Button::new("OK"));
            if first_frame {
                ok.request_focus();
            }
            accept |= ok.clicked();
            cancel = ui
                .put(rect(no, ch + 26.0, 80.0, 22.0), egui::Button::new("Cancel"))
                .clicked();
            cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            if !multiline_focused {
                accept |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            }
        });
        if cancel {
            return;
        }
        if accept {
            let d = &form.definition;
            if let Some(doc) = self.loaded.as_mut().and_then(|l| l.events.as_mut()) {
                match doc.apply(&[EventCommand::Edit {
                    index: form.index,
                    x: d.x,
                    y: d.y,
                    prefix: d.prefix.clone(),
                    suffix: d.suffix.clone(),
                }]) {
                    Ok(()) => self.status = "Event updated".into(),
                    Err(e) => {
                        self.error = Some(e.to_string());
                        self.event_ui.form = Some(form);
                    }
                }
            }
        } else {
            self.event_ui.form = Some(form);
        }
    }
}

fn outside_field_label(ui: &egui::Ui, control: Rect, text: &str, font: &egui::FontId) {
    // Fl_Group::draw_outside_label leaves three pixels. Multiline inputs use
    // FL_ALIGN_LEFT_TOP; single-line labels are vertically centered.
    let (y, align) = if control.height() > 22.0 {
        (control.top(), egui::Align2::RIGHT_TOP)
    } else {
        (control.center().y, egui::Align2::RIGHT_CENTER)
    };
    ui.painter().text(
        Pos2::new(control.left() - 3.0, y),
        align,
        text,
        font.clone(),
        ui.visuals().text_color(),
    );
}

/// FLTK fl_measure/expand_text_ with draw_symbols=false: wrap at ASCII spaces,
/// never split an oversized first word, and do not count a trailing newline
/// as an extra line. Width measurements remain supplied by the active font.
pub(in crate::editor) fn measured_field_lines(
    text: &str,
    width: f32,
    measure: impl Fn(&str) -> f32,
) -> usize {
    let chars: Vec<_> = text.chars().collect();
    let mut position = 0;
    let mut lines = 0;
    while position < chars.len() {
        let mut buffer = String::new();
        let mut word_end = 0;
        let mut word_start = position;
        let mut used_width = 0.0;
        loop {
            let c = chars.get(position).copied().unwrap_or('\0');
            if matches!(c, '\0' | ' ' | '\n') {
                if word_start < position {
                    let new_width = used_width + measure(&buffer[word_end..]);
                    if word_end > 0 && new_width.trunc() > width {
                        position = word_start;
                        break;
                    }
                    word_end = buffer.len();
                    used_width = new_width;
                }
                if c == '\0' {
                    position = chars.len();
                    break;
                }
                if c == '\n' {
                    position += 1;
                    break;
                }
                word_start = position + 1;
            }
            match c {
                '\t' => buffer.extend(std::iter::repeat_n(' ', 8 - buffer.chars().count() % 8)),
                c if c < ' ' || c == '\u{7f}' => {
                    buffer.push('^');
                    buffer.push(char::from(c as u8 ^ 0x40));
                }
                c => buffer.push(c),
            }
            position += 1;
        }
        lines += 1;
    }
    lines
}

/// A source input keeps its opening bounds and scrolls internally as text grows.
const EVENT_FIELD_MARGIN: egui::Margin = egui::Margin::symmetric(4, 3);

fn event_field(ui: &mut egui::Ui, rect: Rect, text: &mut String, name: &str) -> egui::Response {
    if rect.height() <= 22.0 {
        return ui.put(rect, egui::TextEdit::singleline(text).cursor_at_end(false));
    }
    // The frame stays outside the scroll area, so its bottom edge remains
    // visible even after editing grows the contents beyond their opening size.
    let background = ui.painter().add(egui::Shape::Noop);
    let inner = rect - EVENT_FIELD_MARGIN;
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
    child.set_clip_rect(inner.intersect(ui.clip_rect()));
    let mut response = egui::ScrollArea::vertical()
        .id_salt(("event-field", name))
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .max_height(inner.height())
        .show(&mut child, |ui| {
            ui.add_sized(
                inner.size(),
                egui::TextEdit::multiline(text)
                    .frame(egui::Frame::NONE)
                    .desired_rows(1)
                    .cursor_at_end(false),
            )
        })
        .inner;
    let visuals = ui.style().interact(&response);
    ui.painter().set(
        background,
        egui::Shape::Rect(egui::epaint::RectShape::new(
            rect,
            visuals.corner_radius,
            ui.visuals().text_edit_bg_color(),
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        )),
    );
    response.rect = rect;
    response.interact_rect = rect.intersect(ui.clip_rect());
    response
}
