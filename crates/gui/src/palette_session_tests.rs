use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn edit(palettes: &mut polished_map_project::PaletteDocument) {
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
}
fn load(h: &mut Harness<'_, Editor>, root: &Path, new_map: bool) {
    fs::write(root.join("Next.blk"), [0, 1]).unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().start_load(
        OpenForm {
            new_map,
            path: root.join("Next.blk"),
            root: root.to_string_lossy().into_owned(),
            tileset: "demo".into(),
            width: 2,
            height: 1,
            ..Default::default()
        },
        &ctx,
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while h.state().loading.is_some() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
        h.step();
    }
    assert!(h.state().loading.is_none(), "load worker timed out");
    h.run();
}
#[test]
fn premap_colors_survive_new_and_second_map_when_default_import_is_skipped() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    h.state_mut().loaded = None;
    h.state_mut().textures.clear();
    edit(&mut h.state_mut().unloaded_palettes);
    let hash = h.state().palette_document().content_hash();
    load(&mut h, dir.path(), true);
    assert!(
        h.state().loaded.is_none(),
        "map stays staged while imports need decisions"
    );
    assert!(h.state().pending_open.is_some());
    #[cfg(feature = "visual-tests")]
    {
        h.remove_cursor();
        h.run();
        h.snapshot("map-palette-import");
    }
    h.get_by_label("Skip").click();
    h.run();
    assert!(h.state().pending_open.is_none());
    assert_eq!(h.state().palette_document().content_hash(), hash);
    assert!(h.state().palette_document().is_modified());
    assert!(
        !h.state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .has_backing_file()
    );
    load(&mut h, dir.path(), false);
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .has_backing_file()
    );
    assert_eq!(h.state().palette_document().content_hash(), hash);
    assert!(h.state().palette_document().is_modified());
}

#[test]
fn accepting_missing_default_clears_edited_flag_without_resetting_colors() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    edit(
        h.state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .tileset
            .as_mut()
            .unwrap()
            .palettes_mut(),
    );
    let hash = h.state().palette_document().content_hash();
    load(&mut h, dir.path(), false);
    h.get_by_label("Load").click();
    h.run();
    assert!(h.state().pending_open.is_none());
    assert_eq!(h.state().palette_document().content_hash(), hash);
    assert!(!h.state().palette_document().is_modified());
    // With no edited flag, a subsequent open needs no import prompt and retains Custom.
    load(&mut h, dir.path(), false);
    assert!(h.state().pending_open.is_none());
    assert_eq!(h.state().palette_document().content_hash(), hash);
}

#[test]
fn skipping_default_and_special_still_prompts_for_roof_without_clearing_edited_flag() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let root = dir.path();
    fs::create_dir_all(root.join("constants")).unwrap();
    fs::create_dir_all(root.join("data/maps")).unwrap();
    fs::write(
        root.join("constants/map_constants.asm"),
        "\tnewgroup\n\tmap_const NEXT, 2, 1\n",
    )
    .unwrap();
    fs::write(
        root.join("data/maps/maps.asm"),
        "map Next, TILESET_DEMO, TOWN, LANDMARK_NEXT, MUSIC, FALSE, PALETTE_AUTO\n",
    )
    .unwrap();
    fs::write(
        root.join("Next.pal"),
        "\tRGB 0,0,0, 0,0,0, 0,0,0, 0,0,0\n".repeat(8),
    )
    .unwrap();
    fs::write(
        root.join("gfx/tilesets/roofs.pal"),
        "\tRGB 0,0,0, 0,0,0, 0,0,0, 0,0,0\n\tRGB 1,2,3, 4,5,6, 7,8,9, 10,11,12\n",
    )
    .unwrap();
    edit(
        h.state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .tileset
            .as_mut()
            .unwrap()
            .palettes_mut(),
    );
    load(&mut h, root, false);
    for name in ["bg.pal", "Next.pal", "roofs.pal"] {
        let path = h
            .state()
            .pending_open
            .as_ref()
            .unwrap()
            .palette_plan
            .as_ref()
            .unwrap()
            .next_import_path()
            .unwrap();
        assert_eq!(path.file_name().unwrap(), name);
        h.get_by_label(if name == "roofs.pal" { "Load" } else { "Skip" })
            .click();
        h.run();
    }
    assert!(h.state().pending_open.is_none());
    assert!(h.state().palette_document().is_modified());
    assert_eq!(
        h.state().palette_document().bank().colors(PaletteSet::Day)[6][2],
        [8, 16, 24]
    );
    assert_eq!(
        h.state()
            .palette_document()
            .bank()
            .colors(PaletteSet::Custom)[0][0],
        [17, 34, 51]
    );
    assert_eq!(h.state().palette_document().selected(), PaletteSet::Custom);
}

#[test]
fn malformed_accepted_import_preserves_previous_map_palettes_and_preview() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    edit(
        h.state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .tileset
            .as_mut()
            .unwrap()
            .palettes_mut(),
    );
    let hash = h.state().palette_document().content_hash();
    let path = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .document
        .path()
        .to_path_buf();
    let image = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .graphics
        .render_block(0)
        .unwrap();
    fs::write(
        dir.path().join("gfx/tilesets/bg_tiles.pal"),
        "\tRGB 32,0,0\n",
    )
    .unwrap();
    load(&mut h, dir.path(), false);
    h.get_by_label("Load").click();
    h.run();
    assert!(h.state().error.is_some());
    assert!(h.state().pending_open.is_none());
    assert_eq!(h.state().palette_document().content_hash(), hash);
    assert!(h.state().palette_document().is_modified());
    assert_eq!(h.state().loaded.as_ref().unwrap().document.path(), path);
    assert_eq!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .graphics
            .render_block(0)
            .unwrap(),
        image
    );
}

#[test]
fn optional_roof_color_failures_warn_and_keep_the_new_map_and_existing_colors() {
    let (dir, mut h) = super::tileset_editor_tests::fixture(false, false);
    let root = dir.path();
    fs::create_dir_all(root.join("constants")).unwrap();
    fs::create_dir_all(root.join("data/maps")).unwrap();
    fs::write(
        root.join("constants/map_constants.asm"),
        "\tnewgroup\n\tmap_const NEXT, 2, 1\n",
    )
    .unwrap();
    fs::write(
        root.join("data/maps/maps.asm"),
        "map Next, TILESET_DEMO, TOWN, LANDMARK_NEXT, MUSIC, FALSE, PALETTE_AUTO\n",
    )
    .unwrap();
    let before = h.state().palette_document().bank().clone();
    fs::write(root.join("gfx/tilesets/roofs.pal"), "\tRGB 32,0,0\n").unwrap();
    for missing in [false, true] {
        if missing {
            fs::remove_file(root.join("gfx/tilesets/roofs.pal")).unwrap();
        }
        load(&mut h, root, false);
        assert!(h.state().pending_open.is_none());
        assert!(h.state().error.is_none());
        assert!(h.state().open_palette_warning.is_some());
        assert!(
            h.state()
                .loaded
                .as_ref()
                .unwrap()
                .document
                .path()
                .ends_with("Next.blk")
        );
        assert_eq!(h.state().palette_document().bank(), &before);
        h.get_by_label("OK").click();
        h.run();
        assert!(h.state().open_palette_warning.is_none());
    }
}
