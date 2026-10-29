use super::*;
use egui_kittest::{Harness, kittest::Queryable};

fn button(h: &mut Harness<'_, Editor>, pos: Pos2, pressed: bool) {
    h.hover_at(pos);
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Middle,
        pressed,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
}

fn large_map() -> (tempfile::TempDir, Harness<'static, Editor>) {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().apply(Command::Resize {
        width: 40,
        height: 40,
        anchor: ResizeAnchor::TopLeft,
    });
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .document
        .map_mut()
        .undo();
    h.state_mut().selected = 1;
    h.run();
    (dir, h)
}

#[test]
fn map_middle_pan_uses_press_anchor_integer_deltas_and_source_bounds_without_editing() {
    for scale in [1.0, 2.0] {
        for (zoom, rulers, maximum) in [
            (false, false, Vec2::new(832.0, 958.0)),
            (false, true, Vec2::new(848.0, 974.0)),
            (true, false, Vec2::new(2432.0, 2430.0)),
            (true, true, Vec2::new(2448.0, 2446.0)),
        ] {
            let (dir, mut h) = large_map();
            h.set_pixels_per_point(scale);
            h.run();
            // A runtime DPI change scales the old logical viewport. Restore
            // the same 800x600 logical client size before comparing bounds.
            h.set_size(Vec2::new(800.0, 600.0));
            h.state_mut().zoom = zoom;
            h.state_mut().rulers = rulers;
            h.run();
            let before = h
                .state()
                .loaded
                .as_ref()
                .unwrap()
                .document
                .map()
                .blocks()
                .to_vec();
            let start = Pos2::new(500.75, 300.75);
            button(&mut h, start, true);
            assert!(
                h.state().canvas_pan.anchor.is_some(),
                "zoom={zoom} rulers={rulers} dpi={scale} viewport={:?}",
                h.ctx.content_rect()
            );
            assert_eq!(
                h.output().platform_output.cursor_icon,
                egui::CursorIcon::Move
            );
            h.hover_at(Pos2::new(499.25, 299.25));
            h.run();
            assert_eq!(
                h.state().canvas_scroll,
                Vec2::splat(1.0),
                "no drag threshold"
            );
            h.hover_at(Pos2::new(-10_000.0, -10_000.0));
            h.run();
            assert_eq!(h.state().canvas_scroll, maximum, "{zoom} {rulers} {scale}");
            assert_eq!(h.state().ruler_scroll, maximum);
            // Returning from a clamped edge still uses the original press anchor.
            h.hover_at(start - Vec2::new(35.0, 45.0));
            h.run();
            assert_eq!(h.state().canvas_scroll, Vec2::new(35.0, 45.0));
            button(&mut h, start - Vec2::new(35.0, 45.0), false);
            h.run();
            assert!(h.state().canvas_pan.anchor.is_none());
            assert_eq!(
                h.state().canvas_scroll,
                Vec2::new(35.0, 45.0),
                "no inertial fling"
            );
            let map = h.state().loaded.as_ref().unwrap().document.map();
            assert_eq!(map.blocks(), before);
            assert!(!map.can_undo() && map.can_redo());
            assert_eq!(h.state().selected, 1);
            assert!(h.state().stroke.is_empty());
            assert_eq!(
                std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
                [0, 1]
            );
        }
    }
}

#[test]
fn sidebar_middle_pan_preempts_child_selection_and_transfer_in_both_modes() {
    for zoom in [false, true] {
        for transfers in [false, true] {
            let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
            let ctx = h.ctx.clone();
            assert!(h.state_mut().apply_block_commands(
                &ctx,
                &[polished_map_core::BlockCommand::Resize { count: 128 }]
            ));
            h.state_mut().zoom = zoom;
            h.state_mut().drag_and_drop = transfers;
            h.state_mut().selected = 1;
            h.run();
            let start = Pos2::new(16.0, 63.0); // Unselected block zero.
            button(&mut h, start, true);
            button(&mut h, start, false);
            assert_eq!(
                h.state().selected,
                1,
                "Workspace consumes middle before Metatile_Button"
            );
            button(&mut h, start, true);
            h.hover_at(Pos2::new(400.0, -2000.0));
            h.run();
            assert_eq!(h.state().sidebar_scroll, if zoom { 1518.0 } else { 494.0 });
            assert_eq!(h.state().canvas_scroll, Vec2::ZERO);
            h.hover_at(start - Vec2::new(0.0, 17.0));
            h.run();
            assert_eq!(h.state().sidebar_scroll, 17.0);
            button(&mut h, start - Vec2::new(0.0, 17.0), false);
            assert_eq!(h.state().selected, 1);
            assert!(
                h.state().block_form.is_none()
                    && h.state().sidebar_press.is_none()
                    && h.state().sidebar_drag.is_none()
            );
            assert!(
                !h.state()
                    .loaded
                    .as_ref()
                    .unwrap()
                    .document
                    .map()
                    .is_modified()
            );
        }
    }
}

#[test]
fn panning_cancels_on_pointer_loss_modals_focus_loss_zoom_reset_and_close() {
    let (_dir, mut h) = large_map();
    let start = Pos2::new(400.0, 300.0);
    for interruption in 0..6 {
        button(&mut h, start, true);
        h.hover_at(start - Vec2::splat(20.0));
        h.run();
        assert!(h.state().canvas_pan.anchor.is_some());
        match interruption {
            0 => h.event(egui::Event::PointerGone),
            1 => h.state_mut().prepare_resize(),
            2 => h.input_mut().focused = false,
            3 => h.state_mut().zoom = !h.state().zoom,
            4 => h.state_mut().reset_canvas_scroll = true,
            _ => h.state_mut().close_map(),
        }
        h.run();
        assert!(
            h.state().canvas_pan.anchor.is_none(),
            "interruption {interruption}"
        );
        let offset = h.state().canvas_scroll;
        h.hover_at(start - Vec2::splat(60.0));
        h.run();
        assert_eq!(h.state().canvas_scroll, offset);
        if interruption == 1 {
            h.get_by_label("Cancel").click();
            h.run();
        }
        h.input_mut().focused = true;
        button(&mut h, start, false);
    }
    assert!(h.state().loaded.is_none());
    button(&mut h, start, true);
    h.hover_at(Pos2::new(-500.0, -500.0));
    h.run();
    button(&mut h, start, false);
    assert_eq!(h.state().canvas_scroll, Vec2::ZERO);
    assert!(!h.state().is_modified());
}

#[test]
fn middle_pan_over_event_markers_and_blank_margin_never_moves_or_opens_events() {
    let (dir, mut h) = large_map();
    let path = dir.path().join("Map.asm");
    let text = "\tbg_event 2, 2, SCRIPT\n";
    std::fs::write(&path, text).unwrap();
    h.state_mut().loaded.as_mut().unwrap().events =
        Some(polished_map_project::EventDocument::open(&path).unwrap());
    h.state_mut().event_ui.mode = true;
    h.run();
    let marker = h.get_by_label("Event 0: bg_event").rect().center();
    for start in [marker, Pos2::new(150.0, 53.0)] {
        let before = h.state().canvas_scroll;
        button(&mut h, start, true);
        h.hover_at(start - Vec2::new(30.0, 20.0));
        h.run();
        assert_eq!(h.state().canvas_scroll, before + Vec2::new(30.0, 20.0));
        assert!(h.state().event_ui.drag.is_none() && h.state().event_ui.form.is_none());
        button(&mut h, start - Vec2::new(30.0, 20.0), false);
    }
    assert!(
        !h.state()
            .loaded
            .as_ref()
            .unwrap()
            .events
            .as_ref()
            .unwrap()
            .script()
            .events()
            .is_modified()
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), text);
}

#[test]
fn workspace_limits_account_for_coupled_bars_and_retained_blank_space() {
    for (content, outer, offset, vertical, expected) in [
        (
            Vec2::new(100.0, 101.0),
            Vec2::splat(100.0),
            Vec2::ZERO,
            false,
            Vec2::new(16.0, 17.0),
        ),
        (
            Vec2::new(101.0, 100.0),
            Vec2::splat(100.0),
            Vec2::ZERO,
            false,
            Vec2::new(17.0, 16.0),
        ),
        (
            Vec2::splat(100.0),
            Vec2::splat(100.0),
            Vec2::ZERO,
            false,
            Vec2::ZERO,
        ),
        (
            Vec2::splat(50.0),
            Vec2::splat(100.0),
            Vec2::splat(40.0),
            false,
            Vec2::ZERO,
        ),
        (
            Vec2::new(128.0, 1024.0),
            Vec2::new(144.0, 530.0),
            Vec2::ZERO,
            true,
            Vec2::new(0.0, 494.0),
        ),
    ] {
        assert_eq!(maximum(content, outer, offset, 16.0, vertical), expected);
    }
}

#[test]
fn minimum_window_pan_uses_the_actual_narrow_viewport() {
    for (rulers, expected) in [
        (false, Vec2::new(2897.0, 2768.0)),
        (true, Vec2::new(2913.0, 2784.0)),
    ] {
        let (_dir, mut h) = large_map();
        h.set_size(Vec2::new(335.0, 262.0));
        h.state_mut().zoom = true;
        h.state_mut().rulers = rulers;
        h.run();
        let bars: Vec<_> = h.get_all_by_role(egui::Role::ScrollBar).collect();
        assert_eq!(bars.len(), 3);
        for bar in bars {
            assert!(
                bar.rect().right() <= 335.0,
                "scrollbar extends past client: {:?}",
                bar.rect()
            );
            assert!(
                bar.rect().bottom() <= 239.0,
                "scrollbar overlaps status: {:?}",
                bar.rect()
            );
        }
        let start = Pos2::new(300.0, 100.0);
        button(&mut h, start, true);
        h.hover_at(Pos2::new(-10_000.0, -10_000.0));
        h.run();
        assert_eq!(h.state().canvas_scroll, expected, "rulers={rulers}");
        assert_eq!(h.state().ruler_scroll, expected);
        button(&mut h, start, false);
    }
}

#[test]
fn pan_clamps_zoom_retained_blank_space_to_the_natural_content() {
    let (_dir, mut h) = large_map();
    h.state_mut().zoom = true;
    h.run();
    let start = Pos2::new(400.0, 300.0);
    button(&mut h, start, true);
    h.hover_at(start - Vec2::splat(2300.0));
    h.run();
    button(&mut h, start - Vec2::splat(2300.0), false);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Equals);
    h.run();
    assert_eq!(h.state().canvas_scroll, Vec2::splat(1150.0));
    button(&mut h, start, true);
    h.hover_at(start - Vec2::splat(1.0));
    h.run();
    assert_eq!(h.state().canvas_scroll, Vec2::new(832.0, 958.0));
    button(&mut h, start - Vec2::splat(1.0), false);
    assert_eq!(h.state().ruler_scroll, h.state().canvas_scroll);
}

#[test]
fn batched_middle_events_keep_the_press_hit_and_release_outside_the_workspace() {
    let (_dir, mut h) = large_map();
    let start = Pos2::new(400.0, 300.0);
    let outside = Pos2::new(20.0, 10.0);
    h.input_mut().events.extend([
        egui::Event::PointerMoved(start),
        egui::Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Middle,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Event::PointerMoved(outside),
        egui::Event::PointerButton {
            pos: outside,
            button: egui::PointerButton::Middle,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    h.step();
    h.run();
    assert_eq!(h.state().canvas_scroll, Vec2::new(380.0, 290.0));
    assert!(h.state().canvas_pan.anchor.is_none());
    assert_eq!(h.state().selected, 1);
    assert!(!h.state().loaded.as_ref().unwrap().document.map().can_undo());
}

#[test]
fn mouse_wheel_during_pan_keeps_the_original_drag_anchor_and_popup_blocks_new_capture() {
    let (_dir, mut h) = large_map();
    let start = Pos2::new(400.0, 300.0);
    button(&mut h, start, true);
    h.hover_at(start - Vec2::splat(20.0));
    h.run();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: Vec2::new(-7.0, -9.0),
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    assert_eq!(h.state().canvas_scroll, Vec2::new(27.0, 29.0));
    h.hover_at(start - Vec2::splat(21.0));
    h.run();
    assert_eq!(h.state().canvas_scroll, Vec2::splat(21.0));
    button(&mut h, start - Vec2::splat(21.0), false);
    h.get_by_label("File").click();
    h.run();
    assert!(h.ctx.any_popup_open());
    button(&mut h, Pos2::new(40.0, 65.0), true);
    assert!(h.state().canvas_pan.anchor.is_none() && h.state().sidebar_pan.anchor.is_none());
    button(&mut h, Pos2::new(40.0, 65.0), false);
    h.key_press(egui::Key::Escape);
    h.run();
}
