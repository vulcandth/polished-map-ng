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
