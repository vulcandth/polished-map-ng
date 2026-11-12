//! Exercise automatic context through actual Open/New dialogs and load workers.
use super::*;
use documents::Profile;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

mod real_assets;

const PINNED_PROJECTS: [(&str, &str, &str, Profile); 5] = [
    ("POLISHED_MAP_POKERED", "PalletTown", "blk", Profile::Master),
    (
        "POLISHED_MAP_POKEYELLOW",
        "PalletTown",
        "blk",
        Profile::Master,
    ),
    (
        "POLISHED_MAP_POKEGOLD",
        "NewBarkTown",
        "blk",
        Profile::Master,
    ),
    (
        "POLISHED_MAP_POKECRYSTAL",
        "NewBarkTown",
        "blk",
        Profile::Master,
    ),
    (
        "POLISHED_CRYSTAL_FIXTURE",
        "NewBarkTown",
        "ablk",
        Profile::PlusPlus,
    ),
];

fn editor() -> Harness<'static, Editor> {
    Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_max_steps(16)
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new_unified(cc))
}

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for name in ["gfx/tilesets", "data/tilesets", "maps"] {
        fs::create_dir_all(root.join(name)).unwrap();
    }
    fs::write(root.join("main.asm"), "; mixed-format project\n").unwrap();
    for (map, tileset, extension) in [("Classic", "classic", "blk"), ("Modern", "modern", "ablk")] {
        fs::write(
            root.join(format!("{map}.2x1.{tileset}.{extension}")),
            [0, 1],
        )
        .unwrap();
        fs::write(
            root.join(format!("gfx/tilesets/{tileset}.2bpp")),
            [0, 255].repeat(8 * 16),
        )
        .unwrap();
        fs::write(
            root.join(format!("data/tilesets/{tileset}_metatiles.bin")),
            [[0; 16], [1; 16]].concat(),
        )
        .unwrap();
        fs::write(
            root.join(format!("maps/{map}.asm")),
            "\twarp 0, 0, 1, OTHER\n\tpokemon_event 1, 0, PIKACHU\n",
        )
        .unwrap();
    }
    fs::write(
        root.join("gfx/tilesets/classic_palette_map.asm"),
        "\ttilepal 0, GRAY, RED, GREEN, TEXT\n",
    )
    .unwrap();
    fs::write(
        root.join("data/tilesets/modern_attributes.bin"),
        [[0; 16], [1; 16]].concat(),
    )
    .unwrap();
    dir
}

fn wait(h: &mut Harness<'_, Editor>) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while h.state().loading.is_some() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
        h.step();
    }
    assert!(h.state().loading.is_none(), "load worker timed out");
    h.run();
}

fn submit(h: &mut Harness<'_, Editor>) {
    h.run();
    h.key_press(egui::Key::Enter);
    h.step();
    wait(h);
}

fn open(h: &mut Harness<'_, Editor>, path: &Path, profile: Profile) {
    h.state_mut().prepare_open(path.into());
    let form = h.state().open_form.as_ref().expect("Open options");
    assert!(form.ready(), "{:?}", form.detection_error);
    assert!(form.profile == profile);
    submit(h);
    // Edited cached palettes still require the existing import decision.
    for _ in 0..4 {
        if h.state().pending_open.is_none() {
            break;
        }
        h.get_by_label("Skip").click();
        h.run();
    }
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert!(
        h.state().profile() == profile,
        "{}: status={}, pending={}, warning={:?}, form={}",
        path.display(),
        h.state().status,
        h.state().pending_open.is_some(),
        h.state().pending_map_warning,
        h.state().open_form.is_some()
    );
    assert!(
        h.state()
            .last_window_title
            .as_ref()
            .unwrap()
            .contains("NG Unified")
    );
}

fn color(h: &mut Harness<'_, Editor>, rgb: [u8; 3]) {
    let ctx = h.ctx.clone();
    let mut palettes = h.state().palette_document().to_owned();
    palettes
        .apply_batch(&[PaletteCommand::SetRgb {
            slot: ColorSlot {
                set: PaletteSet::Day,
                palette: 0,
                hue: 1,
            },
            rgb,
        }])
        .unwrap();
    h.state_mut().commit_palettes(&ctx, palettes).unwrap();
}

fn choose_tileset(h: &mut Harness<'_, Editor>, from: &str, to: &str) {
    h.state_mut().request_tileset_change();
    h.run();
    h.get_by_label(from).click();
    h.run();
    h.get_by_label(to).scroll_to_me();
    h.run();
    h.get_by_label(to).click();
    h.run();
    choice_preview::wait_for_preview(h);
    h.get_by_label("Selected graphics preview");
    h.get_by_label("OK").click();
    h.run();
}

#[test]
fn native_print_copy_works_after_automatic_storage_changes() {
    let dir = fixture();
    let mut h = editor();
    for (filename, profile) in [
        ("Classic.2x1.classic.blk", Profile::Master),
        ("Modern.2x1.modern.ablk", Profile::PlusPlus),
    ] {
        open(&mut h, &dir.path().join(filename), profile);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::P);
        h.run();
        assert!(h.state().print_form.is_some());
        // egui-winit translates native Ctrl/Cmd+C key-down into Copy, while
        // key-up and modifier release can arrive in the same frame.
        h.input_mut().events.extend([
            egui::Event::Copy,
            egui::Event::Key {
                key: egui::Key::C,
                physical_key: Some(egui::Key::C),
                pressed: false,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            },
            egui::Event::ModifiersChanged(egui::Modifiers::NONE),
        ]);
        h.run();
        assert!(
            h.state().print_request.is_some(),
            "native Print Copy ignored"
        );
        assert!(h.state().print_form.is_some());
        assert!(h.state().block_clipboard.is_none());
        assert!(!h.state().is_modified());
        h.state_mut().print_request = None;
        h.state_mut().print_form = None;
    }
}

#[test]
fn unified_tileset_change_adapts_both_ways_preserving_map_events_and_palette_sessions() {
    let dir = fixture();
    let root = dir.path();
    let mut h = editor();
    let path = root.join("Classic.2x1.classic.blk");
    open(&mut h, &path, Profile::Master);
    color(&mut h, [17, 34, 51]);
    let classic_colors = h.state().palette_document().color_snapshot();
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .document
        .map_mut()
        .undo();
    let events = h
        .state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .events
        .as_mut()
        .unwrap();
    events
        .apply(&[polished_map_core::EventCommand::Move {
            index: 0,
            x: 1,
            y: 0,
        }])
        .unwrap();
    let event_edits = events.script().events().clone();
    choose_tileset(&mut h, "classic", "modern");
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert!(h.state().profile() == Profile::PlusPlus);
    color(&mut h, [68, 85, 102]);
    let plus_colors = h.state().palette_document().color_snapshot();
    let doc = &mut h.state_mut().loaded.as_mut().unwrap().document;
    assert!(doc.map().can_redo());
    doc.map_mut().redo();
    assert_eq!(doc.map().blocks(), [1, 1]);
    assert!(doc.needs_save_as());
    assert!(doc.save().is_err());
    assert_eq!(doc.default_extension(), "ablk");
    choose_tileset(&mut h, "modern", "classic");
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert!(h.state().profile() == Profile::Master);
    assert_eq!(
        h.state().palette_document().color_snapshot(),
        classic_colors
    );
    assert!(h.state().loaded.as_ref().unwrap().document.map().can_undo());
    choose_tileset(&mut h, "classic", "modern");
    assert_eq!(h.state().palette_document().color_snapshot(), plus_colors);
    let loaded = h.state_mut().loaded.as_mut().unwrap();
    assert_eq!(
        loaded.events.as_ref().unwrap().script().events(),
        &event_edits
    );
    loaded.document.save_as(&root.join("Copy.ablk")).unwrap();
    assert!(!loaded.document.needs_save_as());
    assert_eq!(fs::read(root.join("Copy.ablk")).unwrap(), [1, 1]);
    assert_eq!(fs::read(path).unwrap(), [0, 1]);
    assert_eq!(
        fs::read(root.join("data/tilesets/modern_attributes.bin")).unwrap(),
        [[0; 16], [1; 16]].concat()
    );
}

#[test]
fn unified_storage_change_save_suggestion_reopens_with_the_selected_assets() {
    let dir = fixture();
    let mut h = editor();
    for (name, from, to, source_profile, target_profile) in [
        (
            "Classic",
            "classic",
            "modern",
            Profile::Master,
            Profile::PlusPlus,
        ),
        (
            "Modern",
            "modern",
            "classic",
            Profile::PlusPlus,
            Profile::Master,
        ),
    ] {
        let source = dir
            .path()
            .join(format!("{name}.2x1.{from}.{}", source_profile.extension()));
        open(&mut h, &source, source_profile);
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        choose_tileset(&mut h, from, to);
        let doc = &mut h.state_mut().loaded.as_mut().unwrap().document;
        // Fixed parity launchers retain their original filename behavior.
        assert_eq!(
            doc.save_path_suggestion(false),
            doc.path().with_extension(target_profile.extension())
        );
        let suggested = doc.save_path_suggestion(true);
        doc.save_as(&suggested).unwrap();
        open(&mut h, &suggested, target_profile);
        let doc = &h.state().loaded.as_ref().unwrap().document;
        assert_eq!(doc.metadata().tileset.as_deref(), Some(to));
        assert_eq!(doc.map().blocks(), [1, 1]);
        assert_eq!(doc.save_path_suggestion(true), doc.path());
        assert_eq!(fs::read(source).unwrap(), [0, 1]);
    }
}

#[test]
fn unified_compound_tileset_save_suggestion_reopens_without_context_correction() {
    let dir = fixture();
    let root = dir.path();
    fs::rename(
        root.join("gfx/tilesets/modern.2bpp"),
        root.join("gfx/tilesets/modern.common.2bpp"),
    )
    .unwrap();
    fs::write(root.join("gfx/tilesets/common.2bpp"), [0; 16 * 16]).unwrap();
    let source = root.join("Classic.2x1.classic.blk");
    let mut h = editor();
    open(&mut h, &source, Profile::Master);
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    choose_tileset(&mut h, "classic", "modern.common");
    let doc = &mut h.state_mut().loaded.as_mut().unwrap().document;
    let suggested = doc.save_path_suggestion(true);
    assert_eq!(suggested.file_name().unwrap(), "Classic.2x1.modern.ablk");
    doc.save_as(&suggested).unwrap();
    open(&mut h, &suggested, Profile::PlusPlus);
    let doc = &h.state().loaded.as_ref().unwrap().document;
    assert_eq!(doc.metadata().tileset.as_deref(), Some("modern.common"));
    assert_eq!(doc.map().blocks(), [1, 1]);
    assert_eq!(fs::read(source).unwrap(), [0, 1]);
}

#[test]
fn unified_failed_cross_storage_tileset_change_preserves_all_live_domains() {
    let dir = fixture();
    let mut h = editor();
    open(
        &mut h,
        &dir.path().join("Classic.2x1.classic.blk"),
        Profile::Master,
    );
    color(&mut h, [17, 34, 51]);
    let colors = h.state().palette_document().color_snapshot();
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    fs::write(dir.path().join("data/tilesets/modern_attributes.bin"), [0]).unwrap();
    choose_tileset(&mut h, "classic", "modern");
    assert!(h.state().error.is_some());
    assert!(h.state().profile() == Profile::Master);
    assert_eq!(h.state().palette_document().color_snapshot(), colors);
    assert!(h.state().inactive_palettes.is_none());
    let loaded = h.state().loaded.as_ref().unwrap();
    assert_eq!(loaded.document.map().blocks(), [1, 1]);
    assert!(loaded.document.map().can_undo());
    assert_eq!(loaded.tileset_location().unwrap().1, "classic");
    assert!(!loaded.document.needs_save_as());
}

#[test]
fn unified_cross_storage_change_obeys_unsaved_asset_cancel_and_discard() {
    let dir = fixture();
    let mut h = editor();
    open(
        &mut h,
        &dir.path().join("Classic.2x1.classic.blk"),
        Profile::Master,
    );
    h.state_mut()
        .loaded
        .as_mut()
        .unwrap()
        .tileset
        .as_mut()
        .unwrap()
        .apply_batch(&[polished_map_core::TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        }])
        .unwrap();
    h.state_mut().request_tileset_change();
    h.run();
    assert!(h.state().confirm_tileset_change);
    h.get_by_label("Cancel").click();
    h.run();
    assert!(h.state().tileset_choice_form.is_none());
    assert!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .tileset
            .as_ref()
            .unwrap()
            .tileset()
            .is_modified()
    );
    h.state_mut().request_tileset_change();
    h.run();
    h.get_by_label("Change Tileset").click();
    h.run();
    h.get_by_label("classic").click();
    h.run();
    h.get_by_label("modern").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert!(h.state().profile() == Profile::PlusPlus);
    assert!(!dir.path().join("gfx/tilesets/classic.png").exists());
}

#[test]
fn unified_cross_storage_change_warns_and_keeps_optional_roof_color_failure() {
    let dir = fixture();
    let root = dir.path();
    fs::create_dir_all(root.join("constants")).unwrap();
    fs::create_dir_all(root.join("data/maps")).unwrap();
    fs::write(
        root.join("constants/map_constants.asm"),
        "\tnewgroup\n\tmap_const CLASSIC, 2, 1\n",
    )
    .unwrap();
    fs::write(
        root.join("data/maps/maps.asm"),
        "map Classic, TILESET_CLASSIC, TOWN, SOMEWHERE, 0, 0, PALETTE_DAY\n",
    )
    .unwrap();
    fs::write(root.join("Classic.blk"), [0, 1]).unwrap();
    let mut h = editor();
    h.state_mut().auto_roof_colors = false;
    open(&mut h, &root.join("Classic.blk"), Profile::Master);
    h.state_mut().auto_roof_colors = true;
    choose_tileset(&mut h, "classic", "modern");
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    assert!(h.state().profile() == Profile::PlusPlus);
    assert!(
        h.state()
            .open_palette_warning
            .as_ref()
            .unwrap()
            .contains("Roof colors were not loaded")
    );
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        [0, 1]
    );
}

#[test]
fn unified_chr_assets_roof_previews_and_reload_use_the_same_source_policy() {
    let dir = fixture();
    let root = dir.path();
    fs::create_dir_all(root.join("MAPDATA")).unwrap();
    let chr = "\tdb 255,0,255,0,255,0,255,0,255,0,255,0,255,0,255,0\n";
    fs::write(root.join("MAPDATA/modern.CHR"), chr.repeat(16)).unwrap();
    fs::remove_file(root.join("gfx/tilesets/modern.2bpp")).unwrap();
    fs::write(root.join("data/tilesets/modern_collision.bin"), [1; 8]).unwrap();
    fs::create_dir_all(root.join("gfx/tilesets/roofs")).unwrap();
    fs::write(root.join("gfx/tilesets/roofs/legacy.CHR"), chr.repeat(9)).unwrap();
    let map = root.join("Modern.2x1.modern.ablk");
    fs::write(&map, [0, 1]).unwrap();
    let mut h = editor();
    open(&mut h, &map, Profile::PlusPlus);
    assert!(h.state().can_change_roof());
    h.state_mut().request_roof_change();
    h.run();
    h.get_by_label("(none)").click();
    h.run();
    h.get_by_label("legacy").click();
    h.run();
    choice_preview::wait_for_preview(&mut h);
    h.get_by_label("Selected graphics preview");
    h.get_by_label("OK").click();
    h.run();
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    let ctx = h.ctx.clone();
    h.state_mut().request_tile_mode_reload(&ctx);
    let loaded = h.state().loaded.as_ref().unwrap();
    let assets = loaded.plus_assets.as_ref().unwrap();
    assert_eq!(assets.roof.as_ref().unwrap().name(), "legacy");
    assert!(
        assets.tiles.sources()[0]
            .path()
            .ends_with("MAPDATA/modern.CHR")
    );
    assert!(
        assets
            .blocks
            .collision_path()
            .unwrap()
            .ends_with("modern_collision.bin")
    );
    assert_eq!(loaded.document.map().blocks(), [1, 1]);
    assert!(loaded.document.map().can_undo());
    assert!(!root.join("MAPDATA/modern.png").exists());
    assert!(!root.join("gfx/tilesets/roofs/legacy.png").exists());
    // MAPDATA now exists, but the selected classic asset lives under gfx.
    open(
        &mut h,
        &root.join("Classic.2x1.classic.blk"),
        Profile::Master,
    );
    assert!(!h.state().palette_monochrome());
}

#[test]
fn unified_alternating_documents_preserves_palette_sessions_and_uses_union_events() {
    let dir = fixture();
    let mut h = editor();
    let classic = dir.path().join("Classic.2x1.classic.blk");
    let modern = dir.path().join("Modern.2x1.modern.ablk");
    open(&mut h, &classic, Profile::Master);
    color(&mut h, [17, 34, 51]);
    let master_colors = h.state().palette_document().color_snapshot();
    open(&mut h, &modern, Profile::PlusPlus);
    assert_eq!(
        h.state().palette_document().colors(PaletteSet::Day).len(),
        8
    );
    color(&mut h, [68, 85, 102]);
    let plus_colors = h.state().palette_document().color_snapshot();
    for (path, profile, colors) in [
        (&classic, Profile::Master, &master_colors),
        (&modern, Profile::PlusPlus, &plus_colors),
    ] {
        open(&mut h, path, profile);
        assert_eq!(&h.state().palette_document().color_snapshot(), colors);
        let loaded = h.state().loaded.as_ref().unwrap();
        assert_eq!(
            loaded
                .events
                .as_ref()
                .unwrap()
                .script()
                .events()
                .entries()
                .len(),
            2
        );
        assert!(!loaded.document.map().can_undo());
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        let target = dir.path().join(format!("saved.{}", profile.extension()));
        let loaded = h.state_mut().loaded.as_mut().unwrap();
        loaded.document.save_as(&target).unwrap();
        assert_eq!(fs::read(target).unwrap(), [1, 1]);
        h.state_mut().close_map();
        assert_eq!(&h.state().palette_document().color_snapshot(), colors);
    }
    assert_eq!(fs::read(classic).unwrap(), [0, 1]);
    assert_eq!(fs::read(modern).unwrap(), [0, 1]);
}

#[test]
fn unified_failed_or_cancelled_open_preserves_active_document_and_history() {
    let dir = fixture();
    let mut h = editor();
    let classic = dir.path().join("Classic.2x1.classic.blk");
    let modern = dir.path().join("Modern.2x1.modern.ablk");
    open(&mut h, &classic, Profile::Master);
    color(&mut h, [17, 34, 51]);
    let colors = h.state().palette_document().color_snapshot();
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.state_mut().prepare_open(modern.clone());
    h.run();
    h.get_by_label("Cancel").click();
    h.run();
    assert!(h.state().open_form.is_none());
    // Detection still identifies plusplus, but its selected attributes are malformed.
    fs::write(dir.path().join("data/tilesets/modern_attributes.bin"), [0]).unwrap();
    h.state_mut().prepare_open(modern);
    submit(&mut h);
    assert!(h.state().error.is_some());
    assert!(h.state().profile() == Profile::Master);
    assert_eq!(h.state().palette_document().color_snapshot(), colors);
    let doc = &h.state().loaded.as_ref().unwrap().document;
    assert_eq!(doc.path(), classic.canonicalize().unwrap());
    assert_eq!(doc.map().blocks(), [1, 1]);
    assert!(doc.map().can_undo());
}

#[test]
fn unified_monochrome_fallback_does_not_override_next_project_but_user_choice_does() {
    let dir = fixture();
    let root = dir.path();
    for (source, target) in [
        ("gfx/tilesets/classic.2bpp", "gfx/tilesets/legacy.2bpp"),
        (
            "data/tilesets/classic_metatiles.bin",
            "data/tilesets/legacy_metatiles.bin",
        ),
    ] {
        fs::copy(root.join(source), root.join(target)).unwrap();
    }
    let legacy = root.join("Legacy.2x1.legacy.blk");
    fs::write(&legacy, [0, 1]).unwrap();
    let classic = root.join("Classic.2x1.classic.blk");
    let mut h = editor();
    open(&mut h, &legacy, Profile::Master);
    assert!(h.state().palette_monochrome());
    assert!(!h.state().monochrome_preference());
    open(&mut h, &classic, Profile::Master);
    assert!(!h.state().palette_monochrome());
    h.get_by_label("Options").click();
    h.run();
    h.get_by_label("Monochrome").click();
    h.run();
    assert!(h.state().palette_monochrome() && h.state().monochrome_preference());
    open(
        &mut h,
        &root.join("Modern.2x1.modern.ablk"),
        Profile::PlusPlus,
    );
    assert!(!h.state().palette_monochrome());
    open(&mut h, &classic, Profile::Master);
    assert!(h.state().palette_monochrome());
    h.get_by_label("Options").click();
    h.run();
    h.get_by_label("Monochrome").click();
    h.run();
    assert!(!h.state().palette_monochrome() && !h.state().monochrome_preference());
}

#[test]
fn unified_new_map_uses_selected_assets_and_shared_map_requires_actual_context() {
    let dir = fixture();
    let mut h = editor();
    for (tileset, profile) in [("modern", Profile::PlusPlus), ("classic", Profile::Master)] {
        h.state_mut().prepare_new_in(dir.path().into());
        let form = h.state_mut().open_form.as_mut().unwrap();
        form.tileset = tileset.into();
        form.resolve_profile();
        assert!(form.ready());
        assert!(form.profile == profile);
        submit(&mut h);
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        let doc = &h.state().loaded.as_ref().unwrap().document;
        assert!(!doc.has_backing_file());
        assert!(doc.profile() == profile);
        assert_eq!(doc.path().extension().unwrap(), profile.extension());
        let suggested = doc.save_path_suggestion(true);
        h.state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .document
            .save_as(&suggested)
            .unwrap();
        open(&mut h, &suggested, profile);
        assert_eq!(
            h.state()
                .loaded
                .as_ref()
                .unwrap()
                .document
                .metadata()
                .tileset
                .as_deref(),
            Some(tileset)
        );
    }
    let path = dir.path().join("Shared.blk");
    fs::write(&path, [0, 1]).unwrap();
    h.state_mut().prepare_open(path);
    let form = h.state().open_form.as_ref().unwrap();
    assert!(!form.ready() && form.tileset.is_empty());
    assert!(form.detection_error.is_some());
    let ctx = h.ctx.clone();
    let form = h.state_mut().open_form.take().unwrap();
    h.state_mut().start_load(form, &ctx);
    assert!(h.state().loading.is_none());
    assert!(!h.state().open_form.as_ref().unwrap().ready());
    // The direct invalid-submit guard produced an error; dismiss it before
    // exercising a corrected selection through the actual Open dialog.
    assert!(h.state_mut().error.take().is_some());
    let form = h.state_mut().open_form.as_mut().unwrap();
    form.tileset = "modern".into();
    form.resolve_profile();
    assert!(form.ready());
    submit(&mut h);
    let document = &h.state().loaded.as_ref().unwrap().document;
    assert!(document.profile() == Profile::PlusPlus);
    assert_eq!(document.metadata().tileset.as_deref(), Some("modern"));
}

#[test]
fn unified_asm_map_uses_occurrence_assets_and_saves_only_to_explicit_binary_target() {
    let dir = fixture();
    let mut h = editor();
    let path = dir.path().join("Source.2x1.modern.map");
    let source = b"\tdb 0, 1 ; source remains untouched\n";
    fs::write(&path, source).unwrap();
    // ASM imports have no binary-map header owner; select their actual tileset.
    h.state_mut().prepare_open(path.clone());
    let form = h.state_mut().open_form.as_mut().unwrap();
    form.tileset = "modern".into();
    form.resolve_profile();
    assert!(form.ready());
    submit(&mut h);
    assert!(h.state().profile() == Profile::PlusPlus);
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    let document = &mut h.state_mut().loaded.as_mut().unwrap().document;
    assert!(document.save().is_err());
    let target = document.save_path_suggestion(true);
    document.save_as(&target).unwrap();
    assert_eq!(fs::read(&target).unwrap(), [1, 1]);
    open(&mut h, &target, Profile::PlusPlus);
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        [1, 1]
    );
    assert_eq!(fs::read(path).unwrap(), source);
}

#[test]
#[ignore = "requires all five pinned project fixture environment variables"]
fn unified_pinned_projects_alternate_open_edit_save_and_reopen() {
    let mut h = editor();
    let output = tempfile::tempdir().unwrap();
    for (variable, name, extension, profile) in PINNED_PROJECTS.into_iter().cycle().take(10) {
        let root = PathBuf::from(std::env::var_os(variable).expect("set pinned project fixture"));
        let path = root.join(format!("maps/{name}.{extension}"));
        let original = fs::read(&path).unwrap();
        open(&mut h, &path, profile);
        assert_eq!(
            h.state().palette_monochrome(),
            matches!(variable, "POLISHED_MAP_POKERED" | "POLISHED_MAP_POKEYELLOW"),
            "{variable}: only Gen 1 should use automatic monochrome fallback"
        );
        assert_eq!(
            h.state().loaded.as_ref().unwrap().document.map().blocks(),
            original
        );
        let block = if original[0] == 0 { 1 } else { 0 };
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block,
        });
        let target = output.path().join(format!("{name}.{extension}"));
        let doc = &mut h.state_mut().loaded.as_mut().unwrap().document;
        let expected = doc.map().blocks().to_vec();
        doc.save_as(&target).unwrap();
        assert_eq!(fs::read(&target).unwrap(), expected);
        h.state_mut().prepare_open(target);
        let form = h.state_mut().open_form.as_mut().unwrap();
        form.root = root.to_string_lossy().into_owned();
        form.refresh_choices().unwrap();
        assert!(form.ready(), "{:?}", form.detection_error);
        submit(&mut h);
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        assert_eq!(
            h.state().loaded.as_ref().unwrap().document.map().blocks(),
            expected
        );
        assert_eq!(fs::read(path).unwrap(), original);
    }
}

#[test]
#[ignore = "requires pinned POLISHED_MAP_POKERED and POLISHED_MAP_POKEYELLOW"]
fn unified_pinned_projects_short_gen1_map_requires_explicit_size_choice() {
    for variable in ["POLISHED_MAP_POKERED", "POLISHED_MAP_POKEYELLOW"] {
        let root = PathBuf::from(std::env::var_os(variable).expect("set pinned project fixture"));
        let path = root.join("maps/UndergroundPathNorthSouth.blk");
        let original = fs::read(&path).unwrap();
        let mut h = editor();
        h.state_mut().prepare_open(path.clone());
        let form = h.state().open_form.as_ref().unwrap();
        assert_eq!((form.width, form.height), (4, 24));
        submit(&mut h);
        assert!(
            h.state()
                .error
                .as_ref()
                .unwrap()
                .contains("expected 96 map bytes, found 92")
        );
        assert!(h.state().loaded.is_none());
        h.state_mut().error = None;
        h.state_mut().prepare_open(path.clone());
        h.run();
        h.get_by_label("Size").click();
        h.run();
        h.get_by_label("4 x 23").click();
        h.run();
        submit(&mut h);
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        let map = h.state().loaded.as_ref().unwrap().document.map();
        assert_eq!((map.width(), map.height()), (4, 23));
        assert_eq!(map.blocks(), original);
        assert_eq!(fs::read(path).unwrap(), original);
    }
}

#[test]
fn unified_options_keep_both_format_groups_and_disable_inapplicable_actions() {
    let dir = fixture();
    let mut h = editor();
    for profile in [None, Some(Profile::Master), Some(Profile::PlusPlus)] {
        if let Some(profile) = profile {
            let path = if profile == Profile::Master {
                "Classic.2x1.classic.blk"
            } else {
                "Modern.2x1.modern.ablk"
            };
            open(&mut h, &dir.path().join(path), profile);
            let loaded = h.state().loaded.as_ref().unwrap();
            assert_eq!(
                h.state().last_window_title.as_deref(),
                Some(
                    format!(
                        "{} [{}] - {}",
                        path,
                        loaded.document.metadata().root.as_ref().unwrap().display(),
                        h.state().application.title()
                    )
                    .as_str()
                )
            );
        }
        h.get_by_label("Options").click();
        h.run();
        for (label, family) in [
            ("Monochrome", Profile::Master),
            ("Tile Priority", Profile::Master),
            ("256 Tiles", Profile::Master),
            ("512 Tiles", Profile::PlusPlus),
        ] {
            assert_eq!(
                h.get_by_label(label).accesskit_node().is_disabled(),
                profile != Some(family),
                "{label}"
            );
        }
        h.key_press(egui::Key::Escape);
        h.run();
    }
}

#[test]
fn project_options_align_labels_controls_and_keep_roof_override_on_cancel() {
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for (dark, windows, scale) in [
        (false, false, 1.0),
        (true, false, 1.0),
        (false, true, 2.0),
        (true, true, 2.0),
    ] {
        let dir = fixture();
        let mut h = editor();
        h.set_pixels_per_point(scale);
        h.ctx.data_mut(|d| {
            d.insert_temp(egui::Id::unique("test-map-options-windows"), windows);
        });
        h.state_mut().dark = dark;
        h.ctx.set_theme(if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        h.state_mut()
            .prepare_open(dir.path().join("Modern.2x1.modern.ablk"));
        h.run();
        h.get_by_label("Project options").click();
        h.run();
        let label = h.get_by_label("Roof palettes:").rect();
        let control = h
            .get_by_role_and_label(egui::Role::ComboBox, "Roof palettes")
            .rect();
        assert!(label.right() < control.left());
        assert!((label.center().y - control.center().y).abs() < 2.0);
        let folder = h
            .get_by_role_and_label(egui::Role::TextInput, "Project folder:")
            .rect();
        assert!((folder.left() - control.left()).abs() < 1.0);
        assert!(h.get_by_label("Browse…").rect().left() >= folder.right());
        assert!(control.bottom() < 600.0);
        let right = h
            .state()
            .open_form
            .as_ref()
            .unwrap()
            .client_bounds
            .unwrap()
            .right();
        for rect in [control, folder, h.get_by_label("Browse…").rect()] {
            assert!(
                right - rect.right() >= 9.5,
                "missing right padding: {right}, {rect:?}"
            );
        }
        // Avoid machine-specific fixture paths in visual baselines.
        h.state_mut().open_form.as_mut().unwrap().root = "example-project".into();
        h.state_mut().open_form.as_mut().unwrap().choices_root = "example-project".into();
        h.remove_cursor();
        h.run();
        #[cfg(feature = "visual-tests")]
        if !windows {
            snapshots.add(h.try_snapshot(format!(
                "project-options-{}",
                if dark { "dark" } else { "light" }
            )));
        }
        let original = h.state().roof_palette_layout;
        h.get_by_role_and_label(egui::Role::ComboBox, "Roof palettes")
            .click();
        h.run();
        h.get_by_label("Custom").click();
        h.run();
        h.key_press(egui::Key::Escape);
        h.run();
        assert!(h.state().open_form.is_none());
        assert_eq!(h.state().roof_palette_layout, original);
    }
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}

#[test]
fn unified_open_exposes_context_and_keeps_roof_override_transactional() {
    let dir = fixture();
    fs::create_dir_all(dir.path().join("constants")).unwrap();
    fs::write(
        dir.path().join("constants/map_constants.asm"),
        "\tnewgroup A\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("gfx/tilesets/roofs.pal"),
        format!(
            "; morn/day, nite, eve\n{}",
            "\tRGB 1,2,3, 4,5,6\n".repeat(6)
        ),
    )
    .unwrap();
    let mut h = editor();
    let original = h.state().roof_palette_layout;
    h.state_mut()
        .prepare_open(dir.path().join("Modern.2x1.modern.ablk"));
    let form = h.state().open_form.as_ref().unwrap();
    assert!(form.detected_context.contains("512-tile"));
    assert_eq!(
        form.roof_layout,
        Some(polished_map_project::RoofPaletteLayout::DayNiteCustom)
    );
    h.run();
    h.key_press(egui::Key::Escape);
    h.run();
    assert_eq!(h.state().roof_palette_layout, original);
    h.state_mut()
        .prepare_open(dir.path().join("Classic.2x1.classic.blk"));
    assert!(
        h.state()
            .open_form
            .as_ref()
            .unwrap()
            .detected_context
            .contains("256-tile")
    );
    h.state_mut().open_form.as_mut().unwrap().roof_layout =
        Some(polished_map_project::RoofPaletteLayout::Custom);
    submit(&mut h);
    assert_eq!(
        h.state().roof_palette_layout,
        polished_map_project::RoofPaletteLayout::Custom
    );
}
