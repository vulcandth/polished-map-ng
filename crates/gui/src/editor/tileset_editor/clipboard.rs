//! Native clipboard image jobs and bounded import into the tile draft. LGPL-3.0-only.
use super::*;

impl TilesetForm {
    pub(super) fn copy_image(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if self.pending_image.is_some() {
            return Ok(());
        }
        // Tile::draw_for_clipboard deliberately exports monochrome samples.
        let pixels: Vec<_> = self
            .draft
            .pixels(self.selected)
            .iter()
            .flat_map(|&h| [[255, 85, 170, 0][usize::from(h)]; 3])
            .collect();
        // The root clipboard owner outlives this draft's native child window.
        let viewport = egui::ViewportId::ROOT;
        let id = egui_winit::image_clipboard::request(
            ctx,
            viewport,
            egui::ColorImage::from_rgb([8, 8], &pixels),
        )?;
        self.pending_image = Some(PendingImage {
            ctx: ctx.clone(),
            viewport,
            id,
        });
        Ok(())
    }

    pub(super) fn import_rgb(
        &mut self,
        width: usize,
        height: usize,
        pixels: Vec<u8>,
    ) -> Result<(), String> {
        let width = u16::try_from(width).map_err(|_| "Clipboard image is too wide")?;
        let height = u16::try_from(height).map_err(|_| "Clipboard image is too tall")?;
        self.apply(TileCommand::ImportRgb {
            tile: self.selected,
            width,
            height,
            pixels,
            allow_512: self.draft.allow_extended(),
        })
    }

    pub(super) fn paste_image(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if self.pending_image.is_some() {
            return Ok(());
        }
        let viewport = ctx.viewport_id();
        let id = egui_winit::image_clipboard::request_paste(ctx, viewport)?;
        self.pending_image = Some(PendingImage {
            ctx: ctx.clone(),
            viewport,
            id,
        });
        Ok(())
    }

    pub(super) fn poll_image(&mut self) -> Result<(), String> {
        let Some(pending) = &self.pending_image else {
            return Ok(());
        };
        let Some(result) = egui_winit::image_clipboard::take_paste_reply(
            &pending.ctx,
            pending.viewport,
            pending.id,
        ) else {
            return Ok(());
        };
        self.pending_image = None;
        if let Some(image) = result? {
            if image.width() > 2048 || image.height() > 2048 {
                return Err("Clipboard image exceeds 2048×2048".into());
            }
            let pixels = image
                .pixels
                .iter()
                .flat_map(|c| {
                    let rgba = c.to_srgba_unmultiplied();
                    [rgba[0], rgba[1], rgba[2]]
                })
                .collect();
            self.import_rgb(image.width(), image.height(), pixels)?;
        }
        Ok(())
    }
}
