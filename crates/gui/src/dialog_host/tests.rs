use super::*;
use egui::{RawInput, ViewportCommand, ViewportId, ViewportInfo};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Default)]
struct Renderer {
    inputs: HashMap<ViewportId, RawInput>,
    outputs: HashMap<ViewportId, egui::FullOutput>,
    builders: HashMap<ViewportId, egui::ViewportBuilder>,
    scale: f32,
}
fn renderer(ctx: &egui::Context, scale: f32) -> Rc<RefCell<Renderer>> {
    let state = Rc::new(RefCell::new(Renderer {
        scale,
        ..Default::default()
    }));
    let copy = state.clone();
    ctx.set_embed_viewports(false);
    egui::Context::set_immediate_viewport_renderer(move |ctx, mut viewport| {
        let id = viewport.ids.this;
        let (mut input, scale) = {
            let mut state = copy.borrow_mut();
            state.builders.insert(id, viewport.builder.clone());
            (state.inputs.remove(&id).unwrap_or_default(), state.scale)
        };
        input.viewport_id = id;
        input.screen_rect = Some(Rect::from_min_size(
            Pos2::ZERO,
            viewport.builder.inner_size.unwrap(),
        ));
        let close = input
            .viewports
            .get(&id)
            .is_some_and(|v| v.close_requested());
        input.viewports = ctx.input(|i| i.raw.viewports.clone());
        input.viewports.insert(
            id,
            ViewportInfo {
                parent: Some(viewport.ids.parent),
                inner_rect: Some(Rect::from_min_size(
                    Pos2::new(110.0, 130.0),
                    viewport.builder.inner_size.unwrap(),
                )),
                outer_rect: Some(Rect::from_min_size(
                    Pos2::new(100.0, 100.0),
                    viewport.builder.inner_size.unwrap() + Vec2::new(20.0, 40.0),
                )),
                native_pixels_per_point: Some(scale),
                events: if close {
                    vec![egui::ViewportEvent::Close]
                } else {
                    vec![]
                },
                ..Default::default()
            },
        );
        egui_winit::modal_windows::filter_input(ctx, &mut input);
        let mut output = ctx.run_ui(input, |ui| (viewport.viewport_ui_cb)(ui));
        output.textures_delta.clear();
        copy.borrow_mut()
            .outputs
            .entry(id)
            .or_default()
            .append(output);
    });
    state
}
fn root_input() -> RawInput {
    let mut input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0))),
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&ViewportId::ROOT)
        .unwrap()
        .inner_rect = Some(Rect::from_min_size(
        Pos2::new(-800.0, 100.0),
        Vec2::new(800.0, 600.0),
    ));
    input
}
fn frame(ctx: &egui::Context, editor: &mut Editor) -> egui::FullOutput {
    let mut output = ctx.run_ui(root_input(), |ui| editor.show(ui));
    output.textures_delta.clear();
    output
}
fn key(key: egui::Key) -> RawInput {
    RawInput {
        events: vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    }
}
fn close(id: ViewportId) -> RawInput {
    let mut input = RawInput::default();
    input.viewports.insert(
        id,
        ViewportInfo {
            events: vec![egui::ViewportEvent::Close],
            ..Default::default()
        },
    );
    input
}

#[test]
fn native_startup_dialog_waits_for_the_parent_first_frame() {
    let ctx = egui::Context::default();
    let renderer = renderer(&ctx, 1.0);
    let id = main_id("startup");
    let mut calls = 0;
    let mut first = ctx.run_ui(root_input(), |ui| {
        show(ui.ctx(), id, |_| calls += 1);
    });
    assert_eq!(calls, 0);
    assert!(!first.viewport_output.contains_key(&id));
    assert!(renderer.borrow().builders.is_empty());
    first.textures_delta.clear();
    let mut second = ctx.run_ui(root_input(), |ui| {
        show(ui.ctx(), id, |_| calls += 1);
    });
    second.textures_delta.clear();
    assert_eq!(calls, 1);
    assert!(renderer.borrow().builders.contains_key(&id));
}

#[test]
fn native_resize_has_exact_client_and_close_cancels_without_changing_document() {
    for scale in [1.0, 2.0] {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let ctx = h.ctx.clone();
        let mut editor = std::mem::take(h.state_mut());
        let renderer = renderer(&ctx, scale);
        editor.prepare_resize();
        editor.resize_form.as_mut().unwrap().width = 17;
        let id = main_id("resize");
        let root_output = frame(&ctx, &mut editor);
        assert_eq!(
            editor.resize_form.as_ref().unwrap().bounds,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(220.0, 132.0))
        );
        let state = renderer.borrow();
        let output = &root_output.viewport_output[&id];
        assert!(
            output
                .commands
                .contains(&ViewportCommand::InnerSize(Vec2::new(220.0, 132.0))),
            "commands={:?}, builder={:?}",
            output.commands,
            output.builder
        );
        assert!(
            output
                .commands
                .contains(&ViewportCommand::OuterPosition(Pos2::new(-520.0, 304.0)))
        );
        assert!(
            output
                .commands
                .contains(&ViewportCommand::Title("Resize Map".into()))
        );
        assert_eq!(state.builders[&id].resizable, Some(false));
        drop(state);
        assert!(egui_winit::modal_windows::is_blocked(
            &ctx,
            ViewportId::ROOT
        ));
        renderer.borrow_mut().inputs.insert(id, close(id));
        frame(&ctx, &mut editor);
        assert!(editor.resize_form.is_none());
        assert!(!editor.is_modified());
        assert_eq!(editor.loaded.as_ref().unwrap().document.map().width(), 2);
        assert!(!egui_winit::modal_windows::is_blocked(
            &ctx,
            ViewportId::ROOT
        ));
    }
}

#[test]
fn nested_error_keeps_draft_and_suppresses_parent_keyboard_and_close() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let ctx = h.ctx.clone();
    let mut editor = std::mem::take(h.state_mut());
    let renderer = renderer(&ctx, 1.0);
    editor.prepare_resize();
    editor.resize_form.as_mut().unwrap().width = 17;
    editor.error = Some("Retain this draft".into());
    let parent = main_id("resize");
    renderer
        .borrow_mut()
        .inputs
        .insert(parent, key(egui::Key::Enter));
    frame(&ctx, &mut editor);
    assert!(renderer.borrow().outputs.contains_key(&parent));
    // The nested native error also waits for its parent viewport's first frame.
    assert!(editor.error.is_some());
    frame(&ctx, &mut editor);
    assert!(renderer.borrow().outputs.contains_key(&error_id()));
    assert!(egui_winit::modal_windows::is_blocked(&ctx, parent));
    assert_eq!(editor.resize_form.as_ref().unwrap().width, 17);
    renderer.borrow_mut().inputs.insert(parent, close(parent));
    frame(&ctx, &mut editor);
    assert_eq!(editor.resize_form.as_ref().unwrap().width, 17);
    renderer
        .borrow_mut()
        .inputs
        .insert(error_id(), close(error_id()));
    frame(&ctx, &mut editor);
    assert!(editor.error.is_none());
    assert_eq!(editor.resize_form.as_ref().unwrap().width, 17);
    assert!(!egui_winit::modal_windows::is_blocked(&ctx, parent));
    renderer
        .borrow_mut()
        .inputs
        .insert(parent, key(egui::Key::Enter));
    frame(&ctx, &mut editor);
    assert!(editor.resize_form.is_none());
    assert_eq!(editor.loaded.as_ref().unwrap().document.map().width(), 17);
}

#[test]
fn root_close_cannot_discard_a_clean_document_with_an_uncommitted_modal_draft() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let ctx = h.ctx.clone();
    let mut editor = std::mem::take(h.state_mut());
    let _renderer = renderer(&ctx, 1.0);
    editor.prepare_resize();
    editor.resize_form.as_mut().unwrap().width = 17;
    let mut input = root_input();
    input
        .viewports
        .get_mut(&ViewportId::ROOT)
        .unwrap()
        .events
        .push(egui::ViewportEvent::Close);
    let mut output = ctx.run_ui(input, |ui| editor.show(ui));
    output.textures_delta.clear();
    assert!(
        output.viewport_output[&ViewportId::ROOT]
            .commands
            .contains(&ViewportCommand::CancelClose)
    );
    assert_eq!(editor.resize_form.as_ref().unwrap().width, 17);
    assert!(!editor.confirm_discard);
}

#[test]
fn ownership_graph_rejects_cycles_siblings_and_reparenting_and_filters_only_parent_input() {
    let ctx = egui::Context::default();
    let a = main_id("a");
    let b = main_id("b");
    use egui_winit::modal_windows::*;
    set_parent(&ctx, a, ViewportId::ROOT).unwrap();
    set_parent(&ctx, a, ViewportId::ROOT).unwrap();
    assert!(set_parent(&ctx, b, ViewportId::ROOT).is_err());
    set_parent(&ctx, b, a).unwrap();
    assert!(set_parent(&ctx, a, b).is_err());
    assert!(set_parent(&ctx, ViewportId::ROOT, b).is_err());
    let mut input = key(egui::Key::Enter);
    input.viewport_id = a;
    filter_input(&ctx, &mut input);
    assert!(
        !input
            .events
            .iter()
            .any(|e| matches!(e, egui::Event::Key { .. }))
    );
    input = key(egui::Key::Enter);
    input.viewport_id = b;
    filter_input(&ctx, &mut input);
    assert!(matches!(input.events[0], egui::Event::Key { .. }));
    detach(&ctx, a);
    assert!(!is_blocked(&ctx, a));
    assert!(!is_blocked(&ctx, ViewportId::ROOT));
}

#[test]
fn native_editor_and_option_families_close_through_their_cancel_paths() {
    for family in [
        "block",
        "tileset",
        "palette",
        "block-count",
        "map-options",
        "about",
    ] {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let ctx = h.ctx.clone();
        let mut editor = std::mem::take(h.state_mut());
        let renderer = renderer(&ctx, 1.0);
        match family {
            "block" => editor.prepare_block(&ctx),
            "tileset" => editor.prepare_tileset(),
            "palette" => editor.prepare_palettes(),
            "block-count" => editor.prepare_block_count(),
            "map-options" => editor.prepare_new_in(dir.path().to_path_buf()),
            "about" => editor.about = Some(Default::default()),
            _ => unreachable!(),
        }
        assert_eq!(editor.dialog_kind(), Some(family));
        let id = main_id(family);
        frame(&ctx, &mut editor);
        assert!(editor.error.is_none(), "{family}: {:?}", editor.error);
        let state = ctx.data(|d| d.get_temp::<State>(state_key(id))).unwrap();
        let size = state.size.unwrap();
        if matches!(family, "block" | "tileset") {
            assert_eq!(size, Vec2::new(466.0, 304.0));
        }
        if family == "block-count" {
            assert_eq!(size, Vec2::new(194.0, 80.0));
        }
        if family == "map-options" {
            assert!(size.y >= 300.0, "native choices need popup space");
        }
        assert!(size.x > 0.0 && size.y > 0.0);
        renderer.borrow_mut().inputs.insert(id, close(id));
        frame(&ctx, &mut editor);
        assert_eq!(editor.dialog_kind(), None, "{family}");
        assert!(!editor.is_modified(), "{family}");
        assert!(!egui_winit::modal_windows::is_blocked(
            &ctx,
            ViewportId::ROOT
        ));
    }
}

#[test]
fn approved_exit_targets_the_root_and_help_close_is_blocked_by_a_modal() {
    let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let ctx = h.ctx.clone();
    let mut editor = std::mem::take(h.state_mut());
    let renderer = renderer(&ctx, 1.0);
    editor.help_window.open();
    editor.confirm_discard = true;
    editor.pending_exit = true;
    let help = ViewportId::from_hash_of("help-window");
    renderer.borrow_mut().inputs.insert(help, close(help));
    let output = frame(&ctx, &mut editor);
    assert!(editor.help_window.open);
    assert!(
        output.viewport_output[&help]
            .commands
            .contains(&ViewportCommand::CancelClose)
    );
    renderer
        .borrow_mut()
        .inputs
        .insert(main_id("discard"), key(egui::Key::Enter));
    let output = frame(&ctx, &mut editor);
    assert!(editor.exit_approved);
    assert!(
        output.viewport_output[&ViewportId::ROOT]
            .commands
            .contains(&ViewportCommand::Close)
    );
    assert!(
        !output.viewport_output[&main_id("discard")]
            .commands
            .contains(&ViewportCommand::Close)
    );
}

#[test]
fn blocked_parent_cannot_retain_a_mouse_press_released_over_its_child() {
    let ctx = egui::Context::default();
    let mut input = root_input();
    input.events.push(egui::Event::PointerButton {
        pos: Pos2::new(10.0, 10.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    let mut output = ctx.run_ui(input, |_| {});
    output.textures_delta.clear();
    assert!(ctx.input(|i| i.pointer.any_down()));
    egui_winit::modal_windows::set_parent(&ctx, main_id("draft"), ViewportId::ROOT).unwrap();
    let mut input = root_input();
    egui_winit::modal_windows::filter_input(&ctx, &mut input);
    let mut output = ctx.run_ui(input, |_| {});
    output.textures_delta.clear();
    assert!(ctx.input(|i| !i.pointer.any_down() && !i.pointer.any_click()));
    egui_winit::modal_windows::detach(&ctx, main_id("draft"));
    let mut output = ctx.run_ui(root_input(), |_| {});
    output.textures_delta.clear();
    assert!(ctx.input(|i| !i.pointer.any_down() && !i.pointer.any_click()));
}

#[test]
fn native_pixel_capture_uses_the_child_layer_at_both_scales() {
    for (plus, roof) in [(false, false), (true, false), (true, true)] {
        for scale in [1.0, 2.0] {
            let (_dir, mut h) = if plus {
                crate::plusplus_session::tests::fixture()
            } else {
                crate::tileset_editor_tests::fixture(false, false)
            };
            let ctx = h.ctx.clone();
            let mut editor = std::mem::take(h.state_mut());
            let renderer = renderer(&ctx, scale);
            if roof {
                editor.prepare_roof();
            } else {
                editor.prepare_tileset();
            }
            let id = main_id(editor.dialog_kind().unwrap());
            frame(&ctx, &mut editor);
            frame(&ctx, &mut editor);
            renderer
                .borrow_mut()
                .inputs
                .insert(id, key(egui::Key::Num4));
            frame(&ctx, &mut editor);
            let form = editor.tileset_form.as_ref().unwrap();
            let selected = form.selected;
            let mut expected = form.draft.pixels(selected);
            let pixel_size = if plus { 17.0 } else { 18.0 };
            let tile_x = if roof {
                70.0
            } else if plus {
                293.0
            } else {
                278.0
            };
            let start = Pos2::new(tile_x + 1.0 + 2.5 * pixel_size, 37.0 + 2.5 * pixel_size);
            let outside = Pos2::new(700.0, start.y);
            renderer.borrow_mut().inputs.insert(
                id,
                RawInput {
                    events: vec![
                        egui::Event::PointerMoved(start),
                        egui::Event::PointerButton {
                            pos: start,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        },
                        egui::Event::PointerMoved(outside),
                        egui::Event::PointerButton {
                            pos: outside,
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    ..Default::default()
                },
            );
            frame(&ctx, &mut editor);
            expected[18] = 3;
            assert_eq!(
                editor.tileset_form.as_ref().unwrap().draft.pixels(selected),
                expected,
                "native child: plus={plus} roof={roof} scale={scale}"
            );
            renderer.borrow_mut().inputs.insert(id, close(id));
            frame(&ctx, &mut editor);
            assert!(editor.tileset_form.is_none());
            assert!(!editor.is_modified());
        }
    }
}

#[test]
fn native_client_slider_and_middle_release_use_registered_hit_layer() {
    let ctx = egui::Context::default();
    let renderer = renderer(&ctx, 1.0);
    let id = main_id("slider-probe");
    let mut value = 7u8;
    let mut reset = default_value::DefaultValue::slider(value);
    let mut slider = Rect::NOTHING;
    let mut run = |input: Option<RawInput>| {
        if let Some(input) = input {
            renderer.borrow_mut().inputs.insert(id, input);
        }
        let mut output = ctx.run_ui(root_input(), |ui| {
            show(ui.ctx(), id, |child| {
                client(child, egui::Id::unique("slider-probe-client"), |ui| {
                    ui.set_min_size(Vec2::new(200.0, 100.0));
                    let response = ui.add(nice_slider::NiceSlider(&mut value));
                    slider = response.rect;
                    reset.restore(ui, &response, &mut value);
                });
            });
        });
        output.textures_delta.clear();
        (value, slider)
    };
    run(None);
    run(None);
    let (_, rect) = run(None);
    let point = Pos2::new(rect.right() - 1.0, rect.center().y);
    let input = |button| RawInput {
        events: vec![
            egui::Event::PointerMoved(point),
            egui::Event::PointerButton {
                pos: point,
                button,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos: point,
                button,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        ..Default::default()
    };
    assert!(run(Some(input(egui::PointerButton::Primary))).0 > 7);
    assert_eq!(run(Some(input(egui::PointerButton::Middle))).0, 7);
}
