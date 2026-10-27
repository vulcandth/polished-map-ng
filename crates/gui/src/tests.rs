use super::*;
use egui_kittest::{Harness, kittest::Queryable};

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
    harness.get_by_label("Zoom").click();
    harness.run();
    assert!(harness.state().zoom);
}

#[test]
fn open_options_cancel_preserves_editor_state() {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| Editor::new(cc));
    harness
        .state_mut()
        .prepare_open(PathBuf::from("Example.2x1.demo.blk"));
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
    tx.send(Ok(Loaded { document, graphics })).unwrap();
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
    tx.send(Ok(Loaded { document, graphics })).unwrap();
    harness.state_mut().loading = Some(rx);
    harness.run();
    harness.state_mut().status = "Synthetic test map".into();
    harness.run();
    harness.snapshot("synthetic-light");
    harness.get_by_label("View").click();
    harness.run();
    harness.get_by_label("Dark appearance").click();
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.remove_cursor();
    harness.run();
    harness.snapshot("synthetic-dark");
}
