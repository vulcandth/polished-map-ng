//! master Modal_Dialog::{initialize,refresh,show} with an empty subject and one
//! or two buttons. LGPL-3.0-only. Original artwork: assets/artwork/NOTICE.md.
use super::*;

#[derive(Clone, Copy, Debug)]
struct Layout {
    size: Vec2,
    icon: Rect,
    body: Rect,
    ok: Rect,
    cancel: Option<Rect>,
}

impl Layout {
    fn measured(body: Vec2, font_size: f32) -> Self {
        // Main_Window gives all three message dialogs width_range(280, 700).
        // Modal_Dialog measures at 700 - 80, then adds OS_FONT_SIZE to width.
        let width = (body.x.ceil() + 80.0 + font_size).max(280.0);
        let button_y = (body.y.ceil() + 20.0).max(70.0);
        Self {
            size: Vec2::new(width, button_y + 32.0),
            icon: Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(50.0)),
            body: Rect::from_min_size(
                Pos2::new(70.0, 10.0),
                Vec2::new(width - 80.0, body.y.ceil()),
            ),
            ok: Rect::from_min_size(Pos2::new(width - 90.0, button_y), Vec2::new(80.0, 22.0)),
            cancel: None,
        }
    }

    fn confirmation(mut self, windows: bool) -> Self {
        let left = self.ok.translate(Vec2::new(-94.0, 0.0));
        self.cancel = Some(if windows { self.ok } else { left });
        if windows {
            self.ok = left;
        }
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Kind {
    Error,
    Warning,
    Success,
}
impl Kind {
    fn title(self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Success => "Success",
        }
    }
    fn image(self) -> artwork::Image {
        match self {
            Self::Error => artwork::Image::Error,
            Self::Warning => artwork::Image::Warning,
            Self::Success => artwork::Image::Success,
        }
    }
}

pub(super) fn show(ctx: &egui::Context, id: &'static str, kind: Kind, message: &str) -> bool {
    show_with_font(ctx, id, kind, message, font_size(ctx)).0
}

/// Source OK/Cancel artwork, measurement and platform order. Descriptive
/// accessibility names identify what OK does without widening the source buttons.
pub(super) fn confirm(
    ctx: &egui::Context,
    id: &'static str,
    message: &str,
    windows: bool,
    actions: [&str; 2],
) -> (bool, bool) {
    let (accept, cancel, _) = render(
        ctx,
        id,
        Kind::Warning,
        message,
        if windows { 12.0 } else { 13.0 },
        Some((windows, actions)),
    );
    (accept && !cancel, cancel)
}

fn font_size(_ctx: &egui::Context) -> f32 {
    #[cfg(test)]
    if let Some(size) = _ctx.data(|d| d.get_temp::<f32>(egui::Id::unique("test-message-font-size")))
    {
        return size;
    }
    if cfg!(windows) { 12.0 } else { 13.0 }
}

#[cfg(test)]
pub(super) fn test_font_size(ctx: &egui::Context, size: f32) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("test-message-font-size"), size));
}

fn show_with_font(
    ctx: &egui::Context,
    id: &'static str,
    kind: Kind,
    message: &str,
    font_size: f32,
) -> (bool, Rect) {
    let (accept, cancel, bounds) = render(ctx, id, kind, message, font_size, None);
    (accept || cancel, bounds)
}

fn render(
    ctx: &egui::Context,
    id: &'static str,
    kind: Kind,
    message: &str,
    font_size: f32,
    confirmation: Option<(bool, [&str; 2])>,
) -> (bool, bool, Rect) {
    let mut done = false;
    let mut canceled = false;
    let mut bounds = Rect::NOTHING;
    egui::Modal::new(egui::Id::unique(id)).show(ctx, |ui| {
        // Caption stands in for native chrome; the allocated client rectangle
        // below follows the source formula independently of that caption.
        ui.heading(kind.title());
        let font = egui::FontId::proportional(font_size);
        let body = ui.painter().layout(
            message.into(),
            font.clone(),
            ui.visuals().text_color(),
            620.0,
        );
        let mut layout = Layout::measured(body.size(), font_size);
        if let Some((windows, _)) = confirmation {
            layout = layout.confirmation(windows);
        }
        bounds = ui.allocate_exact_size(layout.size, Sense::hover()).0;
        let rect = |r: Rect| r.translate(bounds.min.to_vec2());
        ui.allocate_rect(rect(layout.icon), Sense::hover());
        kind.image().paint(ui, rect(layout.icon));
        // Ui::put centers widgets inside their allocation, including short
        // labels. Source FL_ALIGN_TOP_LEFT needs the actual text origin here.
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(rect(layout.body))
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                ui.add(
                    egui::Label::new(egui::RichText::new(message).font(font))
                        .wrap()
                        .halign(egui::Align::Min),
                );
            },
        );
        let ok = ui
            .put(
                rect(layout.ok),
                egui::Button::new(egui::RichText::new("OK").size(font_size)),
            )
            .on_hover_text("OK (Enter)");
        done = ok.clicked();
        if let Some((_, actions)) = confirmation {
            ok.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, actions[0]));
            let cancel = ui
                .put(
                    rect(layout.cancel.expect("confirmation has Cancel")),
                    egui::Button::new(egui::RichText::new("Cancel").size(font_size)),
                )
                .on_hover_text(if actions[1] == "Skip" {
                    "Keep these colors and continue opening the map (Esc)"
                } else {
                    "Cancel (Esc)"
                });
            cancel.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, actions[1]));
            canceled = cancel.clicked();
        }
        // Default_Button binds Enter. FLTK routes unhandled Escape to the
        // modal window's cancel callback even when it has no Cancel button.
        done |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        canceled |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    });
    (done, canceled, bounds)
}

#[cfg(test)]
mod tests;
