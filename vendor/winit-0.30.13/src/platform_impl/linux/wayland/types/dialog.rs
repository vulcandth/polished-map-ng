//! Polished Map NG transient/modal hints, Apache-2.0.
use crate::platform_impl::wayland::state::WinitState;
use sctk::reexports::client::{delegate_noop, globals::GlobalList, QueueHandle};
pub(crate) use wayland_protocols::xdg::dialog::v1::client::{
    xdg_dialog_v1::XdgDialogV1, xdg_wm_dialog_v1::XdgWmDialogV1,
};
pub(crate) fn bind(globals: &GlobalList, queue: &QueueHandle<WinitState>) -> Option<XdgWmDialogV1> {
    globals.bind(queue, 1..=1, ()).ok()
}
pub(crate) struct Dialog(pub XdgDialogV1);
impl Drop for Dialog {
    fn drop(&mut self) {
        self.0.destroy();
    }
}
delegate_noop!(WinitState: ignore XdgWmDialogV1);
delegate_noop!(WinitState: ignore XdgDialogV1);
