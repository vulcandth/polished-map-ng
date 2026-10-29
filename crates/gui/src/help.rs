//! Pinned master Help_Window and HTML_View, LGPL-3.0-only.
//! Full attributed help text is generated from the bundled original by help_assets.py.
use super::*;
use serde::Deserialize;
use std::sync::OnceLock;

const TITLE: &str = "Polished Map NG Help";
const REFERENCE_NOTE: &str = "Polished Map 4.7.1 reference — NG parity is in progress.";

pub(super) fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Ubuntu lacks the source's right-arrow glyph. The already-bundled Hack
    // face supplies it; keep the existing proportional faces first.
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .push("Hack".into());
    ctx.set_fonts(fonts);
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Heading,
    Paragraph,
    Bullet,
    Rule,
}
#[derive(Deserialize)]
struct Span {
    text: String,
    bold: bool,
    italic: bool,
    mono: bool,
    small: bool,
    link: Option<String>,
}
#[derive(Deserialize)]
struct Block {
    kind: Kind,
    spans: Vec<Span>,
}

fn document(windows: bool) -> &'static [Block] {
    static WINDOWS: OnceLock<Vec<Block>> = OnceLock::new();
    static UNIX: OnceLock<Vec<Block>> = OnceLock::new();
    let (document, source) = if windows {
        (&WINDOWS, include_str!("../assets/help/windows.json"))
    } else {
        (&UNIX, include_str!("../assets/help/unix.json"))
    };
    document.get_or_init(|| {
        // Compiled, hash-verified assets, never user-provided JSON.
        serde_json::from_str(source).expect("verified bundled help document")
    })
}

fn viewport_id() -> egui::ViewportId {
    egui::ViewportId::from_hash_of("help-window")
}

pub(super) struct HelpWindow {
    pub(super) open: bool,
    reposition: bool,
    reset_scroll: bool,
    native_size: Vec2,
    windows_paths: bool,
    #[cfg(test)]
    pub(super) bounds: Rect,
    #[cfg(test)]
    scroll: f32,
}

impl Default for HelpWindow {
    fn default() -> Self {
        Self {
            open: false,
            reposition: false,
            reset_scroll: false,
            native_size: Vec2::new(700.0, 500.0),
            windows_paths: cfg!(windows),
            #[cfg(test)]
            bounds: Rect::NOTHING,
            #[cfg(test)]
            scroll: 0.0,
        }
    }
}

impl HelpWindow {
    pub(super) fn open(&mut self) {
        self.open = true;
        self.reposition = true;
        self.reset_scroll = true;
    }

    fn builder(&self, parent: Pos2) -> egui::ViewportBuilder {
        let builder = egui::ViewportBuilder::default()
            .with_title(TITLE)
            .with_inner_size(self.native_size)
            .with_resizable(true);
        if self.reposition {
            builder
                .with_position(parent + Vec2::splat(48.0))
                .with_active(true)
        } else {
            builder
        }
    }

    pub(super) fn show(&mut self, ctx: &egui::Context, enabled: bool) {
        if !self.open {
            return;
        }
        let parent = ctx.input(|i| i.viewport().outer_rect.map_or(Pos2::ZERO, |r| r.min));
        if ctx.embed_viewports() {
            // Egui's automatic embedded viewport currently omits a close button
            // and ignores position. Use its ordinary nonmodal Window fallback.
            let mut open = self.open;
            let mut window = egui::Window::new(TITLE)
                .id(egui::Id::unique("embedded-help-window"))
                .open(&mut open)
                .collapsible(false)
                .resizable(true)
                .default_size([700.0, 500.0]);
            if self.reposition {
                window = window.current_pos(Pos2::new(48.0, 48.0));
            }
            window.show(ctx, |ui| {
                if !enabled {
                    ui.disable();
                }
                self.content(ui);
            });
            self.open &= open;
        } else {
            ctx.show_viewport_immediate(viewport_id(), self.builder(parent), |ui, _class| {
                if !enabled {
                    ui.disable();
                }
                self.content(ui);
                // Upstream hides its existing Help window; retain its resized
                // client size when eframe recreates our closed child viewport.
                self.native_size = ui.max_rect().size();
                if ui.input(|i| i.viewport().close_requested()) {
                    self.open = false;
                }
            });
        }
        self.reposition = false;
    }

    fn content(&mut self, ui: &mut egui::Ui) {
        let area = ui.max_rect();
        #[cfg(test)]
        {
            self.bounds = area;
        }
        let body = Rect::from_min_size(
            area.min + Vec2::splat(10.0),
            (area.size() - Vec2::new(20.0, 52.0)).max(Vec2::ZERO),
        );
        let ok = Rect::from_min_size(
            Pos2::new(area.right() - 90.0, area.bottom() - 32.0),
            Vec2::new(80.0, 22.0),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(body), |ui| {
            ui.set_clip_rect(body.intersect(ui.clip_rect()));
            egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .inner_margin(4.0)
                .show(ui, |ui| {
                    let mut scroll = egui::ScrollArea::vertical()
                        .id_salt("help-content")
                        .max_height((body.height() - 8.0).max(0.0))
                        .auto_shrink([false, false]);
                    if self.reset_scroll {
                        scroll = scroll.vertical_scroll_offset(0.0);
                    }
                    let result = scroll.show(ui, |ui| {
                        ui.label(egui::RichText::new(REFERENCE_NOTE).size(13.0));
                        ui.add_space(8.0);
                        for block in document(self.windows_paths) {
                            render_block(ui, block);
                        }
                    });
                    #[cfg(test)]
                    {
                        self.scroll = result.state.clamped_offset().y;
                    }
                    #[cfg(not(test))]
                    let _ = result;
                });
        });
        self.reset_scroll = false;
        if ui
            .put(ok, egui::Button::new("OK"))
            .on_hover_text("OK (Enter)")
            .clicked()
            || (ui.is_enabled()
                && ui.input_mut(|i| {
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                        || i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                }))
        {
            self.open = false;
        }
        // Keep the resizable body independent of document length.
        ui.allocate_rect(area, Sense::hover());
    }
}

fn render_block(ui: &mut egui::Ui, block: &Block) {
    if matches!(block.kind, Kind::Rule) {
        ui.separator();
        return;
    }
    let bullet = matches!(block.kind, Kind::Bullet);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if bullet {
            ui.label(egui::RichText::new("• ").size(16.0));
            ui.add_space(4.0);
        }
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            for span in &block.spans {
                let size = if matches!(block.kind, Kind::Heading) {
                    32.0
                } else if span.small {
                    14.0
                } else {
                    16.0
                };
                let mut text = egui::RichText::new(&span.text).size(size);
                if span.bold || matches!(block.kind, Kind::Heading) {
                    text = text.strong();
                }
                if span.italic {
                    text = text.italics();
                }
                if span.mono {
                    text = text.monospace();
                }
                if let Some(link) = &span.link {
                    ui.hyperlink_to(text, link);
                } else {
                    ui.add(egui::Label::new(text).wrap());
                }
            }
        });
    });
    if !bullet {
        ui.add_space(8.0);
    }
}

#[cfg(test)]
mod tests;
