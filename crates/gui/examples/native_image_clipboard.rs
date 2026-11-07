//! Interactive two-process clipboard probe; run only for explicit native testing.
//! Captures a 320×288 acceptance map, or copies a generated 8×8 grayscale tile.
use eframe::egui;
use std::{io::Write, path::PathBuf};

struct Probe {
    output: PathBuf,
    pending: Option<u64>,
    status: String,
}

impl eframe::App for Probe {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = &ui.ctx().clone();
        if let Some(id) = self.pending
            && let Some(result) =
                egui_winit::image_clipboard::take_paste_reply(ctx, ctx.viewport_id(), id)
        {
            self.pending = None;
            self.status = match result.and_then(|image| {
                let Some(image) = image else {
                    return Ok("Native copy accepted or no image offered".to_owned());
                };
                if image.size != [320, 288] {
                    return Err(format!("Unexpected image dimensions: {:?}", image.size));
                }
                let mut file = std::io::BufWriter::new(
                    std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&self.output)
                        .map_err(|e| e.to_string())?,
                );
                file.write_all(&320u32.to_le_bytes())
                    .and_then(|()| file.write_all(&288u32.to_le_bytes()))
                    .map_err(|e| e.to_string())?;
                for color in image.pixels {
                    file.write_all(&color.to_srgba_unmultiplied())
                        .map_err(|e| e.to_string())?;
                }
                file.flush().map_err(|e| e.to_string())?;
                Ok("Captured 320×288 pixels".to_owned())
            }) {
                Ok(status) => status,
                Err(error) => format!("Error: {error}"),
            };
            println!("{}", self.status);
        }
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Native clipboard acceptance");
            ui.label("Use only with the disposable acceptance project.");
            ui.add_enabled_ui(self.pending.is_none(), |ui| {
                if ui.button("Capture copied map").clicked() {
                    match egui_winit::image_clipboard::request_paste(ctx, ctx.viewport_id()) {
                        Ok(id) => self.pending = Some(id),
                        Err(e) => self.status = e,
                    }
                }
                if ui.button("Copy generated tile").clicked() {
                    let pixels: Vec<_> = (0..64)
                        .flat_map(|i| [[255, 85, 170, 0][i % 4]; 3])
                        .collect();
                    match egui_winit::image_clipboard::request(
                        ctx,
                        ctx.viewport_id(),
                        egui::ColorImage::from_rgb([8, 8], &pixels),
                    ) {
                        Ok(id) => self.pending = Some(id),
                        Err(e) => self.status = e,
                    }
                }
            });
            ui.label(&self.status);
        });
    }
}

fn main() -> eframe::Result {
    assert_eq!(
        std::env::var("POLISHED_MAP_NATIVE_IMAGE_PROBE").as_deref(),
        Ok("1")
    );
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("new capture path required"),
    );
    eframe::run_native(
        "Native image clipboard probe",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 180.0]),
            ..Default::default()
        },
        Box::new(move |_| {
            Ok(Box::new(Probe {
                output,
                pending: None,
                status: "Ready".into(),
            }))
        }),
    )
}
