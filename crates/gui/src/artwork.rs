//! Original Polished Map artwork at the pinned master revision.
//! Source placement: Modal_Dialog::refresh, FL_ALIGN_CENTER and Main_Window icon.
//! See assets/artwork/NOTICE.md and docs/artwork-inventory.json for provenance.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum Image {
    Error,
    Warning,
    Success,
    App,
}

impl Image {
    fn pixels(self) -> &'static [u8; 48 * 48 * 4] {
        match self {
            Self::Error => include_bytes!("../assets/artwork/error.rgba"),
            Self::Warning => include_bytes!("../assets/artwork/warning.rgba"),
            Self::Success => include_bytes!("../assets/artwork/success.rgba"),
            Self::App => include_bytes!("../assets/artwork/app.rgba"),
        }
    }

    pub(super) fn paint(self, ui: &egui::Ui, slot: Rect) {
        if !ui.is_rect_visible(slot) {
            return;
        }
        // Source pictures are 48px centered inside 50px slots: one-point inset.
        // Draw integer pixel cells to retain hard edges at either DPI scale.
        let origin = slot.center() - Vec2::splat(24.0);
        let scale = ui.ctx().pixels_per_point();
        let origin = Pos2::new(
            geometry::snap_to_pixel(origin.x, scale),
            geometry::snap_to_pixel(origin.y, scale),
        );
        let mut mesh = egui::Mesh::default();
        for (i, pixel) in self.pixels().as_chunks::<4>().0.iter().enumerate() {
            if pixel[3] != 0 {
                mesh.add_colored_rect(
                    Rect::from_min_size(
                        origin + Vec2::new((i % 48) as f32, (i / 48) as f32),
                        Vec2::splat(1.0),
                    ),
                    Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]),
                );
            }
        }
        ui.painter()
            .with_clip_rect(slot.intersect(ui.clip_rect()))
            .add(mesh);
    }
}

/// Original upstream application picture for the desktop window adapter.
/// Linux uses its original 16px XPM; other platforms use the 48px About/ICO frame.
/// Platform packaging retains the original multi-resolution ICO separately.
pub fn application_icon() -> egui::IconData {
    icon_for_linux(cfg!(target_os = "linux"))
}

fn icon_for_linux(linux: bool) -> egui::IconData {
    let (size, pixels): (u32, &[u8]) = if linux {
        (16, include_bytes!("../assets/artwork/app-icon.rgba"))
    } else {
        (48, Image::App.pixels())
    };
    egui::IconData {
        rgba: pixels.to_vec(),
        width: size,
        height: size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_icon_payloads_match_original_xpm_and_ico_frame() {
        for (linux, size, expected) in [
            (
                true,
                16,
                include_bytes!("../assets/artwork/app-icon.rgba").as_slice(),
            ),
            (
                false,
                48,
                include_bytes!("../assets/artwork/app.rgba").as_slice(),
            ),
        ] {
            let icon = icon_for_linux(linux);
            assert_eq!((icon.width, icon.height), (size, size));
            assert_eq!(icon.rgba, expected);
        }
    }

    #[cfg(feature = "visual-tests")]
    #[test]
    fn artwork_pixels_center_in_source_slots_at_both_scales_and_themes() {
        use egui_kittest::Harness;
        let images = [Image::Error, Image::Warning, Image::Success, Image::App];
        for scale in [1_u32, 2] {
            let mut h = Harness::builder()
                .with_size(Vec2::new(256.0, 128.0))
                .with_pixels_per_point(scale as f32)
                .build_ui(|ui| {
                    for (row, dark) in [false, true].into_iter().enumerate() {
                        let bg = if dark {
                            Color32::from_rgb(83, 83, 83)
                        } else {
                            Color32::from_rgb(206, 206, 206)
                        };
                        for (col, image) in images.into_iter().enumerate() {
                            let slot = Rect::from_min_size(
                                Pos2::new(8.0 + col as f32 * 60.0, 8.0 + row as f32 * 60.0),
                                Vec2::splat(50.0),
                            );
                            ui.painter().rect_filled(slot, 0.0, bg);
                            image.paint(ui, slot);
                        }
                    }
                });
            h.run();
            let rendered = h.render().unwrap();
            for row in 0..2_u32 {
                for (col, image) in images.into_iter().enumerate() {
                    for y in 0..50 * scale {
                        for x in 0..50 * scale {
                            let (lx, ly) = (x / scale, y / scale);
                            let gray = if row == 0 { 206 } else { 83 };
                            let mut expected = [gray, gray, gray, 255];
                            if (1..49).contains(&lx) && (1..49).contains(&ly) {
                                let i = ((ly - 1) * 48 + lx - 1) as usize * 4;
                                let source = &image.pixels()[i..i + 4];
                                if source[3] != 0 {
                                    expected.copy_from_slice(source);
                                }
                            }
                            assert_eq!(
                                rendered
                                    .get_pixel(
                                        (8 + col as u32 * 60) * scale + x,
                                        (8 + row * 60) * scale + y
                                    )
                                    .0,
                                expected,
                                "{image:?} ({x},{y}) scale {scale}"
                            );
                        }
                    }
                }
            }
            if scale == 1 {
                h.snapshot("original-message-artwork-atlas");
            }
        }
    }
}
