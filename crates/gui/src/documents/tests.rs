use super::*;
use crate::{
    Editor, OpenOptions,
    event_workflows::{EventAction, EventChange},
};
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{Command, Point, ResizeAnchor, plusplus::EVENT_KINDS};
use std::fs;

// The shared editor loads a real plusplus map and attribute-aware renderer.
// Synthetic colors/tile pixels avoid redistributing external fixture artwork.
fn fixture() -> (tempfile::TempDir, Harness<'static, Editor>) {
    use polished_map_core::{
        Tile,
        plusplus::{AttributeBlockset, BankOrder, RawTileset},
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Map.2x1.ablk");
    fs::write(&path, [0, 1]).unwrap();
    fs::write(dir.path().join("Map.2x1.blk"), [0, 1]).unwrap();
    let document =
        plusplus::MapDocument::open_in_project(&path, &OpenOptions::default(), Some(dir.path()))
            .unwrap();
    let mut tiles = vec![Tile::new([0; 64]).unwrap(); 512];
    let mut hues = [1; 64];
    hues[0] = 3;
    tiles[511] = Tile::new(hues).unwrap();
    let raw = RawTileset::new(tiles, vec![true; 512]).unwrap();
    let mut offsets = [0; 32];
    offsets[0] = 255; // default bank order: bank 0 upper half is ordinal 511.
    let mut attributes = [0; 32];
    attributes[0] = 0xe7; // palette 7, bank 0, X/Y flip, priority.
    let blocks = AttributeBlockset::from_bytes(&offsets, &attributes).unwrap();
    let mut palettes = [[[255; 3]; 4]; 8];
    palettes[7][1] = [17, 18, 19];
    palettes[7][3] = [97, 98, 99];
    let graphics =
        polished_map_core::plusplus::Graphics::new(raw, blocks, palettes, BankOrder::default());
    let loaded = crate::Loaded {
        roof_layout: None,
        recent_path: Some(path),
        trailing_blocks: 0,
        events: None,
        event_warning: None,
        palette_plan: None,
        plus_assets: None,
        document: document.into(),
        graphics: graphics.into(),
        blockset: None,
        tileset: None,
        allow_256: false,
    };
    let mut h = Harness::builder()
        .with_size(crate::egui::vec2(800.0, 600.0))
        .with_theme(crate::egui::Theme::Light)
        .build_eframe(|cc| Editor::new_plusplus(cc));
    let (tx, rx) = std::sync::mpsc::channel();
    tx.send(Ok(loaded)).unwrap();
    h.state_mut().loading = Some(rx);
    h.run();
    assert_eq!(h.state().textures.len(), 2);
    (dir, h)
}

#[test]
fn plusplus_map_gui_keeps_history_save_formats_conflicts_and_print_names() {
    let (dir, mut h) = fixture();
    let original = dir.path().join("Map.2x1.ablk");
    assert_eq!(
        h.state().loaded.as_ref().unwrap().print_filename(),
        "Map.2x1.png"
    );
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.run();
    h.key_press_modifiers(crate::egui::Modifiers::COMMAND, crate::egui::Key::Z);
    h.run();
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        &[0, 1]
    );
    h.key_press_modifiers(crate::egui::Modifiers::COMMAND, crate::egui::Key::Y);
    h.run();
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        &[1, 1]
    );
    h.state_mut().save_map();
    crate::save_feedback::tests::dismiss(&mut h);
    assert_eq!(fs::read(&original).unwrap(), [1, 1]);
    for (name, expected) in [("Copy", "Copy.ablk"), ("Other.blk", "Other.blk")] {
        h.state_mut().save_as_path(dir.path().join(name));
        crate::save_feedback::tests::dismiss(&mut h);
        let loaded = h.state().loaded.as_ref().unwrap();
        assert_eq!(
            loaded.document.path(),
            fs::canonicalize(dir.path().join(expected)).unwrap()
        );
        assert_eq!(
            loaded.recent_path.as_ref().unwrap(),
            &dir.path().join(expected)
        );
        assert_eq!(fs::read(dir.path().join(expected)).unwrap(), [1, 1]);
    }
    h.state_mut().save_as_path(dir.path().join("Wrong.map"));
    assert!(
        h.state_mut()
            .error
            .take()
            .unwrap()
            .contains(".ablk or .blk")
    );
    assert!(!dir.path().join("Wrong.map").exists());
    assert!(matches!(
        h.state().loaded.as_ref().unwrap().document,
        MapDocument::PlusPlus(_)
    ));
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 0,
    });
    fs::write(dir.path().join("Other.blk"), [99, 99]).unwrap();
    h.state_mut().save_map();
    assert!(
        h.state()
            .error
            .as_ref()
            .unwrap()
            .contains("changed on disk")
    );
    assert!(h.state().is_modified());
    assert_eq!(fs::read(dir.path().join("Other.blk")).unwrap(), [99, 99]);
}

#[test]
fn all_plusplus_events_use_shared_options_and_save_reload_without_changing_catalog() {
    let (dir, mut h) = fixture();
    let path = dir.path().join("Map.asm");
    let untouched = "Header:\r\n\twarp 1, 2, 1, MASTER_ONLY\r\n\tdb 99\r\n";
    let mut source = untouched.to_owned();
    for kind in EVENT_KINDS {
        source.push_str(&format!(
            "\t{} {}1, 2, {}, {} ; keep\r\n",
            kind.macro_name(),
            if kind.has_prefix() { "SPRITE," } else { "" },
            if kind.id_before_map() {
                "1"
            } else {
                "LOCAL_MAP"
            },
            if kind.id_before_map() {
                "LOCAL_MAP"
            } else {
                "1"
            }
        ));
    }
    fs::write(&path, &source).unwrap();
    h.state_mut()
        .request_event_change(EventChange::Load(path.clone()));
    let ctx = h.ctx.clone();
    for (index, kind) in EVENT_KINDS.into_iter().enumerate() {
        assert_eq!(
            h.state()
                .loaded
                .as_ref()
                .unwrap()
                .events
                .as_ref()
                .unwrap()
                .script()
                .events()
                .entries()[index]
                .definition()
                .kind,
            kind
        );
        h.state_mut()
            .event_action(EventAction::Options(index), &ctx);
        h.run();
        assert_eq!(
            h.state().event_ui.form.as_ref().unwrap().bounds.width(),
            300.0
        );
        h.get_by_label("Event X").set_value_accesskit(4.0);
        h.get_by_label("Event Y").set_value_accesskit(5.0);
        h.run();
        h.get_by_label("OK").click();
        h.run();
        let d = h
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .events
            .as_ref()
            .unwrap()
            .script()
            .events()
            .entries()[index]
            .definition();
        assert_eq!((d.x, d.y), (4, 5), "{}", kind.macro_name());
    }
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    h.state_mut().save_events();
    crate::save_feedback::tests::dismiss(&mut h);
    let saved = fs::read_to_string(&path).unwrap();
    assert!(saved.starts_with(untouched));
    assert_eq!(saved.matches("; keep\r\n").count(), 26);
    h.state_mut().request_event_change(EventChange::Unload);
    assert!(h.state().loaded.as_ref().unwrap().events.is_none());
    h.state_mut().request_event_change(EventChange::Load(path));
    h.state_mut().request_event_change(EventChange::Reload);
    let events = h.state().loaded.as_ref().unwrap().events.as_ref().unwrap();
    assert!(matches!(events, EventDocument::PlusPlus(_)));
    assert_eq!(events.script().events().entries().len(), 26);
    for (event, kind) in events.script().events().entries().iter().zip(EVENT_KINDS) {
        let d = event.definition();
        assert_eq!(d.kind, kind);
        assert_eq!((d.x, d.y), (4, 5));
    }
    assert!(!h.state().is_modified());
}

#[test]
fn plusplus_resize_and_event_reload_keep_atomicity_and_dirty_guards() {
    let (dir, mut h) = fixture();
    let path = dir.path().join("Map.asm");
    fs::write(&path, "\tpokemon_event 249, 0, PIKACHU\n").unwrap();
    h.state_mut()
        .request_event_change(EventChange::Load(path.clone()));
    h.state_mut().apply(Command::Resize {
        width: 0,
        height: 1,
        anchor: ResizeAnchor::MiddleRight,
    });
    assert!(h.state_mut().error.take().is_some());
    assert_eq!(h.state().loaded.as_ref().unwrap().document.map().width(), 2);
    assert!(!h.state().is_modified());
    let ctx = h.ctx.clone();
    h.state_mut().event_action(EventAction::Move(0, 0, 0), &ctx);
    h.state_mut().apply(Command::Resize {
        width: 3,
        height: 1,
        anchor: ResizeAnchor::MiddleRight,
    });
    assert_eq!(h.state().loaded.as_ref().unwrap().document.map().width(), 3);
    assert_eq!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .events
            .as_ref()
            .unwrap()
            .script()
            .events()
            .entries()[0]
            .definition()
            .x,
        2
    );
    h.state_mut().request_event_change(EventChange::Reload);
    h.run();
    h.get_by_label("Cancel").click();
    h.run();
    assert_eq!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .events
            .as_ref()
            .unwrap()
            .script()
            .events()
            .entries()[0]
            .definition()
            .x,
        2
    );
    fs::write(&path, "external\n").unwrap();
    h.state_mut().save_events();
    assert!(
        h.state_mut()
            .error
            .take()
            .unwrap()
            .contains("changed on disk")
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "external\n");
    assert!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .events
            .as_ref()
            .unwrap()
            .script()
            .events()
            .is_modified()
    );
}

#[test]
fn plusplus_automatic_events_and_warp_paths_follow_the_owning_document() {
    let (dir, h) = fixture();
    fs::create_dir_all(dir.path().join("maps")).unwrap();
    fs::write(
        dir.path().join("maps/Map.asm"),
        "\tpokemon_event 1, 2, PIKACHU\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("Map.2x1.asm"),
        "\ttreebase_left_event_1 3, 4, BASE\n",
    )
    .unwrap();
    let document = &h.state().loaded.as_ref().unwrap().document;
    for (root, expected) in [
        (true, polished_map_core::EventKind::PokemonEvent),
        (false, polished_map_core::EventKind::TreebaseLeftEvent1),
    ] {
        if !root {
            fs::remove_file(dir.path().join("maps/Map.asm")).unwrap();
        }
        let (events, warning) = crate::event_workflows::auto_load_events(document, true, false);
        assert!(warning.is_none());
        assert_eq!(
            events.unwrap().script().events().entries()[0]
                .definition()
                .kind,
            expected
        );
    }
    assert!(
        crate::event_workflows::auto_load_events(document, false, false)
            .0
            .is_none()
    );
    assert!(
        document
            .warp_path("NextMap")
            .unwrap()
            .ends_with("NextMap.ablk")
    );
    for bad in ["../Other", "Other/Map", "C:\\Other", ""] {
        assert!(document.warp_path(bad).is_err());
    }
    let new: MapDocument = plusplus::MapDocument::new_map(dir.path(), 2, 1)
        .unwrap()
        .into();
    assert!(
        crate::event_workflows::auto_load_events(&new, true, false)
            .0
            .is_none()
    );
    let master: MapDocument = polished_map_project::MapDocument::open(
        &dir.path().join("Map.2x1.blk"),
        &OpenOptions::default(),
    )
    .unwrap()
    .into();
    let events = master.auto_load_events().unwrap().unwrap();
    assert!(
        events.script().events().entries().is_empty(),
        "master must not adopt plusplus macros"
    );
}

#[test]
fn plusplus_canvas_and_print_copy_keep_upper_bank_flips_and_occurrence_palette() {
    use crate::{egui, printing};
    let (_dir, mut h) = fixture();
    let loaded = h.state().loaded.as_ref().unwrap();
    let image = loaded.graphics.render_map(loaded.document.map()).unwrap();
    assert_eq!((image.width, image.height), (64, 32));
    // The first occurrence is tile 511 with palette 7 and both flips. Only its
    // bottom-right pixel has hue 3; the other block and slots use tile 0.
    for y in 0..32 {
        for x in 0..64 {
            let expected = if x == 7 && y == 7 {
                [97, 98, 99]
            } else if x < 8 && y < 8 {
                [17, 18, 19]
            } else {
                [255; 3]
            };
            assert_eq!(
                image.pixels[(y * 64 + x) * 3..(y * 64 + x + 1) * 3],
                expected
            );
        }
    }
    let priorities = loaded.graphics.block_priorities(0).unwrap();
    assert!(priorities[0]);
    assert!(priorities[1..].iter().all(|flag| !flag));
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::P);
    h.run();
    h.get_by_label("Copy").click();
    h.run();
    let job = h.state_mut().print_request.take().unwrap();
    // Print captures a snapshot; subsequent map edits must not alter it.
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    let printing::PrintOutput::Copied(copied) = printing::run(job).unwrap() else {
        panic!("expected clipboard image")
    };
    assert_eq!(copied.size, [64, 32]);
    assert_eq!(
        copied
            .pixels
            .iter()
            .flat_map(|p| [p.r(), p.g(), p.b()])
            .collect::<Vec<_>>(),
        image.pixels
    );
}
