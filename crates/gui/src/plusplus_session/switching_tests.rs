use super::*;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use polished_map_core::{
    ColorSlot, PaletteCommand, PaletteSet,
    plusplus::{AttributeCommand, RawTileCommand},
};
use std::fs;

fn assets<'a>(h: &'a Harness<'_, Editor>) -> &'a PlusPlusAssets {
    h.state()
        .loaded
        .as_ref()
        .unwrap()
        .plus_assets
        .as_ref()
        .unwrap()
}
fn menu(h: &mut Harness<'_, Editor>, group: &str, label: &str) {
    h.get_by_label(group).click();
    h.run();
    h.get_by_label(label).click();
    h.run();
}
fn choose(h: &mut Harness<'_, Editor>, old: &str, new: &str) {
    h.get_by_label(old).click();
    h.run();
    h.get_by_label(new).click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
}
fn add_assets(dir: &std::path::Path) {
    fs::write(dir.join("gfx/tilesets/other.2bpp"), [255; 16 * 16]).unwrap();
    fs::write(dir.join("data/tilesets/other_metatiles.bin"), [1; 16]).unwrap();
    fs::write(dir.join("data/tilesets/other_attributes.bin"), [0x67; 16]).unwrap();
    fs::write(dir.join("tilesets/other_collision.bin"), [4; 4]).unwrap();
    fs::write(dir.join("gfx/tilesets/roofs/other.2bpp"), [255; 16 * 9]).unwrap();
    fs::write(dir.join("gfx/tilesets/roofs/bad.2bpp"), [1]).unwrap();
}
fn edit_other_domains(h: &mut Harness<'_, Editor>) {
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    let a = h
        .state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .plus_assets
        .as_mut()
        .unwrap();
    a.palettes
        .apply_batch(&[PaletteCommand::SetRgb {
            slot: ColorSlot {
                set: PaletteSet::Custom,
                palette: 0,
                hue: 3,
            },
            rgb: [23, 45, 67],
        }])
        .unwrap();
}

#[test]
fn tileset_selector_preserves_map_events_colors_and_prunes_removed_blocks() {
    let (dir, mut h) = tests::fixture();
    add_assets(dir.path());
    edit_other_domains(&mut h);
    let palettes = assets(&h).palettes.content_hash();
    let events = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .events
        .as_ref()
        .unwrap()
        .script()
        .events()
        .clone();
    h.state_mut().selected = 1;
    h.state_mut().copy_block();
    h.state_mut().sidebar_reveal = Some(1);
    menu(&mut h, "Tools", "Change Tileset…");
    assert!(!h.state().confirm_tileset_change);
    assert_eq!(
        h.state()
            .tileset_choice_form
            .as_ref()
            .unwrap()
            .bounds
            .size(),
        Vec2::new(280.0, 300.0)
    );
    choose(&mut h, "demo", "other");
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    let a = assets(&h);
    assert_eq!(a.tiles.name(), "other");
    assert_eq!(a.blocks.blockset().attribute_bytes(), vec![0x67; 16]);
    assert_eq!(a.blocks.blockset().offsets().bytes(), [1; 16]);
    assert_eq!(a.roof.as_ref().unwrap().name(), "demo");
    assert_eq!(a.palettes.content_hash(), palettes);
    assert!(!a.is_modified());
    let loaded = h.state().loaded.as_ref().unwrap();
    assert!(loaded.document.map().can_undo() && loaded.document.map().is_modified());
    assert_eq!(loaded.document.map().blocks(), [1, 1]);
    assert_eq!(loaded.events.as_ref().unwrap().script().events(), &events);
    assert_eq!(h.state().selected, 0);
    assert!(h.state().block_clipboard.is_none() && h.state().sidebar_reveal.is_none());
    assert_eq!(h.state().textures.len(), 1);
    assert!(!dir.path().join("gfx/tilesets/other.png").exists());
}

#[test]
fn tileset_dirty_guards_and_failed_replacement_preserve_live_assets() {
    for domain in 0..3 {
        let (dir, mut h) = tests::fixture();
        add_assets(dir.path());
        let a = h
            .state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .plus_assets
            .as_mut()
            .unwrap();
        match domain {
            0 => a
                .blocks
                .blockset_mut()
                .apply_batch(&[AttributeCommand::Set {
                    block: 0,
                    offsets: [1; 16],
                    attributes: [8; 16],
                }])
                .unwrap(),
            1 => a
                .tiles
                .apply_batch(&[RawTileCommand::Paint {
                    tile: 0,
                    points: vec![Point { x: 0, y: 0 }],
                    hue: 3,
                }])
                .unwrap(),
            _ => a
                .roof
                .as_mut()
                .unwrap()
                .apply_batch(&[RawTileCommand::Paint {
                    tile: 0,
                    points: vec![Point { x: 0, y: 0 }],
                    hue: 3,
                }])
                .unwrap(),
        }
        let hashes = (
            assets(&h).tiles.content_hash(),
            assets(&h).blocks.blockset().attribute_bytes(),
            assets(&h).roof.as_ref().unwrap().content_hash(),
        );
        for proceed in [false, true] {
            menu(&mut h, "Tools", "Change Tileset…");
            assert!(h.state().confirm_tileset_change);
            h.get_by_label(if proceed { "Change Tileset" } else { "Cancel" })
                .click();
            h.run();
            if proceed {
                h.get_by_label("Cancel").click();
                h.run();
            }
            assert_eq!(assets(&h).tiles.content_hash(), hashes.0);
            assert_eq!(assets(&h).blocks.blockset().attribute_bytes(), hashes.1);
            assert_eq!(assets(&h).roof.as_ref().unwrap().content_hash(), hashes.2);
        }
        // A malformed replacement must not discard any accepted unsaved domain.
        fs::write(dir.path().join("gfx/tilesets/other.2bpp"), [1]).unwrap();
        menu(&mut h, "Tools", "Change Tileset…");
        h.get_by_label("Change Tileset").click();
        h.run();
        choose(&mut h, "demo", "other");
        assert!(h.state().error.is_some());
        assert_eq!(assets(&h).tiles.name(), "demo");
        assert_eq!(assets(&h).tiles.content_hash(), hashes.0);
        assert_eq!(assets(&h).blocks.blockset().attribute_bytes(), hashes.1);
        assert_eq!(assets(&h).roof.as_ref().unwrap().content_hash(), hashes.2);
        assert!(assets(&h).is_modified());
    }
}

#[test]
fn roof_switch_preserves_other_domains_retains_dirty_flag_and_removes_overlay() {
    let (dir, mut h) = tests::fixture();
    add_assets(dir.path());
    edit_other_domains(&mut h);
    let a = h
        .state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .plus_assets
        .as_mut()
        .unwrap();
    a.blocks
        .blockset_mut()
        .apply_batch(&[AttributeCommand::Set {
            block: 0,
            offsets: [10; 16],
            attributes: [0; 16],
        }])
        .unwrap();
    a.tiles
        .apply_batch(&[RawTileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        }])
        .unwrap();
    let before = (
        a.tiles.content_hash(),
        a.palettes.content_hash(),
        a.blocks.blockset().offsets().bytes().to_vec(),
    );
    menu(&mut h, "Tools", "Change Roof…");
    assert!(
        !h.state().confirm_roof_change,
        "other dirty domains must not prompt"
    );
    assert_eq!(
        h.state().roof_choice_form.as_ref().unwrap().bounds.size(),
        Vec2::new(280.0, 300.0)
    );
    choose(&mut h, "demo", "other");
    assert!(!assets(&h).roof.as_ref().unwrap().is_modified());
    assert_eq!(
        &h.state()
            .loaded
            .as_ref()
            .unwrap()
            .graphics
            .render_block(0)
            .unwrap()
            .pixels[..3],
        &[23, 45, 67]
    );
    h.state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .plus_assets
        .as_mut()
        .unwrap()
        .roof
        .as_mut()
        .unwrap()
        .apply_batch(&[RawTileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 2,
        }])
        .unwrap();
    for proceed in [false, true] {
        menu(&mut h, "Tools", "Change Roof…");
        assert!(h.state().confirm_roof_change);
        h.get_by_label(if proceed { "Change Roof" } else { "Cancel" })
            .click();
        h.run();
        if proceed {
            h.get_by_label("Cancel").click();
            h.run();
        }
        assert_eq!(assets(&h).roof.as_ref().unwrap().tiles()[0].hues()[0], 2);
    }
    menu(&mut h, "Tools", "Change Roof…");
    h.get_by_label("Change Roof").click();
    h.run();
    choose(&mut h, "other", "demo");
    assert_eq!(assets(&h).roof.as_ref().unwrap().tiles()[0].hues()[0], 0);
    assert!(
        assets(&h).roof.as_ref().unwrap().is_modified(),
        "successful replacement retains roof dirty flag"
    );
    menu(&mut h, "Tools", "Change Roof…");
    h.get_by_label("Change Roof").click();
    h.run();
    choose(&mut h, "demo", "(none)");
    assert!(assets(&h).roof.is_none() && !h.state().can_edit_roof());
    assert!(h.state().can_change_roof());
    assert_eq!(assets(&h).tiles.content_hash(), before.0);
    assert_eq!(assets(&h).palettes.content_hash(), before.1);
    assert_eq!(assets(&h).blocks.blockset().offsets().bytes(), before.2);
    assert!(h.state().loaded.as_ref().unwrap().document.map().can_undo());
    menu(&mut h, "Tools", "Change Roof…");
    choose(&mut h, "(none)", "bad");
    assert!(h.state().error.is_some() && assets(&h).roof.is_none());
    assert!(!dir.path().join("gfx/tilesets/roofs/other.png").exists());
}

#[test]
fn late_roof_read_failure_keeps_the_complete_tileset_and_preview() {
    let (dir, mut h) = tests::fixture();
    add_assets(dir.path());
    let before = assets(&h).tiles.content_hash();
    let preview = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .graphics
        .render_block(0)
        .unwrap();
    fs::write(dir.path().join("gfx/tilesets/roofs/demo.2bpp"), [1]).unwrap();
    menu(&mut h, "Tools", "Change Tileset…");
    choose(&mut h, "demo", "other");
    assert!(h.state().error.is_some());
    assert_eq!(assets(&h).tiles.content_hash(), before);
    assert_eq!(assets(&h).tiles.name(), "demo");
    assert_eq!(h.state().textures.len(), 2);
    assert_eq!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .graphics
            .render_block(0)
            .unwrap(),
        preview
    );
    assert!(!assets(&h).is_modified());
}

#[test]
fn mode_options_are_profile_specific_reload_guarded_and_bank_order_only_redraws() {
    let (dir, mut h) = tests::fixture();
    h.get_by_label("Options").click();
    h.run();
    for label in ["Monochrome", "Tile Priority", "256 Tiles"] {
        assert!(h.query_by_label(label).is_none());
    }
    assert!(
        h.get_by_label("$0:80-FF Before $1:80-FF")
            .accesskit_node()
            .is_disabled()
    );
    h.key_press(egui::Key::B);
    h.run();
    assert_eq!(h.state().bank_order, BankOrder::Bank1First);
    h.key_press(egui::Key::Escape);
    h.run();
    menu(&mut h, "Options", "512 Tiles");
    edit_other_domains(&mut h);
    let a = h
        .state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .plus_assets
        .as_mut()
        .unwrap();
    a.blocks
        .blockset_mut()
        .apply_batch(&[AttributeCommand::Set {
            block: 0,
            offsets: [0x80; 16],
            attributes: [0; 16],
        }])
        .unwrap();
    for (id, hue) in [(256, 1), (384, 2)] {
        a.tiles
            .apply_batch(&[RawTileCommand::Set {
                tile: id,
                value: polished_map_core::plusplus::RawTileSnapshot {
                    pixels: vec![hue; 64],
                    defined: true,
                },
            }])
            .unwrap();
    }
    let before = (
        a.tiles.content_hash(),
        a.blocks.blockset().attribute_bytes(),
        a.palettes.content_hash(),
    );
    for (order, hue) in [(BankOrder::Bank0First, 1), (BankOrder::Bank1First, 2)] {
        // Original B mnemonic, through the same menu row as pointer activation.
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::O);
        h.run();
        h.key_press(egui::Key::B);
        h.run();
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        assert_eq!(h.state().bank_order, order);
        assert_eq!(assets(&h).order, order);
        assert_eq!(
            &h.state()
                .loaded
                .as_ref()
                .unwrap()
                .graphics
                .render_block(0)
                .unwrap()
                .pixels[..3],
            &assets(&h).palettes.colors(PaletteSet::Custom)[0][hue]
        );
        assert_eq!(assets(&h).tiles.content_hash(), before.0);
        assert_eq!(assets(&h).blocks.blockset().attribute_bytes(), before.1);
        assert_eq!(assets(&h).palettes.content_hash(), before.2);
        assert!(!h.state().confirm_tileset_change);
    }
    menu(&mut h, "Options", "512 Tiles");
    assert!(h.state().confirm_tileset_change && !h.state().allow_512);
    h.get_by_label("Cancel").click();
    h.run();
    assert!(!h.state().allow_512 && !assets(&h).allow_512);
    assert_eq!(assets(&h).tiles.content_hash(), before.0);
    menu(&mut h, "Options", "512 Tiles");
    h.get_by_label("Change Tileset").click();
    h.run();
    assert!(h.state().allow_512 && assets(&h).allow_512);
    assert!(!assets(&h).is_modified());
    assert!(h.state().tileset_choice_form.is_none());
    assert_eq!(assets(&h).tiles.model().tiles()[0].hues()[0], 1);
    assert_eq!(assets(&h).palettes.content_hash(), before.2);
    assert!(h.state().loaded.as_ref().unwrap().document.map().can_undo());
    menu(&mut h, "Options", "512 Tiles");
    assert!(!h.state().allow_512);
    fs::write(dir.path().join("gfx/tilesets/demo.2bpp"), [255; 16 * 288]).unwrap();
    menu(&mut h, "Options", "512 Tiles");
    assert!(h.state().allow_512);
    menu(&mut h, "Options", "512 Tiles");
    assert!(
        h.state().allow_512 && assets(&h).allow_512,
        "large input re-enables option after reload"
    );
    assert_eq!(assets(&h).tiles.loaded_tile_count(), 288);
    assert!(!dir.path().join("gfx/tilesets/demo.png").exists());
}
