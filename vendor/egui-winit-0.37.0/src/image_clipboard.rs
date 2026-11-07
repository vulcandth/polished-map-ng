//! Polished Map NG native image-copy acknowledgment (MIT OR Apache-2.0).
//! A rendered image is not proof that the OS clipboard accepted it.

#[derive(Clone, Default)]
struct State {
    next_id: u64,
    active: Option<u64>,
    queued: Option<(u64, Request)>,
    reply: Option<(u64, Result<Option<egui::ColorImage>, String>)>,
}

#[derive(Clone)]
pub(crate) enum Request {
    Copy(egui::ColorImage),
    Paste,
}

fn key(viewport: egui::ViewportId) -> egui::Id {
    egui::Id::unique(("polished-map-native-image-copy-v1", viewport))
}

/// Queue one image on the owning viewport. A pending result cannot be replaced.
pub fn request(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    image: egui::ColorImage,
) -> Result<u64, String> {
    enqueue(ctx, viewport, Request::Copy(image))
}

/// Request an image using the owning native connection, without blocking input.
pub fn request_paste(ctx: &egui::Context, viewport: egui::ViewportId) -> Result<u64, String> {
    enqueue(ctx, viewport, Request::Paste)
}

fn enqueue(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    request: Request,
) -> Result<u64, String> {
    let id = ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<State>(key(viewport));
        if state.active.is_some() {
            return Err("An image clipboard request is already pending".to_owned());
        }
        state.next_id = state.next_id.wrapping_add(1);
        let id = state.next_id;
        state.active = Some(id);
        state.queued = Some((id, request));
        Ok(id)
    })?;
    ctx.request_repaint_of(viewport);
    Ok(id)
}

/// Dispatch outside Context's lock. Tests inject an OS-independent adapter;
/// production calls the same clipboard owned by egui-winit's event loop.
pub fn dispatch_with(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    copy: impl FnOnce(&egui::ColorImage) -> Result<(), String>,
) {
    if let Some((id, request)) = take_request(ctx, viewport) {
        let result = match request {
            Request::Copy(image) => copy(&image).map(|()| None),
            Request::Paste => Err("Copy-only test dispatcher received a paste".into()),
        };
        finish(ctx, viewport, id, result);
    }
}

/// Paste counterpart for deterministic native-adapter tests.
pub fn dispatch_paste_with(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    paste: impl FnOnce() -> Result<Option<egui::ColorImage>, String>,
) {
    if let Some((id, request)) = take_request(ctx, viewport) {
        let result = match request {
            Request::Paste => paste(),
            Request::Copy(_) => Err("Paste-only test dispatcher received a copy".into()),
        };
        finish(ctx, viewport, id, result);
    }
}

pub(crate) fn take_request(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
) -> Option<(u64, Request)> {
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<State>(key(viewport))
            .queued
            .take()
    })
}

pub(crate) fn finish(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    id: u64,
    result: Result<Option<egui::ColorImage>, String>,
) {
    ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<State>(key(viewport));
        if state.active == Some(id) {
            state.reply = Some((id, result));
        }
    });
    ctx.request_repaint_of(viewport);
}

/// Cancel a closed editor's request. A late reply cannot affect the next editor.
pub fn cancel(ctx: &egui::Context, viewport: egui::ViewportId, id: u64) {
    ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<State>(key(viewport));
        if state.active == Some(id) {
            state.active = None;
            state.queued = None;
            state.reply = None;
        }
    });
}

pub(crate) fn active(ctx: &egui::Context, viewport: egui::ViewportId, id: u64) -> bool {
    ctx.data_mut(|data| data.get_temp_mut_or_default::<State>(key(viewport)).active == Some(id))
}

/// Consume only the matching request's result, once.
pub fn take_reply(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    id: u64,
) -> Option<Result<(), String>> {
    take_paste_reply(ctx, viewport, id).map(|result| result.map(|_| ()))
}

/// Consume an image request reply once. `Ok(None)` means no image was offered.
pub fn take_paste_reply(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    id: u64,
) -> Option<Result<Option<egui::ColorImage>, String>> {
    ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<State>(key(viewport));
        if state
            .reply
            .as_ref()
            .is_some_and(|(reply_id, _)| *reply_id == id)
        {
            state.active = None;
            state.reply.take().map(|(_, result)| result)
        } else {
            None
        }
    })
}
