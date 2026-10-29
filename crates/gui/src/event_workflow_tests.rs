use super::event_workflows::{EventAction, EventChange};
use super::*;
use egui_kittest::kittest::Queryable;

#[test]
fn event_cursor_and_game_boy_guide_follow_hover_zoom_and_modal_guards() {
    let (_dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    // The cursor and guide work before any event script is loaded.
    h.state_mut().event_ui.mode = true;
    h.hover_at(Pos2::new(264.0, 167.0));
    h.run();
    assert_eq!(
        h.state().event_ui.cursor,
        Some(geometry::EventCursor {
            block_x: 0,
            block_y: 0,
            event_x: 1,
            event_y: 1,
        })
    );
    assert!(h.state().event_ui.screen.is_none());
    h.get_by_label("ID: 0    X/Y (0, 0)    Event: X/Y (1, 1)");
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::M);
    h.run();
    assert!(!h.state().event_ui.game_boy);
    h.key_press_modifiers(
        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        egui::Key::M,
    );
    h.run();
    assert_eq!(
        h.state().event_ui.screen,
        Some(Rect::from_min_size(
            Pos2::new(192.0, 95.0),
            Vec2::new(160.0, 144.0)
        ))
    );
    #[cfg(feature = "visual-tests")]
    h.snapshot("gameboy-screen-normal");
    h.state_mut().zoom = true;
    h.hover_at(Pos2::new(512.0, 287.0));
    h.run();
    assert_eq!(h.state().event_ui.cursor.unwrap().event_x, 1);
    assert_eq!(
        h.state().event_ui.screen,
        Some(Rect::from_min_size(
            Pos2::new(368.0, 143.0),
            Vec2::new(320.0, 288.0)
        ))
    );
    #[cfg(feature = "visual-tests")]
    h.snapshot("gameboy-screen-zoomed");
    h.state_mut().prepare_resize();
    h.run();
    assert!(h.state().event_ui.cursor.is_none());
    assert!(h.state().event_ui.screen.is_none());
    h.key_press_modifiers(
        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        egui::Key::M,
    );
    h.run();
    assert!(h.state().event_ui.game_boy); // blocked shortcut cannot change it
    h.get_by_label("Cancel").click();
    h.run();
    h.hover_at(Pos2::new(512.0, 287.0));
    h.run();
    assert!(h.state().event_ui.screen.is_some());
    h.key_press(egui::Key::Tab);
    h.run();
    assert!(!h.state().event_ui.mode);
    assert!(h.state().event_ui.screen.is_none());
    assert!(h.state().event_ui.cursor.is_none());
    assert!(!h.state().is_modified());
}

#[test]
fn event_hover_uses_topmost_marker_and_original_negative_division() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let path = dir.path().join("Map.asm");
    std::fs::write(
        &path,
        "\tbg_event -1, 0, FIRST\n\tbg_event -1, 0, TOP\n\tbg_event -2, 0, OUTSIDE\n",
    )
    .unwrap();
    h.state_mut().request_event_change(EventChange::Load(path));
    h.state_mut().event_ui.mode = true;
    h.state_mut().event_ui.game_boy = true;
    h.run();
    let pos = h.get_by_label("Event 1: bg_event").rect().center();
    h.hover_at(pos);
    h.run();
    assert_eq!(h.state().event_ui.hovered_event, Some(1));
    assert_eq!(h.state().event_ui.cursor.unwrap().event_x, 0);
    assert_eq!(
        h.state().event_ui.screen.unwrap().size(),
        Vec2::new(160.0, 144.0)
    );
    let pos = h.get_by_label("Event 2: bg_event").rect().center();
    h.hover_at(pos);
    h.run();
    assert_eq!(h.state().event_ui.hovered_event, Some(2));
    assert!(h.state().event_ui.cursor.is_none());
    assert!(h.state().event_ui.screen.is_none());
    h.hover_at(Pos2::new(145.0, 48.0)); // visible canvas margin, no event/block
    h.run();
    assert!(h.state().event_ui.hovered_event.is_none());
    assert!(h.state().event_ui.cursor.is_none());
    h.hover_at(Pos2::new(30.0, 65.0)); // sidebar
    h.run();
    assert!(h.state().event_ui.cursor.is_none());
    assert!(h.state().event_ui.screen.is_none());
    assert!(!h.state().is_modified());
}

#[test]
fn game_boy_guide_tracks_scrolled_content_and_clips_at_viewport_edge() {
    let (_dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    h.state_mut().apply(Command::Resize {
        width: 20,
        height: 20,
        anchor: ResizeAnchor::TopLeft,
    });
    h.state_mut().event_ui.mode = true;
    h.state_mut().event_ui.game_boy = true;
    h.hover_at(Pos2::new(280.0, 183.0));
    h.run();
    let before = h.state().event_ui.cursor.unwrap();
    let frame = h.state().event_ui.screen.unwrap();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: Vec2::new(0.0, -128.0),
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    let after = h.state().event_ui.cursor.unwrap();
    assert_eq!(after.event_x, before.event_x);
    assert_eq!(after.event_y, before.event_y + 8);
    assert_eq!(h.state().event_ui.screen.unwrap(), frame);
    // Move close to the scroll area's top edge: the logical guide extends
    // outside the viewport, while the painter clips its fill and outlines.
    h.hover_at(Pos2::new(280.0, 55.0));
    h.run();
    assert!(h.state().event_ui.screen.unwrap().top() < 47.0);
    #[cfg(feature = "visual-tests")]
    h.snapshot("gameboy-screen-clipped");
    let blocks = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .document
        .map()
        .blocks()
        .to_vec();
    h.get_by_label("View").click();
    h.run();
    assert!(h.state().event_ui.screen.is_none());
    h.get_by_label("Game Boy Screen").click();
    h.run();
    assert!(!h.state().event_ui.game_boy);
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        blocks
    );
}

#[test]
fn event_editor_requests_preserve_saved_script_and_dispatch_errors() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let path = dir.path().join("Map ü &.asm");
    let original = "Header:\n\tdb 42\n\tbg_event 0, 0, SIGN\n";
    std::fs::write(&path, original).unwrap();
    h.state_mut()
        .request_event_change(EventChange::Load(path.clone()));
    h.state_mut().event_ui.mode = true;
    h.run();
    let ctx = h.ctx.clone();
    h.state_mut().event_action(EventAction::Move(0, 0, 0), &ctx);
    let pos = h.get_by_label("Event 0: bg_event").rect().center();
    h.hover_at(pos);
    h.run();
    for _ in 0..2 {
        for pressed in [true, false] {
            h.input_mut().time = Some(ctx.input(|i| i.time) + 0.05);
            h.event(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            h.step();
        }
    }
    h.input_mut().time = None;
    h.run();
    let request = h
        .state_mut()
        .event_ui
        .script_request
        .take()
        .expect("double-click request");
    assert_eq!(request.path(), path.canonicalize().unwrap());
    assert_eq!(request.line(), Some(3));
    assert!(h.state().is_modified());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);

    // Shift has priority even on the second click; a non-warp event does not
    // launch an editor or navigate anywhere.
    for _ in 0..2 {
        for pressed in [true, false] {
            h.input_mut().time = Some(ctx.input(|i| i.time) + 0.05);
            h.event_modifiers(
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::SHIFT,
                },
                egui::Modifiers::SHIFT,
            );
            h.step();
        }
    }
    h.input_mut().time = None;
    h.run();
    assert!(h.state().event_ui.script_request.is_none());

    // Control/Command-U opens the whole saved script, without a line argument.
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::U);
    h.run();
    assert_eq!(
        h.state().event_ui.script_request.as_ref().unwrap().line(),
        None
    );
    h.state_mut().dispatch_event_script(&ctx, |_| {
        Err(polished_map_platform::ScriptOpenError::Platform(
            "Synthetic editor failure".into(),
        ))
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while h.state().error.is_none() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
        h.step();
    }
    assert_eq!(h.state().error.as_deref(), Some("Synthetic editor failure"));
    assert!(h.state().event_ui.script_request.is_none());
    assert!(h.state().is_modified());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    h.state_mut().error = None;
    h.state_mut().event_action(EventAction::ViewScript(0), &ctx);
    h.state_mut().dispatch_event_script(&ctx, |request| {
        assert_eq!(request.line(), Some(3));
        Ok(())
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while h.state().status != "Event script opener completed"
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(std::time::Duration::from_millis(5));
        h.step();
    }
    assert_eq!(h.state().status, "Event script opener completed");
    assert!(h.state().error.is_none());
    h.get_by_label("Data").click();
    h.run();
    h.get_by_label("View Event Script").click();
    h.run();
    assert_eq!(
        h.state().event_ui.script_request.as_ref().unwrap().line(),
        None
    );
}

#[test]
fn event_mode_shortcuts_respect_modal_and_text_focus() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let path = dir.path().join("Map.asm");
    std::fs::write(&path, "\tbg_event 0, 0, SIGN\n").unwrap();
    h.state_mut().request_event_change(EventChange::Load(path));
    h.run();
    h.key_press(egui::Key::Tab);
    h.run();
    assert!(h.state().event_ui.mode);
    h.key_press(egui::Key::Tab);
    h.run();
    assert!(!h.state().event_ui.mode);
    let shift = egui::Modifiers::COMMAND | egui::Modifiers::SHIFT;
    h.key_press_modifiers(shift, egui::Key::Num3);
    h.run();
    assert!(!h.state().event_ui.warp_ids);
    h.key_press_modifiers(shift, egui::Key::E);
    h.run();
    assert!(h.state().event_ui.mode);
    let ctx = h.ctx.clone();
    h.state_mut().event_action(EventAction::Options(0), &ctx);
    h.run();
    h.get_by_label("Event X").click();
    h.run();
    h.key_press(egui::Key::Tab);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::U);
    h.key_press_modifiers(shift, egui::Key::B);
    h.run();
    assert!(h.state().event_ui.mode);
    assert!(h.state().event_ui.script_request.is_none());
    h.get_by_label("Cancel").click();
    h.run();
    h.key_press_modifiers(shift, egui::Key::B);
    h.run();
    assert!(!h.state().event_ui.mode);
}

#[test]
#[ignore = "requires POLISHED_MAP_POKECRYSTAL; local headless GUI integration"]
fn real_new_bark_town_auto_events_edit_and_save_on_temporary_script() {
    let root = PathBuf::from(
        std::env::var_os("POLISHED_MAP_POKECRYSTAL").expect("set POLISHED_MAP_POKECRYSTAL"),
    );
    let path = root.join("maps/NewBarkTown.asm");
    let original = std::fs::read(&path).unwrap();
    let mut h = egui_kittest::Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .build_eframe(|cc| Editor::new(cc));
    let ctx = h.ctx.clone();
    h.state_mut().start_load(
        OpenForm {
            path: root.join("maps/NewBarkTown.blk"),
            root: root.to_string_lossy().into_owned(),
            tileset: "johto".into(),
            ..Default::default()
        },
        &ctx,
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while h.state().loading.is_some() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
        h.step();
    }
    h.run();
    assert!(h.state().loading.is_none());
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    let loaded = h.state().loaded.as_ref().unwrap();
    let expected = polished_map_project::EventDocument::open(&path).unwrap();
    assert_eq!(
        loaded.events.as_ref().unwrap().script().events(),
        expected.script().events()
    );
    assert!(!expected.script().events().entries().is_empty());
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("NewBarkTown.asm");
    std::fs::write(&copy, &original).unwrap();
    h.state_mut()
        .request_event_change(EventChange::Load(copy.clone()));
    h.state_mut().event_action(EventAction::Options(0), &ctx);
    h.run();
    h.get_by_label("Event X").click();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Event X").type_text("6");
    h.run();
    h.get_by_label("OK").click();
    h.run();
    h.state_mut().save_events();
    h.run();
    assert!(h.state().error.is_none());
    assert!(!h.state().is_modified());
    assert_eq!(
        polished_map_project::EventDocument::open(&copy)
            .unwrap()
            .script()
            .events()
            .entries()[0]
            .definition()
            .x,
        6
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

#[test]
fn event_draft_cancel_accept_reload_unload_and_save_are_independent() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let path = dir.path().join("Map.asm");
    let original = "Label:\r\n\twarp_event 0, 0, MAP, 2\r\n\twarp_event 2, 0, MAP, 1\r\n\tdb 42";
    std::fs::write(&path, original).unwrap();
    h.state_mut()
        .request_event_change(EventChange::Load(path.clone()));
    h.run();
    let ctx = h.ctx.clone();
    h.state_mut().event_action(EventAction::Options(0), &ctx);
    h.state_mut().status = "Synthetic event script".into();
    h.run();
    #[cfg(feature = "visual-tests")]
    {
        h.state_mut()
            .event_ui
            .form
            .as_mut()
            .unwrap()
            .windows_buttons = true;
        h.remove_cursor();
        h.run();
        // Inspected Linux CI artifact 40780636085: one dimmed glyph pixel
        // differs by 2/255 RGB; the rest differ by at most 1/255. Keep the
        // default elsewhere and permit only this small perceptual-color delta.
        h.snapshot_options(
            "event-options-warp",
            &egui_kittest::SnapshotOptions::default()
                .threshold(egui_kittest::OsThreshold::<f32>::default().linux(1.0)),
        );
    }
    h.get_by_label("Cancel").click();
    h.run();
    assert!(!h.state().is_modified());
    h.state_mut().event_action(EventAction::Move(0, 3, 4), &ctx);
    h.run();
    assert!(h.state().is_modified());
    assert!(!h.state().loaded.as_ref().unwrap().document.map().can_undo());
    h.state_mut().request_event_change(EventChange::Reload);
    h.run();
    h.get_by_label("Cancel").click();
    h.run();
    assert!(h.state().is_modified());
    h.state_mut().save_events();
    h.run();
    assert!(!h.state().is_modified());
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("warp_event  3,  4")
    );
    h.state_mut().event_action(EventAction::Move(0, 5, 6), &ctx);
    h.state_mut().request_event_change(EventChange::Unload);
    h.run();
    h.get_by_label("Discard Event Changes").click();
    h.run();
    assert!(h.state().loaded.as_ref().unwrap().events.is_none());
    assert!(!h.state().is_modified());
}

#[test]
fn event_options_geometry_fields_and_actual_canvas_drag_match_source() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let path = dir.path().join("Map.asm");
    std::fs::write(
        &path,
        "\tperson_event SPRITE_RED, $00, $00, FLAGS ; note\n\twarp_event 2, 0, MAP, 1\n",
    )
    .unwrap();
    h.state_mut().request_event_change(EventChange::Load(path));
    h.state_mut().status = "Synthetic event script".into();
    h.key_press_modifiers(
        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        egui::Key::E,
    );
    h.run();
    assert!(h.state().event_ui.mode);
    let pos = h.get_by_label("Event 0: person_event").rect().center();
    assert_eq!(pos, Pos2::new(248.0, 151.0));
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Secondary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Secondary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    assert_eq!(
        h.state().event_ui.form.as_ref().unwrap().bounds.size(),
        Vec2::new(300.0, 158.0)
    );
    for windows in [true, false] {
        h.state_mut()
            .event_ui
            .form
            .as_mut()
            .unwrap()
            .windows_buttons = windows;
        h.run();
        let area = h.state().event_ui.form.as_ref().unwrap().bounds;
        let ok = h.get_by_label("OK").rect();
        assert!((ok.left() - area.left() - if windows { 116.0 } else { 210.0 }).abs() < 0.1);
        assert!((ok.top() - area.top() - 126.0).abs() < 0.1);
        #[cfg(feature = "visual-tests")]
        {
            h.remove_cursor();
            h.run();
            h.snapshot(if windows {
                "event-options-prefix-windows"
            } else {
                "event-options-prefix-unix"
            });
        }
    }
    h.get_by_label("Event prefix").focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Event prefix").type_text("SPRITE_BLUE");
    h.run();
    h.get_by_label("Event X").click();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Event X").type_text("03");
    h.run();
    h.get_by_label("OK").click();
    h.run();
    let event = &h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .events
        .as_ref()
        .unwrap()
        .script()
        .events()
        .entries()[0];
    assert_eq!(event.definition().prefix.as_deref(), Some("SPRITE_BLUE,"));
    assert_eq!(event.definition().x, 3);
    let from = h.get_by_label("Event 0: person_event").rect().center();
    let to = Pos2::new(264.0, 167.0); // event cell (1,1), independent of block coordinates
    h.hover_at(from);
    h.drag_at(from);
    h.run();
    h.hover_at(to);
    h.run();
    h.event(egui::Event::PointerButton {
        pos: to,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    let loaded = h.state().loaded.as_ref().unwrap();
    let d = loaded.events.as_ref().unwrap().script().events().entries()[0].definition();
    assert_eq!((d.x, d.y), (1, 1));
    assert!(!loaded.document.map().is_modified());
    assert!(h.state().event_ui.drag.is_none());
    #[cfg(feature = "visual-tests")]
    {
        h.remove_cursor();
        h.run();
        h.snapshot("event-overlays");
        h.state_mut().zoom = true;
        h.run();
        assert_eq!(
            h.get_by_label("Event 0: person_event").rect().center(),
            Pos2::new(512.0, 287.0)
        );
        h.snapshot("event-overlays-zoomed");
    }
    for (p, expected) in [
        (-16.0, -2),
        (-15.0, -1),
        (0.0, 0),
        (16.0, 1),
        (9999.0, 9),
        (-9999.0, -6),
    ] {
        assert_eq!(
            super::event_workflows::drag_coordinate(p, 16.0, 2),
            expected
        );
        assert_eq!(
            super::event_workflows::drag_coordinate(p * 2.0, 32.0, 2),
            expected
        );
    }
}

#[test]
fn resize_and_save_integrate_event_state_without_touching_unrelated_scripts() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let path = dir.path().join("Map.asm");
    let original = "Header:\n\tbg_event 0, 0, SIGN\n\tdb 42\n";
    std::fs::write(&path, original).unwrap();
    h.state_mut()
        .request_event_change(EventChange::Load(path.clone()));
    h.run();
    h.state_mut().apply(Command::Resize {
        width: 4,
        height: 3,
        anchor: ResizeAnchor::BottomRight,
    });
    h.run();
    let loaded = h.state().loaded.as_ref().unwrap();
    let d = loaded.events.as_ref().unwrap().script().events().entries()[0].definition();
    assert_eq!((d.x, d.y), (4, 4));
    assert!(loaded.document.map().is_modified());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    h.state_mut().apply(Command::Resize {
        width: 0,
        height: 3,
        anchor: ResizeAnchor::Center,
    });
    assert_eq!(h.state().loaded.as_ref().unwrap().document.map().width(), 4);
    h.state_mut().error = None;
    h.state_mut().save();
    h.run();
    assert!(!h.state().is_modified());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "Header:\n\tbg_event  4,  4, SIGN\n\tdb 42\n"
    );
    assert_eq!(
        std::fs::read(dir.path().join("Map.2x1.blk")).unwrap().len(),
        12
    );
    let ctx = h.ctx.clone();
    h.state_mut().event_action(EventAction::Move(0, 5, 5), &ctx);
    std::fs::write(dir.path().join("Map.2x1.blk"), [99]).unwrap();
    h.state_mut().save();
    h.run(); // event-only save does not overwrite the clean map
    assert!(h.state().error.is_none());
    assert!(!h.state().is_modified());
    assert_eq!(std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [99]);
    h.state_mut().event_action(EventAction::Move(0, 6, 6), &ctx);
    std::fs::write(&path, "external").unwrap();
    h.state_mut().save_events();
    assert!(h.state().error.is_some());
    assert!(h.state().is_modified());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "external");
    h.state_mut().error = None;
    h.state_mut()
        .request_event_change(EventChange::Load(dir.path().join("missing.asm")));
    h.run();
    h.get_by_label("Discard Event Changes").click();
    h.run();
    assert!(h.state().error.is_some());
    assert!(h.state().is_modified());
}

#[test]
fn automatic_events_discovery_new_map_skip_and_failed_script_warning() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    std::fs::create_dir_all(dir.path().join("maps")).unwrap();
    std::fs::write(dir.path().join("maps/Map.asm"), "\tbg_event 1, 2, MAP\n").unwrap();
    std::fs::write(dir.path().join("Map.2x1.asm"), "\tbg_event 3, 4, SIBLING\n").unwrap();
    let doc = MapDocument::open_in_project(
        &dir.path().join("Map.2x1.blk"),
        &OpenOptions::default(),
        Some(dir.path()),
    )
    .unwrap();
    let (events, warning) = super::event_workflows::auto_load_events(&doc, true);
    assert!(warning.is_none());
    assert_eq!(
        events.unwrap().script().events().entries()[0]
            .definition()
            .x,
        1
    );
    assert!(
        super::event_workflows::auto_load_events(&doc, false)
            .0
            .is_none()
    );
    let new = MapDocument::new_map(dir.path(), 2, 1).unwrap();
    assert!(
        super::event_workflows::auto_load_events(&new, true)
            .0
            .is_none()
    );
    std::fs::remove_file(dir.path().join("maps/Map.asm")).unwrap();
    assert_eq!(
        super::event_workflows::auto_load_events(&doc, true)
            .0
            .unwrap()
            .script()
            .events()
            .entries()[0]
            .definition()
            .x,
        3
    );
    std::fs::write(
        dir.path().join("Map.2x1.asm"),
        "\tbg_event WIDTH-1, 2, BAD\n",
    )
    .unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().start_load(
        OpenForm {
            path: dir.path().join("Map.2x1.blk"),
            root: dir.path().to_string_lossy().into_owned(),
            tileset: "demo".into(),
            width: 2,
            height: 1,
            ..Default::default()
        },
        &ctx,
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while h.state().loading.is_some() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
        h.step();
    }
    h.run();
    assert!(h.state().loading.is_none());
    assert!(h.state().loaded.as_ref().unwrap().events.is_none());
    assert!(
        h.state()
            .error
            .as_ref()
            .unwrap()
            .contains("event script could not be loaded")
    );
}

#[test]
fn shift_click_warp_honors_unsaved_cancel_and_loads_destination_events() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let root = dir.path();
    std::fs::write(root.join("Makefile"), "").unwrap();
    std::fs::create_dir_all(root.join("constants")).unwrap();
    std::fs::create_dir_all(root.join("data/maps")).unwrap();
    std::fs::write(
        root.join("constants/map_constants.asm"),
        "\tmap_const DEST, 2, 1\n",
    )
    .unwrap();
    std::fs::write(
        root.join("data/maps/maps.asm"),
        "map Dest, TILESET_DEMO, TOWN, LANDMARK_TEST, MUSIC, FALSE, PALETTE_DAY, FISH\n",
    )
    .unwrap();
    std::fs::write(root.join("Dest.blk"), [1, 0]).unwrap();
    std::fs::write(root.join("Dest.asm"), "\tbg_event 1, 1, SCRIPT\n").unwrap();
    std::fs::write(root.join("Map.asm"), "\twarp_event 0, 0, DEST, 1\n").unwrap();
    h.state_mut()
        .request_event_change(EventChange::Load(root.join("Map.asm")));
    h.state_mut().event_ui.mode = true;
    h.run();
    let ctx = h.ctx.clone();
    h.state_mut().event_action(EventAction::Move(0, 0, 0), &ctx);
    let pos = h.get_by_label("Event 0: warp_event").rect().center();
    h.hover_at(pos);
    h.run();
    for pressed in [true, false] {
        h.event_modifiers(
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::SHIFT,
            },
            egui::Modifiers::SHIFT,
        );
        h.step();
    }
    h.run();
    assert!(
        h.state().confirm_discard,
        "error={:?}, modified={}, mode={}, drag={:?}",
        h.state().error,
        h.state().is_modified(),
        h.state().event_ui.mode,
        h.state().event_ui.drag
    );
    h.get_by_label("Cancel").click();
    h.run();
    assert!(h.state().event_ui.warp_path.is_none());
    assert!(h.state().is_modified());
    h.state_mut().event_action(EventAction::Warp(0), &ctx);
    h.run();
    h.get_by_label("OK").click();
    h.run();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while h.state().loading.is_some() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
        h.step();
    }
    h.run();
    assert!(h.state().loading.is_none());
    let loaded = h.state().loaded.as_ref().unwrap();
    assert!(
        loaded.document.path().ends_with("Dest.blk"),
        "error={:?}",
        h.state().error
    );
    assert_eq!(loaded.document.map().blocks(), [1, 0]);
    assert_eq!(
        loaded.events.as_ref().unwrap().script().events().entries()[0]
            .definition()
            .kind,
        polished_map_core::EventKind::BgEvent
    );
    assert!(!h.state().is_modified());
    assert_eq!(
        std::fs::read_to_string(root.join("Map.asm")).unwrap(),
        "\twarp_event 0, 0, DEST, 1\n"
    );
}
