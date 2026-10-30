//! Integer controls following master hex-spinner.cpp and FLTK 1.4.5
//! Fl_Spinner/Fl_Input/Fl_Repeat_Button. See NOTICE.md and LICENSES/FLTK.txt.
use super::egui;
use egui::{Key, Modifiers, Response, Sense, TextBuffer, Ui, Vec2, Widget};
use std::ops::RangeInclusive;

#[cfg(test)]
mod tests;

#[derive(Clone, Default)]
struct EditState {
    text: String,
    value: i32,
    frame: u64,
    repeat: Option<(bool, f64)>,
}

/// Discard uncommitted text too when a middle-click restores a default.
pub(super) fn reset_text(ctx: &egui::Context, id: egui::Id) {
    ctx.data_mut(|d| d.remove::<EditState>(id));
}

pub(super) struct IntegerInput<'a, T> {
    value: &'a mut T,
    range: RangeInclusive<i32>,
    hex: bool,
}

impl<'a, T: egui::emath::Numeric> IntegerInput<'a, T> {
    pub fn new(value: &'a mut T, range: RangeInclusive<i32>) -> Self {
        Self {
            value,
            range,
            hex: false,
        }
    }

    pub fn hex(mut self) -> Self {
        self.hex = true;
        self
    }
}

fn display(value: i32, hex: bool) -> String {
    if hex {
        format!("{:02X}", value as u32)
    } else {
        value.to_string()
    }
}

fn step(value: i32, up: bool, range: &RangeInclusive<i32>) -> i32 {
    if up {
        if value >= *range.end() {
            *range.start()
        } else {
            value + 1
        }
    } else if value <= *range.start() {
        *range.end()
    } else {
        value - 1
    }
}

fn valid_paste(text: &str, hex: bool) -> bool {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if hex {
        digits.bytes().all(|c| c.is_ascii_hexdigit())
    } else if let Some(digits) = digits.strip_prefix("0x") {
        digits.bytes().all(|c| c.is_ascii_hexdigit())
    } else {
        digits.bytes().all(|c| c.is_ascii_digit())
    }
}

fn raw_number(text: &str, hex: bool) -> i64 {
    let negative = text.starts_with('-');
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    let (radix, digits) = if hex {
        (16, digits)
    } else if let Some(digits) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        (16, digits)
    } else {
        (10, digits)
    };
    let end = digits
        .bytes()
        .take_while(|c| {
            if radix == 16 {
                c.is_ascii_hexdigit()
            } else {
                c.is_ascii_digit()
            }
        })
        .count();
    let digits = &digits[..end];
    // Incomplete edits (empty/sign-only) have strtol/atof's zero value. Bound
    // oversized input without inheriting C's platform-dependent overflow.
    let magnitude = if digits.is_empty() {
        0
    } else {
        i64::from_str_radix(digits, radix).unwrap_or(i64::MAX)
    };
    let signed = if negative { -magnitude } else { magnitude };
    if hex && (i64::from(i32::MIN)..=i64::from(u32::MAX)).contains(&signed) {
        i64::from(signed as i32)
    } else {
        signed
    }
}

fn parse(text: &str, hex: bool, range: &RangeInclusive<i32>) -> i32 {
    raw_number(text, hex).clamp(i64::from(*range.start()), i64::from(*range.end())) as i32
}

struct InputBuffer<'a> {
    text: &'a mut String,
    hex: bool,
}

fn allowed_char(text: &str, at: usize, c: char, hex: bool) -> bool {
    c.is_ascii_digit()
        || (hex && c.is_ascii_hexdigit())
        || (at == 0 && matches!(c, '+' | '-'))
        || (!hex && at == 1 && text.starts_with('0') && matches!(c, 'x' | 'X'))
        || (!hex
            && at > 1
            && (text.starts_with("0x") || text.starts_with("0X"))
            && c.is_ascii_hexdigit())
}

impl TextBuffer for InputBuffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<InputBuffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.text
    }
    fn insert_text(&mut self, text: &str, index: egui::text::CharIndex) -> usize {
        let mut inserted = 0;
        for c in text.chars() {
            let at = index.0 + inserted;
            let allowed = allowed_char(self.text, at, c, self.hex);
            if allowed && self.text.len() < 64 && at <= self.text.len() {
                self.text.insert(at, c);
                inserted += 1;
            }
        }
        inserted
    }
    fn delete_char_range(&mut self, range: std::ops::Range<egui::text::CharIndex>) {
        self.text.delete_char_range(range);
    }
}

impl<T: egui::emath::Numeric> Widget for IntegerInput<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let old = self.value.to_f64() as i32;
        let mut value = old;
        let font = ui
            .style()
            .override_font_id
            .clone()
            .unwrap_or_else(|| egui::TextStyle::Body.resolve(ui.style()));
        // ui.put supplies the measured source width; flowing forms retain their
        // existing minimum sizing until their client geometry is ported.
        let minimum = ui.spacing().interact_size.x + 2.0 * ui.spacing().button_padding.x;
        let size = Vec2::new(minimum.min(ui.available_width()), 22.0);
        let (id, area) = ui.allocate_space(size);
        let frame = ui.ctx().cumulative_frame_nr();
        let mut state = ui
            .data_mut(|d| d.remove_temp::<EditState>(id))
            .filter(|s| s.value == old && frame <= s.frame + 1)
            .unwrap_or_else(|| EditState {
                text: display(old, self.hex),
                value: old,
                ..Default::default()
            });
        let active = ui.is_enabled() && ui.input(|i| i.focused) && !ui.ctx().any_popup_open();
        let focused = active && ui.memory(|m| m.has_focus(id));
        let group_focused = focused
            || (active && ui.memory(|m| m.has_focus(id.with(true)) || m.has_focus(id.with(false))));
        let mut entered = false;
        let mut pasted = false;
        let mut key_steps = Vec::new();
        if group_focused {
            // FLTK validates a key before replacing the selection. Rejecting it
            // in TextBuffer alone is too late: TextEdit has already deleted it.
            let selection = egui::TextEdit::load_state(ui.ctx(), id)
                .and_then(|s| s.cursor.char_range())
                .map(|r| r.sorted_cursors())
                .map(|[a, b]| a.index.0..b.index.0)
                .unwrap_or(state.text.len()..state.text.len());
            let mut virtual_text = state.text.clone();
            let mut selection =
                selection.start.min(virtual_text.len())..selection.end.min(virtual_text.len());
            ui.input_mut(|input| {
                entered = focused && input.consume_key(Modifiers::NONE, Key::Enter);
                input.events.retain_mut(|event| {
                    if let egui::Event::Key {
                        key: key @ (Key::ArrowUp | Key::ArrowDown),
                        pressed: true,
                        ..
                    } = event
                    {
                        key_steps.push(*key == Key::ArrowUp);
                        false
                    } else if let egui::Event::Text(text) = event
                        && focused
                    {
                        let mut accepted = String::new();
                        for c in text.chars() {
                            if allowed_char(&virtual_text, selection.start, c, self.hex)
                                && virtual_text.len() - selection.len() < 64
                            {
                                virtual_text.replace_range(selection.clone(), &c.to_string());
                                selection = selection.start + 1..selection.start + 1;
                                accepted.push(c);
                            }
                        }
                        *text = accepted;
                        !text.is_empty()
                    } else if let egui::Event::Paste(text) = event
                        && focused
                    {
                        let text = text.trim_matches(|c: char| c.is_ascii_whitespace());
                        if !text.is_empty() && text.len() <= 64 && valid_paste(text, self.hex) {
                            state.text = text.into();
                            pasted = true;
                        }
                        false
                    } else {
                        true
                    }
                });
            });
        }
        let arrows = area.height() / 2.0 + 2.0;
        let input_rect =
            egui::Rect::from_min_max(area.min, egui::pos2(area.right() - arrows, area.bottom()));
        let mut output = egui::TextEdit::singleline(&mut InputBuffer {
            text: &mut state.text,
            hex: self.hex,
        })
        .id(id)
        .font(font)
        .desired_width(input_rect.width())
        .min_size(input_rect.size())
        .show(&mut ui.new_child(egui::UiBuilder::new().max_rect(input_rect)));
        let escape = ui.input(|i| i.key_pressed(Key::Escape));
        if active && (entered || output.response.lost_focus()) && !escape {
            value = parse(&state.text, self.hex, &self.range);
        }
        if entered || pasted {
            use egui::text::{CCursor, CCursorRange};
            let end = CCursor::new(state.text.chars().count());
            output.state.cursor.set_char_range(Some(if entered {
                CCursorRange::two(CCursor::new(0), end)
            } else {
                CCursorRange::one(end)
            }));
            output.state.store(ui.ctx(), id);
        }
        let mut stepped = false;
        if !key_steps.is_empty() && !self.hex {
            // Fl_Spinner_Input commits/clips before passing arrows to its parent.
            value = parse(&state.text, false, &self.range);
        }
        for up in key_steps {
            value = step(value, up, &self.range);
            stepped = true;
        }
        let now = ui.input(|i| i.time);
        let mut held = None;
        for up in [true, false] {
            let top = if up { area.top() } else { area.center().y };
            let rect = egui::Rect::from_min_size(
                egui::pos2(input_rect.right(), top),
                Vec2::new(arrows, area.height() / 2.0),
            );
            let button = ui.interact(rect, id.with(up), Sense::click_and_drag());
            if button.has_focus() {
                ui.memory_mut(|m| {
                    m.set_focus_lock_filter(
                        button.id,
                        egui::EventFilter {
                            vertical_arrows: true,
                            ..Default::default()
                        },
                    )
                });
            }
            if active
                && button.has_focus()
                && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Space))
            {
                value = step(value, up, &self.range);
                stepped = true;
            }
            let visuals = ui.style().interact(&button);
            ui.painter().rect(
                rect,
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );
            let center = rect.center();
            let dy = if up { -2.0 } else { 2.0 };
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    center + Vec2::new(0.0, dy),
                    center + Vec2::new(-3.0, -dy),
                    center + Vec2::new(3.0, -dy),
                ],
                visuals.fg_stroke.color,
                egui::Stroke::NONE,
            ));
            if active
                && button.is_pointer_button_down_on()
                && ui.input(|i| {
                    (i.pointer.primary_down() || i.pointer.secondary_down())
                        && i.pointer.interact_pos().is_some_and(|p| rect.contains(p))
                })
            {
                held = Some(up);
            }
        }
        match (held, state.repeat) {
            (Some(up), Some((was_up, deadline))) if up == was_up => {
                if now >= deadline {
                    value = step(value, up, &self.range);
                    stepped = true;
                    state.repeat = Some((up, now + 0.1));
                }
            }
            (Some(up), _) => {
                ui.memory_mut(|m| {
                    m.request_focus(id.with(up));
                    m.set_focus_lock_filter(
                        id.with(up),
                        egui::EventFilter {
                            vertical_arrows: true,
                            ..Default::default()
                        },
                    );
                });
                value = step(value, up, &self.range);
                stepped = true;
                state.repeat = Some((up, now + 0.5));
            }
            (None, _) => state.repeat = None,
        }
        if let Some((_, deadline)) = state.repeat {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64(
                    (deadline - now).max(0.0),
                ));
        }
        if active {
            use egui::accesskit::{Action, ActionData};
            ui.input(|i| {
                for request in i.accesskit_action_requests(id, Action::SetValue) {
                    if let Some(ActionData::NumericValue(n)) = request.data
                        && n.is_finite()
                    {
                        value = (n as i32).clamp(*self.range.start(), *self.range.end());
                        stepped = true;
                    }
                }
                for (action, up) in [(Action::Increment, true), (Action::Decrement, false)] {
                    for _ in 0..i.num_accesskit_action_requests(id, action) {
                        value = step(value, up, &self.range);
                        stepped = true;
                    }
                }
            });
        }
        let committed = active && (entered || output.response.lost_focus()) && !escape;
        if stepped || (committed && raw_number(&state.text, self.hex) != i64::from(value)) {
            state.text = display(value, self.hex);
            ui.ctx().request_repaint();
        }
        state.value = value;
        state.frame = frame;
        ui.data_mut(|d| d.insert_temp(id, state));
        *self.value = T::from_f64(f64::from(value));
        let mut response = output.response.response;
        response.rect = area;
        response.interact_rect = area.intersect(ui.clip_rect());
        // Text edits are drafts: report changes only when the numeric value commits.
        response.flags.remove(egui::response::Flags::CHANGED);
        if value != old {
            response.mark_changed();
            ui.ctx().request_repaint();
        }
        response.widget_info(|| egui::WidgetInfo::drag_value(ui.is_enabled(), f64::from(value)));
        ui.ctx().accesskit_node_builder(id, |node| {
            use egui::accesskit::Action;
            node.set_min_numeric_value(f64::from(*self.range.start()));
            node.set_max_numeric_value(f64::from(*self.range.end()));
            node.set_numeric_value_step(1.0);
            for action in [Action::SetValue, Action::Increment, Action::Decrement] {
                node.add_action(action);
            }
        });
        response
    }
}
