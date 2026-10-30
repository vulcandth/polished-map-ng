//! Polished Map NG native-transfer extension (Apache-2.0, like winit).
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::dpi::PhysicalPosition;
use crate::event::WindowEvent;

pub(crate) const MAX_FILES: usize = 4096;
pub(crate) const MAX_BYTES: usize = 4 * 1024 * 1024;

pub(crate) fn completed(
    paths: Vec<PathBuf>,
    position: PhysicalPosition<f64>,
) -> Option<WindowEvent> {
    static NEXT_TRANSFER: AtomicU64 = AtomicU64::new(1);
    if paths.is_empty()
        || paths.len() > MAX_FILES
        || !position.x.is_finite()
        || !position.y.is_finite()
    {
        return None;
    }
    Some(WindowEvent::FilesDropped {
        transfer_id: NEXT_TRANSFER.fetch_add(1, Ordering::Relaxed),
        paths,
        position,
    })
}

/// Decode each URI separately so escaped newlines cannot create extra files.
/// Preserve Unix filename bytes; never canonicalize or fetch remote URLs.
#[cfg(unix)]
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub(crate) fn uri_list(bytes: &[u8]) -> Option<Vec<PathBuf>> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    if bytes.len() > MAX_BYTES {
        return None;
    }
    let mut paths = Vec::new();
    for line in bytes.split(|b| *b == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() || line.starts_with(b"#") {
            continue;
        }
        let rest = line.strip_prefix(b"file://")?;
        let path = if rest.starts_with(b"/") {
            rest
        } else {
            rest.strip_prefix(b"localhost")?
        };
        if !path.starts_with(b"/") || path.contains(&b'?') || path.contains(&b'#') {
            return None;
        }
        let mut decoded = Vec::with_capacity(path.len());
        let mut i = 0;
        while i < path.len() {
            let byte = if path[i] == b'%' {
                let hex = |b: u8| (b as char).to_digit(16).map(|n| n as u8);
                let byte = hex(*path.get(i + 1)?)? * 16 + hex(*path.get(i + 2)?)?;
                i += 3;
                byte
            } else {
                let byte = path[i];
                i += 1;
                byte
            };
            if byte == 0 {
                return None;
            }
            decoded.push(byte);
        }
        paths.push(PathBuf::from(OsString::from_vec(decoded)));
        if paths.len() > MAX_FILES {
            return None;
        }
    }
    (!paths.is_empty()).then_some(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_drop_has_unique_identity_and_rejects_invalid_records() {
        let paths = vec![PathBuf::from("first.blk"), PathBuf::from("second.blk")];
        let position = PhysicalPosition::new(-200.0, 900.25);
        let Some(WindowEvent::FilesDropped {
            transfer_id: first,
            paths: returned,
            position: got,
        }) = completed(paths.clone(), position)
        else {
            panic!("missing complete drop")
        };
        assert_eq!(got, position);
        assert_eq!(returned, paths);
        let Some(WindowEvent::FilesDropped {
            transfer_id: second,
            ..
        }) = completed(paths.clone(), position)
        else {
            panic!("missing second drop")
        };
        assert_ne!(first, second);
        assert!(completed(Vec::new(), position).is_none());
        assert!(completed(vec![PathBuf::new(); MAX_FILES + 1], position).is_none());
        assert!(completed(paths, PhysicalPosition::new(f64::NAN, 0.0)).is_none());
        assert_eq!(MAX_BYTES, 4 * 1024 * 1024);
    }

    #[cfg(unix)]
    #[test]
    fn uri_list_preserves_native_bytes_and_transfer_order() {
        use std::os::unix::ffi::OsStrExt;
        let paths =
            uri_list(b"#comment\r\nfile:///tmp/a%20b.blk\r\nfile://localhost/tmp/%FF%0Anext.blk\n")
                .unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].as_os_str().as_bytes(), b"/tmp/a b.blk");
        assert_eq!(paths[1].as_os_str().as_bytes(), b"/tmp/\xff\nnext.blk");
        // A valid first file must not allow a later unsupported URL through.
        for bytes in [
            b"https://example.com/map.blk".as_slice(),
            b"file://remote/tmp/map.blk",
            b"file:///tmp/%00.blk",
            b"file:///tmp/a%2",
            b"file:///tmp/a%zz",
            b"file:///tmp/ok\nhttp://bad",
            b"file:///tmp/a?query",
            b"#nothing\n",
        ] {
            assert!(uri_list(bytes).is_none(), "{bytes:?}");
        }
        assert!(uri_list(&vec![b'a'; MAX_BYTES + 1]).is_none());
        assert!(uri_list(&b"file:///a\n".repeat(MAX_FILES + 1)).is_none());
    }
}
