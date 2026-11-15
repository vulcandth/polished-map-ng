//! Whole-window opacity from master Main_Window::apply_transparency (LGPL-3.0-only).
//! macOS uses NSWindow's equivalent because upstream's non-Windows path is X11.
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[cfg(any(windows, target_os = "linux", target_os = "macos"))]
mod native;

/// The two source settings, without an arbitrary-opacity extension.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowOpacity {
    #[default]
    /// Full opacity.
    Opaque,
    /// The supported translucent mode: 75% opacity.
    Transparent,
}

impl WindowOpacity {
    /// Normalized alpha used by floating-point native APIs.
    pub const fn alpha(self) -> f64 {
        match self {
            Self::Opaque => 1.0,
            Self::Transparent => 0.75,
        }
    }

    /// Win32 uses floor(alpha * 255), hence 191 rather than rounded 192.
    pub const fn alpha8(self) -> u8 {
        match self {
            Self::Opaque => u8::MAX,
            Self::Transparent => 191,
        }
    }

    /// X11 CARDINAL / Wayland multiplier, retaining floor(alpha * UINT32_MAX).
    pub const fn alpha32(self) -> u32 {
        match self {
            Self::Opaque => u32::MAX,
            Self::Transparent => 0xbfff_ffff,
        }
    }
}

#[derive(Debug, thiserror::Error)]
/// Failure to apply a window-opacity request; callers retain the last accepted state.
pub enum OpacityError {
    #[error(
        "Wayland opacity must be dispatched through the window's native connection and event queue"
    )]
    /// Use the existing winit Wayland connection rather than this handle-only API.
    WaylandProtocolRequired,
    #[error("Native window handle unavailable: {0}")]
    /// A borrowed window or display handle is unavailable.
    Handle(#[from] raw_window_handle::HandleError),
    #[error("{0}")]
    /// This platform or handle type has no supported opacity implementation.
    Unsupported(&'static str),
    #[error("Could not set window opacity: {0}")]
    /// The native API rejected the request, with a platform diagnostic.
    Native(String),
}

/// Set the owner's whole-window opacity on its UI thread. Handles are borrowed
/// only for this call and never retained. X11 completion means the property was
/// sent; a compositor must honor it. This does not alter client drawing/exports.
/// Wayland returns [`OpacityError::WaylandProtocolRequired`]; the caller must
/// dispatch through the patched winit window on its existing connection/queue.
pub fn set_window_opacity(
    owner: &(impl HasWindowHandle + HasDisplayHandle),
    opacity: WindowOpacity,
) -> Result<(), OpacityError> {
    #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
    {
        native::apply(owner, opacity)
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = (owner, opacity);
        Err(OpacityError::Unsupported(
            "Window opacity is not supported on this platform",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_opacity_encodings_truncate_instead_of_rounding() {
        assert_eq!(WindowOpacity::Transparent.alpha(), 3.0 / 4.0);
        assert_eq!(WindowOpacity::Transparent.alpha8(), (255.0 * 0.75) as u8);
        assert_eq!(
            WindowOpacity::Transparent.alpha32(),
            (4_294_967_295.0 * 0.75) as u32
        );
        assert_eq!(WindowOpacity::Opaque.alpha(), 1.0);
        assert_eq!(WindowOpacity::Opaque.alpha8(), 255);
        assert_eq!(WindowOpacity::Opaque.alpha32(), 4_294_967_295);
    }

    struct MissingHandle;
    impl HasWindowHandle for MissingHandle {
        fn window_handle(
            &self,
        ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
            Err(raw_window_handle::HandleError::Unavailable)
        }
    }
    impl HasDisplayHandle for MissingHandle {
        fn display_handle(
            &self,
        ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
            Err(raw_window_handle::HandleError::Unavailable)
        }
    }

    #[test]
    fn missing_handles_return_without_native_access() {
        assert!(matches!(
            set_window_opacity(&MissingHandle, WindowOpacity::Transparent),
            Err(OpacityError::Handle(_))
        ));
    }
}
