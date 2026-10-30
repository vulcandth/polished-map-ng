use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{BlockCommand, EventCommand, TileCommand};
use polished_map_project::{EventDocument, prepare_palette_export};
use std::{collections::BTreeMap, fs};

pub(crate) fn dismiss(h: &mut Harness<'_, Editor>) {
    h.run();
    for _ in 0..16 {
        if !h.state().save_ui.busy() && h.state().overworld_warning.is_none() {
            return;
        }
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        h.get_by_label("OK").click();
        h.run();
    }
    panic!("save feedback did not finish");
}

fn message<'a>(h: &'a Harness<'_, Editor>) -> &'a str {
    &h.state().save_ui.messages.front().unwrap().text
}

fn dirty_all(h: &mut Harness<'_, Editor>, root: &std::path::Path) {
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    let loaded = h.state_mut().loaded.as_mut().unwrap();
    let paint = TileCommand::Paint {
        tile: 0,
        points: vec![Point { x: 0, y: 0 }],
        hue: 3,
    };
    let tiles = loaded.tileset.as_mut().unwrap();
    tiles.apply_batch(std::slice::from_ref(&paint)).unwrap();
    tiles
        .roof_mut()
        .unwrap()
        .roof_mut()
        .apply_batch(&[paint])
        .unwrap();
    loaded
        .blockset
        .as_mut()
        .unwrap()
        .blockset_mut()
        .apply(&BlockCommand::Set {
            block: 0,
            tiles: [1; 16],
        })
        .unwrap();
    let path = root.join("Map.asm");
    fs::write(&path, "Map_Events:\n\twarp_event 0, 0, MAP, 1 ; keep\n").unwrap();
    let mut events = EventDocument::open(&path).unwrap();
    events
        .apply(&[EventCommand::Move {
            index: 0,
            x: 1,
            y: 0,
        }])
        .unwrap();
    loaded.events = Some(events);
}

#[test]
fn save_all_waits_for_each_domain_and_feedback_blocks_parent_input_and_exit() {
    let (dir, mut h) = crate::roof_editor_tests::fixture();
    dirty_all(&mut h, dir.path());
    h.state_mut().save();
    h.run();
    let expected = [
        "Saved demo.png!",
        "Saved demo_palette_map.asm!",
        "Saved demo.png!",
        "Saved demo_metatiles.bin!",
        "Saved Map.asm!",
        "Saved Map.2x1.blk!",
    ];
    for (index, text) in expected.into_iter().enumerate() {
        assert_eq!(message(&h), text);
        h.get_by_label(text);
        let loaded = h.state().loaded.as_ref().unwrap();
        assert_eq!(loaded.document.map().is_modified(), index < 5);
        assert_eq!(
            loaded
                .events
                .as_ref()
                .unwrap()
                .script()
                .events()
                .is_modified(),
            index < 4
        );
        assert_eq!(
            loaded.blockset.as_ref().unwrap().blockset().is_modified(),
            index < 3
        );
        assert_eq!(
            loaded
                .tileset
                .as_ref()
                .unwrap()
                .roof()
                .unwrap()
                .roof()
                .tileset()
                .is_modified(),
            index < 2
        );
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::S);
        h.run();
        assert_eq!(message(&h), text, "Save cannot restart an active flow");
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
        h.run();
        assert!(h.state().loaded.is_some());
        h.input_mut()
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .events
            .push(egui::ViewportEvent::Close);
        h.step();
        assert!(h.output().viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::CancelClose))
        }));
        h.key_press(egui::Key::Enter);
        h.run();
    }
    assert!(!h.state().save_ui.busy());
    assert!(!h.state().is_modified());
    assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [1, 1]);
    assert!(
        fs::read_to_string(dir.path().join("Map.asm"))
            .unwrap()
            .contains("warp_event  1,  0, MAP, 1 ; keep")
    );
}

#[test]
fn save_all_conflict_after_acknowledgment_stops_remaining_writes() {
    let (dir, mut h) = crate::roof_editor_tests::fixture();
    dirty_all(&mut h, dir.path());
    h.state_mut().save();
    h.run();
    // The external edit happens while the source-style success dialog is open.
    let roof = dir.path().join("gfx/tilesets/roofs/demo.2bpp");
    fs::write(&roof, [0x5a; 144]).unwrap();
    for _ in 0..2 {
        h.key_press(egui::Key::Enter);
        h.run();
    }
    assert!(h.state().error.is_some());
    assert!(h.state().save_ui.steps.is_empty());
    assert!(h.state().save_ui.messages.is_empty());
    let loaded = h.state().loaded.as_ref().unwrap();
    assert!(loaded.document.map().is_modified());
    assert!(loaded.blockset.as_ref().unwrap().blockset().is_modified());
    assert!(
        loaded
            .events
            .as_ref()
            .unwrap()
            .script()
            .events()
            .is_modified()
    );
    assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [0, 1]);
    assert_eq!(fs::read(roof).unwrap(), [0x5a; 144]);
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(
        !h.state().save_ui.busy(),
        "dismissing an error must not resume writes"
    );
}

#[test]
fn clean_saves_acknowledge_without_touching_external_files_and_export_names_destination() {
    let (dir, mut h) = crate::roof_editor_tests::fixture();
    let path = dir.path().join("Map.asm");
    fs::write(&path, "\twarp_event 0, 0, MAP, 1\n").unwrap();
    h.state_mut().loaded.as_mut().unwrap().events = Some(EventDocument::open(&path).unwrap());
    fs::write(&path, "external edit").unwrap();
    for (action, expected) in [
        (SaveStep::Tileset, "Saved demo!"),
        (SaveStep::Roof, "Saved demo!"),
        (SaveStep::Blocks, "Saved demo_metatiles.bin!"),
        (SaveStep::Events, "Saved Map.asm!"),
        (SaveStep::Map, "Saved Map.2x1.blk!"),
    ] {
        h.state_mut().save_ui.steps.push_back(action);
        h.run();
        assert!(h.state().error.is_none());
        assert_eq!(message(&h), expected);
        dismiss(&mut h);
    }
    assert_eq!(fs::read_to_string(path).unwrap(), "external edit");
    assert!(!dir.path().join("gfx/tilesets/demo.png").exists());
    let target = prepare_palette_export(&dir.path().join("Colors.pal")).unwrap();
    h.state_mut().export_palette_target(&target).unwrap();
    h.run();
    assert_eq!(message(&h), "Exported Colors.pal!");
    assert!(target.path().is_file());
    dismiss(&mut h);
}

#[test]
fn map_capacity_warning_precedes_success_and_save_as_uses_committed_name() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    h.state_mut().overworld_map_size = 1;
    h.state_mut().save_as_path(dir.path().join("Copy"));
    h.run();
    assert!(h.state().overworld_warning.is_some());
    h.get_by_label("Warning");
    assert_eq!(message(&h), "Saved Copy.blk!");
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(h.state().overworld_warning.is_none());
    h.get_by_label("Saved Copy.blk!");
    #[cfg(feature = "visual-tests")]
    {
        h.remove_cursor();
        h.run();
        h.snapshot("save-map-success");
    }
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(!h.state().save_ui.busy());
    h.state_mut().save_map();
    h.run();
    assert!(
        h.state().overworld_warning.is_none(),
        "clean save does not warn again"
    );
    assert_eq!(message(&h), "Saved Copy.blk!");
}

#[derive(Default)]
struct Storage(BTreeMap<String, String>);
impl eframe::Storage for Storage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.into(), value);
    }
    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }
    fn flush(&mut self) {}
}

#[test]
fn block_limit_warning_threshold_collision_gate_acknowledgment_and_preferences() {
    for count in [128, 129, 256] {
        for collisions in [false, true] {
            let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
            let path = dir.path().join("Blocks.bin");
            let collision = dir.path().join("Collision.bin");
            fs::write(&path, vec![0; count * 16]).unwrap();
            fs::write(&collision, vec![0; count * 4]).unwrap();
            h.state_mut().loaded.as_mut().unwrap().blockset = Some(
                BlocksetDocument::open_with_collisions(
                    &path,
                    collisions.then_some(collision.as_path()),
                )
                .unwrap(),
            );
            h.state_mut().save_blockset();
            h.run();
            assert_eq!(
                message(&h),
                if collisions {
                    "Saved Blocks.bin\nand Collision.bin!"
                } else {
                    "Saved Blocks.bin!"
                }
            );
            assert!(!h.state().meta128);
            h.key_press(egui::Key::Enter);
            h.run();
            let warn = collisions && count > 128;
            assert_eq!(h.state().save_ui.busy(), warn);
            if warn {
                assert!(!h.state().meta128, "display is not acknowledgment");
                let mut before = Storage::default();
                h.state_mut().store_preferences(&mut before);
                let mut restored = Editor::default();
                restored.restore_preferences(Some(&before));
                assert!(!restored.meta128);
                assert!(message(&h).contains(&format!("Blocks.bin has {count} blocks")));
                assert!(message(&h).contains("LoadMetatiles wraps around past 128 blocks"));
                #[cfg(feature = "visual-tests")]
                if count == 129 {
                    h.remove_cursor();
                    h.run();
                    h.snapshot("save-block-limit-warning");
                }
                h.key_press(egui::Key::Escape);
                h.run();
                assert!(
                    h.state().meta128,
                    "source also acknowledges closing the warning"
                );
                let mut stored = Storage::default();
                h.state_mut().store_preferences(&mut stored);
                let mut restored = Editor::default();
                restored.restore_preferences(Some(&stored));
                assert!(restored.meta128);
                h.state_mut().save_blockset();
                h.run();
                assert_eq!(h.state().save_ui.messages.len(), 1);
                dismiss(&mut h);
                h.state_mut().close_map();
                assert!(
                    h.state().meta128,
                    "Close retains the session acknowledgment"
                );
            }
            assert!(!h.state().is_modified());
        }
    }
    let mut legacy = Storage::default();
    legacy.0.insert(
        "polished-map-ng.preferences".into(),
        "{\"version\":1}".into(),
    );
    let mut restored = Editor::default();
    restored.restore_preferences(Some(&legacy));
    assert!(!restored.meta128 && restored.preferences_writable);
}

#[test]
fn save_as_cancellation_and_invalid_destinations_stop_without_false_success() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    let source = dir.path().join("Blocks.CEL");
    let collision = dir.path().join("Collision.bin");
    let original = "\tdb 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0\n";
    fs::write(&source, original).unwrap();
    fs::write(&collision, [0; 4]).unwrap();
    let mut doc = BlocksetDocument::open_with_collisions(&source, Some(&collision)).unwrap();
    doc.blockset_mut()
        .apply(&BlockCommand::Set {
            block: 0,
            tiles: [1; 16],
        })
        .unwrap();
    h.state_mut().loaded.as_mut().unwrap().blockset = Some(doc);
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    // These are the same result handlers used after native dialogs. Either
    // blockset chooser cancel is None; no destination has been prepared yet.
    for map in [false, true] {
        h.state_mut().save_ui.steps.push_back(SaveStep::Map);
        if map {
            h.state_mut().save_map_choice(None);
        } else {
            h.state_mut().save_blockset_choice(None);
        }
        h.run();
        assert!(!h.state().save_ui.busy());
        assert!(h.state().error.is_none() && h.state().is_modified());
        assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [0, 1]);
        assert_eq!(fs::read_to_string(&source).unwrap(), original);
    }
    let block_copy = dir.path().join("Copy.bin");
    // A paired destination is mandatory; then reject an invalid collision
    // extension before either output is committed.
    for collision_copy in [None, Some(dir.path().join("Bad.blk"))] {
        h.state_mut().save_ui.steps.push_back(SaveStep::Map);
        h.state_mut()
            .save_blockset_choice(Some((block_copy.clone(), collision_copy)));
        h.run();
        assert!(h.state().error.is_some());
        assert!(!h.state().save_ui.busy());
        assert!(!block_copy.exists());
        assert!(!h.state().meta128);
        h.key_press(egui::Key::Escape);
        h.run();
    }
    h.state_mut()
        .save_map_choice(Some(dir.path().join("Bad.map")));
    h.run();
    assert!(h.state().error.is_some());
    assert!(!h.state().save_ui.busy());
    h.key_press(egui::Key::Enter);
    h.run();
    let collision_copy = dir.path().join("CopyCollision.bin");
    h.state_mut()
        .save_blockset_choice(Some((block_copy.clone(), Some(collision_copy.clone()))));
    h.run();
    assert_eq!(message(&h), "Saved Copy.bin\nand CopyCollision.bin!");
    assert_eq!(fs::read(block_copy).unwrap(), [1; 16]);
    assert_eq!(fs::read(collision_copy).unwrap(), [0; 4]);
    assert_eq!(fs::read_to_string(source).unwrap(), original);
    assert_eq!(fs::read(collision).unwrap(), [0; 4]);
    dismiss(&mut h);
    assert!(
        h.state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .map()
            .is_modified()
    );
}

#[test]
fn dirty_block_limit_warning_follows_success_but_never_a_conflict() {
    for conflict in [false, true] {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let path = dir.path().join("Blocks.bin");
        let collision = dir.path().join("Collision.bin");
        fs::write(&path, vec![0; 129 * 16]).unwrap();
        fs::write(&collision, vec![0; 129 * 4]).unwrap();
        let mut doc = BlocksetDocument::open_with_collisions(&path, Some(&collision)).unwrap();
        doc.blockset_mut()
            .apply(&BlockCommand::Set {
                block: 0,
                tiles: [1; 16],
            })
            .unwrap();
        h.state_mut().loaded.as_mut().unwrap().blockset = Some(doc);
        if conflict {
            fs::write(&collision, vec![2; 129 * 4]).unwrap();
        }
        h.state_mut().save();
        h.run();
        if conflict {
            assert!(h.state().error.is_some());
            assert!(!h.state().save_ui.busy());
            assert!(!h.state().meta128 && h.state().is_modified());
            assert_eq!(fs::read(path).unwrap(), vec![0; 129 * 16]);
            assert_eq!(fs::read(collision).unwrap(), vec![2; 129 * 4]);
        } else {
            assert_eq!(message(&h), "Saved Blocks.bin\nand Collision.bin!");
            assert!(!h.state().meta128);
            assert!(!h.state().is_modified());
            assert_eq!(&fs::read(path).unwrap()[..16], &[1; 16]);
            h.key_press(egui::Key::Enter);
            h.run();
            assert!(message(&h).contains("129 blocks"));
            assert!(!h.state().meta128);
            dismiss(&mut h);
            assert!(h.state().meta128);
        }
    }
}

#[test]
fn tileset_success_uses_main_compound_name_and_skips_palette_for_monochrome() {
    for monochrome in [false, true] {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(true, monochrome);
        let root = dir.path();
        fs::write(
            root.join("gfx/tilesets/demo_palette_map.asm"),
            "\ttilepal 0, GRAY\n".repeat(48),
        )
        .unwrap();
        for (name, bytes) in [
            ("demo.before,after", [0, 255].repeat(128)),
            ("before", [255, 0].repeat(128)),
            ("after", vec![255; 256]),
        ] {
            fs::write(root.join(format!("gfx/tilesets/{name}.2bpp")), bytes).unwrap();
        }
        let mut doc = MasterTilesetDocument::open_in_project(
            root,
            "demo.before,after",
            &MasterGraphicsOptions {
                allow_256: true,
                monochrome,
                ..Default::default()
            },
        )
        .unwrap();
        doc.apply_batch(&[TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        }])
        .unwrap();
        h.state_mut().loaded.as_mut().unwrap().tileset = Some(doc);
        h.state_mut().save_tileset();
        h.run();
        assert!(h.state().error.is_none(), "{:?}", h.state().error);
        assert_eq!(message(&h), "Saved demo.before,after.png!");
        assert_eq!(
            h.state().save_ui.messages.len(),
            if monochrome { 1 } else { 2 }
        );
        if !monochrome {
            assert_eq!(
                h.state().save_ui.messages[1].text,
                "Saved demo_palette_map.asm!"
            );
        }
        for name in ["demo.before,after", "before", "after"] {
            assert!(root.join(format!("gfx/tilesets/{name}.png")).is_file());
        }
        dismiss(&mut h);
    }
}
