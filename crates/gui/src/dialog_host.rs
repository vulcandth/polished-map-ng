//! Native hosting for the pinned master's Modal_Dialog/Option_Dialog and editor
//! windows. Client geometry stays independent of native decorations. LGPL-3.0-only.
use super::*;

#[derive(Clone, Default)]
struct State {
    size: Option<Vec2>,
    positioned: bool,
    title: String,
    client: Option<egui::Id>,
}
fn state_key(id: egui::ViewportId) -> egui::Id {
    egui::Id::unique(("native-dialog-state", id))
}
fn native(ctx: &egui::Context) -> bool {
    let id = ctx.viewport_id();
    !ctx.embed_viewports()
        && ctx.data(|d| {
            d.get_temp::<bool>(egui::Id::unique(("native-dialog-client", id)))
                .unwrap_or(false)
        })
}
pub(super) fn main_id(kind: &str) -> egui::ViewportId {
    egui::ViewportId::from_hash_of(("polished-map-dialog", kind))
}
fn error_id() -> egui::ViewportId {
    main_id("nested-error")
}

/// Release a previous native owner before opening another primary dialog.
pub(super) fn active(ctx: &egui::Context, kind: Option<&str>) {
    let key = egui::Id::unique("active-native-dialog");
    let next = kind.map(main_id);
    let previous = ctx.data_mut(|d| {
        let old = d.get_temp::<Option<egui::ViewportId>>(key).flatten();
        d.insert_temp(key, next);
        old
    });
    if previous != next
        && let Some(id) = previous
    {
        egui_winit::modal_windows::detach(ctx, id);
        ctx.data_mut(|d| {
            d.remove::<State>(state_key(id));
        });
        if next.is_none() {
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
        }
    }
}

pub(super) fn block_input(ctx: &egui::Context) {
    ctx.input_mut(|i| {
        i.events.clear();
        i.keys_down.clear();
        i.pointer = Default::default();
        i.modifiers = egui::Modifiers::NONE;
    });
    ctx.stop_dragging();
}

fn centered(parent: Rect, size: Vec2, decoration_offset: Vec2) -> Pos2 {
    parent.center() - size * 0.5 - decoration_offset
}

/// Keep the entire form operation inside the viewport callback: pre-render
/// clipboard shortcuts and post-render Enter/Escape must read the child input.
pub(super) fn show(
    ctx: &egui::Context,
    id: egui::ViewportId,
    mut content: impl FnMut(&egui::Context),
) {
    let parent = ctx.viewport_id();
    let parent_rect = ctx.input(|i| i.viewport().inner_rect.or(i.viewport().outer_rect));
    let state = ctx
        .data(|d| d.get_temp::<State>(state_key(id)))
        .unwrap_or_default();
    let size = state.size.unwrap_or(Vec2::new(360.0, 200.0));
    let mut builder = egui::ViewportBuilder::default()
        .with_title(if state.title.is_empty() {
            "Polished Map NG"
        } else {
            &state.title
        })
        .with_icon(artwork::application_icon())
        .with_inner_size(size)
        .with_resizable(false)
        .with_window_type(egui::X11WindowType::Dialog)
        .with_minimize_button(false)
        .with_maximize_button(false)
        .with_taskbar(false)
        .with_visible(state.size.is_some());
    if !state.positioned
        && let Some(parent_rect) = parent_rect
    {
        builder = builder.with_position(centered(parent_rect, size, Vec2::ZERO));
    }
    if let Err(reason) = egui_winit::modal_windows::set_parent(ctx, id, parent) {
        // Internal relationship errors must be visible, never silently degrade
        // ownership. They do not authorize discarding the retained draft.
        log_native_error(ctx, reason);
    }
    ctx.show_viewport_immediate(id, builder, |ui, _| {
        let child_ctx = ui.ctx();
        child_ctx.data_mut(|d| d.insert_temp(egui::Id::unique(("native-dialog-client", id)), true));
        let blocked = egui_winit::modal_windows::is_blocked(child_ctx, id);
        if blocked {
            block_input(child_ctx);
        }
        if child_ctx.input(|i| i.viewport().close_requested()) {
            child_ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if !blocked {
                // All source forms route unhandled Escape through Cancel.
                // Close cancels even when a popup would normally consume Esc.
                egui::Popup::close_all(child_ctx);
                child_ctx.input_mut(|i| {
                    i.events.push(egui::Event::Key {
                        key: egui::Key::Escape,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    })
                });
            }
        }
        content(child_ctx);
        let measured = child_ctx
            .data(|d| d.get_temp::<State>(state_key(id)))
            .unwrap_or_default();
        if let Some(size) = measured.size {
            if child_ctx.content_rect().size() != size {
                child_ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
            if !measured.positioned {
                if let Some(parent_rect) = parent_rect {
                    let offset = child_ctx.input(|i| {
                        let v = i.viewport();
                        v.inner_rect
                            .zip(v.outer_rect)
                            .map_or(Vec2::ZERO, |(inner, outer)| inner.min - outer.min)
                    });
                    child_ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(centered(
                        parent_rect,
                        size,
                        offset,
                    )));
                }
                child_ctx.data_mut(|d| {
                    d.get_temp_mut_or_default::<State>(state_key(id)).positioned = true
                });
                child_ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                child_ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
    });
}
fn log_native_error(ctx: &egui::Context, reason: &'static str) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("native-dialog-error"), reason));
}
pub(super) fn take_error(ctx: &egui::Context) -> Option<&'static str> {
    ctx.data_mut(|d| d.remove_temp::<&'static str>(egui::Id::unique("native-dialog-error")))
        .or_else(|| egui_winit::modal_windows::take_error(ctx))
}

pub(super) fn error_parent(ctx: &egui::Context) {
    if let Err(reason) = egui_winit::modal_windows::set_parent(ctx, error_id(), ctx.viewport_id()) {
        log_native_error(ctx, reason);
    }
    block_input(ctx);
}
pub(super) fn error(ctx: &egui::Context, message: &str) -> bool {
    let mut done = false;
    show(ctx, error_id(), |ctx| {
        done = message_dialog::show(ctx, "error", message_dialog::Kind::Error, message);
    });
    if done {
        clear_error(ctx);
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }
    done
}
pub(super) fn clear_error(ctx: &egui::Context) {
    egui_winit::modal_windows::detach(ctx, error_id());
    ctx.data_mut(|d| {
        d.remove::<State>(state_key(error_id()));
    });
}

pub(super) struct ClientResponse {
    #[allow(dead_code)] // inspected by geometry tests
    pub response: egui::Response,
    pub any_popup_open: bool,
}
pub(super) fn client(
    ctx: &egui::Context,
    id: egui::Id,
    content: impl FnOnce(&mut egui::Ui),
) -> ClientResponse {
    if !native(ctx) {
        let response = egui::Modal::new(id).show(ctx, content);
        return ClientResponse {
            response: response.response,
            any_popup_open: response.any_popup_open,
        };
    }
    let any_popup_open = egui::Popup::is_any_open(ctx);
    let mut ui = egui::Ui::new(
        ctx.clone(),
        id,
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::new(egui::Order::Middle, id))
            .max_rect(ctx.content_rect()),
    );
    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, ui.visuals().panel_fill);
    if egui_winit::modal_windows::is_blocked(ctx, ctx.viewport_id()) {
        ui.disable();
    }
    let key = state_key(ctx.viewport_id());
    ctx.data_mut(|d| {
        let state = d.get_temp_mut_or_default::<State>(key);
        if state.client != Some(id) {
            state.positioned = false;
            state.client = Some(id);
        }
    });
    content(&mut ui);
    let size = ui.min_rect().size().ceil().max(Vec2::splat(1.0));
    ctx.data_mut(|d| d.get_temp_mut_or_default::<State>(key).size = Some(size));
    ClientResponse {
        response: ui.response(),
        any_popup_open,
    }
}
/// The old caption remains only for embedded/headless modal clients.
pub(super) fn heading(ui: &mut egui::Ui, title: &str) {
    if native(ui.ctx()) {
        let key = state_key(ui.ctx().viewport_id());
        ui.ctx()
            .data_mut(|d| d.get_temp_mut_or_default::<State>(key).title = title.to_owned());
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Title(title.to_owned()));
    } else {
        ui.heading(title);
    }
}

#[cfg(test)]
mod tests;
