//! Polished Map NG owned modal-window lifetime, Apache-2.0.
//! Native hints do not replace application input filtering on Unix.
use crate::window::Window;
use std::sync::Arc;

/// Retains a dialog and its parent until their native relationship is detached.
/// Drop this before dropping the application's last window references. Windows
/// restores the owner's previous enabled state before releasing the child.
/// Create a tree (never a cycle), with at most one modal child for each parent.
pub struct ModalWindow {
    child: Arc<Window>,
    parent: Arc<Window>,
    #[cfg(windows_platform)]
    parent_was_enabled: bool,
}
impl ModalWindow {
    /// Both windows must belong to the same event loop. Application code must
    /// suppress parent input independently of the compositor's modal hints.
    pub fn new(child: Arc<Window>, parent: Arc<Window>) -> Result<Self, &'static str> {
        #[cfg(any(windows_platform, macos_platform, x11_platform, wayland_platform))]
        {
            if child.id() == parent.id() {
                return Err("A dialog cannot own itself");
            }
            child.set_modal_parent(Some(&parent))?;
            #[cfg(windows_platform)]
            let parent_was_enabled = {
                let enabled = parent.window.modal_input_enabled();
                parent.window.set_enable(false);
                enabled
            };
            Ok(Self {
                child,
                parent,
                #[cfg(windows_platform)]
                parent_was_enabled,
            })
        }
        #[cfg(not(any(windows_platform, macos_platform, x11_platform, wayland_platform)))]
        {
            let _ = (child, parent);
            Err("Native modal ownership is unsupported on this platform")
        }
    }
}
impl Drop for ModalWindow {
    fn drop(&mut self) {
        #[cfg(windows_platform)]
        self.parent.window.set_enable(self.parent_was_enabled);
        #[cfg(any(windows_platform, macos_platform, x11_platform, wayland_platform))]
        if let Err(reason) = self.child.set_modal_parent(None) {
            tracing::warn!("Could not detach native dialog: {reason}");
        }
        let _ = (&self.child, &self.parent);
    }
}
