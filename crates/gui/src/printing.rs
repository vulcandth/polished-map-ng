//! master Print_Options_Dialog, Main_Window::print_cb and print preferences.
//! LGPL-3.0-only. Native picker/worker dispatch is separate from headless UI tests.
use super::*;
use polished_map_core::{
    Events, Map, PrintError, PrintFont, PrintOptions, PrintStyle, PrintTextMask,
};
use polished_map_project::{PngExportTarget, PrintFonts, prepare_png_export, write_png_export};

#[cfg(test)]
mod tests;

pub(super) struct PrintForm {
    options: PrintOptions,
    windows_buttons: bool,
    focus_export: bool,
    #[cfg(test)]
    bounds: Rect,
}

pub(super) enum Destination {
    ChooseFile,
    File(PngExportTarget),
    Clipboard,
}
pub(super) struct PrintJob {
    map: Map,
    graphics: Graphics,
    events: Option<Events>,
    options: PrintOptions,
    style: PrintStyle,
    destination: Destination,
}
pub(super) enum PrintOutput {
    Exported(PathBuf),
    Copied(egui::ColorImage),
}

impl PrintJob {
    fn run_with(
        self,
        rasterize: impl FnMut(PrintFont, &str) -> Result<PrintTextMask, PrintError>,
    ) -> Result<PrintOutput, String> {
        let image = self
            .graphics
            .render_print(
                &self.map,
                self.events.as_ref(),
                self.options,
                self.style,
                rasterize,
            )
            .map_err(|e| e.to_string())?;
        match self.destination {
            Destination::File(target) => {
                write_png_export(&target, &image).map_err(|e| e.to_string())?;
                Ok(PrintOutput::Exported(target.path().to_owned()))
            }
            Destination::Clipboard => Ok(PrintOutput::Copied(egui::ColorImage::from_rgb(
                [image.width as usize, image.height as usize],
                &image.pixels,
            ))),
            Destination::ChooseFile => Err("Print destination was not selected".into()),
        }
    }
}

pub(super) fn run(job: PrintJob) -> Result<PrintOutput, String> {
    let needs_fonts = job.options.ids
        || (job.options.events && job.events.as_ref().is_some_and(|e| !e.entries().is_empty()));
    let fonts = needs_fonts
        .then(PrintFonts::system)
        .transpose()
        .map_err(|e| e.to_string())?;
    job.run_with(|font, text| {
        fonts
            .as_ref()
            .ok_or_else(|| PrintError::Text("font adapter not initialized".into()))?
            .rasterize(font, text)
    })
}

/// Native save choosers provide their normal overwrite confirmation. If adding
/// the source's default suffix changes the destination, confirm that file too.
#[cfg(not(test))]
pub(super) fn choose_export() -> Result<Option<PngExportTarget>, String> {
    let Some(path) = rfd::FileDialog::new()
        .set_title("Print Screenshot")
        .add_filter("PNG image", &["png"])
        .set_file_name("screenshot.png")
        .save_file()
    else {
        return Ok(None);
    };
    let target = prepare_png_export(&path).map_err(|e| e.to_string())?;
    if target.extension_added()
        && target.exists()
        && rfd::MessageDialog::new()
            .set_title("Replace screenshot?")
            .set_description(format!(
                "{} already exists. Replace it?",
                target.path().display()
            ))
            .set_level(rfd::MessageLevel::Warning)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            != rfd::MessageDialogResult::Yes
    {
        return Ok(None);
    }
    Ok(Some(target))
}

impl Editor {
    pub(super) fn prepare_print(&mut self) {
        if self.loaded.is_none()
            || self.print_worker.is_some()
            || self.print_request.is_some()
            || self.print_form.is_some()
        {
            return;
        }
        self.print_form = Some(PrintForm {
            options: self.print_options,
            windows_buttons: cfg!(windows),
            focus_export: true,
            #[cfg(test)]
            bounds: Rect::NOTHING,
        });
    }

    fn queue_print(&mut self, destination: Destination) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        self.print_request = Some(PrintJob {
            map: loaded.document.map().clone(),
            graphics: loaded.graphics.clone(),
            events: loaded.events.as_ref().map(|d| d.script().events().clone()),
            options: self.print_options,
            style: PrintStyle {
                hexadecimal_ids: self.hex,
                grid_rgb: [if self.dark { 38 } else { 85 }; 3],
            },
            destination,
        });
    }

    pub(super) fn print_dialog(&mut self, ctx: &egui::Context) {
        if let Some(message) = &self.print_success {
            let mut done = false;
            egui::Modal::new(egui::Id::unique("print-success")).show(ctx, |ui| {
                ui.heading("Success");
                ui.label(message);
                done = ui.button("OK").clicked();
                done |= ui.input_mut(|i| {
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                        || i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                });
            });
            if done {
                self.print_success = None;
            }
            return;
        }
        let Some(mut form) = self.print_form.take() else {
            return;
        };
        let (mut export, mut copy, mut cancel) = (false, false, false);
        egui::Modal::new(egui::Id::unique("print-options")).show(ctx, |ui| {
            ui.heading("Print Options");
            let font = egui::FontId::proportional(13.0);
            let measure = |label: &str| {
                ui.painter()
                    .layout_no_wrap(label.into(), font.clone(), ui.visuals().text_color())
                    .size()
                    .x
                    .ceil()
            };
            // text_width(label,pad) adds pad on BOTH sides in master utils.cpp.
            let heading = measure("Show:") + 8.0;
            let labels = ["Grid", "IDs", "Priority", "Events", "Warp IDs"];
            let widths = labels.map(|label| measure(label) + 4.0 + 22.0);
            let width =
                (10.0 + heading + 4.0 + widths.iter().sum::<f32>() + 4.0 * 4.0 + 10.0).max(288.0);
            let (area, _) = ui.allocate_exact_size(Vec2::new(width, 80.0), Sense::hover());
            #[cfg(test)]
            {
                form.bounds = area;
            }
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            ui.put(
                rect(10.0, 10.0, heading, 22.0),
                egui::Label::new(egui::RichText::new("Show:").font(font.clone())),
            );
            let mut x = 10.0 + heading + 4.0;
            for ((label, w), value) in labels.into_iter().zip(widths).zip([
                &mut form.options.grid,
                &mut form.options.ids,
                &mut form.options.priority,
                &mut form.options.events,
                &mut form.options.warp_ids,
            ]) {
                ui.put(
                    rect(x, 10.0, w, 22.0),
                    egui::Checkbox::new(value, egui::RichText::new(label).font(font.clone())),
                );
                x += w + 4.0;
            }
            let (export_x, cancel_x) = if form.windows_buttons {
                (width - 278.0, width - 90.0)
            } else {
                (width - 90.0, width - 278.0)
            };
            let response = ui
                .put(
                    rect(export_x, 48.0, 80.0, 22.0),
                    egui::Button::new("Export…"),
                )
                .on_hover_text("Export (Enter)");
            if form.focus_export {
                response.request_focus();
                form.focus_export = false;
            }
            export = response.clicked();
            copy = ui
                .put(
                    rect(width - 184.0, 48.0, 80.0, 22.0),
                    egui::Button::new("Copy"),
                )
                .on_hover_text("Copy (Ctrl+C)")
                .clicked();
            cancel = ui
                .put(
                    rect(cancel_x, 48.0, 80.0, 22.0),
                    egui::Button::new("Cancel"),
                )
                .on_hover_text("Cancel (Esc)")
                .clicked();
            cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            copy |= ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::C));
            export |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        });
        if cancel || export || copy {
            // The source stores all five values BEFORE checking cancellation.
            self.print_options = form.options;
            if !cancel {
                self.queue_print(if copy {
                    Destination::Clipboard
                } else {
                    Destination::ChooseFile
                });
            }
        } else {
            self.print_form = Some(form);
        }
    }

    /// Native App adapter injects picker and worker; interaction tests can supply
    /// deterministic adapters without opening a chooser or touching the clipboard.
    pub(super) fn dispatch_print(
        &mut self,
        ctx: &egui::Context,
        choose: impl FnOnce() -> Result<Option<PngExportTarget>, String>,
        work: fn(PrintJob) -> Result<PrintOutput, String>,
    ) {
        let Some(mut job) = self.print_request.take() else {
            return;
        };
        if matches!(job.destination, Destination::ChooseFile) {
            match choose() {
                Ok(Some(target)) => job.destination = Destination::File(target),
                Ok(None) => return,
                Err(e) => {
                    self.error = Some(e);
                    return;
                }
            }
        }
        let (tx, rx) = mpsc::channel();
        let context = ctx.clone();
        match std::thread::Builder::new()
            .name("map-print".into())
            .spawn(move || {
                let _ = tx.send(work(job));
                context.request_repaint();
            }) {
            Ok(_) => {
                self.print_worker = Some(rx);
                self.status = "Printing…".into();
            }
            Err(e) => self.error = Some(format!("Could not start Print worker: {e}")),
        }
    }

    pub(super) fn poll_print(&mut self, ctx: &egui::Context) {
        let result = self
            .print_worker
            .as_ref()
            .and_then(|rx| match rx.try_recv() {
                Ok(r) => Some(r),
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("Print worker stopped unexpectedly".into()))
                }
                Err(_) => None,
            });
        let Some(result) = result else {
            return;
        };
        self.print_worker = None;
        match result {
            Ok(output) => {
                let message = match output {
                    PrintOutput::Exported(path) => format!(
                        "Printed {}!",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    PrintOutput::Copied(image) => {
                        ctx.copy_image(image);
                        "Copied to clipboard!".into()
                    }
                };
                self.status = message.clone();
                self.print_success = Some(message);
            }
            Err(e) => {
                self.status = "Print failed".into();
                self.error = Some(e);
            }
        }
    }
}
