use super::*;
use sctk::reexports::client::globals::{registry_queue_init, GlobalListContents};
use sctk::reexports::client::protocol::{wl_compositor, wl_registry, wl_surface};
use sctk::reexports::client::{delegate_noop, Connection, Dispatch};
use std::os::unix::net::UnixStream;

#[test]
fn cancellation_and_request_budget_survive_queued_commands() {
    let count = Arc::new(AtomicUsize::new(0));
    let pairs: Vec<_> = (0..MAX_TRANSFERS).map(|_| pair::<()>(&count).unwrap()).collect();
    assert!(pair::<()>(&count).is_err());
    let (transfer, reply) = pairs.into_iter().next().unwrap();
    drop(transfer);
    assert_eq!(reply.invalid().unwrap().kind(), ErrorKind::Interrupted);
    assert_eq!(count.load(Ordering::Acquire), 1);
    drop(reply);
    assert_eq!(count.load(Ordering::Acquire), 0);
}

#[test]
fn bounded_nonblocking_fragmented_binary_transfer() {
    let payload: Vec<_> = (0..QUANTUM * 3 + 23).map(|i| (i % 251) as u8).collect();
    let (mut reader, writer) = UnixStream::pair().unwrap();
    reader.set_nonblocking(true).unwrap();
    writer.set_nonblocking(true).unwrap();
    let mut received = Vec::new();
    let mut offset = 0;
    assert!(read_chunk(&mut reader, &mut received).is_none());
    let mut writer = Some(writer);
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        assert!(Instant::now() < until);
        if let Some(pipe) = &mut writer {
            if let Some(done) = write_chunk(pipe, &payload, &mut offset) {
                done.unwrap();
                writer = None;
            }
        }
        if let Some(done) = read_chunk(&mut reader, &mut received) {
            done.unwrap();
            break;
        }
    }
    assert_eq!(received, payload);
    let mut full = vec![0; MAX_BYTES];
    assert!(read_chunk(&mut &b"x"[..], &mut full).unwrap().is_err());
    assert_eq!(full.len(), MAX_BYTES);
    struct Zero;
    impl Write for Zero {
        fn write(&mut self, _: &[u8]) -> Result<usize> {
            Ok(0)
        }
        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }
    assert_eq!(
        write_chunk(&mut Zero, b"x", &mut 0).unwrap().unwrap_err().kind(),
        ErrorKind::WriteZero
    );
}

#[test]
fn payload_validation_rejects_ambiguous_or_unbounded_offers() {
    assert!(
        validate_payload(&[("image/png".into(), vec![]), ("image/png".into(), vec![])]).is_err()
    );
    assert!(validate_mimes(&["image/png\0text/plain".into()]).is_err());
    assert!(validate_mimes(&vec!["image/png".into(); 17]).is_err());
    assert!(validate_payload(&[("image/png".into(), vec![0; MAX_BYTES + 1])]).is_err());
}

#[derive(Default)]
struct Client {
    offers: Vec<(sctk::reexports::client::protocol::wl_data_offer::WlDataOffer, Vec<String>)>,
    selected: Option<(sctk::reexports::client::protocol::wl_data_offer::WlDataOffer, Vec<String>)>,
}
use sctk::reexports::client::protocol::{
    wl_data_device, wl_data_device_manager, wl_data_offer, wl_seat,
};
delegate_noop!(Client: ignore wl_data_device_manager::WlDataDeviceManager);
delegate_noop!(Client: ignore wl_seat::WlSeat);
impl Dispatch<wl_data_device::WlDataDevice, ()> for Client {
    fn event(
        state: &mut Self,
        _: &wl_data_device::WlDataDevice,
        event: wl_data_device::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_device::Event::DataOffer { id } => state.offers.push((id, vec![])),
            wl_data_device::Event::Selection { id } => {
                state.selected =
                    state.offers.iter().find(|(offer, _)| Some(offer) == id.as_ref()).cloned()
            },
            _ => (),
        }
    }
    sctk::reexports::client::event_created_child!(Client, wl_data_device::WlDataDevice, [0 => (wl_data_offer::WlDataOffer, ())]);
}
impl Dispatch<wl_data_offer::WlDataOffer, ()> for Client {
    fn event(
        state: &mut Self,
        offer: &wl_data_offer::WlDataOffer,
        event: wl_data_offer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_data_offer::Event::Offer { mime_type } = event {
            state.offers.iter_mut().find(|(id, _)| id == offer).unwrap().1.push(mime_type);
        }
    }
}
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Client {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
delegate_noop!(Client: ignore wl_compositor::WlCompositor);
delegate_noop!(Client: ignore wl_surface::WlSurface);

fn wait<T>(transfer: Transfer<T>) -> Result<T> {
    let deadline = Instant::now() + Duration::from_secs(12);
    loop {
        if let Some(result) = transfer.poll() {
            return result;
        }
        assert!(Instant::now() < deadline, "binary request never completed");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
#[ignore = "tools/wayland_clipboard_probe.py supplies an isolated protocol peer"]
fn native_wayland_binary_clipboard() {
    let scenario = std::env::var("POLISHED_MAP_WAYLAND_CLIPBOARD_PROBE").unwrap();
    let connection = Connection::connect_to_env().unwrap();
    let (globals, mut queue) = registry_queue_init::<Client>(&connection).unwrap();
    let compositor: wl_compositor::WlCompositor = globals.bind(&queue.handle(), 1..=4, ()).unwrap();
    let surface = compositor.create_surface(&queue.handle(), ());
    surface.commit();
    connection.flush().unwrap();
    // SAFETY: the original connection outlives this adapter and is not dispatched
    // or destroyed concurrently. The adapter owns a separate queue on that connection.
    let mut clipboard = unsafe { crate::Clipboard::new(connection.backend().display_ptr().cast()) };
    let ready_deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match wait(clipboard.load_bytes(vec!["application/x-ready".into()]).unwrap()) {
            Ok(selection) => {
                assert_eq!(selection.bytes, b"ready");
                break;
            },
            Err(_) => {
                assert!(Instant::now() < ready_deadline, "seat did not receive focus");
                std::thread::sleep(Duration::from_millis(5));
            },
        }
    }
    let mut host = Client::default();
    if scenario == "external-png" {
        let manager: wl_data_device_manager::WlDataDeviceManager =
            globals.bind(&queue.handle(), 1..=3, ()).unwrap();
        let seat: wl_seat::WlSeat = globals.bind(&queue.handle(), 1..=7, ()).unwrap();
        let _device = manager.get_data_device(&seat, &queue.handle(), ());
        queue.roundtrip(&mut host).unwrap();
        clipboard.set_external_selection(Some(host.selected.clone().unwrap()));
    } else if scenario == "external-empty" {
        clipboard.set_external_selection(None);
        assert_eq!(clipboard.load().unwrap_err().kind(), ErrorKind::NotFound);
    }
    if scenario == "copy" || scenario == "source-snapshot" {
        let payload: Vec<_> = (0..700_013).map(|i| (i % 251) as u8).collect();
        wait(
            clipboard
                .store_bytes(vec![
                    ("image/png".into(), payload),
                    ("image/bmp".into(), b"BMP payload".to_vec()),
                ])
                .unwrap(),
        )
        .unwrap();
        if scenario == "source-snapshot" {
            wait(
                clipboard
                    .store_bytes(vec![("image/png".into(), b"new selection".to_vec())])
                    .unwrap(),
            )
            .unwrap();
        }
        // The peer installs this acknowledgment selection only after it has read
        // every advertised representation from the original source.
        loop {
            if let Ok(value) =
                wait(clipboard.load_bytes(vec!["application/x-done".into()]).unwrap())
            {
                assert_eq!(value.bytes, b"done");
                break;
            }
            assert!(Instant::now() < ready_deadline + Duration::from_secs(8));
            std::thread::sleep(Duration::from_millis(5));
        }
    } else if scenario == "cancel" {
        let transfer = clipboard.load_bytes(vec!["image/png".into()]).unwrap();
        std::thread::sleep(Duration::from_millis(30));
        drop(transfer);
        std::thread::sleep(Duration::from_millis(30));
    } else {
        let transfer = clipboard.load_bytes(vec!["image/png".into(), "image/bmp".into()]).unwrap();
        let result = wait(transfer);
        match scenario.as_str() {
            "png" | "bmp" | "external-png" => {
                let selection = result.unwrap();
                assert_eq!(
                    selection.mime,
                    format!("image/{}", if scenario == "bmp" { "bmp" } else { "png" })
                );
                assert_eq!(
                    selection.bytes,
                    (0..700_013).map(|i| (i % 251) as u8).collect::<Vec<_>>()
                );
            },
            "no-mime" | "external-empty" => {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound)
            },
            "timeout" => assert_eq!(result.unwrap_err().kind(), ErrorKind::TimedOut),
            "oversize" | "seat-removed" => {
                assert!(result.is_err());
            },
            _ => panic!("unknown scenario"),
        }
    }
    drop(clipboard);
    for (offer, _) in host.offers {
        offer.destroy();
    }
    surface.destroy();
    connection.flush().unwrap();
}
