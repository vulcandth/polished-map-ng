use super::*;
use egui_kittest::{Harness, kittest::Queryable};

fn wheel(h: &mut Harness<'_, Editor>, position: Pos2, delta: Vec2) {
    h.hover_at(position);
    h.run();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
}

#[test]
fn zoom_controls_scale_map_scroll_and_reset_sidebar_without_edits() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().apply(Command::Resize {
        width: 40,
        height: 40,
        anchor: ResizeAnchor::TopLeft,
    });
    let ctx = h.ctx.clone();
    assert!(h.state_mut().apply_block_commands(
        &ctx,
        &[polished_map_core::BlockCommand::Resize { count: 128 }]
    ));
    h.state_mut().rulers = true;
    h.state_mut().status = "Zoom scroll fixture".into();
    h.run();
    let map_before = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .document
        .map()
        .blocks()
        .to_vec();
    wheel(&mut h, Pos2::new(400.0, 300.0), Vec2::new(-701.0, -603.0));
    assert_eq!(h.state().canvas_scroll, Vec2::new(701.0, 603.0));
    wheel(&mut h, Pos2::new(80.0, 100.0), Vec2::new(0.0, -67.0));
    assert_eq!(h.state().sidebar_scroll, 67.0);
    h.remove_cursor();
    h.run();
    #[cfg(feature = "visual-tests")]
    h.snapshot("zoom-scrolled-normal");
    h.get_by_label("Zoom").click();
    h.run();
    assert!(h.state().zoom && h.state().canvas_zoom);
    // Both targets exceed the old viewport's scroll limits. Applying the old
    // layout before the new offset would incorrectly clamp this transition.
    assert_eq!(h.state().canvas_scroll, Vec2::new(1402.0, 1206.0));
    assert_eq!(h.state().ruler_scroll, h.state().canvas_scroll);
    assert_eq!(h.state().sidebar_scroll, 0.0);
    h.remove_cursor();
    h.run();
    #[cfg(feature = "visual-tests")]
    h.snapshot("zoom-scrolled-enlarged");
    wheel(&mut h, Pos2::new(400.0, 300.0), Vec2::new(-1.0, -1.0));
    assert_eq!(h.state().canvas_scroll, Vec2::new(1403.0, 1207.0));
    h.get_by_label("View").click();
    h.run();
    h.get_by_role_and_label(egui::Role::CheckBox, "Zoom")
        .click();
    h.run();
    assert!(!h.state().zoom);
    assert_eq!(h.state().canvas_scroll, Vec2::new(701.0, 603.0));
    for (modifiers, key) in [
        (egui::Modifiers::COMMAND, egui::Key::Equals),
        (
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::Plus,
        ),
        (
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::Equals,
        ),
        (egui::Modifiers::COMMAND, egui::Key::Plus),
    ] {
        let old = h.state().zoom;
        h.key_press_modifiers(modifiers, key);
        h.run();
        assert_eq!(h.state().zoom, !old);
        assert_eq!(
            h.state().canvas_scroll,
            if old {
                Vec2::new(701.0, 603.0)
            } else {
                Vec2::new(1402.0, 1206.0)
            }
        );
    }
    h.state_mut().prepare_resize();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Equals);
    h.run();
    assert!(!h.state().zoom);
    h.get_by_label("Cancel").click();
    h.run();
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        map_before
    );
    // The pre-existing explicit Resize commands remain the only edits.
    assert!(!h.state().loaded.as_ref().unwrap().document.map().can_undo());
    assert!(h.state().is_modified());
}

#[test]
fn zoom_out_retains_far_edge_space_until_scrolling_back() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().apply(Command::Resize {
        width: 40,
        height: 40,
        anchor: ResizeAnchor::TopLeft,
    });
    h.state_mut().zoom = true;
    h.state_mut().status = "Zoom far-edge fixture".into();
    h.run();
    for _ in 0..4 {
        wheel(&mut h, Pos2::new(400.0, 300.0), Vec2::new(-575.0, -575.25));
    }
    assert_eq!(h.state().canvas_scroll, Vec2::new(2300.0, 2301.0));
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Equals);
    h.run();
    assert_eq!(h.state().canvas_scroll, Vec2::splat(1150.0));
    assert_eq!(h.state().ruler_scroll, Vec2::splat(1150.0));
    h.remove_cursor();
    h.run();
    #[cfg(feature = "visual-tests")]
    h.snapshot("zoom-far-edge-space");
    // The extended range is only large enough to retain the current position.
    // Returning toward the map removes it, as in FLTK's scrollvalue calculation.
    wheel(&mut h, Pos2::new(400.0, 300.0), Vec2::splat(600.0));
    assert_eq!(h.state().canvas_scroll, Vec2::splat(550.0));
    wheel(&mut h, Pos2::new(400.0, 300.0), Vec2::splat(-600.0));
    assert_eq!(h.state().canvas_scroll, Vec2::new(832.0, 958.0));
}

#[test]
fn explicit_scroll_reset_wins_over_zoom_and_empty_view_can_toggle() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().apply(Command::Resize {
        width: 40,
        height: 40,
        anchor: ResizeAnchor::TopLeft,
    });
    wheel(&mut h, Pos2::new(400.0, 300.0), Vec2::new(-57.0, -115.0));
    assert_eq!(h.state().canvas_scroll, Vec2::new(57.0, 115.0));
    h.state_mut().reset_canvas_scroll = true;
    h.state_mut().zoom = true;
    h.run();
    assert_eq!(h.state().canvas_scroll, Vec2::ZERO);
    assert!(h.state().canvas_zoom);
    let mut empty = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .build_eframe(|cc| Editor::new(cc));
    empty.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Equals);
    empty.run();
    assert!(empty.state().zoom && empty.state().canvas_zoom);
    assert_eq!(empty.state().canvas_scroll, Vec2::ZERO);
    assert!(!empty.state().is_modified());
}
