//! Keep restore geometry current even when an app disables periodic saves.
//! Polished Map NG modification, 2026-10-30; MIT OR Apache-2.0.
use winit::event::WindowEvent;

pub(super) fn remember<T>(
    geometry: &mut Option<T>,
    event: &WindowEvent,
    is_root: bool,
    is_normal: bool,
    capture: impl FnOnce() -> Option<T>,
) {
    if is_root
        && is_normal
        && matches!(
            event,
            WindowEvent::Moved(_)
                | WindowEvent::Resized(_)
                | WindowEvent::ScaleFactorChanged { .. }
        )
        && let Some(current) = capture()
    {
        *geometry = Some(current);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::{PhysicalPosition, PhysicalSize};

    #[test]
    fn resize_then_maximize_retains_latest_normal_geometry_without_a_save() {
        let mut normal = Some((48, 800));
        let resized = WindowEvent::Resized(PhysicalSize::new(960, 720));
        remember(&mut normal, &resized, true, true, || Some((48, 960)));
        let moved = WindowEvent::Moved(PhysicalPosition::new(-120, 90));
        remember(&mut normal, &moved, true, true, || Some((-120, 960)));
        // Maximized, fullscreen and minimized events must not overwrite it.
        for _ in 0..3 {
            remember(&mut normal, &resized, true, false, || panic!("capture"));
        }
        assert_eq!(normal, Some((-120, 960)));
        // Restoring and resizing updates it again, independently of storage.
        remember(&mut normal, &resized, true, true, || Some((-120, 1000)));
        assert_eq!(normal, Some((-120, 1000)));
    }

    #[test]
    fn child_windows_unrelated_events_and_invalid_sizes_leave_root_unchanged() {
        let mut normal = Some(800);
        let resized = WindowEvent::Resized(PhysicalSize::new(0, 0));
        remember(&mut normal, &resized, false, true, || panic!("child"));
        remember(&mut normal, &WindowEvent::Focused(true), true, true, || {
            panic!("focus")
        });
        remember(&mut normal, &resized, true, true, || None);
        assert_eq!(normal, Some(800));
    }
}
