//! Polished Map NG whole-window opacity extension (Apache-2.0).
//! Resources use the window's connection/queue and die before their wl_surface.
use crate::platform_impl::wayland::state::WinitState;
use sctk::reexports::client::{delegate_noop, globals::GlobalList, QueueHandle};
use wayland_protocols::wp::alpha_modifier::v1::client::{
    wp_alpha_modifier_surface_v1::WpAlphaModifierSurfaceV1, wp_alpha_modifier_v1::WpAlphaModifierV1,
};

pub(crate) fn bind(
    globals: &GlobalList,
    queue: &QueueHandle<WinitState>,
) -> Option<WpAlphaModifierV1> {
    globals.bind(queue, 1..=1, ()).ok()
}

/// The surface owner must drop this before destroying the underlying surface.
pub(crate) struct AlphaSurface(pub WpAlphaModifierSurfaceV1);
impl Drop for AlphaSurface {
    fn drop(&mut self) {
        self.0.destroy();
    }
}

delegate_noop!(WinitState: ignore WpAlphaModifierV1);
delegate_noop!(WinitState: ignore WpAlphaModifierSurfaceV1);
