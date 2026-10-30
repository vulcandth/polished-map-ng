//! Protocol integration test. tools/wayland_opacity_probe.py supplies a private
//! Wayland peer; no running desktop, GPU, screenshot or production FFI is needed.
#![cfg(target_os = "linux")]
#![allow(unsafe_code)]

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use wayland_sys::client::wayland_client_handle;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::wayland::{EventLoopBuilderExtWayland, WindowExtWayland},
    window::{Window, WindowId},
};

fn flush(owner: &impl HasDisplayHandle) {
    let RawDisplayHandle::Wayland(display) = owner.display_handle().unwrap().as_raw() else {
        panic!("probe must use its private Wayland connection");
    };
    // SAFETY: the owner keeps this libwayland display alive for the call. The
    // probe is single-threaded, has no concurrent renderer, and never owns it.
    assert!(
        unsafe { (wayland_client_handle().wl_display_flush)(display.display.as_ptr().cast()) } >= 0
    );
}

fn commit(window: &Window) {
    let RawWindowHandle::Wayland(surface) = window.window_handle().unwrap().as_raw() else {
        panic!("probe must use a Wayland surface");
    };
    // SAFETY: live borrowed wl_surface, opcode 6 is the zero-argument commit
    // request in wayland.xml. This substitutes only the renderer's commit; all
    // opacity/frame requests originate in the actual vendored adapter.
    unsafe {
        (wayland_client_handle().wl_proxy_marshal)(surface.surface.as_ptr().cast(), 6);
    }
    flush(window);
}

struct Probe {
    scenario: String,
    window: Option<Window>,
    started: bool,
    lost: bool,
    destroyed: bool,
}

impl ApplicationHandler for Probe {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes().with_title("Isolated opacity probe"))
                .unwrap(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested if !self.started => {
                self.started = true;
                let window = self.window.as_ref().unwrap();
                let result = window.set_whole_window_opacity(0xbfff_ffff);
                match self.scenario.as_str() {
                    "client-frame" => {
                        result.unwrap();
                        commit(window);
                        window.set_whole_window_opacity(u32::MAX).unwrap();
                        commit(window);
                        window.set_whole_window_opacity(0xbfff_ffff).unwrap();
                        commit(window);
                        // Peer now changes to server-owned decorations. Wait
                        // for the adapter's native capability-loss event.
                    }
                    "server-frame" => {
                        assert!(result.unwrap_err().contains("compositor-owned"));
                        self.window.take();
                    }
                    "missing-protocol" => {
                        assert!(result.unwrap_err().contains("alpha-modifier"));
                        self.window.take();
                    }
                    other => panic!("unexpected probe scenario {other}"),
                }
            }
            WindowEvent::WindowOpacityUnavailable { reason } => {
                assert_eq!(self.scenario, "client-frame");
                assert!(!self.lost, "capability loss delivered twice");
                assert!(reason.contains("compositor-owned"));
                self.lost = true;
                commit(self.window.as_ref().unwrap());
                self.window.take();
            }
            WindowEvent::Destroyed => {
                self.destroyed = true;
                // WindowState has now destroyed the native resources; flush
                // them while the event loop still owns the connection.
                flush(event_loop);
                let RawDisplayHandle::Wayland(display) =
                    event_loop.display_handle().unwrap().as_raw()
                else {
                    unreachable!();
                };
                // SAFETY: live borrowed display on the sole dispatch thread,
                // outside libwayland dispatch. This default-queue roundtrip
                // ensures the peer has consumed teardown before we disconnect.
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
}

#[test]
#[ignore = "tools/wayland_opacity_probe.py must supply the isolated protocol peer"]
fn native_wayland_opacity() {
    let scenario = std::env::var("POLISHED_MAP_WAYLAND_OPACITY_PROBE").unwrap();
    let event_loop = EventLoop::builder()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let mut probe = Probe {
        scenario,
        window: None,
        started: false,
        lost: false,
        destroyed: false,
    };
    event_loop.run_app(&mut probe).unwrap();
    assert!(probe.started && probe.destroyed);
    assert_eq!(probe.lost, probe.scenario == "client-frame");
}
