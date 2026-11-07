//! Real keyboard routing through client decorations against a private Wayland peer.
#![cfg(target_os = "linux")]
#![allow(unsafe_code)]

use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use wayland_sys::client::wayland_client_handle;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    platform::wayland::EventLoopBuilderExtWayland,
    window::{Window, WindowId},
};

#[derive(Default)]
struct Probe {
    window: Option<Window>,
    focus: Vec<bool>,
    keys: usize,
    close_requested: bool,
    destroyed: bool,
}

impl ApplicationHandler for Probe {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        window.set_title("ready");
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(window) = &self.window {
            assert_eq!(window.id(), id, "frame input must target its owning window");
        }
        match event {
            WindowEvent::Focused(value) => self.focus.push(value),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                assert_eq!(event.logical_key, Key::Named(NamedKey::F11));
                assert_eq!(self.focus.last(), Some(&true));
                self.keys += 1;
                self.window
                    .as_ref()
                    .unwrap()
                    .set_title(&format!("key-{}", self.keys));
            }
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::Destroyed => {
                self.destroyed = true;
                let RawDisplayHandle::Wayland(display) =
                    event_loop.display_handle().unwrap().as_raw()
                else {
                    panic!("probe must use its private Wayland connection");
                };
                // SAFETY: the event loop owns this display on the sole dispatch
                // thread. Drain native resource teardown before exiting.
                assert!(
                    unsafe {
                        (wayland_client_handle().wl_display_roundtrip)(
                            display.display.as_ptr().cast(),
                        )
                    } >= 0
                );
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        // Winit queues focus events separately from the close request. Verify
        // after the complete native batch has been dispatched, not mid-batch.
        if self.close_requested && self.window.is_some() {
            assert_eq!(self.keys, 7, "root, all five frame parts, and root again");
            assert_eq!(self.focus, [true, false].repeat(7));
            assert!(!self.window.as_ref().unwrap().has_focus());
            self.window.take();
        }
    }
}

#[test]
#[ignore = "tools/wayland_frame_focus_probe.py supplies the isolated peer"]
fn native_wayland_frame_focus() {
    assert_eq!(
        std::env::var("POLISHED_MAP_WAYLAND_FRAME_FOCUS_PROBE").unwrap(),
        "frame-focus"
    );
    let event_loop = EventLoop::builder()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let mut probe = Probe::default();
    event_loop.run_app(&mut probe).unwrap();
    assert!(probe.destroyed);
}
