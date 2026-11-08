//! Main_Window About identity and Modal_Dialog::refresh at pinned master/plusplus.
//! LGPL-3.0-only. Original artwork attribution: assets/artwork/NOTICE.md.
use super::*;

const TITLE: &str = "About Polished Map NG";
const SUBJECT: &str = concat!(
    "Polished Map NG ",
    env!("CARGO_PKG_VERSION"),
    " — development"
);
const MESSAGE: &str = concat!(
    "Rust port of Polished Map 4.7.1 by Rangi42.\n",
    "Original: Copyright © 2021 Rangi.\n",
    "Rust port: Copyright © 2026 Polished Map NG contributors.\n\n",
    "Source code is available at:\n",
    "https://github.com/vulcandth/polished-map-ng\n",
    "Upstream: https://github.com/Rangi42/polished-map\n",
    "Code: ",
    env!("CARGO_PKG_LICENSE"),
    "; see LICENSE.md and NOTICE.md.\n\n",
    "Some icons by Yusuke Kamiyamane (Fugue Icons).\n",
    "CC BY 3.0; adapted by Polished Map.\n",
    "https://p.yusukekamiyamane.com/\n",
    "https://creativecommons.org/licenses/by/3.0/"
);

#[derive(Clone, Copy)]
struct Layout {
    size: Vec2,
    icon: Rect,
    heading: Rect,
    body: Rect,
    ok: Rect,
}

impl Layout {
    /// Source APP_ICON case: measure at max width 700 minus 80; preserve the
    /// extra OS_FONT_SIZE allowance, minimum width 280 and 50x50 icon slot.
    fn measured(heading_width: f32, body: Vec2, font_size: f32) -> Self {
        let width = (heading_width.ceil().max(body.x.ceil()) + 80.0 + font_size).max(280.0);
        let button_y = (45.0 + body.y.ceil() + 10.0).max(70.0);
        Self {
            size: Vec2::new(width, button_y + 22.0 + 10.0),
            icon: Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(50.0)),
            heading: Rect::from_min_size(Pos2::new(70.0, 10.0), Vec2::new(width - 80.0, 25.0)),
            body: Rect::from_min_size(
                Pos2::new(70.0, 45.0),
                Vec2::new(width - 80.0, body.y.ceil()),
            ),
            ok: Rect::from_min_size(Pos2::new(width - 90.0, button_y), Vec2::new(80.0, 22.0)),
        }
    }
}

pub(super) struct AboutDialog {
    font_size: f32,
    #[cfg(test)]
    bounds: Option<Rect>,
}

impl Default for AboutDialog {
    fn default() -> Self {
        Self {
            // themes.h uses 12 on Windows and 13 elsewhere.
            font_size: if cfg!(windows) { 12.0 } else { 13.0 },
            #[cfg(test)]
            bounds: None,
        }
    }
}

impl AboutDialog {
    pub(super) fn show(&mut self, ctx: &egui::Context, application: Application) -> bool {
        let plus = application == Application::Parity(documents::Profile::PlusPlus);
        let unified = application.is_unified();
        let title = if unified {
            "About Polished Map NG Unified"
        } else if plus {
            "About Polished Map NG++"
        } else {
            TITLE
        };
        let subject = if unified {
            SUBJECT.replacen("Polished Map NG", "Polished Map NG Unified", 1)
        } else if plus {
            SUBJECT.replacen("Polished Map NG", "Polished Map NG++", 1)
        } else {
            SUBJECT.into()
        };
        let message = if unified {
            MESSAGE.replacen(
                "Polished Map 4.7.1",
                "Polished Map 4.7.1 and Polished Map++ 2.7.1",
                1,
            )
        } else if plus {
            MESSAGE.replacen("Polished Map 4.7.1", "Polished Map++ 2.7.1", 1)
        } else {
            MESSAGE.into()
        };
        let mut close = false;
        dialog_host::client(ctx, egui::Id::unique("about"), |ui| {
            dialog_host::heading(ui, title);
            let text = ui.visuals().text_color();
            let font_size = self.font_size;
            let heading_font = egui::FontId::proportional(font_size + 4.0);
            let body_font = egui::FontId::proportional(font_size);
            let heading = ui
                .painter()
                .layout(subject.clone(), heading_font.clone(), text, 620.0);
            let body = ui
                .painter()
                .layout(message.clone(), body_font.clone(), text, 620.0);
            let layout = Layout::measured(heading.size().x, body.size(), font_size);
            let (area, _) = ui.allocate_exact_size(layout.size, Sense::hover());
            #[cfg(test)]
            {
                self.bounds = Some(area);
            }
            let translated = |r: Rect| r.translate(area.min.to_vec2());
            // FL_ALIGN_CENTER places the original 48px picture inside its 50px slot.
            ui.allocate_rect(translated(layout.icon), Sense::hover());
            artwork::Image::App.paint(ui, translated(layout.icon));
            ui.put(
                translated(layout.heading),
                egui::Label::new(egui::RichText::new(&subject).font(heading_font))
                    .selectable(true)
                    .halign(egui::Align::Center),
            );
            // Retain measured line positions, with selectable credits and real
            // hyperlinks instead of one inert multiline label.
            for row in &body.rows {
                let line: String = row.glyphs.iter().map(|glyph| glyph.chr).collect();
                let (prefix, url) = line
                    .find("https://")
                    .map_or((line.as_str(), None), |at| (&line[..at], Some(&line[at..])));
                let origin = translated(layout.body).min + row.pos.to_vec2();
                let prefix_width = ui
                    .painter()
                    .layout_no_wrap(prefix.into(), body_font.clone(), text)
                    .size()
                    .x;
                if !prefix.is_empty() {
                    ui.put(
                        Rect::from_min_size(origin, Vec2::new(prefix_width, row.size.y)),
                        egui::Label::new(egui::RichText::new(prefix).font(body_font.clone()))
                            .selectable(true),
                    );
                }
                if let Some(url) = url {
                    ui.put(
                        Rect::from_min_size(
                            origin + Vec2::new(prefix_width, 0.0),
                            Vec2::new(row.size.x - prefix_width, row.size.y),
                        ),
                        egui::Hyperlink::from_label_and_url(
                            egui::RichText::new(url).font(body_font.clone()),
                            url,
                        ),
                    );
                }
            }
            close = ui
                .put(
                    translated(layout.ok),
                    egui::Button::new(egui::RichText::new("OK").size(font_size)),
                )
                .on_hover_text("OK (Enter)")
                .clicked();
            close |= ui.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                    || i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
            });
        });
        close
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn original_geometry_vectors_keep_font_allowance_margins_and_button_anchor() {
        for (heading, body, size, button) in [
            (
                0.0,
                Vec2::ZERO,
                Vec2::new(280.0, 102.0),
                Pos2::new(190.0, 70.0),
            ),
            (
                180.0,
                Vec2::new(260.0, 104.0),
                Vec2::new(353.0, 191.0),
                Pos2::new(263.0, 159.0),
            ),
            (
                620.0,
                Vec2::new(600.0, 30.0),
                Vec2::new(713.0, 117.0),
                Pos2::new(623.0, 85.0),
            ),
        ] {
            let layout = Layout::measured(heading, body, 13.0);
            assert_eq!(layout.size, size);
            assert_eq!(
                layout.icon,
                Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(50.0))
            );
            assert_eq!(layout.heading.min, Pos2::new(70.0, 10.0));
            assert_eq!(layout.heading.height(), 25.0);
            assert_eq!(layout.body.min, Pos2::new(70.0, 45.0));
            assert_eq!(layout.body.width(), size.x - 80.0);
            assert_eq!(layout.ok.min, button);
            assert_eq!(layout.ok.size(), Vec2::new(80.0, 22.0));
            let windows = Layout::measured(heading, body, 12.0);
            assert_eq!(
                windows.size.x,
                if size.x == 280.0 { 280.0 } else { size.x - 1.0 }
            );
            assert_eq!(windows.size.y, size.y);
        }
    }

    fn open(h: &mut Harness<'_, Editor>) {
        h.get_by_label("Help").click();
        h.run();
        h.get_by_role_and_label(egui::Role::MenuItem, "About")
            .click();
        h.run();
    }

    #[test]
    fn plusplus_about_retains_attribution_and_closes_through_shared_controls() {
        let mut h = Harness::builder()
            .with_size(Vec2::new(900.0, 700.0))
            .build_eframe(|cc| Editor::new_plusplus(cc));
        open(&mut h);
        h.get_by_label("About Polished Map NG++");
        h.get_by_label(&SUBJECT.replacen("Polished Map NG", "Polished Map NG++", 1));
        h.get_by_label("Rust port of Polished Map++ 2.7.1 by Rangi42.");
        let area = h.state().about.as_ref().unwrap().bounds.unwrap();
        let ok = h.get_by_label("OK").rect();
        assert_eq!(ok.size(), Vec2::new(80.0, 22.0));
        assert_eq!(area.bottom() - ok.bottom(), 10.0);
        h.key_press(egui::Key::Escape);
        h.run();
        assert!(h.state().about.is_none());
        assert!(h.state().profile() == documents::Profile::PlusPlus);
    }

    #[test]
    fn about_contains_source_identity_and_asset_attribution_with_measured_layout() {
        #[cfg(feature = "visual-tests")]
        let mut snapshots = egui_kittest::SnapshotResults::new();
        for (font_size, dark, _name) in [
            (12.0, false, "about-light"),
            (13.0, false, "about-unix"),
            (12.0, true, "about-dark"),
        ] {
            let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
            // Select platform metrics before the first layout pass; a native
            // platform's initial size must not leak into another platform's image.
            h.state_mut().about = Some(AboutDialog {
                font_size,
                ..Default::default()
            });
            h.state_mut().dark = dark;
            h.ctx.set_theme(if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            });
            h.run();
            h.get_by_label(SUBJECT);
            h.get_by_label("Rust port of Polished Map 4.7.1 by Rangi42.");
            assert!(MESSAGE.contains("Copyright © 2021 Rangi."));
            assert!(MESSAGE.contains("LGPL-3.0-only") && MESSAGE.contains("CC BY 3.0"));
            assert!(
                MESSAGE.contains("Yusuke Kamiyamane")
                    && MESSAGE.contains("adapted by Polished Map")
            );
            let area = h.state().about.as_ref().unwrap().bounds.unwrap();
            assert!((area.center().x - 400.0).abs() <= 0.5);
            let ok = h.get_by_label("OK").rect();
            assert_eq!(ok.size(), Vec2::new(80.0, 22.0));
            assert_eq!(area.bottom() - ok.bottom(), 10.0);
            assert_eq!(area.right() - ok.right(), 10.0);
            let body = h
                .get_by_label("Rust port of Polished Map 4.7.1 by Rangi42.")
                .rect();
            assert_eq!(body.left() - area.left(), 70.0);
            assert_eq!(body.top() - area.top(), 45.0);
            h.remove_cursor();
            h.run();
            #[cfg(feature = "visual-tests")]
            snapshots.add(h.try_snapshot(_name));
            h.get_by_label("OK").click();
            h.run();
            assert!(h.state().about.is_none());
        }
        #[cfg(feature = "visual-tests")]
        snapshots.unwrap();
    }

    #[test]
    fn about_links_emit_browser_requests_without_dismissing_the_dialog() {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        open(&mut h);
        for url in [
            "https://github.com/vulcandth/polished-map-ng",
            "https://github.com/Rangi42/polished-map",
            "https://p.yusukekamiyamane.com/",
            "https://creativecommons.org/licenses/by/3.0/",
        ] {
            h.get_by_role_and_label(egui::Role::Link, url).click();
            let mut requests = Vec::new();
            for _ in 0..4 {
                h.step();
                for command in &h.output().platform_output.commands {
                    if let egui::OutputCommand::OpenUrl(request) = command {
                        requests.push(request.url.clone());
                    }
                }
            }
            assert_eq!(requests, [url]);
            assert!(h.state().about.is_some());
        }
    }

    #[test]
    fn shortcut_and_dismissal_preserve_dirty_documents_and_block_parent_commands() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        for key in [egui::Key::Enter, egui::Key::Escape] {
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Slash);
            h.run();
            assert!(h.state().about.is_some());
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
            h.run();
            assert!(h.state().loaded.is_some() && !h.state().confirm_discard);
            h.key_press(egui::Key::F1);
            h.run();
            assert!(!h.state().help_window.open);
            h.input_mut()
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .events
                .push(egui::ViewportEvent::Close);
            h.step();
            assert!(h.output().viewport_output.values().any(|v| {
                v.commands
                    .iter()
                    .any(|c| matches!(c, egui::ViewportCommand::CancelClose))
            }));
            assert!(!h.state().confirm_discard);
            h.key_press(key);
            h.run();
            assert!(h.state().about.is_none());
        }
        let map = h.state().loaded.as_ref().unwrap().document.map();
        assert_eq!(map.blocks(), [1, 1]);
        assert!(map.is_modified() && map.can_undo());
        assert_eq!(
            std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
            [0, 1]
        );
        h.state_mut().close_map();
        open(&mut h);
        assert!(h.state().about.is_some());
    }
}
