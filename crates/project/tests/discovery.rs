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
fn compound_save_hints_require_a_unique_current_discovery_match() {
    use polished_map_project::{MapMetadata, discover_unified_map, map_path_with_context};
    for (extra, expected) in [
        ("gfx/tilesets/johto.common.2bpp", Some("johto")),
        ("MAPDATA/johto.common.CHR", Some("johto")),
        ("gfx/tilesets/johto.other.png", None),
        ("gfx/tilesets/johto.png", None),
        ("MAPDATA/johto.other.CHR", None),
        ("gfx/tilesets/01.png", None),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, "gfx/tilesets/johto.common.png", "");
        write(
            root,
            "constants/tileset_constants.asm",
            "\tconst TILESET_JOHTO\n",
        );
        let metadata = MapMetadata {
            root: Some(root.into()),
            tileset: Some("johto.common".into()),
            available_tilesets: vec!["johto.common".into()],
            ..Default::default()
        };
        assert_eq!(metadata.filename_tileset_hint().unwrap(), Some("johto"));
        // The saved metadata is unchanged; fresh inventory must notice a newly
        // competing base or numeric alias, including the other graphics root.
        write(root, extra, "");
        let hint = metadata.filename_tileset_hint().unwrap();
        assert_eq!(hint, expected, "{extra}");
        if hint.is_some() {
            let path = map_path_with_context(&root.join("Town.ablk"), (2, 1), hint).unwrap();
            let reopened = discover_unified_map(&path, Some(root)).unwrap();
            assert_eq!(reopened.tileset, metadata.tileset);
        }
    }
}

#[test]
fn compound_save_hints_do_not_guess_missing_or_invalid_context() {
    use polished_map_project::MapMetadata;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "gfx/tilesets/2x3.common.png", "");
    for name in ["missing.common", "2x3.common", "../escape", "", "é"] {
        let metadata = MapMetadata {
            root: Some(root.into()),
            tileset: Some(name.into()),
            ..Default::default()
        };
        assert_eq!(metadata.filename_tileset_hint().unwrap(), None, "{name}");
    }
    let mut metadata = MapMetadata {
        tileset: Some("johto.common".into()),
        ..Default::default()
    };
    assert_eq!(metadata.filename_tileset_hint().unwrap(), None);
    metadata.root = Some(root.into());
    write(root, "gfx/tilesets/johto.common.png", "");
    write(root, "constants/tileset_constants.asm", [255]);
    assert!(metadata.filename_tileset_hint().is_err());
}

#[test]
fn suggested_context_paths_roundtrip_hints_without_inventing_tileset_aliases() {
    use polished_map_project::{discover_unified_map, map_path_with_context};
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "gfx/tilesets/current-tiles.2bpp", "");
    for extension in ["blk", "ablk", "BLK"] {
        let source = root.join(format!("Town.9x9.old.{extension}"));
        let suggested = map_path_with_context(&source, (2, 1), Some("current-tiles")).unwrap();
        assert_eq!(
            suggested,
            root.join(format!("Town.2x1.current-tiles.{extension}"))
        );
        let metadata = discover_unified_map(&suggested, Some(root)).unwrap();
        assert_eq!(metadata.dimensions, Some((2, 1)));
        assert_eq!(metadata.tileset.as_deref(), Some("current-tiles"));
        assert!(!suggested.exists());
        for unsupported in ["a.b", "../escape", "", "é", "3x4"] {
            let suggested = map_path_with_context(&source, (2, 1), Some(unsupported)).unwrap();
            assert_eq!(suggested, root.join(format!("Town.2x1.{extension}")));
            let metadata = discover_unified_map(&suggested, Some(root)).unwrap();
            assert_eq!(metadata.dimensions, Some((2, 1)));
            assert!(metadata.tileset.is_none());
        }
    }
    assert!(map_path_with_context(Path::new("Town.map"), (2, 1), None).is_none());
    assert!(map_path_with_context(Path::new("Town.blk"), (0, 1), None).is_none());
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

#[test]
fn legacy_db_headers_keep_the_original_whitespace_token_and_group_is_a_byte() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "gfx/tilesets/overworld.png", "");
    for (header, hint, resolved) in [
        ("\tdb OVERWORLD ; tileset\n", "overworld", Some("overworld")),
        ("\tdb OVERWORLD, 0\n", "overworld,", None),
        ("\tdb OVERWORLD,0\n", "overworld,0", None),
    ] {
        write(root, "data/mapHeaders/Town.asm", header);
        let metadata = discover_map(&root.join("Town.blk"), Some(root)).unwrap();
        assert_eq!(metadata.tileset_hint.as_deref(), Some(hint));
        assert_eq!(metadata.tileset.as_deref(), resolved);
        assert!(metadata.environment.is_empty());
    }
    for (groups, expected) in [(255, 255), (256, 0), (257, 1)] {
        let source = "\tnewgroup\n".repeat(groups) + "\tmap_const TOWN, 3, 2\n";
        let path = write(root, "constants/map_constants.asm", &source);
        let metadata = discover_map(&root.join("Town.blk"), Some(root)).unwrap();
        assert_eq!(metadata.group, expected);
        assert_eq!(metadata.dimensions, Some((3, 2)));
        assert_eq!(fs::read_to_string(path).unwrap(), source);
    }
}
