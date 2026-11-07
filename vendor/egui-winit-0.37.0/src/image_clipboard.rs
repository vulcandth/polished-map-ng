//! Polished Map NG native image-copy acknowledgment (MIT OR Apache-2.0).
//! A rendered image is not proof that the OS clipboard accepted it.

#[derive(Clone, Default)]
struct State {
    next_id: u64,
    active: Option<u64>,
    queued: Option<(u64, egui::ColorImage)>,
    reply: Option<(u64, Result<(), String>)>,
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
    let id = ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<State>(key(viewport));
        if state.active.is_some() {
            return Err("An image clipboard request is already pending".to_owned());
        }
        state.next_id = state.next_id.wrapping_add(1);
        let id = state.next_id;
        state.active = Some(id);
        state.queued = Some((id, image));
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
    let queued = ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<State>(key(viewport))
            .queued
            .take()
    });
    if let Some((id, image)) = queued {
        let result = copy(&image);
        ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<State>(key(viewport)).reply = Some((id, result));
        });
        ctx.request_repaint_of(viewport);
    }
}

/// Consume only the matching request's result, once.
pub fn take_reply(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    id: u64,
) -> Option<Result<(), String>> {
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
