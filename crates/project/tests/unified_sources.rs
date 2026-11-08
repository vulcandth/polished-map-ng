//! Source syntax is independent of attribute storage. Fixtures are synthetic;
//! source behavior is pinned master config.cpp / tiled-image.cpp and plusplus tileset.cpp.
use polished_map_core::{
    Point,
    plusplus::{AttributeBlockset, BankOrder, RawTileCommand},
};
use polished_map_project::{
    MasterGraphicsOptions, MasterTilesetDocument, ProjectError, ProjectFormat, plusplus,
    read_unified_tiles,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn file(root: &Path, name: &str, bytes: impl AsRef<[u8]>) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}

fn chr(tiles: usize) -> String {
    // Independent bitplane vector: the first plane maps to NG internal hue 2.
    "\tdb 255,0,255,0,255,0,255,0,255,0,255,0,255,0,255,0\n".repeat(tiles)
}

#[test]
fn chr_capacity_follows_storage_and_parity_stays_binary_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = file(dir.path(), "gfx/tilesets/demo.CHR", chr(512));
    let tiles = read_unified_tiles(&path, Default::default(), ProjectFormat::PlusPlus).unwrap();
    assert_eq!(tiles.len(), 512);
    assert!(tiles.iter().all(|t| t.hues() == &[2; 64]));
    assert!(read_unified_tiles(&path, Default::default(), ProjectFormat::Master).is_err());
    assert!(plusplus::read_tiles(&path, Default::default()).is_err());
    assert!(
        plusplus::TilesetDocument::open_in_project(dir.path(), "demo", Default::default()).is_err()
    );
    fs::write(&path, chr(513)).unwrap();
    assert!(read_unified_tiles(&path, Default::default(), ProjectFormat::PlusPlus).is_err());
    for invalid in ["\tdb 256\n", "\tdb MISSING\n", "\tdb 1\n"] {
        fs::write(&path, invalid).unwrap();
        assert!(read_unified_tiles(&path, Default::default(), ProjectFormat::PlusPlus).is_err());
    }
}

#[test]
fn attributed_chr_in_mapdata_edits_to_png_without_rewriting_source() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let source = file(root, "MAPDATA/demo.CHR", chr(257));
    let mut tiles =
        plusplus::TilesetDocument::open_unified_in_project(root, "demo", Default::default())
            .unwrap();
    assert!(tiles.enables_512_tiles());
    assert_eq!(tiles.sources()[0].path(), source.canonicalize().unwrap());
    assert_eq!(
        tiles.outputs()[0].path(),
        root.canonicalize().unwrap().join("MAPDATA/demo.png")
    );
    let blocks = AttributeBlockset::from_bytes(&[0; 16], &[8; 16]).unwrap();
    tiles
        .apply_batch(&[RawTileCommand::Paint {
            tile: 256,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        }])
        .unwrap();
    tiles.save(&blocks, BankOrder::default()).unwrap();
    assert_eq!(fs::read_to_string(source).unwrap(), chr(257));
    let reopened = tiles.replacement("demo").unwrap();
    assert_eq!(reopened.model().tiles()[256].hues()[0], 3);
    assert!(!root.join("gfx/tilesets/demo.png").exists());
}

#[test]
fn selected_asset_controls_directory_and_compound_layout() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    file(root, "MAPDATA/unrelated.CHR", chr(1));
    file(root, "gfx/tilesets/demo.before,after.CHR", chr(17));
    file(root, "gfx/tilesets/before.CHR", chr(1));
    file(root, "gfx/tilesets/after.CHR", chr(2));
    let name = "demo.before,after";
    let options = MasterGraphicsOptions {
        allow_256: true,
        ..Default::default()
    };
    assert!(MasterTilesetDocument::open_in_project(root, name, &options).is_err());
    let master = MasterTilesetDocument::open_unified_in_project(root, name, &options).unwrap();
    assert_eq!(master.tileset().tiles()[16].hues(), &[2; 64]);
    let plus =
        plusplus::TilesetDocument::open_unified_in_project(root, name, Default::default()).unwrap();
    assert_eq!(plus.loaded_tile_count(), 50); // before padded to 16, main to 32, after 2
    for (index, hue) in [(0, 2), (1, 0), (16, 2), (32, 2), (33, 0), (48, 2), (49, 2)] {
        assert_eq!(plus.model().tiles()[index].hues(), &[hue; 64]);
    }
    assert!(
        plus.outputs()
            .iter()
            .all(|p| p.path().parent().unwrap().ends_with("gfx/tilesets"))
    );
    let names = polished_map_project::unified_tileset_choices(root).unwrap();
    assert!(names.iter().any(|c| c.name == "unrelated"));
    assert!(names.iter().any(|c| c.name == name));
    file(root, "gfx/tilesets/demo.before,after.2bpp", [1]);
    // A preferred malformed file is an error, not permission to try the CHR.
    assert!(
        plusplus::TilesetDocument::open_unified_in_project(root, name, Default::default()).is_err()
    );
}

#[test]
fn attributed_collision_union_preserves_parity_and_precedence() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    file(root, "data/tilesets/demo_metatiles.bin", [0; 16]);
    file(root, "data/tilesets/demo_attributes.bin", [0; 16]);
    let binary = file(root, "data/tilesets/demo_collision.bin", [1, 2, 3, 4]);
    let asm = file(
        root,
        "tilesets/demo_collision.asm",
        "\ttilecoll WALL, WALL, WALL, WALL\n",
    );
    let union = plusplus::BlocksetDocument::open_unified_in_project(root, "demo").unwrap();
    assert_eq!(
        union.collision_path().unwrap(),
        binary.canonicalize().unwrap()
    );
    let parity = plusplus::BlocksetDocument::open_in_project(root, "demo").unwrap();
    assert_eq!(
        parity.collision_path().unwrap(),
        asm.canonicalize().unwrap()
    );
    fs::write(binary, [0]).unwrap();
    assert!(plusplus::BlocksetDocument::open_unified_in_project(root, "demo").is_err());
}

#[test]
fn chr_roof_keeps_nine_tile_contract_source_and_conflict_guard() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let source = file(root, "gfx/tilesets/roofs/demo.CHR", chr(9));
    assert!(plusplus::RoofDocument::open_in_project(root, "demo", Default::default()).is_err());
    let mut roof =
        plusplus::RoofDocument::open_unified_in_project(root, "demo", Default::default()).unwrap();
    roof.apply_batch(&[RawTileCommand::Paint {
        tile: 8,
        points: vec![Point { x: 0, y: 0 }],
        hue: 3,
    }])
    .unwrap();
    fs::write(&source, chr(8)).unwrap();
    assert!(matches!(roof.save(), Err(ProjectError::Conflict(_))));
    assert!(!root.join("gfx/tilesets/roofs/demo.png").exists());
    assert!(
        plusplus::RoofDocument::open_unified_in_project(root, "demo", Default::default()).is_err()
    );
    fs::write(&source, chr(9)).unwrap();
    roof.save().unwrap();
    assert_eq!(fs::read_to_string(source).unwrap(), chr(9));
    let reopened =
        plusplus::RoofDocument::open_unified_in_project(root, "demo", Default::default()).unwrap();
    assert_eq!(reopened.tiles()[8].hues()[0], 3);
}

#[test]
fn source_path_errors_never_fall_through_to_another_layout() {
    let dir = tempfile::tempdir().unwrap();
    file(dir.path(), "gfx/tilesets", "not a directory");
    file(dir.path(), "MAPDATA/demo.CHR", chr(1));
    assert!(matches!(
        plusplus::TilesetDocument::open_unified_in_project(dir.path(), "demo", Default::default()),
        Err(ProjectError::Io { .. })
    ));
}

#[test]
fn contextual_palette_import_follows_the_selected_graphics_directory() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    file(root, "MAPDATA/attributed.CHR", chr(1));
    file(root, "gfx/tilesets/classic.CHR", chr(1));
    file(
        root,
        "gfx/tilesets/classic_palette_map.asm",
        "\ttilepal 0, GRAY\n",
    );
    for name in ["MAPDATA/attributed.pal", "gfx/tilesets/classic.pal"] {
        file(root, name, "\tRGB 31, 31, 31\n".repeat(32));
    }
    let classic = file(root, "Classic.1x1.classic.blk", [0]);
    let doc = polished_map_project::MapDocument::open_in_project(
        &classic,
        &Default::default(),
        Some(root),
    )
    .unwrap();
    let (mut tiles, mut plan) = MasterTilesetDocument::begin_unified_for_map(
        &doc,
        "classic",
        &MasterGraphicsOptions::default(),
        &Default::default(),
    )
    .unwrap();
    plan.advance(tiles.palettes_mut(), false).unwrap();
    assert_eq!(
        plan.next_import_path().unwrap(),
        root.join("gfx/tilesets/classic.pal")
    );
    let attributed = file(root, "Attributed.1x1.attributed.ablk", [0]);
    let doc = plusplus::MapDocument::open_in_project(&attributed, &Default::default(), Some(root))
        .unwrap();
    let mut plan =
        plusplus::MapPalettePlan::for_unified_map(&doc, "attributed", &Default::default()).unwrap();
    plan.advance(&mut plusplus::PaletteDocument::default(), false)
        .unwrap();
    assert_eq!(
        plan.next_import_path().unwrap(),
        root.join("MAPDATA/attributed.pal")
    );
}
