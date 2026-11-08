//! Five-project acceptance on private temporary copies; no game assets are shipped.
use super::*;
use polished_map_core::{BlockCommand, CollisionRow, EventCommand};
use std::collections::BTreeMap;

// Copy discovery inputs and editable assets byte-for-byte. Deliberately omit
// repository metadata, build products and unrelated game graphics.
fn copy_input(path: &Path, target: &Path, originals: &mut BTreeMap<PathBuf, Vec<u8>>) {
    let metadata = fs::symlink_metadata(path).unwrap();
    assert!(
        !metadata.file_type().is_symlink(),
        "fixture symlink: {}",
        path.display()
    );
    if metadata.is_dir() {
        fs::create_dir_all(target).unwrap();
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            copy_input(&entry.path(), &target.join(entry.file_name()), originals);
        }
    } else {
        let bytes = fs::read(path).unwrap();
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, &bytes).unwrap();
        originals.insert(path.into(), bytes);
    }
}

fn copy_inputs(root: &Path, destination: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut originals = BTreeMap::new();
    for name in [
        "Makefile",
        "makefile",
        "main.asm",
        "layout.link",
        "constants",
        "data",
        "maps",
        "tilesets",
        "gfx/tilesets",
        "gfx/blocksets",
        "color",
        "MAPDATA",
        "SXY",
    ] {
        let path = root.join(name);
        if path.exists() {
            copy_input(&path, &destination.join(name), &mut originals);
        }
    }
    originals
}

fn assert_originals(originals: BTreeMap<PathBuf, Vec<u8>>) {
    for (path, bytes) in originals {
        assert_eq!(
            fs::read(&path).unwrap(),
            bytes,
            "original {}",
            path.display()
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Assets {
    blocks: Vec<u8>,
    attributes: Vec<u8>,
    collisions: Option<Vec<CollisionRow>>,
    tiles: Vec<[u8; 64]>,
    roof: Vec<[u8; 64]>,
    events: String,
}

fn assets(h: &Harness<'_, Editor>) -> Assets {
    let loaded = h.state().loaded.as_ref().unwrap();
    let events = loaded.events.as_ref().unwrap().script().render();
    if let Some(plus) = &loaded.plus_assets {
        Assets {
            blocks: plus.blocks.blockset().offsets().bytes(),
            attributes: plus.blocks.blockset().attribute_bytes(),
            collisions: plus
                .blocks
                .blockset()
                .offsets()
                .collisions()
                .map(<[_]>::to_vec),
            tiles: plus
                .tiles
                .model()
                .tiles()
                .iter()
                .map(|t| *t.hues())
                .collect(),
            roof: plus
                .roof
                .as_ref()
                .map_or_else(Vec::new, |r| r.tiles().iter().map(|t| *t.hues()).collect()),
            events,
        }
    } else {
        let blocks = loaded.blockset.as_ref().unwrap().blockset();
        let tiles = loaded.tileset.as_ref().unwrap();
        Assets {
            blocks: blocks.bytes(),
            // Classic monochrome PNGs synthesize palette attributes on import.
            // Row padding can revive unused blank slots after trim; compare the
            // attributes actually referenced by blocks, not that transient tail.
            attributes: blocks
                .bytes()
                .iter()
                .map(|&id| tiles.tileset().attributes()[usize::from(id)])
                .collect(),
            collisions: blocks.collisions().map(<[_]>::to_vec),
            tiles: tiles.tileset().tiles().iter().map(|t| *t.hues()).collect(),
            roof: tiles.roof().map_or_else(Vec::new, |r| {
                r.roof()
                    .tileset()
                    .tiles()
                    .iter()
                    .map(|t| *t.hues())
                    .collect()
            }),
            events,
        }
    }
}

fn menu(h: &mut Harness<'_, Editor>, group: &str, item: &str) {
    h.get_by_label(group).click();
    h.run();
    h.get_by_label(item).click();
    h.run();
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
}

fn save(h: &mut Harness<'_, Editor>, item: &str) {
    menu(h, "File", item);
    crate::save_feedback::tests::dismiss(h);
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
}

fn select_roof(h: &mut Harness<'_, Editor>, name: &str) {
    if name == "PalletTown" {
        assert!(
            !h.state().can_change_roof(),
            "Gen 1 fixture has no roof assets"
        );
        return;
    }
    menu(h, "Tools", "Change Roof…");
    h.get_by_label("(none)").click();
    h.run();
    h.get_by_label("new_bark").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert!(!assets(h).roof.is_empty());
}

fn edit_pixels(h: &mut Harness<'_, Editor>, roof: bool) {
    menu(
        h,
        "Tools",
        if roof {
            "Edit Roof…"
        } else {
            "Edit Tileset…"
        },
    );
    let form = h.state().tileset_form.as_ref().unwrap();
    let old = form.draft.pixels(form.selected)[0];
    h.get_by_label(if old == 3 { "Swatch 1" } else { "Swatch 4" })
        .click();
    h.run();
    h.get_by_label("Pixel 0, 0").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    save(h, if roof { "Save Roof" } else { "Save Tileset" });
}

fn edit_blocks(h: &mut Harness<'_, Editor>, before: &Assets) {
    let profile = h.state().profile();
    crate::block_workflow_tests::open_block(h, 0);
    let id = if before.blocks[0] == 0 { 1 } else { 0 };
    let label = if profile == Profile::PlusPlus {
        format!("Tile $0:{id:02X}")
    } else {
        format!("Tile ${id:02X}")
    };
    h.get_by_label(&label).click();
    h.run();
    if profile == Profile::PlusPlus {
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num8);
        h.run();
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::X);
        h.run();
    }
    h.get_by_label("Block tile 0, 0").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(assets(h).blocks[0], id);
    if profile == Profile::PlusPlus {
        assert_eq!(assets(h).attributes[0], 0x27);
    }
    if let Some(rows) = &before.collisions {
        let mut collision = rows[0].clone();
        match &mut collision {
            CollisionRow::Asm(values) => {
                values[0] = if values[0] == "FLOOR" {
                    "WALL"
                } else {
                    "FLOOR"
                }
                .into();
            }
            CollisionRow::Binary(values) => values[0] ^= 1,
        }
        let ctx = h.ctx.clone();
        assert!(h.state_mut().apply_block_commands(
            &ctx,
            &[BlockCommand::SetCollisions {
                block: 0,
                collision,
            }]
        ));
    }
    save(h, "Save Blockset");
}

fn edit_event(h: &mut Harness<'_, Editor>) {
    let events = h
        .state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .events
        .as_mut()
        .unwrap();
    let old = events.script().events().entries()[0].definition();
    let (x, y) = (old.x + 1, old.y);
    events
        .apply(&[EventCommand::Move { index: 0, x, y }])
        .unwrap();
    h.state_mut().save_events();
    crate::save_feedback::tests::dismiss(h);
}

fn edit_palette(h: &mut Harness<'_, Editor>, root: &Path) {
    // Exercise the shared palette dialog, including Gen 1's monochrome row.
    let palette = if h.state().palette_monochrome() { 8 } else { 0 };
    let slot = ColorSlot {
        set: h.state().palette_document().selected(),
        palette,
        hue: 0,
    };
    let (hex, exported) = if h.state().palette_document().color(slot).unwrap() == [18, 52, 86] {
        ("ABCDEF", [173, 206, 239])
    } else {
        ("123456", [16, 49, 82])
    };
    menu(h, "Tools", "Edit Current Palettes…");
    crate::palette_editor_tests::hex(h, hex);
    h.get_by_label("OK").click();
    h.run();
    let target = polished_map_project::prepare_palette_export(&root.join("edited.pal")).unwrap();
    h.state_mut().export_palette_target(&target).unwrap();
    crate::save_feedback::tests::dismiss(h);
    let mut colors = polished_map_project::PaletteDocument::default();
    colors.load(target.path()).unwrap();
    assert_eq!(colors.bank().color(slot).unwrap(), exported);
}

fn edit_assets(h: &mut Harness<'_, Editor>, root: &Path) -> Assets {
    let before = assets(h);
    edit_blocks(h, &before);
    edit_pixels(h, false);
    if !before.roof.is_empty() {
        edit_pixels(h, true);
    }

    edit_event(h);
    edit_palette(h, root);
    let expected = assets(h);
    assert_ne!(expected.tiles[0], before.tiles[0]);
    assert_ne!(expected.events, before.events);
    if !before.roof.is_empty() {
        assert_ne!(expected.roof, before.roof);
    }
    if before.collisions.is_some() {
        assert_ne!(expected.collisions, before.collisions);
    }
    expected
}

fn assert_assets(h: &Harness<'_, Editor>, expected: &Assets, label: &str) {
    let reopened = assets(h);
    assert!(
        reopened.blocks == expected.blocks,
        "{label}: reopened block bytes"
    );
    assert!(
        reopened.attributes == expected.attributes,
        "{label}: reopened attributes"
    );
    assert!(
        reopened.collisions == expected.collisions,
        "{label}: reopened collisions"
    );
    assert!(
        reopened.tiles == expected.tiles,
        "{label}: reopened tile pixels"
    );
    assert!(
        reopened.roof == expected.roof,
        "{label}: reopened roof pixels"
    );
    assert!(
        reopened.events == expected.events,
        "{label}: reopened event source"
    );
}

#[test]
#[ignore = "requires all five pinned project fixture environment variables"]
fn unified_pinned_projects_asset_edits_save_and_reopen_without_touching_fixtures() {
    let mut h = editor();
    for (variable, name, extension, profile) in PINNED_PROJECTS {
        let root = PathBuf::from(std::env::var_os(variable).expect("set pinned project fixture"));
        let copy = tempfile::tempdir().unwrap();
        let originals = copy_inputs(&root, copy.path());
        let path = copy.path().join(format!("maps/{name}.{extension}"));
        let map_bytes = fs::read(&path).unwrap();
        open(&mut h, &path, profile);
        select_roof(&mut h, name);
        let expected = edit_assets(&mut h, copy.path());
        open(&mut h, &path, profile);
        select_roof(&mut h, name);
        assert_assets(&h, &expected, variable);
        assert_eq!(
            fs::read(&path).unwrap(),
            map_bytes,
            "asset saves must not change map blocks"
        );
        assert_originals(originals);
        eprintln!(
            "{variable}: copied block/tile/palette/event assets saved and reopened; roof={}, collisions={}",
            !expected.roof.is_empty(),
            expected.collisions.is_some()
        );
    }
}

#[test]
#[ignore = "requires pinned POLISHED_MAP_POKECRYSTAL and POLISHED_CRYSTAL_FIXTURE"]
fn unified_pinned_projects_cross_storage_asset_saves_follow_selected_context() {
    let classic = PathBuf::from(std::env::var_os("POLISHED_MAP_POKECRYSTAL").unwrap());
    let modern = PathBuf::from(std::env::var_os("POLISHED_CRYSTAL_FIXTURE").unwrap());
    let copy = tempfile::tempdir().unwrap();
    let root = copy.path();
    let mut originals = copy_inputs(&classic, root);
    // Keep real classic assets and add a complete attributed compound tileset.
    // The synthetic map/events avoid implying that either game can mix formats.
    let attributed_files = [
        "gfx/tilesets/johto_traditional.johto_common.png",
        "gfx/tilesets/johto_common.png",
        "data/tilesets/johto_traditional_metatiles.bin",
        "data/tilesets/johto_traditional_attributes.bin",
        "data/tilesets/johto_traditional_collision.asm",
    ];
    for name in attributed_files {
        assert!(!root.join(name).exists());
        copy_input(&modern.join(name), &root.join(name), &mut originals);
    }
    let source = root.join("Mixed.2x1.johto.blk");
    fs::write(&source, [0, 1]).unwrap();
    fs::write(root.join("maps/Mixed.asm"), "\twarp_event 0, 0, MAP, 1\n").unwrap();
    let mut h = editor();
    open(&mut h, &source, Profile::Master);
    select_roof(&mut h, "NewBarkTown");
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    let classic_files = [
        "gfx/tilesets/johto.png",
        "gfx/tilesets/johto_palette_map.asm",
        "data/tilesets/johto_metatiles.bin",
        "data/tilesets/johto_collision.asm",
    ];
    for (from, to, profile, untouched) in [
        (
            "johto",
            "johto_traditional.johto_common",
            Profile::PlusPlus,
            classic_files.as_slice(),
        ),
        (
            "johto_traditional.johto_common",
            "johto",
            Profile::Master,
            attributed_files.as_slice(),
        ),
    ] {
        let previous: Vec<_> = untouched
            .iter()
            .map(|name| (root.join(name), fs::read(root.join(name)).unwrap()))
            .collect();
        choose_tileset(&mut h, from, to);
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        assert!(
            h.state().profile() == profile,
            "{from} -> {to}: status={}, dirty_guard={}, chooser={}, roof={}, warning={:?}",
            h.state().status,
            h.state().confirm_tileset_change,
            h.state().tileset_choice_form.is_some(),
            h.state().roof_choice_form.is_some(),
            h.state().open_palette_warning
        );
        assert_eq!(
            h.state().loaded.as_ref().unwrap().document.map().blocks(),
            [1, 1]
        );
        assert!(
            !assets(&h).roof.is_empty(),
            "selected roof survives storage changes"
        );
        let expected = edit_assets(&mut h, root);
        for (path, bytes) in previous {
            assert_eq!(
                fs::read(&path).unwrap(),
                bytes,
                "inactive assets {}",
                path.display()
            );
        }
        // The storage guard remains even after all companion assets are saved.
        let document = &mut h.state_mut().loaded.as_mut().unwrap().document;
        assert!(document.needs_save_as());
        assert!(document.save().is_err());
        let suggested = document.save_path_suggestion(true);
        let filename = format!(
            "Mixed.2x1.{}.{}",
            to.split('.').next().unwrap(),
            profile.extension()
        );
        assert_eq!(suggested.file_name().unwrap(), filename.as_str());
        let path = root.join("maps").join(suggested.file_name().unwrap());
        document.save_as(&path).unwrap();
        open(&mut h, &path, profile);
        select_roof(&mut h, "NewBarkTown");
        assert_assets(&h, &expected, to);
        assert_eq!(fs::read(&path).unwrap(), [1, 1]);
        assert_eq!(fs::read(&source).unwrap(), [0, 1]);
    }
    // Both outputs live under maps/, leaving the root-level source untouched.
    assert_originals(originals);
}
