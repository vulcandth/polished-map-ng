//! Exercise the production clipboard reader against an independent X11 owner.
//! FLTK 1.4.5 Fl_x.cxx advertises only image/bmp for copied graphics.
#![cfg(target_os = "linux")]

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{
            ConnectionExt, CreateWindowAux, EventMask, PropMode, SELECTION_NOTIFY_EVENT,
            SelectionNotifyEvent, WindowClass,
        },
    },
    wrapper::ConnectionExt as _,
};

fn bmp() -> Vec<u8> {
    // Independent 2x1, 24-bit BITMAPINFOHEADER: red then green, padded scanline.
    let mut bytes = vec![0; 62];
    bytes[..2].copy_from_slice(b"BM");
    bytes[2..6].copy_from_slice(&62u32.to_le_bytes());
    bytes[10..14].copy_from_slice(&54u32.to_le_bytes());
    bytes[14..18].copy_from_slice(&40u32.to_le_bytes());
    bytes[18..22].copy_from_slice(&2i32.to_le_bytes());
    bytes[22..26].copy_from_slice(&1i32.to_le_bytes());
    bytes[26..28].copy_from_slice(&1u16.to_le_bytes());
    bytes[28..30].copy_from_slice(&24u16.to_le_bytes());
    bytes[34..38].copy_from_slice(&8u32.to_le_bytes());
    bytes[54..60].copy_from_slice(&[0, 0, 255, 0, 255, 0]);
    bytes
}

fn read_from_owner(
    offers: Vec<(&'static str, Vec<u8>)>,
) -> (
    Result<arboard::ImageData<'static>, arboard::Error>,
    Vec<String>,
) {
    let (ready_tx, ready_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();
    let owner = thread::spawn(move || {
        let (conn, screen) = x11rb::connect(None).unwrap();
        let window = conn.generate_id().unwrap();
        conn.create_window(
            0,
            window,
            conn.setup().roots[screen].root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_ONLY,
            0,
            &CreateWindowAux::new(),
        )
        .unwrap()
        .check()
        .unwrap();
        let atom = |name: &[u8]| conn.intern_atom(false, name).unwrap().reply().unwrap().atom;
        let clipboard = atom(b"CLIPBOARD");
        let offers: Vec<_> = offers
            .into_iter()
            .map(|(name, bytes)| (atom(name.as_bytes()), name, bytes))
            .collect();
        conn.set_selection_owner(window, clipboard, x11rb::CURRENT_TIME)
            .unwrap()
            .check()
            .unwrap();
        assert_eq!(
            conn.get_selection_owner(clipboard)
                .unwrap()
                .reply()
                .unwrap()
                .owner,
            window
        );
        conn.flush().unwrap();
        ready_tx.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut requests = Vec::new();
        while Instant::now() < deadline && stop_rx.try_recv().is_err() {
            if let Some(Event::SelectionRequest(event)) = conn.poll_for_event().unwrap() {
                requests.push(
                    String::from_utf8(
                        conn.get_atom_name(event.target)
                            .unwrap()
                            .reply()
                            .unwrap()
                            .name,
                    )
                    .unwrap(),
                );
                let property = if event.property == 0 {
                    event.target
                } else {
                    event.property
                };
                let served = offers.iter().find(|(target, _, _)| *target == event.target);
                if let Some((_, _, bytes)) = served {
                    conn.change_property8(
                        PropMode::REPLACE,
                        event.requestor,
                        property,
                        event.target,
                        bytes,
                    )
                    .unwrap()
                    .check()
                    .unwrap();
                }
                conn.send_event(
                    false,
                    event.requestor,
                    EventMask::NO_EVENT,
                    SelectionNotifyEvent {
                        response_type: SELECTION_NOTIFY_EVENT,
                        sequence: 0,
                        time: event.time,
                        requestor: event.requestor,
                        selection: event.selection,
                        target: event.target,
                        property: if served.is_some() { property } else { 0 },
                    },
                )
                .unwrap()
                .check()
                .unwrap();
                conn.flush().unwrap();
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }
        requests
    });
    ready_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let result = arboard::Clipboard::new().unwrap().get_image();
    stop_tx.send(()).unwrap();
    (result, owner.join().unwrap())
}

#[test]
#[ignore = "requires isolated Xvfb; changes that server's clipboard selection"]
fn original_bmp_clipboard_and_png_precedence() {
    assert_eq!(
        std::env::var("POLISHED_MAP_CLIPBOARD_PROBE").as_deref(),
        Ok("1")
    );
    assert!(
        std::env::var_os("WAYLAND_DISPLAY").is_none(),
        "run on isolated X11"
    );
    let (image, requests) = read_from_owner(vec![("image/bmp", bmp())]);
    let image = image.unwrap();
    assert_eq!((image.width, image.height), (2, 1));
    assert_eq!(image.bytes.as_ref(), [255, 0, 0, 255, 0, 255, 0, 255]);
    assert_eq!(requests, ["image/png", "image/bmp"]);

    let dir = tempfile::tempdir().unwrap();
    let png = dir.path().join("pixel.png");
    polished_map_project::write_png(
        &png,
        &polished_map_core::RgbImage {
            width: 1,
            height: 1,
            pixels: vec![12, 34, 56],
        },
    )
    .unwrap();
    let (image, requests) = read_from_owner(vec![
        ("image/bmp", bmp()),
        ("image/png", std::fs::read(png).unwrap()),
    ]);
    let image = image.unwrap();
    assert_eq!((image.width, image.height), (1, 1));
    assert_eq!(image.bytes.as_ref(), [12, 34, 56, 255]);
    assert_eq!(requests, ["image/png"]);

    for bad in [b"BM truncated".to_vec(), {
        let mut bytes = bmp();
        bytes[18..22].copy_from_slice(&i32::MAX.to_le_bytes());
        bytes[22..26].copy_from_slice(&i32::MAX.to_le_bytes());
        bytes
    }] {
        let (result, _) = read_from_owner(vec![("image/bmp", bad)]);
        assert!(matches!(result, Err(arboard::Error::ConversionFailure)));
    }
    let (result, requests) = read_from_owner(vec![
        ("image/png", b"bad PNG".to_vec()),
        ("image/bmp", bmp()),
    ]);
    assert!(matches!(result, Err(arboard::Error::ConversionFailure)));
    assert_eq!(
        requests,
        ["image/png"],
        "corruption must not select a different payload"
    );
    let (result, _) = read_from_owner(vec![]);
    assert!(matches!(result, Err(arboard::Error::ContentNotAvailable)));
}
