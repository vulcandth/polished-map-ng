use super::*;
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

fn open(h: &mut Harness<'_, Editor>, path: &Path) {
    h.state_mut().prepare_open(path.to_owned());
    let form = h.state_mut().open_form.as_mut().unwrap();
    form.root = path.parent().unwrap().to_string_lossy().into_owned();
    form.tileset = "demo".into();
    form.width = 2;
    form.height = 1;
    h.run();
    h.get_by_label("Open map").click();
    h.run();
    let deadline = Instant::now() + Duration::from_secs(5);
    while h.state().loading.is_some() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
        h.step();
    }
    assert!(h.state().loading.is_none(), "open worker timed out");
    h.run();
}

#[test]
fn oversized_map_warns_after_palette_choice_and_acknowledgment_publishes_only_the_prefix() {
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for (extension, key) in [
        ("blk", None),
        ("blk", Some(egui::Key::Enter)),
        ("MAP", Some(egui::Key::Escape)),
    ] {
        let (dir, mut h) = tileset_editor_tests::fixture(false, false);
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        h.state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .tileset
            .as_mut()
            .unwrap()
            .palettes_mut()
            .apply_batch(&[PaletteCommand::SetRgb {
                slot: ColorSlot {
                    set: PaletteSet::Custom,
                    palette: 0,
                    hue: 0,
                },
                rgb: [17, 34, 51],
            }])
            .unwrap();
        let palette_hash = h.state().palette_document().content_hash();
        let before = h
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .path()
            .to_owned();
        let history = h.state().recent_maps.clone();
        let path = dir.path().join(format!("Trailing.2x1.{extension}"));
        let bytes = if extension == "blk" {
            vec![1, 0, 1, 1]
        } else {
            b"\tdb $01, $00, $01, $01 ; preserve\n".to_vec()
        };
        fs::write(&path, &bytes).unwrap();
        open(&mut h, &path);
        assert!(h.state().pending_open.is_some() && h.state().pending_map_warning.is_none());
        h.get_by_label("Skip").click();
        h.run();
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        assert_eq!(h.state().pending_map_warning.as_deref(), Some(format!("Warning: Trailing.2x1.{extension}:\n\nThe .blk file is larger than the specified size.").as_str()));
        assert_eq!(h.state().loaded.as_ref().unwrap().document.path(), before);
        assert_eq!(h.state().recent_maps, history);
        assert_eq!(
            h.state().loaded.as_ref().unwrap().document.map().blocks(),
            [1, 1]
        );
        assert!(h.state().loaded.as_ref().unwrap().document.map().can_undo());
        assert!(h.get_by_label("File").accesskit_node().is_disabled());
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
        h.run();
        assert_eq!(
            h.state().loaded.as_ref().unwrap().document.map().blocks(),
            [1, 1]
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
        #[cfg(feature = "visual-tests")]
        if key.is_none() {
            h.remove_cursor();
            h.run();
            snapshots.add(h.try_snapshot("map-length-warning"));
        }
        if let Some(key) = key {
            h.key_press(key);
        } else {
            h.get_by_label("OK").click();
        }
        h.run();
        assert!(h.state().pending_map_warning.is_none() && h.state().pending_open.is_none());
        let map = h.state().loaded.as_ref().unwrap().document.map();
        assert_eq!(map.blocks(), [1, 0]);
        assert!(!map.is_modified() && !map.can_undo());
        assert_ne!(h.state().recent_maps, history);
        assert_eq!(h.state().palette_document().content_hash(), palette_hash);
        assert!(h.state().palette_document().is_modified());
        assert_eq!(fs::read(&path).unwrap(), bytes);
        if extension == "blk" {
            // Main_Window::save_map(false) leaves a clean map alone, then
            // writes the dimension-sized prefix after an actual edit.
            h.state_mut().save_map();
            h.run();
            assert_eq!(fs::read(&path).unwrap(), bytes);
            h.state_mut().apply(Command::Paint {
                points: vec![Point { x: 0, y: 0 }],
                block: 0,
            });
            h.state_mut().save_map();
            h.run();
            assert_eq!(fs::read(&path).unwrap(), [0, 0]);
            assert!(h.state().error.is_none());
        }
        assert!(h.state().pending_map_warning.is_none());
    }
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}

#[test]
fn short_map_preserves_the_current_document_and_exact_length_opens_without_warning() {
    let (dir, mut h) = tileset_editor_tests::fixture(false, false);
    let before = h
        .state()
        .loaded
        .as_ref()
        .unwrap()
        .document
        .path()
        .to_owned();
    let history = h.state().recent_maps.clone();
    let path = dir.path().join("Next.2x1.blk");
    fs::write(&path, [1]).unwrap();
    open(&mut h, &path);
    assert!(
        h.state()
            .error
            .as_ref()
            .unwrap()
            .contains("expected 2 map bytes, found 1")
    );
    assert!(h.state().pending_open.is_none() && h.state().pending_map_warning.is_none());
    assert_eq!(h.state().loaded.as_ref().unwrap().document.path(), before);
    assert_eq!(h.state().recent_maps, history);
    h.get_by_label("OK").click();
    h.run();
    fs::write(&path, [1, 0]).unwrap();
    open(&mut h, &path);
    assert!(h.state().error.is_none() && h.state().pending_map_warning.is_none());
    assert!(h.state().pending_open.is_none());
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().blocks(),
        [1, 0]
    );
    assert_eq!(fs::read(&path).unwrap(), [1, 0]);
}
