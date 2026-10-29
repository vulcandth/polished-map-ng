use super::*;
use crate::event_workflows::EventChange;
use egui_kittest::{Harness, kittest::Queryable};
use std::fs;

fn open(h: &mut Harness<'_, Editor>) {
    h.get_by_label("File").click();
    h.run();
    h.get_by_role_and_label(egui::Role::MenuItem, "Print…")
        .click();
    h.run();
}

#[test]
fn geometry_independent_flags_cancel_and_reopen_match_source() {
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for windows in [false, true] {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        h.state_mut().allow_priority = false;
        open(&mut h);
        assert_eq!(
            h.state().print_form.as_ref().unwrap().options,
            PrintOptions::default()
        );
        assert!(h.state().grid && !h.state().allow_priority);
        h.state_mut().print_form.as_mut().unwrap().windows_buttons = windows;
        h.remove_cursor();
        h.run();
        let area = h.state().print_form.as_ref().unwrap().bounds;
        assert_eq!(area.height(), 80.0);
        for (label, offset) in [
            ("Export…", if windows { 278.0 } else { 90.0 }),
            ("Copy", 184.0),
            ("Cancel", if windows { 90.0 } else { 278.0 }),
        ] {
            let rect = h.get_by_label(label).rect();
            assert_eq!(rect.size(), Vec2::new(80.0, 22.0));
            assert_eq!(area.right() - rect.left(), offset);
            assert_eq!(rect.top() - area.top(), 48.0);
            assert_eq!(area.bottom() - rect.bottom(), 10.0);
        }
        let font = egui::FontId::proportional(13.0);
        let measure = |label: &str| {
            h.ctx.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(label.into(), font.clone(), Color32::BLACK)
                    .size()
                    .x
                    .ceil()
            })
        };
        let mut x = 10.0 + measure("Show:") + 8.0 + 4.0;
        for label in ["Grid", "IDs", "Priority", "Events", "Warp IDs"] {
            let rect = h.get_by_role_and_label(egui::Role::CheckBox, label).rect();
            assert_eq!(rect.left() - area.left(), x);
            assert!(rect.height() <= 22.0);
            assert!(rect.width() <= measure(label) + 26.0);
            x += measure(label) + 26.0 + 4.0;
        }
        assert_eq!(area.width(), (x - 4.0 + 10.0).max(288.0));
        #[cfg(feature = "visual-tests")]
        snapshots.add(h.try_snapshot(if windows {
            "print-options-windows"
        } else {
            "print-options-unix"
        }));
        // Warp IDs and priority remain independently selectable with Events off
        // and the editor's Allow Priority option off.
        for label in ["Warp IDs", "Priority"] {
            h.get_by_role_and_label(egui::Role::CheckBox, label).click();
            h.run();
        }
        let options = h.state().print_form.as_ref().unwrap().options;
        assert!(options.warp_ids && options.priority && !options.events);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
        h.run();
        assert!(h.state().loaded.is_some());
        assert!(!h.state().confirm_discard);
        h.key_press(egui::Key::Escape);
        h.run();
        assert!(h.state().print_form.is_none() && h.state().print_request.is_none());
        assert_eq!(h.state().print_options, options);
        open(&mut h);
        assert_eq!(h.state().print_form.as_ref().unwrap().options, options);
        h.get_by_role_and_label(egui::Role::CheckBox, "Grid")
            .click();
        h.run();
        h.get_by_label("Cancel").click();
        h.run();
        assert!(h.state().print_options.grid);
        assert!(!h.state().is_modified());
        assert!(!h.state().loaded.as_ref().unwrap().document.map().can_undo());
        assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [0, 1]);
    }
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}

#[test]
fn command_shortcut_toolbar_and_default_button_share_print_workflow() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::P);
    h.run();
    assert!(h.state().print_form.is_some());
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::C);
    h.run();
    assert!(h.state().print_form.is_none());
    assert!(matches!(
        h.state().print_request.as_ref().unwrap().destination,
        Destination::Clipboard
    ));
    h.state_mut().print_request = None;
    h.get_by_role_and_label(egui::Role::Button, "Print…")
        .click();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(matches!(
        h.state().print_request.as_ref().unwrap().destination,
        Destination::ChooseFile
    ));
    let ctx = h.ctx.clone();
    h.state_mut().dispatch_print(&ctx, || Ok(None), run);
    assert!(h.state().print_request.is_none() && h.state().print_worker.is_none());
    assert!(h.state().print_success.is_none() && h.state().error.is_none());
    // Command+Shift+P remains the separate priority shortcut.
    h.state_mut().allow_priority = true;
    h.key_press_modifiers(
        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        egui::Key::P,
    );
    h.run();
    assert!(h.state().print_form.is_none());
    assert!(!h.state().show_priority);
    h.state_mut().close_map();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::P);
    h.run();
    assert!(h.state().print_form.is_none());
}

fn mask(_: PrintFont, _: &str) -> Result<PrintTextMask, PrintError> {
    PrintTextMask::new(2, 2, vec![255, 64, 128, 0])
}

#[test]
fn copy_and_png_capture_unsaved_map_palettes_events_and_print_flags() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let script = dir.path().join("Map.asm");
    fs::write(
        &script,
        "\twarp_event 0, 0, DESTINATION, 1\n\tbg_event 1, 0, SIGN\n",
    )
    .unwrap();
    h.state_mut()
        .request_event_change(EventChange::Load(script.clone()));
    h.state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .events
        .as_mut()
        .unwrap()
        .apply(&[polished_map_core::EventCommand::Move {
            index: 0,
            x: 2,
            y: 0,
        }])
        .unwrap();
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.state_mut().prepare_palettes();
    h.run();
    h.get_by_label("RED color 2").click();
    h.run();
    h.get_by_label("Hex color").focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Hex color").type_text("123456");
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    h.get_by_label("OK").click();
    h.run();
    h.state_mut().print_options = PrintOptions {
        grid: true,
        ids: true,
        priority: true,
        events: true,
        warp_ids: true,
    };
    h.state_mut().hex = true;
    h.state_mut().dark = true;
    h.state_mut().queue_print(Destination::Clipboard);
    let job = h.state_mut().print_request.take().unwrap();
    assert_eq!(job.map.blocks(), [1, 1]);
    assert_eq!(
        &job.graphics.render_block(1).unwrap().pixels[..3],
        &[0x12, 0x34, 0x56]
    );
    assert_eq!(job.events.as_ref().unwrap().entries().len(), 2);
    assert_eq!(job.events.as_ref().unwrap().entries()[0].definition().x, 2);
    assert_eq!(
        job.style,
        PrintStyle {
            hexadecimal_ids: true,
            grid_rgb: [38; 3]
        }
    );
    let expected = job
        .graphics
        .render_print(&job.map, job.events.as_ref(), job.options, job.style, mask)
        .unwrap();
    let target = prepare_png_export(&dir.path().join("shot")).unwrap();
    h.state_mut().queue_print(Destination::File(target.clone()));
    let png_job = h.state_mut().print_request.take().unwrap();
    // Changes after capture cannot change the queued output.
    h.state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .document
        .map_mut()
        .undo();
    h.state_mut().print_options = PrintOptions::default();
    let PrintOutput::Copied(copied) = job.run_with(mask).unwrap() else {
        panic!("expected clipboard output")
    };
    assert_eq!(copied.size, [64, 32]);
    assert_eq!(
        copied
            .pixels
            .iter()
            .flat_map(|p| p.to_array()[..3].to_vec())
            .collect::<Vec<_>>(),
        expected.pixels
    );
    assert!(matches!(
        png_job.run_with(mask).unwrap(),
        PrintOutput::Exported(_)
    ));
    let expected_path = dir.path().join("expected.png");
    polished_map_project::write_new_png(&expected_path, &expected).unwrap();
    assert_eq!(
        fs::read(target.path()).unwrap(),
        fs::read(expected_path).unwrap()
    );
    assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [0, 1]);
    assert_eq!(
        fs::read_to_string(script).unwrap(),
        "\twarp_event 0, 0, DESTINATION, 1\n\tbg_event 1, 0, SIGN\n"
    );
    assert!(h.state().loaded.as_ref().unwrap().document.map().can_redo());
}

#[test]
fn worker_output_handoff_success_errors_and_modal_close_guard() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().prepare_print();
    h.run();
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
    h.get_by_label("Copy").click();
    h.run();
    let ctx = h.ctx.clone();
    h.state_mut()
        .dispatch_print(&ctx, || panic!("Copy must not open a picker"), run);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let copied = loop {
        h.step();
        if let Some(image) = h
            .output()
            .platform_output
            .commands
            .iter()
            .find_map(|c| match c {
                egui::OutputCommand::CopyImage(image) => Some(image.clone()),
                _ => None,
            })
        {
            break image;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "worker failed: {:?}",
            h.state().error
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(copied.size, [64, 32]);
    assert_eq!(
        h.state().print_success.as_deref(),
        Some("Copied to clipboard!")
    );
    assert!(h.state().print_worker.is_none());
    h.remove_cursor();
    h.run();
    #[cfg(feature = "visual-tests")]
    h.snapshot("print-copy-success");
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(h.state().print_success.is_none());
    for result in [Some(Err("synthetic encode failure".into())), None] {
        let (tx, rx) = mpsc::channel();
        if let Some(result) = result {
            tx.send(result).unwrap();
        }
        drop(tx);
        h.state_mut().print_worker = Some(rx);
        h.step();
        assert!(h.state().error.is_some());
        assert_eq!(h.state().status, "Print failed");
        assert!(h.state().print_worker.is_none() && h.state().print_success.is_none());
        h.state_mut().error = None;
    }
    assert!(!h.state().is_modified());
}

#[test]
fn picker_cancel_error_and_selected_destination_dispatch() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let ctx = h.ctx.clone();
    for cancel in [true, false] {
        h.state_mut().queue_print(Destination::ChooseFile);
        h.state_mut().dispatch_print(
            &ctx,
            || {
                if cancel {
                    Ok(None)
                } else {
                    Err("picker failure".into())
                }
            },
            run,
        );
        assert!(h.state().print_worker.is_none() && h.state().print_request.is_none());
        assert_eq!(h.state().error.is_some(), !cancel);
    }
    h.state_mut().error = None;
    let path = dir.path().join("existing.png");
    fs::write(&path, b"old screenshot").unwrap();
    let target = prepare_png_export(&path).unwrap();
    h.state_mut().queue_print(Destination::ChooseFile);
    h.state_mut().dispatch_print(&ctx, || Ok(Some(target)), run);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while h.state().print_worker.is_some() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
        h.step();
    }
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert_eq!(
        h.state().print_success.as_deref(),
        Some("Printed existing.png!")
    );
    assert!(fs::read(path).unwrap().starts_with(b"\x89PNG"));
    assert!(!h.state().is_modified());
}
