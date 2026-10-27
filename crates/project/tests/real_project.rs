//! Real project smoke tests use an ignored, pinned checkout; no game assets are redistributed.
use std::{collections::BTreeMap, fs, path::PathBuf};

use polished_map_project::{MapDocument, OpenOptions, discover_map, load_master_graphics};

#[test]
#[ignore = "requires POLISHED_MAP_POKECRYSTAL; run in the real-project CI job"]
fn pinned_pokecrystal_maps_discover_open_and_render() {
    let root = PathBuf::from(
        std::env::var_os("POLISHED_MAP_POKECRYSTAL").expect("set POLISHED_MAP_POKECRYSTAL"),
    );
    let mut graphics = BTreeMap::new();
    let mut opened = 0;
    let mut unresolved = Vec::new();
    for entry in fs::read_dir(root.join("maps")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "blk") {
            continue;
        }
        let meta = discover_map(&path, Some(&root)).unwrap();
        if meta.dimensions.is_none() || meta.tileset.is_none() {
            unresolved.push(path.file_name().unwrap().to_string_lossy().to_string());
            continue;
        }
        let doc = MapDocument::open_in_project(&path, &OpenOptions::default(), Some(&root))
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let tileset = meta.tileset.unwrap();
        if !graphics.contains_key(&tileset) {
            let value = load_master_graphics(&root, &tileset, false)
                .unwrap_or_else(|e| panic!("{tileset}: {e}"));
            graphics.insert(tileset.clone(), value);
        }
        let image = graphics[&tileset].render_map(doc.map()).unwrap();
        assert_eq!(image.width, u32::from(doc.map().width()) * 32);
        assert_eq!(image.height, u32::from(doc.map().height()) * 32);
        assert_eq!(fs::read(&path).unwrap(), doc.map().blocks());
        opened += 1;
    }
    unresolved.sort();
    eprintln!(
        "opened {opened} maps using {} tilesets; unresolved: {unresolved:?}",
        graphics.len()
    );
    assert_eq!(opened, 236);
    assert_eq!(graphics.len(), 35);
    // These shared files have no matching map-name metadata in the pinned project.
    // Upstream also asks the user to choose dimensions/tileset. Use a real owning
    // map for explicit options here; do not invent a unique owner in discovery.
    let shared = [
        ("DeptStore1F", "GoldenrodDeptStore1F"),
        ("DeptStore2F", "GoldenrodDeptStore2F"),
        ("DeptStore3F", "GoldenrodDeptStore3F"),
        ("DeptStore4F", "GoldenrodDeptStore4F"),
        ("DeptStore5F", "GoldenrodDeptStore5F"),
        ("DeptStore6F", "GoldenrodDeptStore6F"),
        ("DeptStoreElevator", "GoldenrodDeptStoreElevator"),
        ("EastWestGate", "Route31VioletGate"),
        ("GiftShop", "MountMoonGiftShop"),
        ("House1", "Route30BerryHouse"),
        ("House2", "VioletKylesHouse"),
        ("Mart", "CherrygroveMart"),
        ("NorthSouthGate", "Route5SaffronGate"),
        ("Pokecenter1F", "CherrygrovePokecenter1F"),
        ("PortPassage", "VermilionPortPassage"),
        ("RuinsOfAlphItemRoom", "RuinsOfAlphAerodactylItemRoom"),
        ("RuinsOfAlphPuzzleChamber", "RuinsOfAlphAerodactylChamber"),
        ("UndergroundPathEntrance", "Route6UndergroundPathEntrance"),
    ];
    assert_eq!(unresolved, shared.map(|(name, _)| format!("{name}.blk")));
    for (name, owner) in shared {
        let metadata = discover_map(&root.join(format!("maps/{owner}.blk")), Some(&root)).unwrap();
        let (width, height) = metadata.dimensions.unwrap();
        let path = root.join(format!("maps/{name}.blk"));
        let doc = MapDocument::open_in_project(
            &path,
            &OpenOptions {
                width: Some(width),
                height: Some(height),
            },
            Some(&root),
        )
        .unwrap();
        let graphics =
            load_master_graphics(&root, metadata.tileset.as_deref().unwrap(), false).unwrap();
        graphics.render_map(doc.map()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), doc.map().blocks());
    }
}
