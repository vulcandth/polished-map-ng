//! master Tile::draw_with_priority and Metatileset::draw_metatile. LGPL-3.0-only.
use super::*;
use polished_map_core::PriorityPattern;

/// Three immutable nearest-filtered textures shared by all views in one context.
#[derive(Clone)]
pub(super) struct Textures([egui::TextureHandle; 3]);

impl Textures {
    pub(super) fn get(ctx: &egui::Context) -> Self {
        let key = egui::Id::unique("priority-pattern-textures");
        if let Some(textures) = ctx.data(|d| d.get_temp::<Self>(key)) {
            return textures;
        }
        let textures = Self(PriorityPattern::ALL.map(|pattern| {
            ctx.load_texture(
                format!("priority-{}", pattern.size()),
                egui::ColorImage::from_rgba_unmultiplied([pattern.size(); 2], &pattern.rgba()),
                egui::TextureOptions::NEAREST,
            )
        }));
        ctx.data_mut(|d| d.insert_temp(key, textures.clone()));
        textures
    }

    pub(super) fn tile(&self, painter: &egui::Painter, rect: Rect, pattern: PriorityPattern) {
        let index = match pattern {
            PriorityPattern::Small => 0,
            PriorityPattern::Large => 1,
            PriorityPattern::Chip => 2,
        };
        painter.image(
            self.0[index].id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    pub(super) fn block(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        flags: Option<[bool; 16]>,
        zoom: bool,
    ) {
        let Some(flags) = flags else {
            return;
        };
        let pattern = if zoom {
            PriorityPattern::Large
        } else {
            PriorityPattern::Small
        };
        let size = pattern.size() as f32;
        for (slot, flag) in flags.into_iter().enumerate() {
            if flag {
                self.tile(
                    painter,
                    Rect::from_min_size(
                        rect.min + Vec2::new((slot % 4) as f32 * size, (slot / 4) as f32 * size),
                        Vec2::splat(size),
                    ),
                    pattern,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::{NodeT, Queryable};
    use polished_map_core::BlockCommand;

    #[test]
    fn priority_controls_keep_bits_and_export_pixels_and_respect_disabled_and_modal_states() {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let ctx = h.ctx.clone();
        let tiles = std::array::from_fn(|i| if i % 2 == 0 { 2 } else { 1 });
        assert!(
            h.state_mut()
                .apply_block_commands(&ctx, &[BlockCommand::Set { block: 0, tiles }])
        );
        h.state_mut().status = "Priority display fixture".into();
        h.remove_cursor();
        h.run();
        let loaded = h.state().loaded.as_ref().unwrap();
        let before = loaded.graphics.render_map(loaded.document.map()).unwrap();
        let attributes = loaded
            .tileset
            .as_ref()
            .unwrap()
            .tileset()
            .attributes()
            .to_vec();
        assert!(h.state().show_priority && h.state().allow_priority);
        #[cfg(feature = "visual-tests")]
        h.snapshot("priority-map-normal");
        h.get_by_role_and_label(egui::Role::Button, "Show Priority")
            .click();
        h.run();
        assert!(!h.state().show_priority);
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::P,
        );
        h.run();
        assert!(h.state().show_priority);
        h.get_by_label("View").click();
        h.run();
        let menu = h.get_by_role_and_label(egui::Role::CheckBox, "Show Priority");
        assert_eq!(
            menu.accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        menu.click();
        h.run();
        assert!(!h.state().show_priority);
        h.key_press(egui::Key::Escape);
        h.run();
        h.get_by_role_and_label(egui::Role::Button, "Show Priority")
            .click();
        h.run();
        h.get_by_label("Options").click();
        h.run();
        h.get_by_label("Tile Priority").click();
        h.run();
        h.key_press(egui::Key::Escape);
        h.run();
        assert!(
            h.get_by_role_and_label(egui::Role::Button, "Show Priority")
                .accesskit_node()
                .is_disabled()
        );
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::P,
        );
        h.run();
        assert!(
            h.state().show_priority,
            "disabled control retains checked/display state"
        );
        h.get_by_label("Zoom").click();
        h.remove_cursor();
        h.run();
        #[cfg(feature = "visual-tests")]
        h.snapshot("priority-map-zoom-disabled");
        let loaded = h.state().loaded.as_ref().unwrap();
        assert_eq!(
            loaded.graphics.render_map(loaded.document.map()).unwrap(),
            before
        );
        assert_eq!(
            loaded.tileset.as_ref().unwrap().tileset().attributes(),
            attributes
        );
        assert!(
            !loaded.document.map().is_modified()
                && !loaded.tileset.as_ref().unwrap().tileset().is_modified()
        );
        h.get_by_label("Options").click();
        h.run();
        h.get_by_label("Tile Priority").click();
        h.run();
        h.key_press(egui::Key::Escape);
        h.run();
        assert!(h.state().allow_priority);
        h.state_mut().prepare_block(&ctx);
        h.run();
        h.state_mut().block_form.as_mut().unwrap().windows_buttons = true;
        h.remove_cursor();
        h.run();
        #[cfg(feature = "visual-tests")]
        h.snapshot("priority-block-editor");
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::P,
        );
        h.run();
        assert!(h.state().show_priority);
        h.get_by_label("Cancel").click();
        h.run();
        h.state_mut().prepare_tileset();
        h.state_mut().tileset_form.as_mut().unwrap().windows_buttons = true;
        h.remove_cursor();
        h.run();
        #[cfg(feature = "visual-tests")]
        h.snapshot("priority-tileset-editor");
        h.get_by_label("Tileset tile $02").click();
        h.run();
        h.input_mut().events.push(egui::Event::Key {
            key: egui::Key::C,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        });
        h.step();
        let copied = h
            .output()
            .platform_output
            .commands
            .iter()
            .find_map(|command| {
                if let egui::OutputCommand::CopyImage(image) = command {
                    Some(image)
                } else {
                    None
                }
            })
            .expect("copy image output");
        assert_eq!(copied.size, [8, 8]);
        assert!(
            copied.pixels.iter().all(|p| *p == Color32::BLACK),
            "clipboard omits the teal overlay"
        );
        h.run();
        h.get_by_label("Cancel").click();
        h.run();
        h.state_mut().close_map();
        assert!(h.state().show_priority && h.state().allow_priority);
    }

    #[cfg(feature = "visual-tests")]
    #[test]
    fn priority_textures_preserve_every_pattern_pixel_at_both_dpi_scales() {
        let colors = [
            [32, 96, 96, 96],
            [50, 126, 126, 96],
            [86, 186, 186, 96],
            [104, 216, 216, 96],
        ];
        for scale in [1_u32, 2] {
            for dark in [false, true] {
                let mut h = egui_kittest::Harness::builder()
                    .with_size(Vec2::new(220.0, 150.0))
                    .with_pixels_per_point(scale as f32)
                    .build_ui(|ui| {
                        ui.painter().rect_filled(
                            ui.max_rect(),
                            0.0,
                            if dark {
                                Color32::from_gray(32)
                            } else {
                                Color32::WHITE
                            },
                        );
                        let textures = Textures::get(ui.ctx());
                        for (i, pattern) in PriorityPattern::ALL.into_iter().enumerate() {
                            textures.tile(
                                ui.painter(),
                                Rect::from_min_size(
                                    Pos2::new(16.0 + i as f32 * 40.0, 16.0),
                                    Vec2::splat(pattern.size() as f32),
                                ),
                                pattern,
                            );
                        }
                        for (i, [r, g, b, a]) in colors.into_iter().enumerate() {
                            ui.painter().rect_filled(
                                Rect::from_min_size(
                                    Pos2::new(160.0, 16.0 + i as f32 * 30.0),
                                    Vec2::splat(24.0),
                                ),
                                0.0,
                                Color32::from_rgba_unmultiplied(r, g, b, a),
                            );
                        }
                    });
                h.run();
                let image = h.render().unwrap();
                let samples = std::array::from_fn::<_, 4, _>(|i| {
                    image.get_pixel(161 * scale, (17 + i as u32 * 30) * scale).0
                });
                for (i, pattern) in PriorityPattern::ALL.into_iter().enumerate() {
                    let rgba = pattern.rgba();
                    let size = pattern.size() as u32;
                    for y in 0..size * scale {
                        for x in 0..size * scale {
                            let offset = ((y / scale * size + x / scale) * 4) as usize;
                            let color = colors
                                .iter()
                                .position(|c| c.as_slice() == &rgba[offset..offset + 4])
                                .unwrap();
                            let actual = image
                                .get_pixel((16 + i as u32 * 40) * scale + x, 16 * scale + y)
                                .0;
                            assert!(
                                actual
                                    .into_iter()
                                    .zip(samples[color])
                                    .all(|(a, b)| a.abs_diff(b) <= 1),
                                "{pattern:?} {x},{y} dpi={scale} dark={dark}: {actual:?} != {:?}",
                                samples[color]
                            );
                        }
                    }
                }
            }
        }
    }
}
