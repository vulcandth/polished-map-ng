use std::{fs, path::Path};

use polished_map_core::{PaletteBank, PaletteSet};
use polished_map_project::{
    MapDocument, MasterGraphicsOptions, OpenOptions, load_master_map_graphics, parse_rgb_colors,
    parse_rgb_palettes,
};

fn write(root: &Path, path: &str, data: impl AsRef<[u8]>) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, data).unwrap();
}

fn setup(root: &Path, environment: &str, palette: &str, group: usize) -> MapDocument {
    write(root, "maps/Test.blk", [0]);
    write(
        root,
        "constants/map_constants.asm",
        format!("{}\tmap_const TEST, 1, 1\n", "\tnewgroup\n".repeat(group)),
    );
    write(
        root,
        "data/maps/maps.asm",
        format!("map Test, TILESET_DEMO, {environment}, LANDMARK_TEST, MUSIC, FALSE, {palette}\n"),
    );
    // First plane=1 gives the LIGHT hue (index2).
    write(root, "gfx/tilesets/demo.2bpp", [255, 0].repeat(8));
    write(
        root,
        "gfx/tilesets/demo_palette_map.asm",
        "\ttilepal 0, ROOF\n",
    );
    write(root, "data/tilesets/demo_metatiles.bin", [0; 16]);
    MapDocument::open_in_project(
        &root.join("maps/Test.blk"),
        &OpenOptions::default(),
        Some(root),
    )
    .unwrap()
}

fn color(doc: &MapDocument, options: &MasterGraphicsOptions) -> [u8; 3] {
    let rgb = load_master_map_graphics(doc, "demo", options)
        .unwrap()
        .render_block(0)
        .unwrap();
    rgb.pixels[..3].try_into().unwrap()
}

#[test]
fn rgb_macros_preserve_hue_order_and_reject_malformed_channels() {
    let text = "RGB 31,31,31\n\tRGB 0,1,2, 3,4,5 ; two colors\n\tRGB 6,7,8, 9,10,31\n";
    let rgb = parse_rgb_colors(text).unwrap();
    assert_eq!(rgb, [[0, 8, 16], [24, 33, 41], [49, 57, 66], [74, 82, 255]]);
    assert_eq!(
        parse_rgb_palettes(text).unwrap()[0],
        [rgb[0], rgb[2], rgb[1], rgb[3]]
    );
    for malformed in [
        "\tRGB 32,0,0",
        "\tRGB -1,0,0",
        "\tRGB $1,0,0",
        "\tRGB 1,2",
        "\tRGB 1,2,3,",
    ] {
        assert!(parse_rgb_colors(malformed).is_err(), "{malformed}");
    }
    assert!(parse_rgb_palettes("\tRGB 1,2,3").is_err());
}

#[test]
fn metadata_selection_and_explicit_overrides_match_source_precedence() {
    use PaletteSet::*;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    for (environment, palette, dark_theme, expected) in [
        ("TOWN", "PALETTE_AUTO", false, Day),
        ("INDOOR", "PALETTE_NITE", false, Nite),
        ("INDOOR", "PALETTE_MORN", false, Indoor),
        ("TOWN", "PALETTE_MORN", true, Morn),
        ("TOWN", "PALETTE_AUTO", true, Nite),
    ] {
        let doc = setup(root, environment, palette, 0);
        let options = MasterGraphicsOptions {
            dark_theme,
            ..Default::default()
        };
        assert_eq!(
            color(&doc, &options),
            PaletteBank::default().colors(expected)[6][2]
        );
        assert_eq!(
            color(
                &doc,
                &MasterGraphicsOptions {
                    palette: Some(Darkness),
                    ..options
                }
            ),
            PaletteBank::default().colors(Darkness)[6][2]
        );
    }
}

#[test]
fn special_palette_precedence_and_roof_group_colors_are_isolated() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let doc = setup(root, "TOWN", "PALETTE_AUTO", 1);
    let palette = |value: u8| {
        format!("\tRGB {value},{value},{value}, {value},{value},{value}, {value},{value},{value}, {value},{value},{value}\n").repeat(8)
    };
    write(root, "gfx/tilesets/demo.pal", palette(1));
    write(root, "gfx/tilesets/landmark_test.pal", palette(2));
    write(root, "maps/Test.pal", palette(3));
    assert_eq!(color(&doc, &MasterGraphicsOptions::default()), [24; 3]);
    // A map-specific custom set wins over automatic night/indoor selection.
    assert_eq!(
        color(
            &doc,
            &MasterGraphicsOptions {
                dark_theme: true,
                ..Default::default()
            }
        ),
        [24; 3]
    );
    fs::remove_file(root.join("maps/Test.pal")).unwrap();
    assert_eq!(color(&doc, &MasterGraphicsOptions::default()), [16; 3]);
    fs::remove_file(root.join("gfx/tilesets/landmark_test.pal")).unwrap();
    assert_eq!(color(&doc, &MasterGraphicsOptions::default()), [8; 3]);
    fs::remove_file(root.join("gfx/tilesets/demo.pal")).unwrap();
    write(
        root,
        "gfx/tilesets/roofs.pal",
        "\tRGB 0,0,0, 0,0,0, 0,0,0, 0,0,0\n\tRGB 1,2,3, 4,5,6, 7,8,9, 10,11,12\n",
    );
    assert_eq!(color(&doc, &MasterGraphicsOptions::default()), [8, 16, 24]);
    assert_eq!(
        color(
            &doc,
            &MasterGraphicsOptions {
                palette: Some(PaletteSet::Nite),
                ..Default::default()
            }
        ),
        [57, 66, 74]
    );
    let second = tempfile::tempdir().unwrap();
    let other_doc = setup(second.path(), "TOWN", "PALETTE_AUTO", 0);
    assert_eq!(
        color(&other_doc, &MasterGraphicsOptions::default()),
        PaletteBank::default().colors(PaletteSet::Day)[6][2]
    );
}
