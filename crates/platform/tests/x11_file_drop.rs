//! Real XDND/selection transfers against the patched winit adapter on a private
//! Xvfb server. No desktop automation, GPU, user files or raw FFI is used.
#![cfg(target_os = "linux")]
#![forbid(unsafe_code)]

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    collections::HashSet,
    os::unix::ffi::OsStrExt,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    platform::x11::EventLoopBuilderExtX11,
    window::{Window, WindowId},
};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{self as x, ConnectionExt as _},
    },
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};

const MAX_BYTES: usize = 4 * 1024 * 1024;
const PAYLOAD: &[u8] = b"# ordered paths\r\nfile:///tmp/one%20space.blk\r\nfile://localhost/tmp/%CF%80.blk\nfile:///tmp/%FF-last.blk\n";

#[derive(Clone, Copy)]
struct Target {
    xid: u32,
    id: WindowId,
}

#[derive(Debug)]
enum Signal {
    Files(WindowId, Vec<Vec<u8>>, (f64, f64)),
    Legacy(WindowId, Vec<u8>),
    Cancelled(WindowId),
    Destroyed(WindowId),
}

#[derive(Debug)]
enum Control {
    Close(WindowId),
    Done(bool),
}

struct App {
    windows: Vec<Window>,
    signals: Sender<Signal>,
    receiver: Option<Receiver<Signal>>,
    proxy: EventLoopProxy<Control>,
    worker: Option<JoinHandle<()>>,
    ids: HashSet<u64>,
    success: bool,
}

impl ApplicationHandler<Control> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut targets = Vec::new();
        for x in [-240, 240] {
            let window = event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Private XDND probe")
                        .with_visible(false)
                        .with_position(PhysicalPosition::new(x, 100))
                        .with_inner_size(PhysicalSize::new(320, 240)),
                )
                .unwrap();
            assert_eq!(window.scale_factor(), 2.0);
            let xid = match window.window_handle().unwrap().as_raw() {
                RawWindowHandle::Xlib(raw) => u32::try_from(raw.window).unwrap(),
                RawWindowHandle::Xcb(raw) => raw.window.get(),
                _ => panic!("private probe must use X11"),
            };
            targets.push(Target {
                xid,
                id: window.id(),
            });
            self.windows.push(window);
        }
        let receiver = self.receiver.take().unwrap();
        let proxy = self.proxy.clone();
        self.worker = Some(thread::spawn(move || {
            let result = catch_unwind(AssertUnwindSafe(|| {
                Peer::new(targets, receiver, proxy.clone()).run()
            }));
            proxy.send_event(Control::Done(result.is_ok())).unwrap();
        }));
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Control) {
        match event {
            Control::Close(id) => self.windows.retain(|window| window.id() != id),
            Control::Done(success) => {
                self.success = success;
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let signal = match event {
            WindowEvent::FilesDropped {
                transfer_id,
                paths,
                position,
            } => {
                assert!(self.ids.insert(transfer_id), "completion replayed");
                Signal::Files(
                    id,
                    paths
                        .iter()
                        .map(|p| p.as_os_str().as_bytes().to_vec())
                        .collect(),
                    (position.x, position.y),
                )
            }
            WindowEvent::DroppedFile(path) => {
                Signal::Legacy(id, path.as_os_str().as_bytes().to_vec())
            }
            WindowEvent::HoveredFileCancelled => Signal::Cancelled(id),
            WindowEvent::Destroyed => Signal::Destroyed(id),
            _ => return,
        };
        self.signals.send(signal).unwrap();
    }
}

struct Atoms {
    enter: u32,
    position: u32,
    status: u32,
    drop: u32,
    leave: u32,
    finished: u32,
    selection: u32,
    uri: u32,
    types: u32,
    aware: u32,
    copy: u32,
    incr: u32,
    stamp: u32,
    text: u32,
}

struct Peer {
    conn: RustConnection,
    source: u32,
    other_source: u32,
    root: u32,
    targets: Vec<Target>,
    atoms: Atoms,
    receiver: Receiver<Signal>,
    proxy: EventLoopProxy<Control>,
}

impl Peer {
    fn new(
        targets: Vec<Target>,
        receiver: Receiver<Signal>,
        proxy: EventLoopProxy<Control>,
    ) -> Self {
        let (conn, screen) = x11rb::connect(None).unwrap();
        let root = conn.setup().roots[screen].root;
        let atom = |name: &[u8]| conn.intern_atom(false, name).unwrap().reply().unwrap().atom;
        let atoms = Atoms {
            enter: atom(b"XdndEnter"),
            position: atom(b"XdndPosition"),
            status: atom(b"XdndStatus"),
            drop: atom(b"XdndDrop"),
            leave: atom(b"XdndLeave"),
            finished: atom(b"XdndFinished"),
            selection: atom(b"XdndSelection"),
            uri: atom(b"text/uri-list"),
            types: atom(b"XdndTypeList"),
            aware: atom(b"XdndAware"),
            copy: atom(b"XdndActionCopy"),
            incr: atom(b"INCR"),
            stamp: atom(b"_NG_PROBE_TIMESTAMP"),
            text: atom(b"text/plain"),
        };
        let mut sources = Vec::new();
        for _ in 0..2 {
            let id = conn.generate_id().unwrap();
            conn.create_window(
                x11rb::COPY_FROM_PARENT as u8,
                id,
                root,
                0,
                0,
                1,
                1,
                0,
                x::WindowClass::INPUT_OUTPUT,
                x11rb::COPY_FROM_PARENT,
                &x::CreateWindowAux::new().event_mask(x::EventMask::PROPERTY_CHANGE),
            )
            .unwrap()
            .check()
            .unwrap();
            sources.push(id);
        }
        for target in &targets {
            // Independent connections can observe creation in different orders.
            let deadline = Instant::now() + Duration::from_secs(5);
            while conn.get_geometry(target.xid).unwrap().reply().is_err() {
                assert!(Instant::now() < deadline, "native window was not created");
                thread::sleep(Duration::from_millis(2));
            }
            loop {
                let reply = conn
                    .get_property(false, target.xid, atoms.aware, x::AtomEnum::ATOM, 0, 16)
                    .unwrap()
                    .reply()
                    .unwrap();
                if let Some(mut values) = reply.value32() {
                    assert_eq!(
                        values.next(),
                        Some(5),
                        "window must advertise XDND version 5"
                    );
                    assert!(values.next().is_none());
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "window never advertised file-drop support"
                );
                thread::sleep(Duration::from_millis(2));
            }
            conn.change_window_attributes(
                target.xid,
                &x::ChangeWindowAttributesAux::new()
                    .event_mask(x::EventMask::PROPERTY_CHANGE | x::EventMask::STRUCTURE_NOTIFY),
            )
            .unwrap()
            .check()
            .unwrap();
        }
        Self {
            conn,
            source: sources[0],
            other_source: sources[1],
            root,
            targets,
            atoms,
            receiver,
            proxy,
        }
    }

    fn event(&self) -> Event {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(event) = self.conn.poll_for_event().unwrap() {
                assert!(
                    !matches!(event, Event::Error(_)),
                    "X server error: {event:?}"
                );
                return event;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for XDND peer response"
            );
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn signal(&self) -> Signal {
        self.receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("native event was not delivered")
    }

    fn message(&self, target: u32, kind: u32, data: [u32; 5]) {
        self.conn
            .send_event(
                false,
                target,
                x::EventMask::NO_EVENT,
                x::ClientMessageEvent::new(32, target, kind, data),
            )
            .unwrap()
            .check()
            .unwrap();
        self.conn.flush().unwrap();
    }

    fn response(&self, kind: u32) -> [u32; 5] {
        loop {
            match self.event() {
                Event::ClientMessage(event) => {
                    assert_eq!(event.window, self.source);
                    assert_eq!(event.type_, kind, "unexpected XDND response");
                    return event.data.as_data32();
                }
                Event::SelectionRequest(_) => panic!("unexpected selection fetch"),
                _ => {}
            }
        }
    }

    fn timestamp(&self) -> u32 {
        self.conn
            .change_property8(
                x::PropMode::REPLACE,
                self.source,
                self.atoms.stamp,
                x::AtomEnum::STRING,
                &[1],
            )
            .unwrap()
            .check()
            .unwrap();
        loop {
            if let Event::PropertyNotify(event) = self.event()
                && event.window == self.source
                && event.atom == self.atoms.stamp
            {
                return event.time;
            }
        }
    }

    fn enter(&self, target: Target, mime: u32, type_list: bool) {
        if type_list {
            self.conn
                .change_property32(
                    x::PropMode::REPLACE,
                    self.source,
                    self.atoms.types,
                    x::AtomEnum::ATOM,
                    &[self.atoms.text, mime],
                )
                .unwrap()
                .check()
                .unwrap();
        }
        self.message(
            target.xid,
            self.atoms.enter,
            [self.source, (5 << 24) | u32::from(type_list), mime, 0, 0],
        );
    }

    fn position(&self, target: Target, source: u32, x: i16, y: i16) {
        let root = self
            .conn
            .translate_coordinates(target.xid, self.root, x, y)
            .unwrap()
            .reply()
            .unwrap();
        assert!(root.same_screen);
        let packed = ((root.dst_x as u16 as u32) << 16) | root.dst_y as u16 as u32;
        self.message(
            target.xid,
            self.atoms.position,
            [source, 0, packed, 0, self.atoms.copy],
        );
    }

    fn status(&self, target: Target, accepted: bool) {
        let response = self.response(self.atoms.status);
        assert_eq!(response[0], target.xid);
        assert_eq!(response[1] & 1, u32::from(accepted));
        assert_eq!(response[4], if accepted { self.atoms.copy } else { 0 });
    }

    fn drop(&self, target: Target, time: u32) {
        self.message(target.xid, self.atoms.drop, [self.source, 0, time, 0, 0]);
    }

    fn selection(&self, target: Target, time: u32) -> x::SelectionRequestEvent {
        loop {
            match self.event() {
                Event::SelectionRequest(event) => {
                    assert_eq!(event.owner, self.source);
                    assert_eq!(event.requestor, target.xid);
                    assert_eq!(event.selection, self.atoms.selection);
                    assert_eq!(event.target, self.atoms.uri);
                    assert_eq!(event.time, time);
                    return event;
                }
                Event::ClientMessage(_) => panic!("drop finished before its selection arrived"),
                _ => {}
            }
        }
    }

    fn notify(&self, request: x::SelectionRequestEvent, property: u32, time: u32, requestor: u32) {
        self.conn
            .send_event(
                false,
                requestor,
                x::EventMask::NO_EVENT,
                x::SelectionNotifyEvent {
                    response_type: x::SELECTION_NOTIFY_EVENT,
                    sequence: 0,
                    time,
                    requestor,
                    selection: request.selection,
                    target: request.target,
                    property,
                },
            )
            .unwrap()
            .check()
            .unwrap();
        self.conn.flush().unwrap();
    }

    fn bytes(&self, request: x::SelectionRequestEvent, bytes: &[u8]) {
        self.conn
            .delete_property(request.requestor, request.property)
            .unwrap()
            .check()
            .unwrap();
        for chunk in bytes.chunks(65536) {
            self.conn
                .change_property8(
                    x::PropMode::APPEND,
                    request.requestor,
                    request.property,
                    self.atoms.uri,
                    chunk,
                )
                .unwrap()
                .check()
                .unwrap();
        }
        self.notify(request, request.property, request.time, request.requestor);
    }

    fn finished(&self, target: Target, accepted: bool) {
        self.check_finished(target, accepted, self.response(self.atoms.finished));
    }

    fn check_finished(&self, target: Target, accepted: bool, response: [u32; 5]) {
        assert_eq!(response[0], target.xid);
        assert_eq!(response[1], u32::from(accepted));
        assert_eq!(response[2], if accepted { self.atoms.copy } else { 0 });
    }

    fn files(&self, target: Target) {
        let expected = vec![
            b"/tmp/one space.blk".to_vec(),
            b"/tmp/\xcf\x80.blk".to_vec(),
            b"/tmp/\xff-last.blk".to_vec(),
        ];
        let Signal::Files(id, paths, position) = self.signal() else {
            panic!("missing complete transfer")
        };
        assert_eq!(id, target.id);
        assert_eq!(paths, expected);
        assert_eq!(
            position,
            (17.0, 29.0),
            "physical coordinates must not be scaled twice or taken from another source/target"
        );
        for path in expected {
            let Signal::Legacy(id, actual) = self.signal() else {
                panic!("missing legacy path event")
            };
            assert_eq!((id, actual), (target.id, path));
        }
    }

    fn barrier(&self, target: Target) {
        // This protocol roundtrip and cancellation pass through the same native
        // event stream as prior notifications. Unexpected late completions must
        // appear before the cancellation, not race a separate user-event queue.
        self.enter(target, self.atoms.uri, false);
        self.position(target, self.source, 17, 29);
        self.status(target, true);
        self.message(target.xid, self.atoms.leave, [self.source, 0, 0, 0, 0]);
        assert!(matches!(self.signal(), Signal::Cancelled(id) if id == target.id));
        assert!(self.receiver.try_recv().is_err());
    }

    fn begin(&self, target: Target, type_list: bool) -> u32 {
        let time = self.timestamp();
        self.conn
            .set_selection_owner(self.source, self.atoms.selection, time)
            .unwrap()
            .check()
            .unwrap();
        assert_eq!(
            self.conn
                .get_selection_owner(self.atoms.selection)
                .unwrap()
                .reply()
                .unwrap()
                .owner,
            self.source
        );
        self.enter(target, self.atoms.uri, type_list);
        self.position(target, self.source, 17, 29);
        self.status(target, true);
        time
    }

    fn incremental(
        &self,
        target: Target,
        request: x::SelectionRequestEvent,
        payload: &[u8],
        accepted: bool,
    ) {
        // A low size hint tests the receiver's cumulative bound, independently
        // of advertised size. Real chunk boundaries split escapes and newlines.
        self.conn
            .change_property32(
                x::PropMode::REPLACE,
                target.xid,
                request.property,
                self.atoms.incr,
                &[0],
            )
            .unwrap()
            .check()
            .unwrap();
        self.notify(request, request.property, request.time, request.requestor);
        let mut offset = 0;
        let mut sent_eof = false;
        loop {
            match self.event() {
                Event::PropertyNotify(event)
                    if event.window == target.xid
                        && event.atom == request.property
                        && event.state == x::Property::DELETE =>
                {
                    if !sent_eof {
                        let end = (offset + if payload.len() > MAX_BYTES { 65536 } else { 13 })
                            .min(payload.len());
                        self.conn
                            .change_property8(
                                x::PropMode::REPLACE,
                                target.xid,
                                request.property,
                                self.atoms.uri,
                                &payload[offset..end],
                            )
                            .unwrap()
                            .check()
                            .unwrap();
                        self.conn.flush().unwrap();
                        sent_eof = offset == payload.len();
                        offset = end;
                    }
                }
                Event::ClientMessage(event) => {
                    assert_eq!(event.type_, self.atoms.finished);
                    self.check_finished(target, accepted, event.data.as_data32());
                    if accepted {
                        assert!(sent_eof);
                    } else {
                        assert!(offset <= MAX_BYTES + 131072);
                    }
                    break;
                }
                _ => {}
            }
        }
        self.conn
            .delete_property(target.xid, request.property)
            .unwrap()
            .check()
            .unwrap();
    }

    fn run(self) {
        let target = self.targets[0];
        for scenario in [
            "inline",
            "type-list",
            "incremental",
            "wrong-source-target",
            "post-drop-position",
            "stale-notify",
            "invalid-uri",
            "wrong-format",
            "oversize",
            "oversized-incr-hint",
            "oversized-incr-stream",
            "refused-selection",
        ] {
            eprintln!("XDND scenario: {scenario}");
            let time = self.begin(target, scenario == "type-list");
            if scenario == "wrong-source-target" {
                self.position(target, self.other_source, 77, 88);
                self.position(self.targets[1], self.source, 99, 111);
            }
            self.drop(target, time);
            let request = self.selection(target, time);
            if scenario == "post-drop-position" {
                self.position(target, self.source, 77, 88);
                self.status(target, false);
            }
            let accepted = matches!(
                scenario,
                "inline"
                    | "type-list"
                    | "incremental"
                    | "wrong-source-target"
                    | "post-drop-position"
                    | "stale-notify"
            );
            match scenario {
                "incremental" => self.incremental(target, request, PAYLOAD, true),
                "oversized-incr-stream" => {
                    self.incremental(target, request, &vec![b'a'; MAX_BYTES + 262144], false)
                }
                _ => {
                    match scenario {
                        "invalid-uri" => self.bytes(
                            request,
                            b"file:///tmp/good.blk\nfile://remote-host/bad.blk\n",
                        ),
                        "oversize" => self.bytes(request, &vec![b'a'; MAX_BYTES + 1]),
                        "wrong-format" | "oversized-incr-hint" => {
                            self.conn
                                .change_property32(
                                    x::PropMode::REPLACE,
                                    target.xid,
                                    request.property,
                                    if scenario == "wrong-format" {
                                        self.atoms.uri
                                    } else {
                                        self.atoms.incr
                                    },
                                    &[MAX_BYTES as u32 + 1],
                                )
                                .unwrap()
                                .check()
                                .unwrap();
                            self.notify(request, request.property, request.time, request.requestor);
                        }
                        "refused-selection" => {
                            self.notify(request, 0, request.time, request.requestor)
                        }
                        _ => {
                            if scenario == "stale-notify" {
                                self.notify(
                                    request,
                                    0,
                                    request.time.wrapping_sub(1),
                                    request.requestor,
                                );
                                self.notify(request, 0, request.time, self.targets[1].xid);
                            }
                            self.bytes(request, PAYLOAD);
                        }
                    }
                    self.finished(target, accepted);
                }
            }
            if accepted {
                self.files(target);
            }
            self.notify(request, request.property, request.time, request.requestor); // stale duplicate
            self.barrier(target);
        }
        eprintln!("XDND scenario: leave-before-drop");
        self.barrier(target);
        eprintln!("XDND scenario: drop-without-position");
        let time = self.timestamp();
        self.enter(target, self.atoms.uri, false);
        self.drop(target, time);
        self.finished(target, false);
        self.barrier(target);
        eprintln!("XDND scenario: unsupported-mime");
        self.enter(target, self.atoms.text, false);
        self.position(target, self.source, 17, 29);
        self.status(target, false);
        self.drop(target, time);
        self.finished(target, false);
        self.barrier(target);
        eprintln!("XDND scenario: new-enter-while-pending");
        let old_time = self.begin(target, false);
        self.drop(target, old_time);
        let old = self.selection(target, old_time);
        let new_time = loop {
            let time = self.timestamp();
            if time != old_time {
                break time;
            }
        };
        self.enter(target, self.atoms.uri, false);
        self.finished(target, false);
        self.position(target, self.source, 17, 29);
        self.status(target, true);
        self.drop(target, new_time);
        let new = self.selection(target, new_time);
        self.notify(old, 0, old.time, old.requestor);
        self.bytes(new, PAYLOAD);
        self.finished(target, true);
        self.files(target);
        self.barrier(target);
        eprintln!("XDND scenario: window-teardown");
        let time = self.begin(target, false);
        self.drop(target, time);
        let _pending = self.selection(target, time);
        self.proxy.send_event(Control::Close(target.id)).unwrap();
        assert!(matches!(self.signal(), Signal::Destroyed(id) if id == target.id));
        let survivor = self.targets[1];
        let time = self.begin(survivor, false);
        self.drop(survivor, time);
        let request = self.selection(survivor, time);
        self.bytes(request, PAYLOAD);
        self.finished(survivor, true);
        self.files(survivor);
        self.barrier(survivor);
        eprintln!("Passed 17 native XDND scenarios");
    }
}

#[test]
#[ignore = "run under private Xvfb with WINIT_X11_SCALE_FACTOR=2; see CI"]
fn native_x11_file_drop() {
    assert_eq!(
        std::env::var("POLISHED_MAP_X11_DROP_PROBE").as_deref(),
        Ok("1"),
        "use the private-server test command"
    );
    let event_loop = EventLoop::<Control>::with_user_event()
        .with_x11()
        .with_any_thread(true)
        .build()
        .unwrap();
    let (signals, receiver) = mpsc::channel();
    let mut app = App {
        windows: Vec::new(),
        signals,
        receiver: Some(receiver),
        proxy: event_loop.create_proxy(),
        worker: None,
        ids: HashSet::new(),
        success: false,
    };
    event_loop.run_app(&mut app).unwrap();
    app.worker.take().unwrap().join().unwrap();
    assert!(app.success, "native X11 protocol assertions failed");
}
