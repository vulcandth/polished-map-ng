//! Exercise automatic context through actual Open/New dialogs and load workers.
use super::*;
use documents::Profile;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

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
    let target = dir.path().join("Imported.ablk");
    document.save_as(&target).unwrap();
    assert_eq!(fs::read(target).unwrap(), [1, 1]);
    assert_eq!(fs::read(path).unwrap(), source);
}

#[test]
#[ignore = "requires all five pinned project fixture environment variables"]
fn unified_pinned_projects_alternate_open_edit_save_and_reopen() {
    let projects = [
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
    let mut h = editor();
    let output = tempfile::tempdir().unwrap();
    for (variable, name, extension, profile) in projects.into_iter().cycle().take(10) {
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
