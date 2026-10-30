//! Real native adapter exercised by tools/wayland_file_drop_probe.py on a private
//! protocol connection. No display server, user files or GPU are involved.
#![cfg(target_os = "linux")]
#![allow(unsafe_code)]

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::{
    collections::{HashSet, VecDeque},
    os::unix::ffi::OsStrExt,
};
use wayland_sys::client::wayland_client_handle;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::wayland::EventLoopBuilderExtWayland,
    window::{Window, WindowId},
};

fn commit(window: &Window) {
    let RawWindowHandle::Wayland(surface) = window.window_handle().unwrap().as_raw() else {
        unreachable!()
    };
    let RawDisplayHandle::Wayland(display) = window.display_handle().unwrap().as_raw() else {
        unreachable!()
    };
    // SAFETY: live borrowed native objects on the sole dispatch thread. Commit
    // is wl_surface's zero-argument opcode 6; no renderer runs in this probe.
    unsafe {
        (wayland_client_handle().wl_proxy_marshal)(surface.surface.as_ptr().cast(), 6);
        assert!((wayland_client_handle().wl_display_flush)(display.display.as_ptr().cast()) >= 0);
    }
}

#[derive(Default)]
struct Probe {
    scenario: String,
    window: Option<Window>,
    ready: bool,
    responsive: bool,
    destroyed: bool,
    ids: HashSet<u64>,
    received: Vec<Vec<Vec<u8>>>,
    legacy: VecDeque<Vec<u8>>,
}

impl ApplicationHandler for Probe {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes().with_title("Private drop probe"))
                .unwrap(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested if !self.ready => {
                let window = self.window.as_ref().unwrap();
                assert_eq!(window.scale_factor(), 2.0);
                self.ready = true;
                window.set_title("drop-probe-ready");
                commit(window);
            }
            WindowEvent::Resized(_) if self.ready && !self.responsive => {
                self.responsive = true;
                let window = self.window.as_ref().unwrap();
                window.set_title("drop-probe-responsive");
                commit(window);
            }
            WindowEvent::FilesDropped {
                transfer_id,
                paths,
                position,
            } => {
                assert!(self.ids.insert(transfer_id), "completion repeated");
                assert!(
                    self.legacy.is_empty(),
                    "legacy events must immediately follow their complete event"
                );
                let paths: Vec<Vec<u8>> = paths
                    .iter()
                    .map(|p| p.as_os_str().as_bytes().to_vec())
                    .collect();
                let second = paths.first().is_some_and(|p| p == b"/tmp/second.blk");
                let expected = if second { (82.5, 111.0) } else { (49.0, 73.5) };
                assert_eq!(
                    (position.x, position.y),
                    expected,
                    "coordinates must belong to this drop, scaled once"
                );
                self.legacy.extend(paths.clone());
                self.received.push(paths);
            }
            WindowEvent::DroppedFile(path) => {
                assert_eq!(
                    Some(path.as_os_str().as_bytes().to_vec()),
                    self.legacy.pop_front()
                );
            }
            WindowEvent::CloseRequested => {
                self.window.take();
            }
            WindowEvent::Destroyed => {
                self.destroyed = true;
                let RawDisplayHandle::Wayland(display) =
                    event_loop.display_handle().unwrap().as_raw()
                else {
                    unreachable!()
                };
                // SAFETY: the event loop still owns this connection. A default
                // queue roundtrip acknowledges teardown before disconnecting;
                // this callback runs outside libwayland's event dispatch.
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
#[ignore = "tools/wayland_file_drop_probe.py must supply the isolated protocol peer"]
fn native_wayland_file_drop() {
    let scenario = std::env::var("POLISHED_MAP_WAYLAND_DROP_PROBE").unwrap();
    let event_loop = EventLoop::builder()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let mut probe = Probe {
        scenario,
        ..Default::default()
    };
    event_loop.run_app(&mut probe).unwrap();
    assert!(probe.ready && probe.destroyed);
    assert!(probe.legacy.is_empty());
    let expected = match probe.scenario.as_str() {
        "ordered" | "legacy" => vec![vec![
            b"/tmp/one space.blk".to_vec(),
            b"/tmp/\xcf\x80.blk".to_vec(),
            b"/tmp/\xff-last.blk".to_vec(),
        ]],
        // The peer finishes the second transfer first, proving independent
        // lifetimes and that motion on a later drag cannot change the first.
        "overlap" => vec![
            vec![b"/tmp/second.blk".to_vec()],
            vec![b"/tmp/first.blk".to_vec()],
        ],
        _ => Vec::new(),
    };
    assert_eq!(probe.received, expected);
    if matches!(
        probe.scenario.as_str(),
        "ordered"
            | "legacy"
            | "overlap"
            | "invalid"
            | "oversize"
            | "timeout"
            | "seat-removed"
            | "window-closed"
            | "queue-limit"
    ) {
        assert!(
            probe.responsive,
            "event loop stalled while receiving a partial pipe"
        );
    }
}
