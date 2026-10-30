//! Binary-map metadata observations from unchanged master discovery functions.
use polished_map_project::discover_map;
use serde_json::json;
use std::{fs, path::Path, process::Command};

fn write(root: &Path, name: &str, contents: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn compare(root: &Path, filename: &str) {
    let map = root.join(filename);
    fs::write(&map, [0; 6]).unwrap();
    let oracle = std::env::var_os("POLISHED_MAP_DISCOVERY_ORACLE")
        .expect("set POLISHED_MAP_DISCOVERY_ORACLE");
    let output = Command::new(oracle).arg(root).arg(&map).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let actual = discover_map(&map, Some(root)).unwrap();
    assert_eq!(
        json!({"constant": actual.constant, "dimensions": actual.dimensions, "group": actual.group,
        "hint": actual.tileset_hint.as_deref().unwrap_or(""), "environment": actual.environment,
        "landmark": actual.landmark, "palette": actual.palette}),
        expected,
        "{filename}, root {}",
        root.display()
    );
}

#[test]
#[ignore = "requires POLISHED_MAP_DISCOVERY_ORACLE; run in reference CI"]
fn metadata_profiles_match_original_source() {
    for dimensions in [
        "constants/map_dimension_constants.asm",
        "constants/map_constants.asm",
        "constants/map_constants.inc",
    ] {
        for (macro_name, wh) in [
            ("map_const", "3, 2"),
            ("mapgroup", "2, 3"),
            ("mapconst", "2, 3"),
        ] {
            for headers in ["data/maps/maps.asm", "maps/map_headers.asm"] {
                for header_macro in ["map", "map_header"] {
                    for tileset in ["TILESET_JOHTO", "1", "01", "$a", "UNKNOWN"] {
                        let dir = tempfile::tempdir().unwrap();
                        write(
                            dir.path(),
                            dimensions,
                            &format!(
                                "\tnewgroup\n{macro_name} TOWN, 99, 99\n\t{macro_name} TOWN, {wh} ; size\n"
                            ),
                        );
                        write(
                            dir.path(),
                            headers,
                            &format!(
                                "; unrelated\nmap Other, TILESET_OTHER\n\t{header_macro} Town, {tileset}, TOWN, LANDMARK_TEST, MUSIC, FALSE, PALETTE_AUTO, FISH ; header\n"
                            ),
                        );
                        compare(dir.path(), "Town.blk");
                        compare(dir.path(), "Town.2x3.johto.BLK");
                    }
                }
            }
        }
    }
    for headers in ["data/maps/headers/Town.asm", "data/mapHeaders/Town.asm"] {
        for header in [
            "\tmap_header Town, TOWN, OVERWORLD, 0\n",
            "\tdb OVERWORLD ; tileset\n",
            "\tdb OVERWORLD, 0\n",
            "\tdb OVERWORLD,0\n",
            "\tmap_header Town, TOWN, TILESET_OVERWORLD, 0\n",
            "\tdb 1\n",
        ] {
            let dir = tempfile::tempdir().unwrap();
            write(dir.path(), headers, header);
            compare(dir.path(), "Town.blk");
        }
    }
    for groups in [0, 1, 255, 256, 257] {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "constants/map_constants.asm",
            &("\tnewgroup\n".repeat(groups) + "\tmap_const TOWN, 3, 2\n"),
        );
        compare(dir.path(), "Town.blk");
    }
    // Each existing file wins even when it has no matching map; do not search
    // lower-priority profiles for a more convenient match.
    let dir = tempfile::tempdir().unwrap();
    for (path, source) in [
        (
            "constants/map_dimension_constants.asm",
            "\tmap_const OTHER, 7, 7\n",
        ),
        ("constants/map_constants.asm", "\tmap_const TOWN, 3, 2\n"),
        ("constants/map_constants.inc", "\tmapconst TOWN, 1, 6\n"),
        ("data/maps/maps.asm", "map Other, TILESET_OTHER\n"),
        (
            "maps/map_headers.asm",
            "map_header Town, $a, TOWN, Landmark, MUSIC, FALSE, PALETTE_DAY\n",
        ),
        (
            "data/maps/headers/Town.asm",
            "\tmap_header Town, TOWN, JOHTO, 0\n",
        ),
        ("data/mapHeaders/Town.asm", "\tdb OVERWORLD\n"),
    ] {
        write(dir.path(), path, source);
    }
    compare(dir.path(), "Town.blk");
    for path in [
        "constants/map_dimension_constants.asm",
        "constants/map_constants.asm",
        "data/maps/maps.asm",
        "maps/map_headers.asm",
        "data/maps/headers/Town.asm",
    ] {
        fs::remove_file(dir.path().join(path)).unwrap();
        compare(dir.path(), "Town.blk");
    }
    for (name, constant) in [
        ("Town", "TOWN"),
        ("town", "TOWN"),
        ("SSAnne2F", "SS_ANNE_2F"),
        ("aB", "AB"),
        ("Route10North", "ROUTE_10_NORTH"),
        ("A-B_2", "A_B_2"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "constants/map_constants.asm",
            &format!("\tmap_const {constant}, 3, 2\n"),
        );
        write(
            dir.path(),
            "data/maps/maps.asm",
            &format!(
                "map {}, TILESET_JOHTO, ROUTE, LANDMARK, MUSIC, FALSE, PALETTE_NITE\n",
                name.to_ascii_uppercase()
            ),
        );
        compare(dir.path(), &format!("{name}.blk"));
    }
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "constants/map_constants.asm",
        "\tmap_const TOWN, 3, 2\n",
    );
    write(
        dir.path(),
        "data/maps/maps.asm",
        "map Town, TILESET_JOHTO, TOWN\n",
    );
    write(
        dir.path(),
        "data/maps/blocks.asm",
        "Town_Blocks:\nOther_Blocks:\n\tINCBIN \"Shared.blk\"\n",
    );
    // Upstream does not follow INCBIN references to attribute shared block files.
    compare(dir.path(), "Shared.blk");
    // The first matching header wins, even when its tileset token is unresolved.
    write(
        dir.path(),
        "data/maps/maps.asm",
        "map Town, UNKNOWN, CAVE\nmap Town, TILESET_JOHTO, TOWN\n",
    );
    compare(dir.path(), "Town.blk");
}
