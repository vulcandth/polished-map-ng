use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{
    ColorSlot, PaletteSet,
    plusplus::{AttributeCommand, RawTileCommand},
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

mod lifecycle;

fn editor() -> Harness<'static, Editor> {
    Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new_plusplus(cc))
}
fn wait(h: &mut Harness<'_, Editor>) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while h.state().loading.is_some() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
        h.step();
    }
    h.run();
    assert!(h.state().loading.is_none(), "load worker timed out");
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
}
fn open(h: &mut Harness<'_, Editor>, path: &Path, roof: bool) {
    h.state_mut().prepare_open(path.into());
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    if roof {
        h.state_mut().open_form.as_mut().unwrap().roof = "demo".into();
    }
    h.run();
    h.key_press(egui::Key::Enter);
    h.step();
    wait(h);
}
pub(crate) fn fixture() -> (tempfile::TempDir, Harness<'static, Editor>) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for name in ["gfx/tilesets/roofs", "data/tilesets", "maps", "tilesets"] {
        fs::create_dir_all(root.join(name)).unwrap();
    }
    fs::write(root.join("main.asm"), "; plusplus root\n").unwrap();
    fs::write(root.join("Map.2x1.demo.ablk"), [0, 1]).unwrap();
    fs::write(root.join("Next.2x1.demo.ablk"), [1, 0]).unwrap();
    fs::write(root.join("maps/Map.asm"), "\tpokemon_event 0, 0, PIKACHU\n").unwrap();
    fs::write(root.join("gfx/tilesets/demo.2bpp"), [0, 255].repeat(8 * 16)).unwrap();
    fs::write(root.join("gfx/tilesets/roofs/demo.2bpp"), [0; 16 * 9]).unwrap();
    fs::write(
        root.join("data/tilesets/demo_metatiles.bin"),
        [[0; 16], [1; 16]].concat(),
    )
    .unwrap();
    fs::write(
        root.join("data/tilesets/demo_attributes.bin"),
        [[0; 16], [1; 16]].concat(),
    )
    .unwrap();
    fs::write(root.join("tilesets/demo_collision.bin"), [0; 8]).unwrap();
    fs::write(
        root.join("gfx/tilesets/bg_tiles.pal"),
        "\tRGB 31,31,31\n\tRGB 16,16,16\n\tRGB 8,8,8\n\tRGB 0,0,0\n".repeat(8),
    )
    .unwrap();
    let mut h = editor();
    open(&mut h, &root.join("Map.2x1.demo.ablk"), true);
    let loaded = h.state().loaded.as_ref().unwrap();
    assert!(loaded.plus_assets.is_some() && loaded.tileset.is_none() && loaded.blockset.is_none());
    assert_eq!(
        loaded.events.as_ref().unwrap().script().events().entries()[0]
            .definition()
            .kind,
        polished_map_core::EventKind::PokemonEvent
    );
    (dir, h)
}
fn edit_color(h: &mut Harness<'_, Editor>, value: &str, accept: bool) {
    h.get_by_label("Edit current palettes").click();
    h.run();
    assert!(h.query_by_label("MONOCHROME color 1").is_none());
    assert_eq!(
        h.state()
            .palette_form
            .as_ref()
            .unwrap()
            .client_bounds
            .height(),
        191.0
    );
    h.get_by_label("GRAY color 3").click();
    h.run();
    h.get_by_label("Hex color").focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Hex color").type_text(value);
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    h.get_by_label(if accept { "OK" } else { "Cancel" }).click();
    h.run();
}

#[test]
fn plusplus_open_palette_controls_preview_export_and_closed_session_are_shared() {
    let (dir, mut h) = fixture();
    let set = h.state().palette_document().selected();
    let before = h.state().palette_document().color_snapshot();
    edit_color(&mut h, "123456", false);
    assert_eq!(h.state().palette_document().color_snapshot(), before);
    edit_color(&mut h, "123456", true);
    assert_eq!(
        h.state()
            .palette_document()
            .color(ColorSlot {
                set,
                palette: 0,
                hue: 1
            })
            .unwrap(),
        [18, 52, 86]
    );
    assert_eq!(
        &h.state()
            .loaded
            .as_ref()
            .unwrap()
            .graphics
            .render_block(0)
            .unwrap()
            .pixels[..3],
        &[18, 52, 86]
    );
    assert!(h.state().palette_document().is_modified());
    assert!(
        !h.state().is_modified(),
        "palette edits have their own export domain"
    );
    assert_eq!(h.state().palette_document().colors(set).len(), 8);
    assert!(
        h.state()
            .palette_document()
            .color(ColorSlot {
                set,
                palette: 8,
                hue: 0
            })
            .is_err()
    );
    let target =
        polished_map_project::prepare_palette_export(&dir.path().join("colors.pal")).unwrap();
    h.state_mut().export_palette_target(&target).unwrap();
    crate::save_feedback::tests::dismiss(&mut h);
    let text = fs::read_to_string(target.path()).unwrap();
    assert!(!text.contains("MONOCHROME"));
    let mut reopened = plusplus::PaletteDocument::default();
    reopened.load(target.path()).unwrap();
    assert_eq!(
        reopened
            .color(ColorSlot {
                set,
                palette: 0,
                hue: 1
            })
            .unwrap(),
        [16, 49, 82]
    );
    let colors = h.state().palette_document().color_snapshot();
    h.state_mut().close_map();
    assert!(matches!(
        h.state().palette_document(),
        documents::PaletteRef::PlusPlus(_)
    ));
    assert_eq!(h.state().palette_document().color_snapshot(), colors);
    let ctx = h.ctx.clone();
    assert!(
        h.state_mut()
            .commit_palettes(&ctx, polished_map_project::PaletteDocument::default())
            .is_err()
    );
}

#[test]
fn plusplus_import_skip_and_failure_keep_live_assets_and_session_colors() {
    let (dir, mut h) = fixture();
    edit_color(&mut h, "123456", true);
    let colors = h.state().palette_document().color_snapshot();
    let original = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .document
        .path()
        .to_owned();
    open(&mut h, &dir.path().join("Next.2x1.demo.ablk"), false);
    assert!(h.state().pending_open.is_some());
    assert_eq!(h.state().loaded.as_ref().unwrap().document.path(), original);
    h.get_by_label("Skip").click();
    h.run();
    assert!(h.state().pending_open.is_none());
    assert_eq!(h.state().palette_document().color_snapshot(), colors);
    assert!(h.state().palette_document().is_modified());
    fs::write(dir.path().join("gfx/tilesets/bg_tiles.pal"), [0xff]).unwrap();
    open(&mut h, &dir.path().join("Map.2x1.demo.ablk"), true);
    h.get_by_label("Load").click();
    h.run();
    assert!(h.state_mut().error.take().is_some());
    assert!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .path()
            .ends_with("Next.2x1.demo.ablk")
    );
    assert_eq!(h.state().palette_document().color_snapshot(), colors);
}

#[test]
fn plusplus_save_all_routes_asset_domains_and_stops_on_attribute_conflict() {
    let (dir, mut h) = fixture();
    let ctx = h.ctx.clone();
    let assets = h
        .state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .plus_assets
        .as_mut()
        .unwrap();
    assets
        .tiles
        .apply_batch(&[RawTileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 2,
        }])
        .unwrap();
    assets
        .roof
        .as_mut()
        .unwrap()
        .apply_batch(&[RawTileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        }])
        .unwrap();
    assets
        .blocks
        .blockset_mut()
        .apply(&AttributeCommand::Set {
            block: 0,
            offsets: [1; 16],
            attributes: [0xa3; 16],
        })
        .unwrap();
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.state_mut()
        .event_action(event_workflows::EventAction::Move(0, 1, 1), &ctx);
    h.state_mut().save();
    crate::save_feedback::tests::dismiss(&mut h);
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert!(!h.state().is_modified());
    for path in ["gfx/tilesets/demo.png", "gfx/tilesets/roofs/demo.png"] {
        assert!(dir.path().join(path).is_file());
    }
    assert_eq!(
        fs::read(dir.path().join("data/tilesets/demo_attributes.bin")).unwrap()[..16],
        [0xa3; 16]
    );
    assert_eq!(
        fs::read(dir.path().join("Map.2x1.demo.ablk")).unwrap(),
        [1, 1]
    );
    assert!(
        !dir.path()
            .join("gfx/tilesets/demo_palette_map.asm")
            .exists()
    );
    let assets = h
        .state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .plus_assets
        .as_mut()
        .unwrap();
    assets
        .blocks
        .blockset_mut()
        .apply(&AttributeCommand::Set {
            block: 0,
            offsets: [0; 16],
            attributes: [0; 16],
        })
        .unwrap();
    fs::write(assets.blocks.attribute_path(), [55; 32]).unwrap();
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 0,
    });
    h.state_mut().save();
    assert!(
        h.state()
            .error
            .as_ref()
            .unwrap()
            .contains("changed on disk")
    );
    assert!(h.state().is_modified());
    assert_eq!(
        fs::read(dir.path().join("Map.2x1.demo.ablk")).unwrap(),
        [1, 1]
    );
    assert_eq!(
        fs::read(dir.path().join("data/tilesets/demo_attributes.bin")).unwrap(),
        [55; 32]
    );
}

#[test]
fn plusplus_clean_tileset_save_trims_unused_definitions_without_writing() {
    let (dir, mut h) = fixture();
    let path = dir.path().join("gfx/tilesets/demo.2bpp");
    let mut source = fs::read(&path).unwrap();
    source.extend([0; 16]);
    fs::write(&path, &source).unwrap();
    open(&mut h, &dir.path().join("Map.2x1.demo.ablk"), false);
    assert!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .plus_assets
            .as_ref()
            .unwrap()
            .tiles
            .model()
            .defined()[16]
    );
    h.state_mut().save_tileset();
    crate::save_feedback::tests::dismiss(&mut h);
    assert!(
        !h.state()
            .loaded
            .as_ref()
            .unwrap()
            .plus_assets
            .as_ref()
            .unwrap()
            .tiles
            .model()
            .defined()[16]
    );
    assert!(!h.state().is_modified());
    assert!(!dir.path().join("gfx/tilesets/demo.png").exists());
    assert_eq!(fs::read(&path).unwrap(), source);
}

#[test]
fn plusplus_dirty_tileset_conflict_preserves_live_definitions() {
    let (dir, mut h) = fixture();
    let path = dir.path().join("gfx/tilesets/demo.2bpp");
    let mut source = fs::read(&path).unwrap();
    source.extend([0; 16]);
    fs::write(&path, &source).unwrap();
    open(&mut h, &dir.path().join("Map.2x1.demo.ablk"), false);
    let assets = h
        .state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .plus_assets
        .as_mut()
        .unwrap();
    assets
        .tiles
        .apply_batch(&[RawTileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 2,
        }])
        .unwrap();
    let before = assets.tiles.content_hash();
    assert!(assets.tiles.model().defined()[16]);
    fs::write(assets.blocks.attribute_path(), [55; 32]).unwrap();
    h.state_mut().save_tileset();
    assert!(
        h.state()
            .error
            .as_ref()
            .unwrap()
            .contains("changed on disk")
    );
    let assets = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .plus_assets
        .as_ref()
        .unwrap();
    assert_eq!(assets.tiles.content_hash(), before);
    assert!(assets.tiles.model().defined()[16]);
    assert!(!dir.path().join("gfx/tilesets/demo.png").exists());
    assert_eq!(fs::read(&path).unwrap(), source);
}

#[test]
#[ignore = "requires pinned POLISHED_CRYSTAL_FIXTURE; no game assets committed"]
fn real_plusplus_open_palette_edit_and_save_copy_use_shared_gui() {
    let root = std::env::var_os("POLISHED_CRYSTAL_FIXTURE")
        .map(PathBuf::from)
        .expect("set POLISHED_CRYSTAL_FIXTURE");
    let original = root.join("maps/NewBarkTown.ablk");
    let bytes = fs::read(&original).unwrap();
    let mut h = editor();
    open(&mut h, &original, false);
    let loaded = h.state().loaded.as_ref().unwrap();
    assert_eq!(
        (
            loaded.document.map().width(),
            loaded.document.map().height()
        ),
        (10, 9)
    );
    let assets = loaded.plus_assets.as_ref().unwrap();
    assert_eq!(assets.tiles.name(), "johto_traditional.johto_common");
    assert_eq!(assets.tiles.loaded_tile_count(), 288);
    assert_eq!(loaded.graphics.block_count(), 248);
    assert!(h.state().allow_512);
    assert_eq!(h.state().palette_document().selected(), PaletteSet::Day);
    edit_color(&mut h, "123456", true);
    let temporary = tempfile::tempdir().unwrap();
    let target =
        polished_map_project::prepare_palette_export(&temporary.path().join("colors.pal")).unwrap();
    h.state_mut().export_palette_target(&target).unwrap();
    crate::save_feedback::tests::dismiss(&mut h);
    let mut colors = plusplus::PaletteDocument::default();
    colors.load(target.path()).unwrap();
    assert_eq!(
        colors
            .color(ColorSlot {
                set: PaletteSet::Day,
                palette: 0,
                hue: 1
            })
            .unwrap(),
        [16, 49, 82]
    );
    let changed = if bytes[0] == 0 { 1 } else { 0 };
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: changed,
    });
    let target = temporary.path().join("NewBarkTown.10x9.ablk");
    h.state_mut().save_as_path(target.clone());
    crate::save_feedback::tests::dismiss(&mut h);
    let reopened = plusplus::MapDocument::open(&target, &OpenOptions::default()).unwrap();
    assert_eq!(
        reopened.map().blocks(),
        h.state().loaded.as_ref().unwrap().document.map().blocks()
    );
    assert_eq!(fs::read(&original).unwrap(), bytes);
    let assets = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .plus_assets
        .as_ref()
        .unwrap();
    assets.tiles.verify_unchanged().unwrap();
    assets.blocks.verify_unchanged().unwrap();
    // Exercise the real project's block draft, then persist its accepted model
    // to separate temporary paths. Never save edits back into the pinned fixture.
    crate::block_workflow_tests::open_block(&mut h, 0);
    h.get_by_label("Tile $0:01").click();
    h.run();
    h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num8);
    h.run();
    h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::X);
    h.run();
    h.get_by_label("Block tile 0, 0").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    let doc = &h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .plus_assets
        .as_ref()
        .unwrap()
        .blocks;
    assert_eq!(doc.blockset().block(0).unwrap()[0].attribute(), 0x27);
    let mut copy =
        plusplus::BlocksetDocument::open(doc.path(), doc.attribute_path(), doc.collision_path())
            .unwrap();
    *copy.blockset_mut() = doc.blockset().clone();
    let offsets = plusplus::prepare_binary_save_as(&temporary.path().join("blocks.bin")).unwrap();
    let attrs = plusplus::prepare_binary_save_as(&temporary.path().join("attributes.bin")).unwrap();
    let collisions = doc.collision_path().map(|path| {
        let ext = path.extension().unwrap();
        polished_map_project::prepare_collision_save_as(
            &temporary.path().join("collisions").with_extension(ext),
            ext == "bin",
        )
        .unwrap()
    });
    copy.save_as(&offsets, &attrs, collisions.as_ref()).unwrap();
    let reopened = plusplus::BlocksetDocument::open(
        offsets.path(),
        attrs.path(),
        collisions.as_ref().map(|t| t.path()),
    )
    .unwrap();
    assert_eq!(
        reopened.blockset().offsets().bytes(),
        doc.blockset().offsets().bytes()
    );
    assert_eq!(
        reopened.blockset().attribute_bytes(),
        doc.blockset().attribute_bytes()
    );
    assert_eq!(
        reopened.blockset().offsets().collisions(),
        doc.blockset().offsets().collisions()
    );
    doc.verify_unchanged().unwrap();
}

#[test]
#[ignore = "requires pinned POLISHED_CRYSTAL_FIXTURE; no game assets committed"]
fn real_plusplus_tile_and_roof_gui_edits_save_and_reopen_copied_assets() {
    let root = std::env::var_os("POLISHED_CRYSTAL_FIXTURE")
        .map(PathBuf::from)
        .expect("set POLISHED_CRYSTAL_FIXTURE")
        .canonicalize()
        .unwrap();
    let name = "johto_traditional.johto_common";
    let original_tiles =
        plusplus::TilesetDocument::open_in_project(&root, name, Default::default()).unwrap();
    let original_blocks = plusplus::BlocksetDocument::open_in_project(&root, name).unwrap();
    let original_roof =
        plusplus::RoofDocument::open_in_project(&root, "new_bark", Default::default()).unwrap();
    let alternate_name = "johto_modern.johto_common";
    let alternate_tiles =
        plusplus::TilesetDocument::open_in_project(&root, alternate_name, Default::default())
            .unwrap();
    let alternate_blocks =
        plusplus::BlocksetDocument::open_in_project(&root, alternate_name).unwrap();
    let alternate_roof =
        plusplus::RoofDocument::open_in_project(&root, "violet", Default::default()).unwrap();
    let copy = tempfile::tempdir().unwrap();
    let mut paths: Vec<_> = original_tiles.sources().iter().map(|t| t.path()).collect();
    paths.extend([
        original_blocks.path(),
        original_blocks.attribute_path(),
        original_roof.source().path(),
    ]);
    paths.extend(original_blocks.collision_path());
    paths.extend(alternate_tiles.sources().iter().map(|t| t.path()));
    paths.extend([
        alternate_blocks.path(),
        alternate_blocks.attribute_path(),
        alternate_roof.source().path(),
    ]);
    paths.extend(alternate_blocks.collision_path());
    let before: Vec<_> = paths.iter().map(|p| fs::read(p).unwrap()).collect();
    for (path, bytes) in paths.iter().zip(&before) {
        let to = copy.path().join(path.strip_prefix(&root).unwrap());
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::write(to, bytes).unwrap();
    }
    fs::write(
        copy.path().join("main.asm"),
        "; isolated integration copy\n",
    )
    .unwrap();
    let map_path = copy.path().join("NewBarkTown.10x9.ablk");
    let map = fs::read(root.join("maps/NewBarkTown.ablk")).unwrap();
    fs::write(&map_path, &map).unwrap();
    let mut h = editor();
    h.state_mut().prepare_open(map_path.clone());
    let form = h.state_mut().open_form.as_mut().unwrap();
    form.tileset = name.into();
    form.roof = "new_bark".into();
    h.run();
    h.key_press(egui::Key::Enter);
    h.step();
    wait(&mut h);
    assert!(h.state().allow_512);
    // Switch compound tilesets and roof overlays, then return before the edit/save
    // checks below. All candidates live in the copied project, never the fixture.
    for (item, from, to) in [
        ("Change Tileset…", name, alternate_name),
        ("Change Roof…", "new_bark", "violet"),
        ("Change Tileset…", alternate_name, name),
        ("Change Roof…", "violet", "new_bark"),
    ] {
        h.get_by_label("Tools").click();
        h.run();
        h.get_by_label(item).click();
        h.run();
        h.get_by_label(from).click();
        h.run();
        h.get_by_label(to).click();
        h.run();
        h.get_by_label("OK").click();
        h.run();
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        let assets = h
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .plus_assets
            .as_ref()
            .unwrap();
        if item == "Change Tileset…" {
            assert_eq!(assets.tiles.name(), to);
            let expected = if to == name {
                &original_tiles
            } else {
                &alternate_tiles
            };
            assert_eq!(assets.tiles.model().tiles(), expected.model().tiles());
        } else {
            assert_eq!(assets.roof.as_ref().unwrap().name(), to);
        }
        assert!(!h.state().is_modified());
    }
    for roof in [false, true] {
        h.get_by_label("Tools").click();
        h.run();
        h.get_by_label(if roof {
            "Edit Roof…"
        } else {
            "Edit Tileset…"
        })
        .click();
        h.run();
        let before = if roof {
            original_roof.tiles()[6].hues()[0]
        } else {
            original_tiles.model().tiles()[0].hues()[0]
        };
        let (swatch, hue) = if before == 3 {
            ("Swatch 1", 0)
        } else {
            ("Swatch 4", 3)
        };
        h.get_by_label(swatch).click();
        h.run();
        h.get_by_label("Pixel 0, 0").click();
        h.run();
        h.get_by_label("OK").click();
        h.run();
        h.get_by_label("File").click();
        h.run();
        h.get_by_label(if roof { "Save Roof" } else { "Save Tileset" })
            .click();
        h.run();
        crate::save_feedback::tests::dismiss(&mut h);
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        let assets = h
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .plus_assets
            .as_ref()
            .unwrap();
        if roof {
            let reopened = plusplus::RoofDocument::open_in_project(
                copy.path(),
                "new_bark",
                Default::default(),
            )
            .unwrap();
            assert_eq!(reopened.tiles()[6].hues()[0], hue);
            assert_eq!(reopened.tiles(), assets.roof.as_ref().unwrap().tiles());
        } else {
            let reopened =
                plusplus::TilesetDocument::open_in_project(copy.path(), name, Default::default())
                    .unwrap();
            assert_eq!(reopened.model().tiles()[0].hues()[0], hue);
            assert_eq!(reopened.model().tiles(), assets.tiles.model().tiles());
        }
        assert!(!h.state().is_modified());
    }
    assert_eq!(fs::read(map_path).unwrap(), map);
    for (path, bytes) in paths.iter().zip(before) {
        assert_eq!(
            fs::read(path).unwrap(),
            bytes,
            "original {}",
            path.display()
        );
    }
    original_tiles.verify_unchanged().unwrap();
    original_blocks.verify_unchanged().unwrap();
}
