use super::*;
use crate::plusplus_session::tests::fixture;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use polished_map_core::plusplus::{AttributeCommand, BankOrder, TileReference};
use std::fs;

fn open(h: &mut Harness<'_, Editor>, roof: bool) {
    if roof {
        h.get_by_label("Tools").click();
        h.run();
        h.get_by_label("Edit Roof…").click();
    } else {
        h.get_by_label("Tileset").click();
    }
    h.run();
    assert!(
        h.state().tileset_form.is_some(),
        "roof={roof}, error={:?}",
        h.state().error
    );
}
fn pixels(h: &Harness<'_, Editor>, id: u16) -> [u8; 64] {
    h.state().tileset_form.as_ref().unwrap().draft.pixels(id)
}
fn wheel(h: &mut Harness<'_, Editor>, delta: f32) {
    let area = h.state().tileset_form.as_ref().unwrap().client_bounds;
    h.hover_at(area.min + Vec2::new(120.0, 150.0));
    h.run();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: Vec2::new(0.0, delta),
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(8);
    h.run();
}
fn menu(h: &mut Harness<'_, Editor>, item: &str) {
    h.get_by_label("File").click();
    h.run();
    h.get_by_label(item).click();
    h.run();
}

#[test]
fn raw_tile_and_roof_dialogs_match_geometry_keep_dirty_domains_and_save() {
    for roof in [false, true] {
        let (dir, mut h) = fixture();
        if roof {
            let ctx = h.ctx.clone();
            assert!(h.state_mut().apply_attribute_commands(
                &ctx,
                &[AttributeCommand::Set {
                    block: 0,
                    offsets: [16; 16],
                    attributes: [0; 16]
                }]
            ));
            h.state_mut()
                .loaded
                .as_mut()
                .unwrap()
                .plus_assets
                .as_mut()
                .unwrap()
                .blocks
                .blockset_mut()
                .mark_saved();
        }
        let source_path = dir.path().join(if roof {
            "gfx/tilesets/roofs/demo.2bpp"
        } else {
            "gfx/tilesets/demo.2bpp"
        });
        let source = fs::read(&source_path).unwrap();
        let attributes = h
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .plus_assets
            .as_ref()
            .unwrap()
            .blocks
            .blockset()
            .attribute_bytes();
        for accept in [false, true] {
            open(&mut h, roof);
            let form = h.state().tileset_form.as_ref().unwrap();
            let origin = form.client_bounds.min;
            assert_eq!(
                form.client_bounds.size(),
                if roof {
                    Vec2::new(226.0, 228.0)
                } else {
                    Vec2::new(441.0, 304.0)
                }
            );
            assert_eq!(form.selected, if roof { 6 } else { 0 });
            let pixel = h.get_by_label("Pixel 0, 0").rect();
            assert_eq!(
                pixel.min - origin,
                Vec2::new(if roof { 71.0 } else { 294.0 }, 37.0)
            );
            assert_eq!(pixel.size(), Vec2::splat(17.0));
            assert_eq!(
                h.get_by_label("Pixel 7, 7").rect().min - pixel.min,
                Vec2::splat(119.0)
            );
            assert!(h.query_by_label("Palette:").is_none());
            assert!(h.query_by_label("Priority (above sprites)").is_none());
            for windows in [false, true] {
                h.state_mut().tileset_form.as_mut().unwrap().windows_buttons = windows;
                h.run();
                let origin = h.state().tileset_form.as_ref().unwrap().client_bounds.min;
                let ok = h.get_by_label("OK").rect().min - origin;
                let cancel = h.get_by_label("Cancel").rect().min - origin;
                assert_eq!(
                    ok,
                    if roof {
                        Vec2::new(if windows { 42.0 } else { 136.0 }, 196.0)
                    } else {
                        Vec2::new(351.0, if windows { 233.0 } else { 272.0 })
                    }
                );
                assert_eq!(
                    cancel,
                    if roof {
                        Vec2::new(if windows { 136.0 } else { 42.0 }, 196.0)
                    } else {
                        Vec2::new(351.0, if windows { 272.0 } else { 233.0 })
                    }
                );
            }
            h.get_by_label("Swatch 4").click();
            h.run();
            h.get_by_label("Pixel 0, 0").click();
            h.run();
            assert_eq!(pixels(&h, if roof { 6 } else { 0 })[0], 3);
            assert!(!h.state().is_modified());
            h.get_by_label(if accept { "OK" } else { "Cancel" }).click();
            h.run();
            let loaded = h.state().loaded.as_ref().unwrap();
            let assets = loaded.plus_assets.as_ref().unwrap();
            assert_eq!(assets.tiles.model().is_modified(), accept && !roof);
            assert_eq!(assets.roof.as_ref().unwrap().is_modified(), accept && roof);
            assert!(!assets.blocks.blockset().is_modified());
            assert_eq!(assets.blocks.blockset().attribute_bytes(), attributes);
            assert!(!loaded.document.map().is_modified() && !loaded.document.map().can_undo());
            if accept {
                assert_eq!(
                    &loaded.graphics.render_block(0).unwrap().pixels[..3],
                    &assets.palettes.colors(assets.palettes.selected())[0][3]
                );
            }
        }
        menu(&mut h, if roof { "Save Roof" } else { "Save Tileset" });
        crate::save_feedback::tests::dismiss(&mut h);
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        assert!(!h.state().is_modified());
        if roof {
            let reopened = polished_map_project::plusplus::RoofDocument::open_in_project(
                dir.path(),
                "demo",
                Default::default(),
            )
            .unwrap();
            assert_eq!(reopened.tiles()[6].hues()[0], 3);
            assert!(!dir.path().join("gfx/tilesets/demo.png").exists());
        } else {
            let reopened = polished_map_project::plusplus::TilesetDocument::open_in_project(
                dir.path(),
                "demo",
                Default::default(),
            )
            .unwrap();
            assert_eq!(reopened.model().tiles()[0].hues()[0], 3);
            assert!(!dir.path().join("gfx/tilesets/roofs/demo.png").exists());
        }
        assert_eq!(fs::read(source_path).unwrap(), source);
        assert!(
            !dir.path()
                .join("gfx/tilesets/demo_palette_map.asm")
                .exists()
        );
        open(&mut h, roof);
        h.get_by_label("OK").click();
        h.run();
        assert!(
            h.state().is_modified(),
            "unchanged acceptance marks the source domain dirty"
        );
        let output = dir.path().join(if roof {
            "gfx/tilesets/roofs/demo.png"
        } else {
            "gfx/tilesets/demo.png"
        });
        fs::write(&output, b"external writer").unwrap();
        menu(&mut h, if roof { "Save Roof" } else { "Save Tileset" });
        assert!(
            h.state()
                .error
                .as_ref()
                .unwrap()
                .contains("changed on disk")
        );
        assert_eq!(fs::read(output).unwrap(), b"external writer");
        assert!(h.state().is_modified());
    }
}

#[test]
fn expanded_raw_sheet_selects_high_ids_and_clips_image_paste_in_both_bank_orders() {
    for (extended, order) in [
        (false, BankOrder::Bank1First),
        (true, BankOrder::Bank1First),
        (true, BankOrder::Bank0First),
    ] {
        let (_dir, mut h) = fixture();
        let assets = h
            .state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .plus_assets
            .as_mut()
            .unwrap();
        assets.allow_512 = extended;
        assets.order = order;
        open(&mut h, false);
        assert!(
            !h.get_by_label("Tileset tile $0:60")
                .accesskit_node()
                .is_disabled(),
            "master's tile gap must not apply to raw graphics"
        );
        wheel(&mut h, -2000.0);
        let id = if extended { 511 } else { 255 };
        let t = TileReference::from_index(id, order).unwrap();
        let label = format!("Tileset tile ${}:{:02X}", u8::from(t.bank1()), t.offset());
        h.get_by_label(&label).click();
        h.run();
        assert_eq!(h.state().tileset_form.as_ref().unwrap().selected, id);
        let source = egui::ColorImage::filled([16, 16], Color32::BLACK);
        h.event_modifiers(
            egui::Event::PasteImage(std::sync::Arc::new(source)),
            egui::Modifiers::SHIFT,
        );
        h.run();
        assert_eq!(pixels(&h, id), [3; 64]);
        if !extended {
            assert_eq!(pixels(&h, 256), [0; 64]);
        }
        h.input_mut().events.push(egui::Event::Copy);
        h.run();
        h.key_press(egui::Key::Delete);
        h.run();
        assert_eq!(pixels(&h, id), [0; 64]);
        assert!(
            matches!(h.state().tileset_form.as_ref().unwrap().draft.snapshot(id).unwrap(), Snapshot::Raw(v) if !v.defined)
        );
        h.input_mut()
            .events
            .push(egui::Event::Paste("unused OS text".into()));
        h.run();
        assert_eq!(pixels(&h, id), [3; 64]);
        assert!(
            matches!(h.state().tileset_form.as_ref().unwrap().draft.snapshot(id).unwrap(), Snapshot::Raw(v) if v.defined)
        );
        // The tile window has no occurrence palette shortcuts.
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num8);
        h.run();
        assert_eq!(pixels(&h, id), [3; 64]);
        h.get_by_label("Cancel").click();
        h.run();
        assert!(!h.state().is_modified());
        open(&mut h, false);
        assert!(h.query_by_label("Tileset tile $0:00").is_some());
        assert_eq!(h.state().tileset_form.as_ref().unwrap().selected, 0);
    }
}

#[test]
fn shared_pixel_gestures_clipboard_and_roof_order_retain_raw_semantics() {
    for roof in [false, true] {
        let (_dir, mut h) = fixture();
        open(&mut h, roof);
        let id = if roof { 6 } else { 0 };
        h.get_by_label("Swatch 4").click();
        h.run();
        h.get_by_label("Pixel 0, 0").click();
        h.run();
        h.get_by_label("Swatch 2").click();
        h.run();
        h.get_by_label("Pixel 7, 7")
            .click_modifiers(egui::Modifiers::SHIFT);
        h.run();
        assert_eq!(pixels(&h, id)[0], 3);
        assert!(pixels(&h, id)[1..].iter().all(|&hue| hue == 2));
        h.get_by_label("Swatch 3").click();
        h.run();
        h.get_by_label("Pixel 7, 7")
            .click_modifiers(egui::Modifiers::CTRL);
        h.run();
        assert!(pixels(&h, id)[1..].iter().all(|&hue| hue == 1));
        h.get_by_label("Swatch 4").click();
        h.run();
        h.get_by_label("Pixel 7, 7")
            .click_modifiers(egui::Modifiers::ALT);
        h.run();
        assert_eq!(pixels(&h, id)[0], 1);
        assert!(pixels(&h, id)[1..].iter().all(|&hue| hue == 3));
        h.input_mut().events.push(egui::Event::Copy);
        h.run();
        let other = if roof {
            "Roof tile $0:0A"
        } else {
            "Tileset tile $0:01"
        };
        h.get_by_label(other).click();
        h.run();
        let target = if roof { 0 } else { 1 };
        h.input_mut().events.push(egui::Event::Paste(String::new()));
        h.run();
        assert_eq!(pixels(&h, target), pixels(&h, id));
        if roof {
            let saved = pixels(&h, target);
            h.key_press(egui::Key::Delete);
            h.run();
            assert_eq!(
                pixels(&h, target),
                saved,
                "roof deletion remains unavailable"
            );
            let top = h.get_by_label("Roof tile $0:10").rect();
            let bottom = h.get_by_label("Roof tile $0:0A").rect();
            assert_eq!(bottom.min - top.min, Vec2::new(0.0, 32.0));
        }
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
            .find_map(|c| {
                if let egui::OutputCommand::CopyImage(i) = c {
                    Some(i)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(copied.size, [8, 8]);
        assert_eq!(copied.pixels[0].to_array(), [85, 85, 85, 255]);
        assert_eq!(copied.pixels[1].to_array(), [0, 0, 0, 255]);
        h.run();
        h.get_by_label("Cancel").click();
        h.run();
        assert!(!h.state().is_modified());
    }
}

#[test]
fn raw_clipboard_live_swap_and_drag_share_input_but_preserve_definition_state() {
    for roof in [false, true] {
        let (_dir, mut h) = fixture();
        open(&mut h, roof);
        let first = if roof { 6 } else { 0 };
        let second = if roof { 0 } else { 1 };
        let first_label = if roof {
            "Roof tile $0:10"
        } else {
            "Tileset tile $0:00"
        };
        let second_label = if roof {
            "Roof tile $0:0A"
        } else {
            "Tileset tile $0:01"
        };
        let initial = pixels(&h, first);
        h.input_mut().events.push(egui::Event::Copy);
        h.run();
        h.get_by_label("Swatch 4").click();
        h.run();
        h.get_by_label("Pixel 0, 0")
            .click_modifiers(egui::Modifiers::CTRL);
        h.run();
        assert_eq!(pixels(&h, first), [3; 64]);
        h.get_by_label(second_label).click();
        h.run();
        h.get_by_label("Swatch 2").click();
        h.run();
        h.get_by_label("Pixel 0, 0")
            .click_modifiers(egui::Modifiers::CTRL);
        h.run();
        assert_eq!(pixels(&h, second), [2; 64]);
        for _ in 0..2 {
            h.input_mut().events.push(egui::Event::Cut);
            h.run();
        }
        assert_eq!(pixels(&h, first), [2; 64]);
        assert_eq!(pixels(&h, second), [3; 64], "held swap must not swap back");
        h.input_mut().events.push(egui::Event::Paste(String::new()));
        h.run();
        assert_eq!(
            pixels(&h, second),
            initial,
            "paste retains the captured snapshot"
        );
        for (enabled, ctrl) in [(false, false), (true, false), (true, true)] {
            h.state_mut().drag_and_drop = enabled;
            let before = [pixels(&h, first), pixels(&h, second)];
            let from = h.get_by_label(first_label).rect().center();
            let to = h.get_by_label(second_label).rect().center();
            h.hover_at(from);
            h.drag_at(from);
            h.run();
            h.hover_at(to);
            h.run();
            let modifiers = if ctrl {
                egui::Modifiers::CTRL
            } else {
                egui::Modifiers::NONE
            };
            h.event_modifiers(
                egui::Event::PointerButton {
                    pos: to,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers,
                },
                modifiers,
            );
            h.run();
            assert_eq!(pixels(&h, first), before[usize::from(enabled && !ctrl)]);
            assert_eq!(pixels(&h, second), before[usize::from(!enabled)]);
        }
        h.get_by_label("Cancel").click();
        h.run();
        assert!(!h.state().is_modified());
    }
}

#[test]
fn roof_image_paste_clips_to_selected_tile_and_wrong_profile_accept_is_rejected() {
    let (_dir, mut h) = fixture();
    open(&mut h, true);
    let image = egui::ColorImage::filled([8, 24], Color32::BLACK);
    h.event_modifiers(
        egui::Event::PasteImage(std::sync::Arc::new(image)),
        egui::Modifiers::SHIFT,
    );
    h.run();
    // Roof_Window::paste_tile_graphics_cb targets only _selected. The shared
    // safe clipping policy bounds upstream's unchecked larger-image loop.
    assert_eq!(pixels(&h, 6), [3; 64]);
    for id in [0, 1, 2, 3, 4, 5, 7, 8] {
        assert_eq!(pixels(&h, id), [0; 64]);
    }
    let mut form = h.state_mut().tileset_form.take().unwrap();
    assert!(
        form.draft
            .set_snapshot(
                0,
                Snapshot::Raw(polished_map_core::plusplus::RawTileSnapshot {
                    pixels: vec![0; 64],
                    defined: false
                })
            )
            .is_err()
    );
    assert_eq!(form.draft.pixels(0), [0; 64]);
    let (_master_dir, mut master) = crate::tileset_editor_tests::fixture(false, false);
    let ctx = master.ctx.clone();
    assert!(
        master
            .state_mut()
            .accept_tileset(&ctx, &mut form)
            .unwrap_err()
            .contains("another editor profile")
    );
    assert!(!master.state().is_modified());
}

#[cfg(feature = "visual-tests")]
#[test]
fn raw_editor_pixels_and_sheet_use_monochrome_and_undefined_colors() {
    let (_dir, mut h) = fixture();
    open(&mut h, false);
    h.remove_cursor();
    h.run();
    let origin = h.get_by_label("Pixel 0, 0").rect().min;
    let raw = h.get_by_label("Tileset tile $0:00").rect().min;
    let image = h.render().unwrap();
    for position in [origin, raw] {
        assert_eq!(
            &image
                .get_pixel(position.x as u32 + 6, position.y as u32 + 6)
                .0[..3],
            &[85; 3]
        );
    }
    h.key_press(egui::Key::Delete);
    h.run();
    h.remove_cursor();
    h.run();
    let image = h.render().unwrap();
    assert_eq!(
        &image.get_pixel(origin.x as u32 + 6, origin.y as u32 + 6).0[..3],
        &[255; 3],
        "undefined raw pixels are still edited in monochrome"
    );
    assert_eq!(
        &image.get_pixel(raw.x as u32 + 6, raw.y as u32 + 6).0[..3],
        &polished_map_core::UNDEFINED_COLORS[0]
    );
}
