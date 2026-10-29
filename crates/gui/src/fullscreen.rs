//! master Main_Window::{initialize,full_screen_cb,exit_cb} and main.cpp startup.
//! LGPL-3.0-only. eframe owns normal geometry and native fullscreen persistence.
use eframe::egui;

fn active(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.viewport().fullscreen.unwrap_or(false))
}

pub(super) fn toggle(ctx: &egui::Context) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!active(ctx)));
}

pub(super) fn menu(ui: &mut egui::Ui) {
    // Use the reported native state, including OS-initiated transitions and
    // restoration. A second app preference could disagree with eframe's store.
    let mut fullscreen = active(ui.ctx());
    if crate::menu_popup::checkbox(ui, true, &mut fullscreen, "Full Screen")
        .on_hover_text("Full Screen (F11)")
        .changed()
    {
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Fullscreen(fullscreen));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Editor, Point, Vec2};
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    fn reported(h: &mut Harness<'_, Editor>, fullscreen: Option<bool>) {
        h.input_mut()
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .fullscreen = fullscreen;
    }

    fn commands(h: &Harness<'_, Editor>) -> Vec<bool> {
        h.output()
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .unwrap()
            .commands
            .iter()
            .filter_map(|command| match command {
                egui::ViewportCommand::Fullscreen(value) => Some(*value),
                _ => None,
            })
            .collect()
    }

    fn press(h: &mut Harness<'_, Editor>) -> Vec<bool> {
        h.key_down(egui::Key::F11);
        h.step();
        let result = commands(h);
        h.key_up(egui::Key::F11);
        h.run();
        result
    }

    #[test]
    fn f11_uses_reported_viewport_state_without_a_loaded_map_or_repeated_requests() {
        let mut h = Harness::builder()
            .with_size(Vec2::new(800.0, 600.0))
            .build_eframe(|cc| Editor::new(cc));
        for state in [None, Some(false), Some(true), Some(false)] {
            reported(&mut h, state);
            h.run();
            assert_eq!(press(&mut h), [!state.unwrap_or(false)]);
            assert!(commands(&h).is_empty(), "no request on key release/idle");
            assert!(h.state().loaded.is_none() && !h.state().is_modified());
        }
        h.key_down(egui::Key::Escape);
        h.step();
        assert!(h.output().viewport_output.values().all(|v| {
            v.commands.iter().all(|c| {
                !matches!(
                    c,
                    egui::ViewportCommand::Close | egui::ViewportCommand::Fullscreen(_)
                )
            })
        }));
    }

    #[test]
    fn view_menu_follows_native_transitions_and_requests_the_opposite_state() {
        let mut h = Harness::builder()
            .with_size(Vec2::new(800.0, 600.0))
            .with_theme(egui::Theme::Light)
            .build_eframe(|cc| Editor::new(cc));
        crate::menu_popup::test_font_size(&h.ctx, 12.0);
        for state in [true, false] {
            reported(&mut h, Some(state));
            h.get_by_label("View").click();
            h.run();
            let node = h.get_by_label("Full Screen");
            assert_eq!(
                node.accesskit_node().toggled(),
                Some(if state {
                    egui::accesskit::Toggled::True
                } else {
                    egui::accesskit::Toggled::False
                })
            );
            assert!(node.rect().top() > h.get_by_label("Palettes").rect().top());
            #[cfg(feature = "visual-tests")]
            if state {
                h.remove_cursor();
                h.run();
                h.snapshot("fullscreen-menu-reported");
            }
            h.get_by_label("Full Screen").click();
            h.step();
            assert_eq!(commands(&h), [!state]);
            h.key_press(egui::Key::Escape);
            h.run();
        }
    }

    #[test]
    fn fullscreen_preserves_dirty_documents_history_and_modal_guards() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        h.run();
        assert_eq!(press(&mut h), [true]);
        let map = h.state().loaded.as_ref().unwrap().document.map();
        assert_eq!(map.blocks(), [1, 1]);
        assert!(map.can_undo() && map.is_modified());
        for modal in 0..3 {
            match modal {
                0 => h.state_mut().prepare_resize(),
                1 => h.state_mut().prepare_palettes(),
                _ => h.state_mut().request_close_map(),
            }
            h.run();
            assert!(press(&mut h).is_empty());
            h.get_by_label("Cancel").click();
            h.run();
            assert!(h.state().loaded.as_ref().unwrap().document.map().can_undo());
        }
        h.state_mut().close_map();
        reported(&mut h, Some(true));
        h.run();
        assert_eq!(press(&mut h), [false]);
        assert_eq!(
            std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
            [0, 1]
        );
    }
}
