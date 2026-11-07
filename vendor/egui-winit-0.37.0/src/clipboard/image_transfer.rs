//! Polished Map NG asynchronous native image transfer (MIT OR Apache-2.0).
use super::Clipboard;

pub(crate) type ImageResult = Result<Option<egui::ColorImage>, String>;
pub(crate) enum ImageTask {
    Ready(Option<ImageResult>),
    #[cfg(all(
        feature = "smithay-clipboard",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ))]
    Wayland(wayland::Task),
}

impl Clipboard {
    pub(crate) fn start_image(&mut self, request: crate::image_clipboard::Request) -> ImageTask {
        #[cfg(all(
            feature = "smithay-clipboard",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ))]
        if let Some(clipboard) = &self.smithay {
            return match wayland::start(clipboard, request) {
                Ok(task) => ImageTask::Wayland(task),
                Err(error) => ImageTask::Ready(Some(Err(error))),
            };
        }
        ImageTask::Ready(Some(match request {
            crate::image_clipboard::Request::Copy(image) => self.set_image(&image).map(|()| None),
            crate::image_clipboard::Request::Paste => self.read_image_result(),
        }))
    }

    pub(crate) fn poll_image(&mut self, task: &mut ImageTask) -> Option<ImageResult> {
        match task {
            ImageTask::Ready(result) => result.take(),
            #[cfg(all(
                feature = "smithay-clipboard",
                any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd"
                )
            ))]
            ImageTask::Wayland(task) => {
                let Some(clipboard) = &self.smithay else {
                    return Some(Err("Wayland clipboard closed".into()));
                };
                wayland::poll(clipboard, task)
            }
        }
    }

    fn read_image_result(&mut self) -> ImageResult {
        #[cfg(all(
            not(any(target_os = "android", target_os = "ios")),
            feature = "arboard"
        ))]
        if let Some(clipboard) = &mut self.arboard {
            return match clipboard.get_image() {
                Ok(image) => Ok(Some(super::color_image_from_arboard(&image))),
                Err(arboard::Error::ContentNotAvailable) => Ok(None),
                Err(error) => Err(error.to_string()),
            };
        }
        Err("The native image clipboard is unavailable on this display".into())
    }
}

#[cfg(all(
    feature = "smithay-clipboard",
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )
))]
mod wayland {
    use super::*;
    use smithay_clipboard::{Selection, Transfer};
    use std::sync::mpsc::{self, Receiver, TryRecvError};
    type Payload = Vec<(String, Vec<u8>)>;

    pub(crate) enum Task {
        Encode(Receiver<Result<Payload, String>>),
        Store(Transfer<()>),
        Receive(Transfer<Selection>),
        Decode(Receiver<ImageResult>),
    }

    fn worker<T: Send + 'static>(
        job: impl FnOnce() -> T + Send + 'static,
    ) -> Result<Receiver<T>, String> {
        // A cancelled UI request may still be in a bounded codec operation.
        // Retain the permit until that thread exits, preventing cancel/retry
        // from creating unbounded workers or image allocations.
        use std::sync::atomic::{AtomicUsize, Ordering};
        static WORKERS: AtomicUsize = AtomicUsize::new(0);
        struct Permit;
        impl Drop for Permit {
            fn drop(&mut self) {
                WORKERS.fetch_sub(1, Ordering::AcqRel);
            }
        }
        WORKERS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < 4).then_some(n + 1)
            })
            .map_err(|_| "Image clipboard codecs are busy; try again".to_owned())?;
        let permit = Permit;
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("image-clipboard-codec".into())
            .spawn(move || {
                let _permit = permit;
                let _ = tx.send(job());
            })
            .map_err(|e| e.to_string())?;
        Ok(rx)
    }

    pub(crate) fn start(
        clipboard: &smithay_clipboard::Clipboard,
        request: crate::image_clipboard::Request,
    ) -> Result<Task, String> {
        match request {
            crate::image_clipboard::Request::Copy(image) => {
                Ok(Task::Encode(worker(move || encode(image))?))
            }
            crate::image_clipboard::Request::Paste => clipboard
                .load_bytes(vec![
                    "image/png".into(),
                    "image/bmp".into(),
                    "image/x-bmp".into(),
                    "image/x-MS-bmp".into(),
                ])
                .map(Task::Receive)
                .map_err(|e| e.to_string()),
        }
    }

    pub(crate) fn poll(
        clipboard: &smithay_clipboard::Clipboard,
        task: &mut Task,
    ) -> Option<ImageResult> {
        match task {
            Task::Encode(rx) => match rx.try_recv() {
                Ok(Ok(payload)) => match clipboard.store_bytes(payload) {
                    Ok(transfer) => {
                        *task = Task::Store(transfer);
                        None
                    }
                    Err(e) => Some(Err(e.to_string())),
                },
                Ok(Err(e)) => Some(Err(e)),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    Some(Err("Clipboard image encoder stopped".into()))
                }
            },
            Task::Store(transfer) => transfer
                .poll()
                .map(|r| r.map(|()| None).map_err(|e| e.to_string())),
            Task::Receive(transfer) => match transfer.poll()? {
                Ok(selection) => match worker(move || decode(selection).map(Some)) {
                    Ok(rx) => {
                        *task = Task::Decode(rx);
                        None
                    }
                    Err(e) => Some(Err(e)),
                },
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Some(Ok(None)),
                Err(e) => Some(Err(e.to_string())),
            },
            Task::Decode(rx) => match rx.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    Some(Err("Clipboard image decoder stopped".into()))
                }
            },
        }
    }

    const MAX_PIXELS: usize = 16 * 1024 * 1024;
    fn encode(image: egui::ColorImage) -> Result<Payload, String> {
        use image::ImageEncoder as _;
        let [width, height] = image.size;
        if width == 0
            || height == 0
            || width > 8192
            || height > 8192
            || width.checked_mul(height) != Some(image.pixels.len())
            || image.pixels.len() > MAX_PIXELS
        {
            return Err("Clipboard image dimensions exceed the supported limits".into());
        }
        let bytes: Vec<u8> = image
            .pixels
            .iter()
            .flat_map(|c| c.to_srgba_unmultiplied())
            .collect();
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(
                &bytes,
                width as u32,
                height as u32,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| e.to_string())?;
        // PNG is lossless and preserves alpha. BMP is additionally offered for legacy peers.
        let mut bmp = Vec::new();
        image::codecs::bmp::BmpEncoder::new(&mut bmp)
            .encode(
                &bytes,
                width as u32,
                height as u32,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| e.to_string())?;
        if png.len().saturating_add(bmp.len()) > smithay_clipboard::MAX_BYTES {
            return Err("Encoded clipboard image exceeds 64 MiB".into());
        }
        Ok(vec![("image/png".into(), png), ("image/bmp".into(), bmp)])
    }

    fn decode(selection: Selection) -> Result<egui::ColorImage, String> {
        let format = match selection.mime.as_str() {
            "image/png" => image::ImageFormat::Png,
            "image/bmp" | "image/x-bmp" | "image/x-MS-bmp" => image::ImageFormat::Bmp,
            _ => return Err("Unsupported clipboard image format".into()),
        };
        let encoded_len = selection.bytes.len();
        let mime = selection.mime.clone();
        let mut reader =
            image::ImageReader::with_format(std::io::Cursor::new(selection.bytes), format);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(8192);
        limits.max_image_height = Some(8192);
        limits.max_alloc = Some((MAX_PIXELS * 4) as u64);
        reader.limits(limits);
        let decoded = reader
            .decode()
            .map_err(|e| format!("Invalid {mime} clipboard image ({encoded_len} bytes): {e}"))?;
        if u64::from(decoded.width()) * u64::from(decoded.height()) > MAX_PIXELS as u64 {
            return Err("Clipboard image exceeds the pixel limit".into());
        }
        let rgba = decoded.into_rgba8();
        Ok(egui::ColorImage::from_rgba_unmultiplied(
            [rgba.width() as usize, rgba.height() as usize],
            rgba.as_raw(),
        ))
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn png_and_bmp_use_exact_dimensions_and_rgb() {
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [2, 1],
                &[17, 34, 51, 255, 255, 0, 85, 255],
            );
            for (mime, bytes) in encode(image.clone()).unwrap() {
                let decoded = decode(Selection { mime, bytes }).unwrap();
                assert_eq!(decoded.size, image.size);
                assert_eq!(decoded.pixels, image.pixels);
            }
        }
        #[test]
        fn malformed_or_excessive_images_are_errors() {
            assert!(
                decode(Selection {
                    mime: "image/png".into(),
                    bytes: b"BM invalid PNG".to_vec()
                })
                .is_err()
            );
            assert!(
                decode(Selection {
                    mime: "image/bmp".into(),
                    bytes: vec![0; 54]
                })
                .is_err()
            );
            assert!(encode(egui::ColorImage::filled([0, 0], egui::Color32::WHITE)).is_err());
            // A complete BMP header with dimensions exceeding the limit, without
            // a pixel allocation or an encoded payload of corresponding size.
            let mut bmp = vec![0u8; 54];
            bmp[..2].copy_from_slice(b"BM");
            bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
            bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
            bmp[18..22].copy_from_slice(&100_000i32.to_le_bytes());
            bmp[22..26].copy_from_slice(&100_000i32.to_le_bytes());
            bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
            bmp[28..30].copy_from_slice(&24u16.to_le_bytes());
            assert!(
                decode(Selection {
                    mime: "image/bmp".into(),
                    bytes: bmp
                })
                .is_err()
            );
        }
    }
}
