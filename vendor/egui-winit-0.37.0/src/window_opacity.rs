//! Polished Map NG native output bridge (MIT OR Apache-2.0).
//! Only a successful native dispatch is an accepted request. Rendering and
//! compositor behavior still require separate native verification.
#[derive(Clone, Default)]
struct State {
    next_id: u64,
    active: Option<u64>,
    queued: Option<(u64, u32)>,
    reply: Option<(u64, Result<(), &'static str>)>,
    unavailable: Option<&'static str>,
}

fn key(viewport: egui::ViewportId) -> egui::Id {
    egui::Id::unique(("polished-map-native-window-opacity-v1", viewport))
}

/// Queue one native request on the UI thread. Do not overwrite a pending reply.
pub fn request(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    multiplier: u32,
) -> Result<u64, &'static str> {
    let id = ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<State>(key(viewport));
        if state.active.is_some() {
            return Err("A native opacity request is already pending");
        }
        state.next_id = state.next_id.wrapping_add(1);
        let id = state.next_id;
        state.active = Some(id);
        state.queued = Some((id, multiplier));
        Ok(id)
    })?;
    ctx.request_repaint_of(viewport);
    Ok(id)
}

/// Dispatch the queued output once, outside Context's data lock. Exposed for
/// headless integration tests; real dispatch receives the owned winit Window.
pub fn dispatch_with(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    apply: impl FnOnce(u32) -> Result<(), &'static str>,
) {
    let queued = ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<State>(key(viewport))
            .queued
            .take()
    });
    if let Some((id, multiplier)) = queued {
        let result = apply(multiplier);
        ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<State>(key(viewport)).reply = Some((id, result))
        });
        ctx.request_repaint_of(viewport);
    }
}

/// Take only the reply for this caller's request and viewport.
pub fn take_reply(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    id: u64,
) -> Option<Result<(), &'static str>> {
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

/// Record a native frame transition that restored opacity after capability loss.
pub fn unavailable(ctx: &egui::Context, viewport: egui::ViewportId, reason: &'static str) {
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<State>(key(viewport))
            .unavailable = Some(reason)
    });
    ctx.request_repaint_of(viewport);
}

pub fn take_unavailable(ctx: &egui::Context, viewport: egui::ViewportId) -> Option<&'static str> {
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<State>(key(viewport))
            .unavailable
            .take()
    })
}
