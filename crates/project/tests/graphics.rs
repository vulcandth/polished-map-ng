//! Independent source-derived vectors for master tileset.cpp composition.
use std::{fs, path::Path};

use polished_map_core::{Graphics, day_palettes};
use polished_map_project::{
    MapDocument, MasterGraphicsOptions, load_master_graphics, load_master_map_graphics,
};

fn write(root: &Path, name: &str, data: impl AsRef<[u8]>) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, data).unwrap();
}

fn setup(root: &Path) {
    // One metatile per tile ID lets us inspect every slot through public rendering.
    let blocks: Vec<_> = (0..=255).flat_map(|id| [id; 16]).collect();
    write(root, "data/tilesets/demo_metatiles.bin", blocks);
    write(
        root,
        "gfx/tilesets/demo_palette_map.asm",
        "\ttilepal 0, GREEN\n".repeat(256),
    );
}

fn pixel(graphics: &Graphics, slot: u8) -> [u8; 3] {
    graphics.render_block(slot).unwrap().pixels[..3]
        .try_into()
        .unwrap()
}

#[test]
fn compound_files_pad_rows_use_full_name_and_apply_global_palette_indices() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    setup(root);
    write(
        root,
        "gfx/tilesets/demo.before,after.2bpp",
        [255, 0].repeat(16),
    );
    write(root, "gfx/tilesets/before.2bpp", [0, 255].repeat(8));
    write(root, "gfx/tilesets/after.2bpp", [255; 16]);
    // The base graphics are deliberately different; only block/palette paths fall back.
    write(root, "gfx/tilesets/demo.2bpp", [0; 16]);
    let mut palettes = "\ttilepal 0, GREEN\n".repeat(32);
    palettes.push_str(&"\ttilepal 0, RED\n".repeat(224));
    write(root, "gfx/tilesets/demo_palette_map.asm", palettes);
    let gfx = load_master_graphics(root, "demo.before,after", true).unwrap();
    assert_eq!(pixel(&gfx, 0), day_palettes()[2][1]);
    for slot in [1, 15, 18, 31] {
        assert_eq!(pixel(&gfx, slot), [255; 3], "padding {slot}");
    }
    for slot in [16, 17] {
        assert_eq!(pixel(&gfx, slot), day_palettes()[2][2]);
    }
    assert_eq!(pixel(&gfx, 32), day_palettes()[1][3]);
    assert_eq!(pixel(&gfx, 33), day_palettes()[1][0]);
    // A full-name blockset and palette map take precedence over base-name files.
    write(
        root,
        "data/tilesets/demo.before,after_metatiles.bin",
        [16; 16],
    );
    write(
        root,
        "gfx/tilesets/demo.before,after_palette_map.asm",
        "\ttilepal 0, RED\n".repeat(256),
    );
    let gfx = load_master_graphics(root, "demo.before,after", true).unwrap();
    assert_eq!(gfx.block_count(), 1);
    assert_eq!(pixel(&gfx, 0), day_palettes()[1][2]);
}

#[test]
fn missing_companions_are_optional_but_malformed_companions_fail() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    setup(root);
    write(
        root,
        "gfx/tilesets/demo.missing,absent.2bpp",
        [255, 0].repeat(8),
    );
    let gfx = load_master_graphics(root, "demo.missing,absent", true).unwrap();
    assert_eq!(pixel(&gfx, 0), day_palettes()[2][2]);
    assert_eq!(pixel(&gfx, 1), day_palettes()[2][0]);
    write(root, "gfx/tilesets/missing.2bpp", [0]);
    assert!(load_master_graphics(root, "demo.missing,absent", true).is_err());
}

#[test]
fn compound_remapping_and_overflow_match_the_first_256_source_slots() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    setup(root);
    write(root, "gfx/tilesets/before.2bpp", [0, 255].repeat(8 * 96));
    write(
        root,
        "gfx/tilesets/demo.before,after.2bpp",
        [255, 0].repeat(8 * 128),
    );
    write(root, "gfx/tilesets/after.2bpp", [255; 16 * 64]);
    let gfx = load_master_graphics(root, "demo.before,after", false).unwrap();
    for slot in [0, 0x5f] {
        assert_eq!(pixel(&gfx, slot), day_palettes()[2][1]);
    }
    for slot in [0x80, 0xff] {
        assert_eq!(pixel(&gfx, slot), day_palettes()[2][2]);
    }
    for slot in [0x60, 0x7f] {
        assert_eq!(pixel(&gfx, slot), day_palettes()[2][3]);
    }
    let gfx = load_master_graphics(root, "demo.before,after", true).unwrap();
    assert_eq!(pixel(&gfx, 0x60), day_palettes()[2][2]);
    assert_eq!(pixel(&gfx, 0xe0), day_palettes()[2][3]);
}

#[test]
fn roof_replaces_exactly_nine_slots_and_uses_destination_palettes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    setup(root);
    write(root, "gfx/tilesets/demo.2bpp", [0, 255].repeat(8 * 32));
    write(root, "gfx/tilesets/roofs/roof.2bpp", [255, 0].repeat(8 * 9));
    let doc = MapDocument::new_map(root, 1, 1).unwrap();
    for allow_256 in [false, true] {
        let options = MasterGraphicsOptions {
            allow_256,
            roof: Some("roof".into()),
            ..Default::default()
        };
        let gfx = load_master_map_graphics(&doc, "demo", &options).unwrap();
        for slot in 0x0a..=0x12 {
            assert_eq!(pixel(&gfx, slot), day_palettes()[2][2]);
        }
        for slot in [9, 0x13] {
            assert_eq!(pixel(&gfx, slot), day_palettes()[2][1]);
        }
    }
    let options = MasterGraphicsOptions {
        roof: Some("roof".into()),
        ..Default::default()
    };
    for count in [8, 10, 256] {
        write(root, "gfx/tilesets/roofs/roof.2bpp", vec![0; count * 16]);
        assert!(load_master_map_graphics(&doc, "demo", &options).is_err());
    }
    fs::remove_file(root.join("gfx/tilesets/roofs/roof.2bpp")).unwrap();
    // Preserve upstream's padded LZ length: even a short roof expands to 256 tiles.
    write(root, "gfx/tilesets/roofs/roof.2bpp.lz", [0xff]);
    assert!(load_master_map_graphics(&doc, "demo", &options).is_err());
    let options = MasterGraphicsOptions {
        roof: Some("../demo".into()),
        ..Default::default()
    };
    assert!(load_master_map_graphics(&doc, "demo", &options).is_err());
}
