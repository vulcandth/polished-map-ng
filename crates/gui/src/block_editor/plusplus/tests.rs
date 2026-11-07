use super::*;
use crate::plusplus_session::tests::fixture;
use egui_kittest::{Harness, kittest::Queryable};

fn blocks<'a>(h: &'a Harness<'_, Editor>) -> &'a AttributeBlockset {
    h.state()
        .loaded
        .as_ref()
        .unwrap()
        .plus_assets
        .as_ref()
        .unwrap()
        .blocks
        .blockset()
}
fn menu(h: &mut Harness<'_, Editor>, group: &str, item: &str) {
    h.get_by_label(group).click();
    h.run();
    h.get_by_label(item).click();
    h.run();
}
fn click(
    h: &mut Harness<'_, Editor>,
    name: &str,
    button: egui::PointerButton,
    modifiers: egui::Modifiers,
) {
    let pos = h.get_by_label(name).rect().center();
    h.hover_at(pos);
    for pressed in [true, false] {
        h.event_modifiers(
            egui::Event::PointerButton {
                pos,
                button,
                pressed,
                modifiers,
            },
            modifiers,
        );
        h.run();
    }
}

#[test]
fn occurrence_sheet_middle_pan_retains_selection_attributes_and_source_limits() {
    for scale in [1.0, 2.0] {
        for expanded in [false, true] {
            let (_dir, mut h) = fixture();
            h.set_pixels_per_point(scale);
            h.set_size(Vec2::new(800.0, 600.0));
            h.run();
            h.state_mut()
                .loaded
                .as_mut()
                .unwrap()
                .plus_assets
                .as_mut()
                .unwrap()
                .allow_512 = expanded;
            crate::block_workflow_tests::open_block(&mut h, 0);
            let form = h.state().block_form.as_ref().unwrap();
            let attributes = form.plus.as_ref().unwrap().attributes;
            let start = form.client_bounds.min + Vec2::new(120.75, 140.75);
            h.hover_at(start);
            h.event(egui::Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Middle,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            });
            h.run();
            // Expanded source client: 512 - 386 + box_dh(2) = 128.
            for (distance, expected) in [(1000.0, 128.0), (70.0, 70.0), (0.0, 0.0)] {
                h.hover_at(start - Vec2::new(0.0, distance));
                h.run();
                let form = h.state().block_form.as_ref().unwrap();
                let plus = form.plus.as_ref().unwrap();
                assert!(
                    (plus.scroll.offset() - if expanded { expected } else { 0.0 }).abs() < 0.01
                );
                assert_eq!(form.selected, 0);
                assert_eq!(plus.attributes, attributes);
            }
            h.event(egui::Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Middle,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            });
            h.run();
            h.hover_at(start - Vec2::new(0.0, 50.0));
            h.run();
            assert_eq!(
                h.state()
                    .block_form
                    .as_ref()
                    .unwrap()
                    .plus
                    .as_ref()
                    .unwrap()
                    .scroll
                    .offset(),
                0.0
            );
            h.get_by_label("Cancel").click();
            h.run();
            assert!(!h.state().is_modified());
        }
    }
}

#[test]
fn attributes_collisions_cancel_accept_and_save_use_shared_block_dialog() {
    let (dir, mut h) = fixture();
    h.state_mut().show_priority = false;
    h.get_by_label("Show Priority").click();
    h.run();
    assert!(
        h.state().show_priority,
        "plusplus priority is independent of master Tile Priority"
    );
    for accept in [false, true] {
        crate::block_workflow_tests::open_block(&mut h, 0);
        let form = h.state().block_form.as_ref().unwrap();
        let origin = form.client_bounds.min;
        assert_eq!(form.client_bounds.size(), Vec2::new(477.0, 432.0));
        assert_eq!(
            h.get_by_label("Block tile 0, 0").rect().min - origin,
            Vec2::new(338.0, 37.0)
        );
        assert_eq!(
            h.get_by_label("Block tile 0, 0").rect().size(),
            Vec2::splat(32.0)
        );
        assert_eq!(
            h.get_by_label("Tile $0:00").rect().min - origin,
            Vec2::new(11.0, 37.0)
        );
        assert_eq!(
            h.get_by_label("Collision top left").rect().min - origin,
            Vec2::new(313.0, 279.0)
        );
        assert_eq!(
            h.get_by_label("Collision bottom left").rect().min.y - origin.y,
            305.0
        );
        assert_eq!(h.get_by_label("OK").rect().min.y - origin.y, 400.0);
        assert_eq!(h.get_by_label("Tile palette").rect().height(), 22.0);
        assert_eq!(
            h.get_by_label("Priority (above sprites)").rect().left(),
            h.get_by_label("X flip").rect().left(),
        );
        h.get_by_label("Tile $0:01").click();
        h.run();
        if accept {
            assert_eq!(
                h.state()
                    .block_form
                    .as_ref()
                    .unwrap()
                    .plus
                    .as_ref()
                    .unwrap()
                    .brush_attributes(),
                [true; 3],
                "attribute controls persist even after Cancel"
            );
            for label in ["X flip", "Y flip", "Priority (above sprites)"] {
                h.get_by_label(label).click();
                h.run();
            }
        }
        for key in [egui::Key::Num8, egui::Key::Y, egui::Key::P] {
            h.key_press_modifiers(egui::Modifiers::ALT, key);
            h.run();
        }
        // A held Alt+X must not toggle repeatedly. Keep the first key down
        // so egui classifies subsequent key-down events as repeats.
        h.input_mut().events.push(egui::Event::Key {
            key: egui::Key::X,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::ALT,
        });
        h.run();
        h.input_mut().events.push(egui::Event::Key {
            key: egui::Key::X,
            physical_key: None,
            pressed: true,
            repeat: true,
            modifiers: egui::Modifiers::ALT,
        });
        h.run();
        h.input_mut().events.push(egui::Event::Key {
            key: egui::Key::X,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers::ALT,
        });
        h.run();
        click(
            &mut h,
            "Block tile 0, 0",
            egui::PointerButton::Primary,
            egui::Modifiers::NONE,
        );
        let form = h.state().block_form.as_ref().unwrap();
        assert_eq!(form.tiles[0], 1);
        assert_eq!(form.plus.as_ref().unwrap().attributes[0], 0xe7);
        h.get_by_label("Collision top left").focus();
        h.run();
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.run();
        h.get_by_label("Collision top left").type_text("AB");
        h.run();
        // Blur commits the spinner without triggering the dialog's Enter action.
        h.get_by_label("Collision top right").focus();
        h.run();
        assert!(
            !blocks(&h).is_modified(),
            "draft must not affect the active document"
        );
        h.get_by_label(if accept { "OK" } else { "Cancel" }).click();
        h.run();
        assert_eq!(blocks(&h).is_modified(), accept);
        assert_eq!(blocks(&h).attributes()[0][0], if accept { 0xe7 } else { 0 });
    }
    assert_eq!(
        blocks(&h).offsets().collisions().unwrap()[0],
        CollisionRow::Binary([0xab, 0, 0, 0])
    );
    assert!(
        !h.state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .map()
            .is_modified()
    );
    h.state_mut().save_blockset();
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    let reopened =
        polished_map_project::plusplus::BlocksetDocument::open_in_project(dir.path(), "demo")
            .unwrap();
    assert_eq!(
        reopened.blockset().offsets().bytes(),
        blocks(&h).offsets().bytes()
    );
    assert_eq!(
        reopened.blockset().attribute_bytes(),
        blocks(&h).attribute_bytes()
    );
    assert_eq!(
        reopened.blockset().offsets().collisions(),
        blocks(&h).offsets().collisions()
    );
}

#[test]
fn pick_high_bank_tile_reveals_selection_and_mirrors_brush_in_both_orders() {
    for order in [BankOrder::Bank1First, BankOrder::Bank0First] {
        let (_dir, mut h) = fixture();
        let ctx = h.ctx.clone();
        h.state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .plus_assets
            .as_mut()
            .unwrap()
            .order = order;
        let base = TileReference::from_index(511, order).unwrap();
        let mut offsets = [0; 16];
        offsets[0] = base.offset();
        let mut attributes = [0; 16];
        attributes[0] = base.attribute() | 0xe7;
        assert!(h.state_mut().apply_attribute_commands(
            &ctx,
            &[AttributeCommand::Set {
                block: 0,
                offsets,
                attributes
            }]
        ));
        crate::block_workflow_tests::open_block(&mut h, 0);
        assert_eq!(
            h.state()
                .block_form
                .as_ref()
                .unwrap()
                .plus
                .as_ref()
                .unwrap()
                .count,
            512,
            "a high occurrence expands the list even with the option off"
        );
        click(
            &mut h,
            "Block tile 0, 0",
            egui::PointerButton::Secondary,
            egui::Modifiers::NONE,
        );
        h.run();
        let form = h.state().block_form.as_ref().unwrap();
        assert_eq!(
            form.selected,
            511,
            "origin {:?}, chip {:?}, draft {:?}",
            form.client_bounds,
            h.get_by_label("Block tile 0, 0").rect(),
            form.tiles
        );
        let plus = form.plus.as_ref().unwrap();
        assert_eq!(plus.palette, 7);
        assert!(plus.x_flip && plus.y_flip && plus.priority);
        let label = format!("Tile ${}:FF", u8::from(base.bank1()));
        let selected = h.get_by_label(&label).rect();
        let origin = form.client_bounds.min;
        assert!(selected.top() >= origin.y + 37.0 && selected.bottom() <= origin.y + 422.0);
        // Choose 0:00, retaining the picked palette/flips/priority, then paint
        // a mirrored 2x2. Scroll first by picking an existing low tile.
        click(
            &mut h,
            "Block tile 1, 0",
            egui::PointerButton::Secondary,
            egui::Modifiers::NONE,
        );
        h.run();
        for key in [egui::Key::Num8, egui::Key::X, egui::Key::Y, egui::Key::P] {
            h.key_press_modifiers(egui::Modifiers::ALT, key);
            h.run();
        }
        click(
            &mut h,
            "Block tile 0, 0",
            egui::PointerButton::Primary,
            egui::Modifiers::CTRL,
        );
        let form = h.state().block_form.as_ref().unwrap();
        assert_eq!(
            [form.tiles[0], form.tiles[1], form.tiles[4], form.tiles[5]],
            [17, 16, 1, 0]
        );
        assert_eq!(form.plus.as_ref().unwrap().attributes[0], 0xe7);
        h.get_by_label("OK").click();
        h.run();
        assert_eq!(blocks(&h).offsets().blocks()[0][0], 17);
        assert_eq!(blocks(&h).attributes()[0][0], 0xe7);
    }
}

#[test]
fn block_clipboard_resize_drag_and_invalid_batches_keep_attributes_and_map_separate() {
    let (_dir, mut h) = fixture();
    let ctx = h.ctx.clone();
    assert!(h.state_mut().apply_attribute_commands(
        &ctx,
        &[
            AttributeCommand::Set {
                block: 0,
                offsets: [0x80; 16],
                attributes: [0xef; 16]
            },
            AttributeCommand::SetCollisions {
                block: 0,
                collision: CollisionRow::Binary([0x12; 4])
            },
        ]
    ));
    menu(&mut h, "Edit", "Copy Block");
    h.state_mut().selected = 1;
    menu(&mut h, "Edit", "Swap Block");
    assert_eq!(blocks(&h).attributes()[0], [1; 16]);
    assert_eq!(blocks(&h).attributes()[1], [0xef; 16]);
    h.state_mut().selected = 0;
    menu(&mut h, "Edit", "Paste Block");
    assert_eq!(blocks(&h).attributes()[0], [0xef; 16]);
    assert_eq!(
        blocks(&h).offsets().collisions().unwrap()[0],
        CollisionRow::Binary([0x12; 4])
    );
    assert!(h.state_mut().apply_attribute_commands(
        &ctx,
        &[AttributeCommand::Set {
            block: 1,
            offsets: [1; 16],
            attributes: [2; 16]
        }]
    ));
    // Actual sidebar drag swaps, Ctrl-drag copies all occurrence attributes.
    for ctrl in [false, true] {
        let from = Pos2::new(16.0, 63.0);
        let to = Pos2::new(48.0, 63.0);
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
        assert_eq!(blocks(&h).attributes()[0], [2; 16]);
        assert_eq!(
            blocks(&h).attributes()[1],
            [if ctrl { 2 } else { 0xef }; 16]
        );
    }
    h.state_mut().selected = 1;
    menu(&mut h, "Edit", "Copy Block");
    h.get_by_label("Resize blockset").click();
    h.run();
    h.get_by_label("Block count").focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Block count").type_text("1");
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(blocks(&h).attributes().len(), 1);
    assert!(h.state().block_clipboard.is_none());
    assert_eq!(h.state().selected, 0);
    let before = blocks(&h).attribute_bytes();
    assert!(!h.state_mut().apply_attribute_commands(
        &ctx,
        &[
            AttributeCommand::Set {
                block: 0,
                offsets: [5; 16],
                attributes: [7; 16]
            },
            AttributeCommand::Resize { count: 0 },
        ]
    ));
    assert_eq!(blocks(&h).attribute_bytes(), before);
    h.state_mut().error = None;
    assert!(!h.state_mut().apply_block_commands(
        &ctx,
        &[BlockCommand::Set {
            block: 0,
            tiles: [4; 16]
        }]
    ));
    assert_eq!(blocks(&h).attribute_bytes(), before);
    assert_eq!(h.state().textures.len(), 1);
    let map = h.state().loaded.as_ref().unwrap().document.map();
    assert_eq!(map.blocks(), [0, 1]);
    assert!(!map.is_modified() && !map.can_undo());
}

#[test]
fn asm_collision_fields_and_attribute_hotkeys_keep_text_input_separate() {
    use egui_kittest::kittest::NodeT;
    let (_dir, mut h) = fixture();
    crate::block_workflow_tests::open_block(&mut h, 0);
    for enabled in [false, true] {
        h.state_mut().block_form.as_mut().unwrap().collision =
            enabled.then(|| CollisionRow::Asm(std::array::from_fn(|_| "FLOOR".into())));
        h.run();
        let origin = h.state().block_form.as_ref().unwrap().client_bounds.min;
        for (i, name) in [
            "Collision top left",
            "Collision top right",
            "Collision bottom left",
            "Collision bottom right",
        ]
        .into_iter()
        .enumerate()
        {
            let node = h.get_by_label(name);
            assert_eq!(
                node.rect().min - origin,
                Vec2::new(313.0, 279.0 + 26.0 * i as f32)
            );
            assert_eq!(node.rect().size(), Vec2::new(154.0, 22.0));
            assert_eq!(node.accesskit_node().is_disabled(), !enabled);
        }
    }
    h.get_by_label("Collision top left").focus();
    h.run();
    h.input_mut().events.extend([
        egui::Event::Key {
            key: egui::Key::X,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::ALT,
        },
        egui::Event::Text("≈".into()),
    ]);
    h.run();
    let form = h.state().block_form.as_ref().unwrap();
    assert!(form.plus.as_ref().unwrap().x_flip);
    assert_eq!(
        form.collision,
        Some(CollisionRow::Asm(std::array::from_fn(|_| "FLOOR".into())))
    );
    h.get_by_label("Cancel").click();
    h.run();
    assert!(!blocks(&h).is_modified());
}

#[cfg(feature = "visual-tests")]
#[test]
fn rendered_block_atlas_preserves_palette_bank_flips_and_roof_pixels() {
    use polished_map_core::{ColorSlot, PaletteCommand, plusplus::RawTileCommand};
    let (_dir, mut h) = fixture();
    let assets = h
        .state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .plus_assets
        .as_mut()
        .unwrap();
    let mut pixels = vec![1; 64];
    pixels[0] = 0;
    pixels[7] = 2;
    pixels[56] = 3;
    assets
        .tiles
        .apply_batch(&[RawTileCommand::Set {
            tile: 511,
            value: polished_map_core::plusplus::RawTileSnapshot {
                pixels,
                defined: true,
            },
        }])
        .unwrap();
    let set = assets.palettes.selected();
    let colors = [[11, 12, 13], [21, 22, 23], [31, 32, 33], [41, 42, 43]];
    for (hue, rgb) in colors.into_iter().enumerate() {
        assets
            .palettes
            .apply_batch(&[PaletteCommand::SetRgb {
                slot: ColorSlot {
                    set,
                    palette: 7,
                    hue: hue as u8,
                },
                rgb,
            }])
            .unwrap();
    }
    let ctx = h.ctx.clone();
    let mut offsets = [255; 16];
    offsets[1] = 10;
    assert!(h.state_mut().apply_attribute_commands(
        &ctx,
        &[AttributeCommand::Set {
            block: 0,
            offsets,
            attributes: [0x67; 16]
        }]
    ));
    crate::block_workflow_tests::open_block(&mut h, 0);
    h.remove_cursor();
    h.run();
    let origin = h.get_by_label("Block tile 0, 0").rect().min;
    let roof = h.get_by_label("Block tile 1, 0").rect().min;
    let image = h.render().unwrap();
    for (x, y, rgb) in [
        (2, 2, colors[1]),
        (30, 2, colors[3]),
        (2, 30, colors[2]),
        (30, 30, colors[0]),
    ] {
        assert_eq!(
            &image.get_pixel(origin.x as u32 + x, origin.y as u32 + y).0[..3],
            &rgb
        );
    }
    // The fixture roof has hue zero, overriding the base sheet's hue one.
    assert_eq!(
        &image.get_pixel(roof.x as u32 + 12, roof.y as u32 + 12).0[..3],
        &colors[0]
    );
}
