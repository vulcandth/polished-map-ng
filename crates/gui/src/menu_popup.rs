//! Popup layout from master widgets.h/Main_Window::initialize and FLTK 1.4.5
//! Fl_Menu.cxx. Based in part on the work of the FLTK project, Copyright
//! 1998-2024 Bill Spitzak and others. See NOTICE.md and LICENSES/FLTK.txt.
//! Rust adaptation: LGPL-3.0-only. Native fonts and theme glyphs remain separate.
use super::*;
use std::sync::Arc;

mod specs;
#[cfg(test)]
mod tests;

const TAG: &str = "polished-map-popup";
const LEADING: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Mark {
    Blank,
    Check,
    Radio,
    Submenu,
}

#[derive(Clone, Copy)]
pub(super) struct Spec<'a> {
    pub name: &'a str,
    mark: Mark,
    shortcut: &'a str,
    divider: bool,
    verbatim: bool,
}

impl<'a> Spec<'a> {
    const fn new(name: &'a str, mark: Mark, shortcut: &'a str, divider: bool) -> Self {
        Self {
            name,
            mark,
            shortcut,
            divider,
            verbatim: false,
        }
    }

    pub(super) fn recent(name: &'a str, shortcut: &'a str, divider: bool) -> Self {
        Self {
            verbatim: true,
            ..Self::new(name, Mark::Blank, shortcut, divider)
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Metrics {
    width: f32,
    row: f32,
    key_width: f32,
    count: usize,
}

impl Metrics {
    /// Input widths already include the source label/check/image measurement.
    /// Popup border inset is one point (OS_PANEL_THIN_UP_BOX, both themes).
    fn measured(items: &[(Vec2, bool, Vec2)], title: f32) -> Self {
        let mut width: f32 = 0.0;
        let mut row: f32 = 1.0;
        let mut modifiers: f32 = 0.0;
        let mut keys: f32 = 0.0;
        for &(label, submenu, shortcut) in items {
            width = width.max(label.x + if submenu { 14.0 } else { 0.0 });
            row = row.max(label.y + LEADING);
            modifiers = modifiers.max(shortcut.x);
            keys = keys.max(shortcut.y);
        }
        Self {
            width: (width + modifiers + keys + 9.0).max(title + 12.0),
            row,
            key_width: keys,
            count: items.len(),
        }
    }

    fn height(self) -> f32 {
        if self.count == 0 {
            5.0
        } else {
            self.count as f32 * self.row + 1.0
        }
    }

    fn indicator(self) -> f32 {
        let h = self.row as i32 - 4;
        (h - 2 * ((h - 14 + 1) / 2)) as f32
    }
}

struct Item {
    name: String,
    mark: Mark,
    divider: bool,
    label: Arc<egui::Galley>,
    modifiers: Arc<egui::Galley>,
    key: Arc<egui::Galley>,
}

struct Prepared {
    items: Vec<Item>,
    metrics: Metrics,
}

fn font_size(_ctx: &egui::Context) -> f32 {
    #[cfg(test)]
    if let Some(size) = _ctx.data(|d| d.get_temp::<f32>(egui::Id::unique("menu-test-font-size"))) {
        return size;
    }
    if cfg!(windows) { 12.0 } else { 13.0 }
}

#[cfg(test)]
pub(super) fn test_font_size(ctx: &egui::Context, size: f32) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("menu-test-font-size"), size));
}

fn shortcut_parts(shortcut: &str, mac: bool) -> (String, String) {
    let Some((mods, key)) = shortcut.rsplit_once('+') else {
        return (String::new(), shortcut.into());
    };
    let mut prefix = String::new();
    if mac {
        if mods.contains('S') {
            prefix.push('⇧');
        }
        if mods.contains('A') {
            prefix.push('⌥');
        }
        if mods.contains('C') {
            prefix.push('⌘');
        }
    } else {
        if mods.contains('C') {
            prefix.push_str("Ctrl+");
        }
        if mods.contains('A') {
            prefix.push_str("Alt+");
        }
        if mods.contains('S') {
            prefix.push_str("Shift+");
        }
    }
    (prefix, key.into())
}

impl Prepared {
    fn new(ui: &egui::Ui, title: &str, specs: &[Spec<'_>]) -> Self {
        let font = egui::FontId::proportional(font_size(ui.ctx()));
        let text = |s: String| ui.painter().layout_no_wrap(s, font.clone(), Color32::WHITE);
        let mac = ui.ctx().os() == egui::os::OperatingSystem::Mac;
        let mut measured = Vec::new();
        let items = specs
            .iter()
            .map(|spec| {
                // OS_MENU_ITEM adds one leading and nine trailing spaces. Public
                // accessible labels retain the app's existing Unicode ellipsis.
                let display = if spec.verbatim {
                    spec.name.into()
                } else {
                    spec.name.replace('…', "...")
                };
                let label = text(format!(" {display}         "));
                let (mods, key) = shortcut_parts(spec.shortcut, mac);
                let modifiers = if mac {
                    ui.painter().layout_no_wrap(
                        mods,
                        egui::FontId::new(font.size, egui::FontFamily::Name("Menu Symbols".into())),
                        Color32::WHITE,
                    )
                } else {
                    text(mods)
                };
                let key = text(key);
                let mut size = label.size().ceil();
                size.x += match spec.mark {
                    Mark::Blank | Mark::Submenu => {
                        size.y = size.y.max(16.0);
                        16.0
                    }
                    Mark::Check | Mark::Radio => 18.0,
                };
                measured.push((
                    size,
                    spec.mark == Mark::Submenu,
                    Vec2::new(
                        modifiers.size().x.floor(),
                        if spec.shortcut.is_empty() {
                            0.0
                        } else {
                            key.size().x.floor() + 4.0
                        },
                    ),
                ));
                Item {
                    name: spec.name.into(),
                    mark: spec.mark,
                    divider: spec.divider,
                    label,
                    modifiers,
                    key,
                }
            })
            .collect();
        Self {
            items,
            metrics: Metrics::measured(&measured, text(title.into()).size().x.ceil()),
        }
    }
}

fn scope<R>(
    ui: &mut egui::Ui,
    title: &str,
    specs: &[Spec<'_>],
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let prepared = Arc::new(Prepared::new(ui, title, specs));
    let size = Vec2::new(
        prepared.metrics.width - 2.0,
        prepared.metrics.height() - 2.0,
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .ui_stack_info(egui::UiStackInfo::default().with_tag_value(TAG, prepared)),
        |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.set_width(size.x);
            ui.set_min_height(size.y);
            contents(ui)
        },
    )
    .inner
}

fn prepared(ui: &egui::Ui) -> Arc<Prepared> {
    ui.stack()
        .iter()
        .find_map(|s| s.info.tags.get_downcast::<Arc<Prepared>>(TAG).cloned())
        .expect("menu rows must be inside their measured popup")
}

pub(super) fn config() -> egui::containers::menu::MenuConfig {
    egui::containers::menu::MenuConfig::new()
        .submenu_gap(0.0)
        .style(|style: &mut egui::Style| {
            egui::containers::menu::menu_style(style);
            style.spacing.menu_margin = egui::Margin::ZERO;
            style.visuals.menu_corner_radius = egui::CornerRadius::ZERO;
            style.visuals.window_stroke.width = 1.0;
            style.visuals.window_fill = if style.visuals.dark_mode {
                Color32::from_gray(83)
            } else {
                Color32::from_gray(206)
            };
        })
}

pub(super) fn menu<R>(ui: &mut egui::Ui, name: &str, contents: impl FnOnce(&mut egui::Ui) -> R) {
    ui.menu_button(name, |ui| scope(ui, name, specs::group(name), contents));
}

pub(super) fn submenu<R>(ui: &mut egui::Ui, name: &str, contents: impl FnOnce(&mut egui::Ui) -> R) {
    submenu_with(ui, name, specs::group(name), contents);
}

pub(super) fn submenu_with<R>(
    ui: &mut egui::Ui,
    name: &str,
    specs: &[Spec<'_>],
    contents: impl FnOnce(&mut egui::Ui) -> R,
) {
    let response = row(ui, true, name, None);
    egui::containers::menu::SubMenu::new().show(ui, &response, |ui| scope(ui, "", specs, contents));
}

pub(super) fn action(ui: &mut egui::Ui, enabled: bool, name: &str) -> bool {
    let response = row(ui, enabled, name, None);
    if response.clicked() {
        ui.close();
        true
    } else {
        false
    }
}

pub(super) fn checkbox(
    ui: &mut egui::Ui,
    enabled: bool,
    value: &mut bool,
    name: &str,
) -> egui::Response {
    let mut response = row(ui, enabled, name, Some(*value));
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
        ui.close();
    }
    response
}

pub(super) fn radio(ui: &mut egui::Ui, value: bool, name: &str) -> egui::Response {
    row(ui, true, name, Some(value))
}

pub(super) fn row(
    ui: &mut egui::Ui,
    enabled: bool,
    name: &str,
    selected: Option<bool>,
) -> egui::Response {
    let prepared = prepared(ui);
    let index = prepared
        .items
        .iter()
        .position(|i| i.name == name)
        .expect("unmeasured menu item");
    row_at(ui, enabled, index, selected)
}

pub(super) fn row_at(
    ui: &mut egui::Ui,
    enabled: bool,
    index: usize,
    selected: Option<bool>,
) -> egui::Response {
    let prepared = prepared(ui);
    let item = &prepared.items[index];
    let name = item.name.as_str();
    let m = prepared.metrics;
    ui.add_enabled_ui(enabled, |ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        let height = m.row - if index + 1 == m.count { 1.0 } else { 0.0 };
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(m.width - 2.0, height), Sense::click());
        let role = match item.mark {
            Mark::Check => egui::Role::CheckBox,
            Mark::Radio => egui::Role::RadioButton,
            _ => egui::Role::MenuItem,
        };
        response.widget_info(|| match selected {
            Some(value) => egui::WidgetInfo::selected(role, ui.is_enabled(), value, name),
            None => egui::WidgetInfo::labeled(role, ui.is_enabled(), name),
        });
        if !item.key.text().is_empty() {
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_keyboard_shortcut(format!("{}{}", item.modifiers.text(), item.key.text()));
            });
        }
        let dark = ui.visuals().dark_mode;
        let hovered = ui.is_enabled() && (response.hovered() || response.has_focus());
        let fg = if hovered {
            if dark { Color32::BLACK } else { Color32::WHITE }
        } else if dark {
            Color32::WHITE
        } else {
            Color32::from_gray(60)
        };
        if hovered {
            ui.painter().rect_filled(
                Rect::from_min_size(
                    rect.min + Vec2::new(1.0, 0.0),
                    Vec2::new(m.width - 5.0, m.row - 2.0),
                ),
                0.0,
                if dark {
                    Color32::from_gray(214)
                } else {
                    Color32::from_rgb(61, 123, 187)
                },
            );
        }
        let h = m.row - LEADING;
        let text_x = match item.mark {
            Mark::Blank | Mark::Submenu => 19.0,
            _ => m.indicator() + 6.0,
        };
        let text_y = rect.top() + 1.0 + (h - item.label.size().y) / 2.0;
        ui.painter().galley_with_override_text_color(
            Pos2::new(rect.left() + text_x, text_y),
            item.label.clone(),
            fg,
        );
        let split = rect.right() - 1.0 - m.key_width;
        if !item.key.text().is_empty() {
            ui.painter().galley_with_override_text_color(
                Pos2::new(
                    split - item.modifiers.size().x,
                    rect.top() + 1.0 + (h - item.modifiers.size().y) / 2.0,
                ),
                item.modifiers.clone(),
                fg,
            );
            ui.painter().galley_with_override_text_color(
                Pos2::new(split, text_y),
                item.key.clone(),
                fg,
            );
        }
        if let Some(selected) = selected {
            let w = m.indicator();
            let r = Rect::from_min_size(
                rect.min + Vec2::new(2.0, 1.0 + (h - w) / 2.0),
                Vec2::splat(w),
            );
            let fill = if dark {
                Color32::from_gray(58)
            } else {
                Color32::from_gray(252)
            };
            let stroke = egui::Stroke::new(1.0, fg);
            if item.mark == Mark::Radio {
                ui.painter().circle(r.center(), w / 2.0, fill, stroke);
                if selected {
                    ui.painter().circle_filled(r.center(), w / 4.0, fg);
                }
            } else {
                ui.painter()
                    .rect(r, 0.0, fill, stroke, egui::StrokeKind::Inside);
                if selected {
                    ui.painter().add(egui::Shape::line(
                        vec![
                            r.min + Vec2::new(3.0, w / 2.0),
                            r.min + Vec2::new(w / 2.0, w - 3.0),
                            r.min + Vec2::new(w - 2.0, 3.0),
                        ],
                        egui::Stroke::new(1.5, fg),
                    ));
                }
            }
        }
        if item.mark == Mark::Submenu {
            let sz = (((h as i32 - 2) & -2) + 1).min(13) as f32;
            let r = Rect::from_min_size(
                Pos2::new(rect.right() - sz - 3.0, rect.top() + 2.0 + (h - sz) / 2.0),
                Vec2::splat(sz),
            );
            ui.painter()
                .add(egui::containers::menu::SubMenuButton::arrow_shape(r, fg));
        }
        if item.divider {
            let y = rect.top() + h + 2.0;
            ui.painter().hline(
                rect.x_range(),
                y,
                egui::Stroke::new(1.0, Color32::from_gray(if dark { 40 } else { 160 })),
            );
            ui.painter().hline(
                rect.x_range(),
                y + 1.0,
                egui::Stroke::new(1.0, Color32::from_gray(if dark { 106 } else { 255 })),
            );
        }
        if response.clicked() && response.has_focus() {
            // A command can open a modal later in this same frame. Do not let
            // its activating Enter also accept/dismiss that newly opened dialog.
            ui.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                i.consume_key(egui::Modifiers::NONE, egui::Key::Space);
            });
        }
        response
    })
    .inner
}
