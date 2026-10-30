use super::*;
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn block_editor_cancel_accept_save_and_conflict_preserve_map_history() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Map.1x1.blk");
    let blocks = dir.path().join("demo.bin");
    let collisions = dir.path().join("collision.asm");
    std::fs::write(&collisions, "\ttilecoll FLOOR, FLOOR, WALL, WALL ; 00\n").unwrap();
    std::fs::write(&path, [0]).unwrap();
    std::fs::write(&blocks, [0; 16]).unwrap();
    let document = MapDocument::open(&path, &OpenOptions::default()).unwrap();
    let blockset = BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap();
    let graphics = Graphics::new(
        (0..256)
            .map(|i| polished_map_core::Tile::new([i as u8 % 4; 64]).unwrap())
            .collect(),
        vec![polished_map_core::day_palettes()[2]; 256],
        vec![[0; 16]],
    )
    .unwrap();
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new(cc));
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(Loaded {
        recent_path: None,
        trailing_blocks: 0,
        palette_plan: None,
        events: None,
        event_warning: None,
        document,
        graphics,
        blockset: Some(blockset),
        tileset: None,
        allow_256: false,
    }))
    .unwrap();
    harness.state_mut().loading = Some(rx);
    harness.run();
    harness.state_mut().status = "Synthetic blockset".into();
    harness.run();
    for accept in [false, true] {
        crate::block_workflow_tests::open_block(&mut harness, 0);
        #[cfg(feature = "visual-tests")]
        if !accept {
            let original = harness.state().block_form.as_ref().unwrap().windows_buttons;
            for windows in [true, false] {
                harness
                    .state_mut()
                    .block_form
                    .as_mut()
                    .unwrap()
                    .windows_buttons = windows;
                harness.remove_cursor();
                harness.run();
                harness.snapshot(if windows {
                    "block-dialog-windows"
                } else {
                    "block-dialog-unix"
                });
            }
            harness
                .state_mut()
                .block_form
                .as_mut()
                .unwrap()
                .windows_buttons = original;
            harness.run();
        }
        harness.get_by_label("Tile $01").click();
        harness.run();
        harness.get_by_label("Block tile 0, 0").click();
        harness.run();
        harness.get_by_label("Collision top left").focus();
        harness.run();
        harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        harness.run();
        harness
            .get_by_label("Collision top left")
            .type_text("WATER");
        harness.run();
        harness
            .get_by_label(if accept { "OK" } else { "Cancel" })
            .click();
        harness.run();
        let loaded = harness.state().loaded.as_ref().unwrap();
        assert_eq!(
            loaded.blockset.as_ref().unwrap().blockset().blocks()[0][0],
            u8::from(accept)
        );
        assert!(!loaded.document.map().is_modified());
        assert!(!loaded.document.map().can_undo());
        assert_eq!(harness.state().is_modified(), accept);
        let polished_map_core::CollisionRow::Asm(values) = &loaded
            .blockset
            .as_ref()
            .unwrap()
            .blockset()
            .collisions()
            .unwrap()[0]
        else {
            panic!("expected named collisions");
        };
        assert_eq!(values[0], if accept { "WATER" } else { "FLOOR" });
    }
    assert_eq!(
        &harness
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .graphics
            .render_block(0)
            .unwrap()
            .pixels[..3],
        &polished_map_core::day_palettes()[2][1]
    );
    harness.state_mut().choose_new();
    harness.run();
    assert!(harness.state().confirm_discard);
    harness.get_by_label("Cancel").click();
    harness.run();
    harness.get_by_label("File").click();
    harness.run();
    harness.get_by_label("Save Blockset").click();
    crate::save_feedback::tests::dismiss(&mut harness);
    assert!(!harness.state().is_modified());
    assert_eq!(std::fs::read(&blocks).unwrap()[0], 1);
    assert_eq!(
        std::fs::read_to_string(&collisions).unwrap(),
        "\ttilecoll WATER, FLOOR, WALL, WALL ; 00\n"
    );
    crate::block_workflow_tests::open_block(&mut harness, 0);
    harness.get_by_label("OK").click();
    harness.run();
    std::fs::write(&blocks, [9; 16]).unwrap();
    harness.state_mut().save_blockset();
    harness.run();
    assert!(
        harness
            .state()
            .error
            .as_ref()
            .is_some_and(|s| s.contains("changed on disk"))
    );
    assert!(harness.state().is_modified());
    assert_eq!(std::fs::read(&blocks).unwrap(), [9; 16]);
    assert_eq!(std::fs::read(path).unwrap(), [0]);
}

#[test]
fn binary_collision_spinners_edit_all_four_quadrants() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Map.1x1.blk");
    let blocks = dir.path().join("blocks.bin");
    let collisions = dir.path().join("collision.bin");
    std::fs::write(&path, [0]).unwrap();
    std::fs::write(&blocks, [0; 16]).unwrap();
    std::fs::write(&collisions, [1, 2, 3, 4]).unwrap();
    let document = MapDocument::open(&path, &OpenOptions::default()).unwrap();
    let blockset = BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap();
    let graphics = Graphics::new(
        vec![polished_map_core::Tile::new([0; 64]).unwrap(); 256],
        vec![polished_map_core::day_palettes()[0]; 256],
        vec![[0; 16]],
    )
    .unwrap();
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new(cc));
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(Loaded {
        recent_path: None,
        trailing_blocks: 0,
        palette_plan: None,
        events: None,
        event_warning: None,
        document,
        graphics,
        blockset: Some(blockset),
        tileset: None,
        allow_256: false,
    }))
    .unwrap();
    harness.state_mut().loading = Some(rx);
    harness.run();
    harness.state_mut().status = "Synthetic binary collisions".into();
    crate::block_workflow_tests::open_block(&mut harness, 0);
    #[cfg(feature = "visual-tests")]
    {
        harness
            .state_mut()
            .block_form
            .as_mut()
            .unwrap()
            .windows_buttons = true;
        harness.remove_cursor();
        harness.run();
        harness.snapshot("block-dialog-binary");
    }
    for (i, label) in [
        "Collision top left",
        "Collision top right",
        "Collision bottom left",
        "Collision bottom right",
    ]
    .into_iter()
    .enumerate()
    {
        harness
            .get_by_label(label)
            .set_value_accesskit(252.0 + i as f64);
        harness.run();
    }
    harness.get_by_label("OK").click();
    harness.run();
    harness.state_mut().save_blockset();
    harness.run();
    assert!(
        harness.state().error.is_none(),
        "{:?}",
        harness.state().error
    );
    assert_eq!(std::fs::read(collisions).unwrap(), [252, 253, 254, 255]);
}

#[test]
fn toolbar_controls_toggle_real_state() {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new(cc));
    harness.run();
    assert!(harness.state().grid);
    harness.get_by_label("Grid").click();
    harness.run();
    assert!(!harness.state().grid);
    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::G);
    harness.run();
    assert!(harness.state().grid);
    harness.get_by_label("Zoom").click();
    harness.run();
    assert!(harness.state().zoom);
}

#[test]
fn open_options_cancel_preserves_editor_state() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("Example.2x1.demo.blk");
    std::fs::write(&path, [0, 0]).unwrap();
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new(cc));
    harness.state_mut().prepare_open(path);
    harness.run();
    harness.get_by_label("Cancel").click();
    harness.run();
    assert!(harness.state().open_form.is_none());
}

#[test]
fn interpolated_paint_visits_intermediate_cells() {
    let mut points = vec![Point { x: 1, y: 1 }];
    append_line(&mut points, Point { x: 1, y: 1 }, Point { x: 4, y: 1 });
    assert_eq!(
        points,
        vec![
            Point { x: 1, y: 1 },
            Point { x: 2, y: 1 },
            Point { x: 3, y: 1 },
            Point { x: 4, y: 1 }
        ]
    );
}

#[test]
fn canvas_drag_is_one_undoable_edit_and_save_uses_the_backend() {
    use polished_map_core::{Tile, day_palettes};
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("Drag.2x1.demo.blk");
    std::fs::write(&path, [0, 0]).unwrap();
    let document = MapDocument::open(&path, &OpenOptions::default()).unwrap();
    let graphics = Graphics::new(
        vec![Tile::new([0; 64]).unwrap()],
        vec![day_palettes()[0]],
        vec![[0; 16]; 2],
    )
    .unwrap();
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .build_eframe(|cc| Editor::new(cc));
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(Loaded {
        recent_path: None,
        trailing_blocks: 0,
        palette_plan: None,
        events: None,
        event_warning: None,
        document,
        graphics,
        blockset: None,
        tileset: None,
        allow_256: true,
    }))
    .unwrap();
    harness.state_mut().loading = Some(rx);
    harness.run();
    harness.state_mut().selected = 1;
    harness.hover_at(Pos2::new(256.0, 159.0));
    harness.drag_at(Pos2::new(256.0, 159.0));
    harness.run();
    harness.hover_at(Pos2::new(288.0, 159.0));
    harness.run();
    harness.drop_at(Pos2::new(288.0, 159.0));
    harness.run();
    assert_eq!(
        harness
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .map()
            .blocks(),
        [1, 1]
    );
    harness.get_by_label("Undo").click();
    harness.run();
    assert_eq!(
        harness
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .map()
            .blocks(),
        [0, 0]
    );
    assert!(
        !harness
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .map()
            .can_undo()
    );
    harness.get_by_label("Redo").click();
    harness.run();
    harness.get_by_label("Save").click();
    harness.run();
    assert_eq!(std::fs::read(&path).unwrap(), [1, 1]);
    assert!(!harness.state().is_modified());
}

#[cfg(feature = "visual-tests")]
#[test]
fn synthetic_editor_appearance() {
    use polished_map_core::{Tile, day_palettes};
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("Synthetic.8x6.demo.blk");
    std::fs::write(
        &path,
        (0..48)
            .map(|i| ((i / 8 + i % 8) % 4) as u8)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let document = MapDocument::open(&path, &OpenOptions::default()).unwrap();
    let tiles = (0..4).map(|h| Tile::new([h; 64]).unwrap()).collect();
    let graphics = Graphics::new(
        tiles,
        vec![day_palettes()[2]; 4],
        (0..4).map(|i| [i; 16]).collect(),
    )
    .unwrap();
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new(cc));
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(Loaded {
        recent_path: None,
        trailing_blocks: 0,
        palette_plan: None,
        events: None,
        event_warning: None,
        document,
        graphics,
        blockset: None,
        tileset: None,
        allow_256: true,
    }))
    .unwrap();
    harness.state_mut().loading = Some(rx);
    harness.run();
    harness.state_mut().status = "Synthetic test map".into();
    harness.run();
    harness.snapshot("synthetic-light");
    harness.get_by_label("View").click();
    harness.run();
    harness.get_by_label("Theme").click();
    harness.run();
    harness.get_by_label("Dark").click();
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.remove_cursor();
    harness.run();
    harness.snapshot("synthetic-dark");
    harness.state_mut().prepare_resize();
    harness.run();
    harness.snapshot("resize-dialog");
}

#[test]
fn resize_dialog_anchor_accept_and_cancel_use_shared_command() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("Resize.2x1.demo.blk");
    std::fs::write(&path, [1, 2]).unwrap();
    let document = MapDocument::open(&path, &OpenOptions::default()).unwrap();
    let graphics = Graphics::new(
        vec![polished_map_core::Tile::new([0; 64]).unwrap()],
        vec![polished_map_core::day_palettes()[0]],
        vec![[0; 16]; 3],
    )
    .unwrap();
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .build_eframe(|cc| Editor::new(cc));
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(Loaded {
        recent_path: None,
        trailing_blocks: 0,
        palette_plan: None,
        events: None,
        event_warning: None,
        document,
        graphics,
        blockset: None,
        tileset: None,
        allow_256: true,
    }))
    .unwrap();
    harness.state_mut().loading = Some(rx);
    harness.run();
    harness.get_by_label("Tools").click();
    harness.run();
    harness.get_by_label("Resize Map…").click();
    harness.run();
    harness.get_by_label("Width").set_value_accesskit(3.0);
    harness.get_by_label("Height").set_value_accesskit(2.0);
    harness.run();
    harness.get_by_label("Bottom right").click();
    harness.run();
    harness.get_by_label("Resize").click();
    harness.run();
    let doc = &harness.state().loaded.as_ref().unwrap().document;
    assert_eq!(doc.map().blocks(), [0, 0, 0, 0, 1, 2]);
    assert!(!doc.map().can_undo());
    assert!(doc.map().is_modified());
    harness.get_by_label("Tools").click();
    harness.run();
    harness.get_by_label("Resize Map…").click();
    harness.run();
    harness.get_by_label("Width").set_value_accesskit(8.0);
    harness.run();
    harness.get_by_label("Cancel").click();
    harness.run();
    assert_eq!(
        harness
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .map()
            .width(),
        3
    );
    assert_eq!(std::fs::read(path).unwrap(), [1, 2]);
}

#[test]
fn new_map_dialog_creates_in_memory_and_cancel_preserves_it() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(directory.path().join("gfx/tilesets/roofs")).unwrap();
    std::fs::create_dir_all(directory.path().join("data/tilesets")).unwrap();
    std::fs::write(directory.path().join("gfx/tilesets/demo.2bpp"), [0; 16]).unwrap();
    std::fs::write(
        directory.path().join("data/tilesets/demo_metatiles.bin"),
        [0x0a; 16],
    )
    .unwrap();
    std::fs::write(
        directory.path().join("gfx/tilesets/roofs/test.2bpp"),
        [255; 9 * 16],
    )
    .unwrap();
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .build_eframe(|cc| Editor::new(cc));
    harness
        .state_mut()
        .prepare_new_in(directory.path().to_path_buf());
    harness.run();
    harness.get_by_label("Width").set_value_accesskit(3.0);
    harness.get_by_label("Height").set_value_accesskit(2.0);
    harness
        .get_by_role_and_label(egui::Role::ComboBox, "Roof")
        .click();
    harness.run();
    harness.get_by_label("test").click();
    harness.run();
    harness.get_by_label("Create map").click();
    // A worker can request another repaint while the harness is settling.
    // Process input/loading frame-by-frame; only demand a settled UI afterwards.
    harness.run_steps(3);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while harness.state().loading.is_some() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
        harness.step();
    }
    assert!(harness.state().loading.is_none(), "map loading timed out");
    harness.run();
    assert!(
        harness.state().error.is_none(),
        "{:?}",
        harness.state().error
    );
    let doc = &harness.state().loaded.as_ref().unwrap().document;
    assert_eq!(doc.map().blocks(), [0; 6]);
    assert!(!doc.has_backing_file());
    assert!(doc.map().is_modified());
    assert_eq!(
        &harness
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .graphics
            .render_block(0)
            .unwrap()
            .pixels[..3],
        &[8, 24, 33]
    );
    assert!(!directory.path().join("NewMap.blk").exists());
    harness
        .state_mut()
        .prepare_new_in(directory.path().to_path_buf());
    harness.run();
    harness.get_by_label("Cancel").click();
    harness.run();
    assert_eq!(
        harness
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .map()
            .width(),
        3
    );
}
