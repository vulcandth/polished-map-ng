//! Real project smoke tests use an ignored, pinned checkout; no game assets are redistributed.
use std::{collections::BTreeMap, fs, path::PathBuf};

use polished_map_project::{
    MapDocument, MasterGraphicsOptions, OpenOptions, discover_map, load_master_graphics,
    load_master_map_graphics,
};

#[test]
#[ignore = "requires POLISHED_MAP_POKECRYSTAL; run in the real-project CI job"]
fn pinned_contextual_palette_bank_edits_switches_and_exports_without_touching_project() {
    use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
    use polished_map_project::{
        BlocksetDocument, MasterTilesetDocument, PaletteDocument, prepare_palette_export,
    };
    let root = PathBuf::from(
        std::env::var_os("POLISHED_MAP_POKECRYSTAL").expect("set POLISHED_MAP_POKECRYSTAL"),
    );
    let doc = MapDocument::open_in_project(
        &root.join("maps/NewBarkTown.blk"),
        &OpenOptions::default(),
        Some(&root),
    )
    .unwrap();
    let mut tiles =
        MasterTilesetDocument::open_for_map(&doc, "johto", &MasterGraphicsOptions::default())
            .unwrap();
    let blocks = BlocksetDocument::open_in_project(&root, "johto").unwrap();
    let selected = tiles.palettes().selected();
    let bank = tiles.palettes().bank().clone();
    let original = tiles
        .graphics(blocks.blockset())
        .unwrap()
        .render_map(doc.map())
        .unwrap();
    let slot = ColorSlot {
        set: PaletteSet::Nite,
        palette: 6,
        hue: 2,
    };
    tiles
        .palettes_mut()
        .apply_batch(&[PaletteCommand::SetRgb {
            slot,
            rgb: [3, 127, 249],
        }])
        .unwrap();
    if selected != PaletteSet::Nite {
        assert_eq!(
            tiles
                .graphics(blocks.blockset())
                .unwrap()
                .render_map(doc.map())
                .unwrap(),
            original
        );
    }
    tiles.palettes_mut().select(PaletteSet::Nite);
    assert_eq!(tiles.colors()[6][2], [3, 127, 249]);
    assert_ne!(
        tiles
            .graphics(blocks.blockset())
            .unwrap()
            .render_map(doc.map())
            .unwrap(),
        original
    );
    assert!(!tiles.tileset().is_modified());
    assert!(!doc.map().is_modified());
    let temp = tempfile::tempdir().unwrap();
    let target = prepare_palette_export(&temp.path().join("colors.pal")).unwrap();
    tiles.palettes_mut().export(&target, false).unwrap();
    let mut exported = PaletteDocument::default();
    exported.load(target.path()).unwrap();
    for set in PaletteSet::ALL {
        for palette in 0..8 {
            for hue in 0..4 {
                let current = ColorSlot { set, palette, hue };
                assert_eq!(
                    exported.bank().color(current).unwrap(),
                    if current == slot {
                        [0, 123, 255]
                    } else {
                        bank.copy_color(current).unwrap()
                    }
                );
            }
        }
    }
    let reloaded =
        MasterTilesetDocument::open_for_map(&doc, "johto", &MasterGraphicsOptions::default())
            .unwrap();
    assert_eq!(reloaded.palettes().bank(), &bank);
    assert_eq!(
        reloaded
            .graphics(blocks.blockset())
            .unwrap()
            .render_map(doc.map())
            .unwrap(),
        original
    );
    let next_map = MapDocument::open_in_project(
        &root.join("maps/VioletCity.blk"),
        &OpenOptions::default(),
        Some(&root),
    )
    .unwrap();
    let session = tiles.palettes().clone();
    let (mut next, mut plan) = MasterTilesetDocument::begin_for_map(
        &next_map,
        "johto",
        &MasterGraphicsOptions::default(),
        &session,
    )
    .unwrap();
    while !plan.is_finished() {
        plan.advance(next.palettes_mut(), false).unwrap();
    }
    assert_eq!(
        next.palettes().bank(),
        session.bank(),
        "skipped imports retain every session set across maps"
    );
    let (mut fresh_session, mut plan) = MasterTilesetDocument::begin_for_map(
        &next_map,
        "johto",
        &MasterGraphicsOptions::default(),
        &PaletteDocument::default(),
    )
    .unwrap();
    while !plan.is_finished() {
        plan.advance(fresh_session.palettes_mut(), true).unwrap();
    }
    let fresh =
        MasterTilesetDocument::open_for_map(&next_map, "johto", &MasterGraphicsOptions::default())
            .unwrap();
    assert_eq!(fresh_session.palettes().bank(), fresh.palettes().bank());
    assert_eq!(
        fresh_session
            .graphics(blocks.blockset())
            .unwrap()
            .render_map(next_map.map())
            .unwrap(),
        fresh
            .graphics(blocks.blockset())
            .unwrap()
            .render_map(next_map.map())
            .unwrap()
    );
}

#[test]
#[ignore = "requires POLISHED_MAP_POKECRYSTAL; run in the real-project CI job"]
fn pinned_roof_edit_save_reopen_preserves_other_pixels_and_reference_source() {
    use polished_map_core::{Point, Tile, TileCommand};
    use polished_map_project::RoofDocument;
    let root = PathBuf::from(
        std::env::var_os("POLISHED_MAP_POKECRYSTAL").expect("set POLISHED_MAP_POKECRYSTAL"),
    );
    let relative = "gfx/tilesets/roofs/new_bark.png";
    let original = fs::read(root.join(relative)).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, &original).unwrap();
    let mut doc = RoofDocument::open_in_project(temp.path(), "new_bark", [0; 9]).unwrap();
    let mut expected = doc.roof().tileset().tiles().to_vec();
    let mut pixels = *expected[6].hues();
    pixels[0] = (pixels[0] + 1) % 4;
    expected[6] = Tile::new(pixels).unwrap();
    doc.roof_mut()
        .apply_batch(&[TileCommand::Paint {
            tile: 6,
            points: vec![Point { x: 0, y: 0 }],
            hue: pixels[0],
        }])
        .unwrap();
    assert!(doc.preview_save().unwrap().is_some());
    doc.save().unwrap();
    let reopened = RoofDocument::open_in_project(temp.path(), "new_bark", [0; 9]).unwrap();
    assert_eq!(reopened.roof().tileset().tiles(), expected);
    assert_eq!(reopened.content_hash(), doc.content_hash());
    assert_eq!(&fs::read(path).unwrap()[24..26], &[2, 0]);
    assert_eq!(fs::read(root.join(relative)).unwrap(), original);
}

#[test]
#[ignore = "requires POLISHED_MAP_POKECRYSTAL; run in the real-project CI job"]
fn pinned_johto_tileset_edit_save_and_reopen_preserves_other_pixels_and_sources() {
    use polished_map_core::{Point, TileCommand};
    use polished_map_project::{BlocksetDocument, MasterTilesetDocument};
    let root = PathBuf::from(
        std::env::var_os("POLISHED_MAP_POKECRYSTAL").expect("set POLISHED_MAP_POKECRYSTAL"),
    );
    let temp = tempfile::tempdir().unwrap();
    let mut originals = Vec::new();
    for relative in [
        "gfx/tilesets/johto.png",
        "gfx/tilesets/johto_palette_map.asm",
        "data/tilesets/johto_metatiles.bin",
    ] {
        let source = root.join(relative);
        let bytes = fs::read(&source).unwrap();
        let target = temp.path().join(relative);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, &bytes).unwrap();
        originals.push((source, bytes));
    }
    let options = MasterGraphicsOptions::default();
    let blocks = BlocksetDocument::open_in_project(temp.path(), "johto").unwrap();
    let mut doc = MasterTilesetDocument::open_in_project(temp.path(), "johto", &options).unwrap();
    let mut expected = doc.tileset().tiles().to_vec();
    let mut first = *expected[0].hues();
    let hue = (first[0] + 1) % 4;
    first[0] = hue;
    expected[0] = polished_map_core::Tile::new(first).unwrap();
    let palette = (doc.tileset().attributes()[0] + 1) % 8;
    doc.apply_batch(&[
        TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue,
        },
        TileCommand::Palette { tile: 0, palette },
    ])
    .unwrap();
    let preview = doc.preview_save(blocks.blockset()).unwrap();
    assert_eq!(preview.len(), 2);
    doc.save(blocks.blockset()).unwrap();
    let reopened = MasterTilesetDocument::open_in_project(temp.path(), "johto", &options).unwrap();
    assert_eq!(reopened.tileset().tiles(), expected);
    assert_eq!(reopened.tileset().attributes()[0], palette);
    assert_eq!(
        doc.content_hash(blocks.blockset()),
        reopened.content_hash(blocks.blockset())
    );
    // Validate decoding and serialization on real data, without claiming an
    // original-application differential or redistributing its assets.
    let encoded = fs::read(temp.path().join("gfx/tilesets/johto.png")).unwrap();
    assert_eq!(&encoded[24..26], &[2, 0]);
    for (path, bytes) in originals {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}

#[test]
#[ignore = "requires POLISHED_MAP_POKECRYSTAL; run in the real-project CI job"]
fn pinned_pokecrystal_roofs_load_without_changing_base_graphics() {
    let root = PathBuf::from(
        std::env::var_os("POLISHED_MAP_POKECRYSTAL").expect("set POLISHED_MAP_POKECRYSTAL"),
    );
    let doc = MapDocument::open_in_project(
        &root.join("maps/NewBarkTown.blk"),
        &OpenOptions::default(),
        Some(&root),
    )
    .unwrap();
    let base = load_master_map_graphics(&doc, "johto", &MasterGraphicsOptions::default())
        .unwrap()
        .render_map(doc.map())
        .unwrap();
    let blocks_for_switch =
        polished_map_project::BlocksetDocument::open_in_project(&root, "johto").unwrap();
    let mut switching = polished_map_project::MasterTilesetDocument::open_for_map(
        &doc,
        "johto",
        &MasterGraphicsOptions::default(),
    )
    .unwrap();
    for roof in ["azalea", "goldenrod", "new_bark", "olivine", "violet"] {
        let options = MasterGraphicsOptions {
            roof: Some(roof.into()),
            ..Default::default()
        };
        let image = load_master_map_graphics(&doc, "johto", &options)
            .unwrap()
            .render_map(doc.map())
            .unwrap();
        switching.change_roof(Some(roof)).unwrap();
        assert_eq!(
            switching
                .graphics(blocks_for_switch.blockset())
                .unwrap()
                .render_map(doc.map())
                .unwrap(),
            image
        );
        assert!(!switching.tileset().is_modified());
        assert!(!switching.roof().unwrap().roof().tileset().is_modified());
        assert_eq!((image.width, image.height), (base.width, base.height));
        if roof == "new_bark" {
            // The base johto sheet already contains New Bark Town's roof.
            assert!(
                image.pixels == base.pixels,
                "New Bark roof should match base"
            );
        } else {
            assert!(
                image.pixels != base.pixels,
                "{roof} should change the rendered roof"
            );
        }
    }
    switching.change_roof(None).unwrap();
    assert_eq!(
        switching
            .graphics(blocks_for_switch.blockset())
            .unwrap()
            .render_map(doc.map())
            .unwrap(),
        base
    );
    let reloaded = load_master_map_graphics(&doc, "johto", &MasterGraphicsOptions::default())
        .unwrap()
        .render_map(doc.map())
        .unwrap();
    assert!(
        base == reloaded,
        "loading a roof must not change a later base load"
    );
    // Edit only a temporary copy of the real blockset; never alter the fixture.
    let mut blocks =
        polished_map_project::BlocksetDocument::open_in_project(&root, "johto").unwrap();
    let source = blocks.path().to_path_buf();
    let before = fs::read(&source).unwrap();
    let collision_source = blocks.collision_path().unwrap().to_path_buf();
    let collision_before = fs::read_to_string(&collision_source).unwrap();
    let mut expected = before.clone();
    expected[..16].copy_from_slice(&before[16..32]);
    blocks
        .blockset_mut()
        .apply(&polished_map_core::BlockCommand::Copy { from: 1, to: 0 })
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let target =
        polished_map_project::prepare_blockset_save_as(&temp.path().join("copy.bin")).unwrap();
    let collision_target =
        polished_map_project::prepare_collision_save_as(&temp.path().join("copy.asm"), false)
            .unwrap();
    blocks.save_as_pair(&target, &collision_target).unwrap();
    let collision_after = fs::read_to_string(collision_target.path()).unwrap();
    assert_eq!(
        collision_after,
        collision_before.replacen("01, 01, 01, 01", "FLOOR, FLOOR, FLOOR, FLOOR", 1)
    );
    assert!(
        fs::read_to_string(collision_source).unwrap() == collision_before,
        "reference collisions must stay unchanged"
    );
    assert!(
        fs::read(target.path()).unwrap() == expected,
        "real blockset copy must preserve all other bytes"
    );
    assert!(
        fs::read(source).unwrap() == before,
        "reference assets must stay unchanged"
    );
}

#[test]
#[ignore = "requires POLISHED_MAP_POKECRYSTAL; run in the real-project CI job"]
fn pinned_pokecrystal_tileset_switch_matches_fresh_assets_and_preserves_session_colors() {
    use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
    use polished_map_project::{BlocksetDocument, MasterTilesetDocument};
    let root = PathBuf::from(
        std::env::var_os("POLISHED_MAP_POKECRYSTAL").expect("set POLISHED_MAP_POKECRYSTAL"),
    );
    let map = MapDocument::open_in_project(
        &root.join("maps/NewBarkTown.blk"),
        &OpenOptions::default(),
        Some(&root),
    )
    .unwrap();
    let options = MasterGraphicsOptions {
        roof: Some("new_bark".into()),
        ..Default::default()
    };
    let mut current = MasterTilesetDocument::open_for_map(&map, "johto", &options).unwrap();
    current
        .palettes_mut()
        .apply_batch(&[PaletteCommand::SetRgb {
            slot: ColorSlot {
                set: PaletteSet::Day,
                palette: 0,
                hue: 0,
            },
            rgb: [17, 34, 51],
        }])
        .unwrap();
    let palette_hash = current.palettes().content_hash();
    for name in ["kanto", "house", "johto"] {
        let (new, blocks) = current.prepare_replacement(name).unwrap();
        let mut fresh = MasterTilesetDocument::open_for_map(&map, name, &options).unwrap();
        *fresh.palettes_mut() = current.palettes().clone();
        let fresh_blocks = BlocksetDocument::open_in_project(&root, name).unwrap();
        assert_eq!(blocks.blockset().blocks(), fresh_blocks.blockset().blocks());
        assert_eq!(
            blocks.blockset().collisions(),
            fresh_blocks.blockset().collisions()
        );
        assert_eq!(
            new.graphics(blocks.blockset())
                .unwrap()
                .render_map(map.map())
                .unwrap(),
            fresh
                .graphics(fresh_blocks.blockset())
                .unwrap()
                .render_map(map.map())
                .unwrap()
        );
        assert_eq!(new.palettes().content_hash(), palette_hash);
        assert!(!blocks.blockset().is_modified());
        assert!(!new.tileset().is_modified());
        assert!(!new.roof().unwrap().roof().tileset().is_modified());
        current = new;
    }
}

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
        // Also exercise project-specific palette files and roof colors for every map.
        let colored = load_master_map_graphics(&doc, &tileset, &MasterGraphicsOptions::default())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        colored.render_map(doc.map()).unwrap();
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
