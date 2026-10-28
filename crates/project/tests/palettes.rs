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

#[test]
fn explicit_session_imports_overlay_only_present_sets_and_select_after_imports() {
    use polished_map_core::{ColorSlot, PaletteCommand};
    use polished_map_project::{MapPalettePlan, PaletteDocument};
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let doc = setup(root, "TOWN", "PALETTE_NITE", 0);
    let mut palettes = PaletteDocument::default();
    palettes
        .apply_batch(&[PaletteCommand::SetRgb {
            slot: ColorSlot {
                set: PaletteSet::Custom,
                palette: 0,
                hue: 0,
            },
            rgb: [17, 34, 51],
        }])
        .unwrap();
    palettes.select(PaletteSet::Custom);
    write(
        root,
        "gfx/tilesets/bg_tiles.pal",
        "\tRGB 1,1,1, 2,2,2, 3,3,3, 4,4,4\n".repeat(16),
    );
    let mut plan =
        MapPalettePlan::for_map(&doc, "demo", &MasterGraphicsOptions::default()).unwrap();
    assert!(plan.next_import_path().unwrap().ends_with("bg_tiles.pal"));
    plan.advance(&mut palettes, true).unwrap();
    assert!(!palettes.is_modified());
    assert_eq!(palettes.selected(), PaletteSet::Day);
    assert_eq!(
        palettes.bank().colors(PaletteSet::Custom)[0][0],
        [17, 34, 51]
    );
    assert_eq!(
        palettes.bank().colors(PaletteSet::Day)[0],
        [[8; 3], [24; 3], [16; 3], [33; 3]]
    );
    assert!(plan.next_import_path().is_none());
    plan.advance(&mut palettes, true).unwrap();
    assert!(plan.is_finished());
    assert_eq!(palettes.selected(), PaletteSet::Nite);
    // Fresh loads are deterministic and cannot inherit the caller's Custom edits.
    let fresh = polished_map_project::MasterTilesetDocument::open_for_map(
        &doc,
        "demo",
        &MasterGraphicsOptions::default(),
    )
    .unwrap();
    assert_ne!(
        fresh.palettes().bank().colors(PaletteSet::Custom)[0][0],
        [17, 34, 51]
    );
}

#[test]
fn failed_plan_step_can_be_retried_or_skipped_without_partial_colors_or_cursor_changes() {
    use polished_map_project::{MapPalettePlan, PaletteDocument};
    let dir = tempfile::tempdir().unwrap();
    let doc = setup(dir.path(), "TOWN", "PALETTE_AUTO", 0);
    write(
        dir.path(),
        "gfx/tilesets/bg_tiles.pal",
        "\tRGB 1,2,3, 32,0,0\n",
    );
    let mut plan =
        MapPalettePlan::for_map(&doc, "demo", &MasterGraphicsOptions::default()).unwrap();
    let mut palettes = PaletteDocument::default();
    palettes.mark_modified();
    let path = plan.next_import_path().unwrap().to_path_buf();
    let before = palettes.content_hash();
    assert!(plan.advance(&mut palettes, true).is_err());
    assert_eq!(palettes.content_hash(), before);
    assert!(palettes.is_modified());
    assert_eq!(plan.next_import_path(), Some(path.as_path()));
    plan.advance(&mut palettes, false).unwrap();
    assert!(palettes.is_modified());
    plan.advance(&mut palettes, true).unwrap();
    assert!(plan.is_finished());
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
