use std::{
    fs,
    path::{Path, PathBuf},
};

use polished_map_project::{
    MapDocument, OpenOptions, discover_map, find_project_root, map_constant,
};

fn write(root: &Path, name: &str, bytes: impl AsRef<[u8]>) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn modern_project_metadata_opens_map_and_resolves_numeric_tilesets() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "Makefile", "");
    let path = write(root, "maps/Route10North.blk", [1; 6]);
    write(
        root,
        "constants/map_constants.asm",
        "\tnewgroup\nmap_const ROUTE_10_NORTH, 8, 8\n\tmap_const ROUTE_10_NORTH, 3, 2 ; supported\n",
    );
    write(
        root,
        "data/maps/maps.asm",
        "map Route10North, TILESET_JOHTO, TOWN, LANDMARK_TEST, MUSIC, FALSE, PALETTE_AUTO, FISH\n",
    );
    write(
        root,
        "constants/tileset_constants.asm",
        "\tconst_def 1\n\tconst TILESET_JOHTO\n",
    );
    write(root, "gfx/tilesets/01.png", "");
    write(root, "gfx/tilesets/01.colored.png", "");
    let doc = MapDocument::open(&path, &OpenOptions::default()).unwrap();
    assert_eq!((doc.map().width(), doc.map().height()), (3, 2));
    let meta = doc.metadata();
    assert_eq!(meta.group, 1);
    assert_eq!(meta.tileset.as_deref(), Some("01"));
    assert_eq!(meta.available_tilesets, ["01"]);
    assert_eq!(meta.environment, "TOWN");
    assert_eq!(meta.landmark, "landmark_test");
    assert_eq!(meta.palette, "PALETTE_AUTO");
    assert!(
        meta.dimensions_source
            .as_ref()
            .unwrap()
            .ends_with("constants/map_constants.asm")
    );
    assert_eq!(find_project_root(&path), Some(root.to_path_buf()));
}

#[test]
fn filename_overrides_metadata_and_explicit_dimensions_override_filename() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let path = write(root, "maps/Test.3x2.demo.BLK", [0; 6]);
    write(
        root,
        "constants/map_constants.asm",
        "\tmap_const TEST, 9, 9\n",
    );
    write(
        root,
        "data/maps/maps.asm",
        "map Test, TILESET_WRONG, CAVE\n",
    );
    write(root, "gfx/tilesets/demo.2bpp", "");
    let meta = discover_map(&path, Some(root)).unwrap();
    assert_eq!(meta.dimensions, Some((3, 2)));
    assert_eq!(meta.tileset.as_deref(), Some("demo"));
    assert!(meta.header_source.is_none());
    let doc = MapDocument::open_in_project(
        &path,
        &OpenOptions {
            width: Some(2),
            height: Some(3),
        },
        Some(root),
    )
    .unwrap();
    assert_eq!((doc.map().width(), doc.map().height()), (2, 3));
    assert_eq!(
        discover_map(&root.join("Odd.3x2.asm"), None)
            .unwrap()
            .dimensions,
        None
    );
}

#[test]
fn legacy_macros_and_path_precedence_match_upstream() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let path = root.join("maps/OldTown.blk");
    write(
        root,
        "constants/map_dimension_constants.asm",
        "\tmapgroup OLD_TOWN, 7, 9\n",
    );
    write(
        root,
        "constants/map_constants.asm",
        "\tmap_const OLD_TOWN, 1, 1\n",
    );
    write(
        root,
        "data/maps/maps.asm",
        "map_header OldTown, $a, OUTDOOR\n",
    );
    write(
        root,
        "maps/map_headers.asm",
        "map_header OldTown, TILESET_WRONG\n",
    );
    write(root, "MAPDATA/10.CHR", "");
    write(root, "gfx/tilesets/wrong.png", "");
    let meta = discover_map(&path, Some(root)).unwrap();
    assert_eq!(meta.dimensions, Some((9, 7)));
    assert_eq!(meta.tileset.as_deref(), Some("10"));
    assert_eq!(meta.available_tilesets, ["10"]);
}

#[test]
fn individual_red_headers_are_only_used_without_global_headers() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let map = root.join("maps/RedTown.blk");
    write(
        root,
        "constants/map_constants.inc",
        "\tmapconst RED_TOWN, 4, 5\n",
    );
    write(
        root,
        "data/maps/headers/RedTown.asm",
        "\tmap_header RedTown, RED_TOWN, OVERWORLD, 0\n",
    );
    write(root, "gfx/tilesets/overworld.png", "");
    let meta = discover_map(&map, Some(root)).unwrap();
    assert_eq!(meta.dimensions, Some((5, 4)));
    assert_eq!(meta.tileset.as_deref(), Some("overworld"));
    write(
        root,
        "data/maps/maps.asm",
        "; exists, but has no matching entry\n",
    );
    assert_eq!(discover_map(&map, Some(root)).unwrap().tileset, None);
}

#[test]
fn nearest_makefile_and_constant_boundaries_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "Makefile", "");
    write(dir.path(), "nested/makefile", "");
    let map = dir.path().join("nested/maps/Test.blk");
    assert_eq!(find_project_root(&map), Some(dir.path().join("nested")));
    for (input, expected) in [
        ("Route10North", "ROUTE_10_NORTH"),
        ("OlivinePokecenter1F", "OLIVINE_POKECENTER_1F"),
        ("SSAnne2F", "SS_ANNE_2F"),
        ("aB", "AB"),
        ("", ""),
    ] {
        assert_eq!(map_constant(input), expected);
    }
}
