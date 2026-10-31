use polished_map_core::Command;
use polished_map_project::{OpenOptions, ProjectError, plusplus::*};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn write(root: &Path, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn discovery_keeps_roots_macros_asset_directories_and_formats_separate() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "main.asm", "");
    write(root, "nested/Makefile", "");
    let map = write(root, "nested/maps/Route10North.ablk", [7; 6]);
    assert_eq!(find_project_root(&map).as_deref(), Some(root));
    assert_eq!(
        polished_map_project::find_project_root(&map),
        Some(root.join("nested"))
    );
    write(root, "nested/layout.link", "");
    assert_eq!(find_project_root(&map), Some(root.join("nested")));
    fs::remove_file(root.join("nested/layout.link")).unwrap();
    write(
        root,
        "constants/map_constants.asm",
        "\tnewgroup\n\tmapconst ROUTE_10_NORTH, 99, 98\n\tmapgroup ROUTE_10_NORTH, 2, 3\n",
    );
    write(
        root,
        "data/maps/maps.asm",
        "map Route10North, TILESET_JOHTO, TOWN, LANDMARK_TEST, MUSIC, FALSE, PALETTE_NITE\n",
    );
    write(
        root,
        "constants/tileset_constants.asm",
        "\tconst TILESET_JOHTO\n",
    );
    for file in [
        "01.png",
        "01.2bpp",
        "02.2bpp.unique.lz",
        "03.CHR",
        "04.colored.png",
        "10.PNG",
    ] {
        write(root, &format!("gfx/tilesets/{file}"), "");
    }
    write(root, "MAPDATA/other.png", "");
    let metadata = discover_map(&map, None).unwrap();
    assert_eq!(metadata.dimensions, Some((3, 2)));
    assert_eq!(metadata.group, 1);
    assert_eq!(metadata.tileset.as_deref(), Some("01"));
    assert_eq!(metadata.available_tilesets, ["01", "02", "10"]);
    assert_eq!(metadata.landmark, "landmark_test");
    assert_eq!(metadata.palette, "PALETTE_NITE");
    assert_eq!(tileset_choices(root).unwrap()[0].label, "01: JOHTO");
    assert_eq!(
        polished_map_project::list_tilesets(root).unwrap(),
        ["other"]
    );
    for file in [
        "roof2.png",
        "roof2.2bpp",
        "roof10.png",
        "bad.PNG",
        "legacy.chr",
    ] {
        write(root, &format!("gfx/tilesets/roofs/{file}"), "");
    }
    assert_eq!(list_roofs(root).unwrap(), ["roof2", "roof10"]);
    fs::remove_file(root.join("data/maps/maps.asm")).unwrap();
    write(
        root,
        "data/maps/headers/Route10North.asm",
        "\tdb OVERWORLD\n",
    );
    assert!(discover_map(&map, None).unwrap().tileset_hint.is_none());
    assert_eq!(
        polished_map_project::discover_map(&map, Some(root))
            .unwrap()
            .tileset_hint
            .as_deref(),
        Some("overworld")
    );
    for suffix in ["ablk", "ABLK", "blk", "BLK"] {
        let hint = discover_map(
            &root.join(format!("Route10North.2x3.demo.{suffix}")),
            Some(root),
        )
        .unwrap();
        assert_eq!(hint.dimensions, Some((2, 3)));
        assert_eq!(hint.tileset_hint.as_deref(), Some("demo"));
        assert_eq!(hint.group, 0);
        assert!(hint.environment.is_empty());
    }
}

#[test]
fn binary_map_lifecycle_uses_shared_history_and_guarded_saves() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "main.asm", "");
    let path = write(root, "maps/Map.2x3.ablk", [0, 128, 255, 3, 4, 5, 99]);
    assert!(MapDocument::open(&path, &OpenOptions::default()).is_err());
    let report =
        MapDocument::open_in_project_with_report(&path, &OpenOptions::default(), None).unwrap();
    assert_eq!(report.trailing_blocks, 1);
    let mut doc = report.document;
    assert_eq!(doc.map().blocks(), [0, 128, 255, 3, 4, 5]);
    doc.map_mut()
        .apply(&Command::Replace { from: 128, to: 254 })
        .unwrap();
    assert!(doc.map_mut().undo());
    assert!(doc.map_mut().redo());
    doc.save().unwrap();
    assert_eq!(fs::read(&path).unwrap(), [0, 254, 255, 3, 4, 5]);
    let newer = write(root, "other/layout.link", "");
    let target = prepare_save_as(&newer.parent().unwrap().join("Copy")).unwrap();
    assert_eq!(target.path().extension().unwrap(), "ablk");
    doc.save_as(&target).unwrap();
    assert_eq!(
        doc.metadata().root,
        Some(fs::canonicalize(newer.parent().unwrap()).unwrap())
    );
    let options = OpenOptions {
        width: Some(2),
        height: Some(3),
    };
    assert_eq!(
        MapDocument::open(target.path(), &options)
            .unwrap()
            .map()
            .blocks(),
        doc.map().blocks()
    );
    let mut fresh = MapDocument::new_map(root, 2, 3).unwrap();
    assert!(fresh.path().ends_with("NewMap.ablk"));
    assert!(matches!(
        fresh.save(),
        Err(ProjectError::DestinationRequired)
    ));
    for suffix in ["map", "asm", "bin"] {
        let bad = write(root, &format!("wrong.{suffix}"), [0; 6]);
        assert!(MapDocument::open(&bad, &options).is_err());
        assert!(inspect_map_sizes(&bad, None).is_err());
        assert!(prepare_save_as(&bad).is_err());
        assert!(
            doc.save_as(&polished_map_project::prepare_palette_export(&bad).unwrap())
                .is_err()
        );
    }
    let hash = doc.content_hash();
    doc.map_mut()
        .apply(&Command::Replace { from: 0, to: 9 })
        .unwrap();
    fs::write(doc.path(), b"external").unwrap();
    assert!(matches!(doc.save(), Err(ProjectError::Conflict(_))));
    assert_eq!(doc.content_hash(), hash);
    assert!(doc.map().is_modified());
    let target = prepare_save_as(&root.join("existing.blk")).unwrap();
    fs::write(target.path(), b"external").unwrap();
    assert!(matches!(
        doc.save_as(&target),
        Err(ProjectError::Conflict(_))
    ));
    assert_eq!(fs::read(target.path()).unwrap(), b"external");
    let short = write(root, "Short.2x3.ablk", [0; 5]);
    assert!(
        MapDocument::open_in_project_with_report(&short, &OpenOptions::default(), None).is_err()
    );
    assert_eq!(
        inspect_map_sizes(&path, Some((2, 3))).unwrap().dimensions(),
        (2, 3)
    );
}

fn compare(root: &Path, filename: &str) {
    let map = write(root, filename, [0; 6]);
    let oracle = std::env::var_os("POLISHED_MAP_PLUSPLUS_DISCOVERY_ORACLE")
        .expect("set POLISHED_MAP_PLUSPLUS_DISCOVERY_ORACLE");
    let result = std::process::Command::new(oracle)
        .arg(root)
        .arg(&map)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let expected: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let actual = discover_map(&map, Some(root)).unwrap();
    assert_eq!(
        serde_json::json!({"constant":actual.constant,"dimensions":actual.dimensions,"group":actual.group,
        "hint":actual.tileset_hint.as_deref().unwrap_or(""),"environment":actual.environment,"landmark":actual.landmark,"palette":actual.palette}),
        expected,
        "{filename}"
    );
}

#[test]
#[ignore = "requires compiled pinned plusplus discovery oracle"]
fn metadata_matches_original_plusplus_functions() {
    for dimensions in [
        "constants/map_dimension_constants.asm",
        "constants/map_constants.asm",
        "constants/map_constants.inc",
    ] {
        for macro_name in ["map_const", "mapgroup", "mapconst"] {
            for headers in ["data/maps/maps.asm", "maps/map_headers.asm"] {
                for header_macro in ["map", "map_header"] {
                    for hint in ["TILESET_JOHTO", "1", "$a", "UNKNOWN"] {
                        let dir = tempfile::tempdir().unwrap();
                        write(
                            dir.path(),
                            dimensions,
                            format!(
                                "\tnewgroup\n{macro_name} TOWN, 99, 99\n\t{macro_name} TOWN, 3, 2\n"
                            ),
                        );
                        write(
                            dir.path(),
                            headers,
                            format!(
                                "{header_macro} Town, {hint}, TOWN, LANDMARK, MUSIC, FALSE, PALETTE_NITE\n"
                            ),
                        );
                        for file in [
                            "Town.ablk",
                            "Town.blk",
                            "Town.2x3.johto.ABLK",
                            "Town.3x2.BLK",
                        ] {
                            compare(dir.path(), file);
                        }
                    }
                }
            }
        }
    }
    for groups in [0, 255, 256, 257] {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "constants/map_constants.asm",
            "\tnewgroup\n".repeat(groups) + "\tmap_const TOWN, 3, 2\n",
        );
        compare(dir.path(), "Town.ablk");
    }
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "data/maps/maps.asm",
        "map Town, TILESET_JOHTO, TOWN, SIGN_WOOD, NEW_BARK_TOWN, MUSIC, 0, PALETTE_AUTO\n",
    );
    compare(dir.path(), "Town.ablk");
    write(
        dir.path(),
        "constants/map_dimension_constants.asm",
        "\tmap_const OTHER, 3, 2\n",
    );
    write(
        dir.path(),
        "constants/map_constants.asm",
        "\tmap_const TOWN, 3, 2\n",
    );
    write(
        dir.path(),
        "maps/map_headers.asm",
        "map Town, TILESET_OTHER, INDOOR, OTHER, MUSIC, FALSE, PALETTE_NITE\n",
    );
    compare(dir.path(), "Town.ablk");
}

#[test]
#[ignore = "requires pinned POLISHED_CRYSTAL_FIXTURE; no game assets committed"]
fn real_map_discovery_edit_save_reopen_preserves_bytes() {
    let root = std::env::var_os("POLISHED_CRYSTAL_FIXTURE")
        .map(PathBuf::from)
        .expect("set POLISHED_CRYSTAL_FIXTURE");
    let original = root.join("maps/NewBarkTown.ablk");
    let mut doc = MapDocument::open(&original, &OpenOptions::default()).unwrap();
    assert_eq!((doc.map().width(), doc.map().height()), (10, 9));
    assert_eq!(
        doc.metadata().tileset.as_deref(),
        Some("johto_traditional.johto_common")
    );
    // The pinned application's fixed field positions predate the SIGN column.
    assert_eq!(doc.metadata().landmark, "sign_wood");
    assert_eq!(doc.metadata().palette, "0");
    let tileset = doc.metadata().tileset.as_deref().unwrap();
    let mut palettes = PaletteDocument::default();
    let mut plan = MapPalettePlan::for_map(&doc, tileset, &PaletteOptions::default()).unwrap();
    while !plan.is_finished() {
        plan.advance(&mut palettes, true).unwrap();
    }
    assert_eq!(palettes.selected(), polished_map_core::PaletteSet::Day);
    let blocks = BlocksetDocument::open_in_project(&root, tileset).unwrap();
    let tiles =
        TilesetDocument::open_in_project(&root, tileset, polished_map_project::PngPlatform::Unix)
            .unwrap();
    let colors = palettes.colors(palettes.selected());
    let graphics = tiles.graphics(
        blocks.blockset(),
        colors,
        polished_map_core::plusplus::BankOrder::default(),
    );
    let pixels = graphics.render_map(doc.map()).unwrap();
    let before = fs::read(&original).unwrap();
    doc.map_mut()
        .apply(&Command::Swap { a: 0, b: 255 })
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let palette_target =
        polished_map_project::prepare_palette_export(&dir.path().join("colors.pal")).unwrap();
    palettes.export(&palette_target).unwrap();
    let mut reloaded_colors = PaletteDocument::default();
    reloaded_colors.load(palette_target.path()).unwrap();
    assert_eq!(reloaded_colors.colors(reloaded_colors.selected()), colors);
    let target = prepare_save_as(&dir.path().join("NewBarkTown.ablk")).unwrap();
    doc.save_as(&target).unwrap();
    let mut reopened =
        MapDocument::open_in_project(target.path(), &OpenOptions::default(), Some(&root)).unwrap();
    assert_eq!(reopened.map().blocks(), doc.map().blocks());
    reopened
        .map_mut()
        .apply(&Command::Swap { a: 0, b: 255 })
        .unwrap();
    reopened.save().unwrap();
    assert_eq!(fs::read(target.path()).unwrap(), before);
    assert_eq!(fs::read(original).unwrap(), before);
    assert_eq!(graphics.render_map(reopened.map()).unwrap(), pixels);
}
