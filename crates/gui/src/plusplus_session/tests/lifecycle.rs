//! Full-profile lifecycle checks through the production plusplus constructor.
use super::*;

fn bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut result = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            result.extend(bytes(&path));
        } else {
            result.push((path.clone(), fs::read(path).unwrap()));
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[test]
fn every_plusplus_dirty_domain_guards_open_close_and_exit_without_writes() {
    for domain in ["map", "events", "tiles", "roof", "blocks"] {
        let (dir, mut h) = fixture();
        let before = bytes(dir.path());
        let ctx = h.ctx.clone();
        match domain {
            "map" => h.state_mut().apply(Command::Paint {
                points: vec![Point { x: 0, y: 0 }],
                block: 1,
            }),
            "events" => h
                .state_mut()
                .event_action(event_workflows::EventAction::Move(0, 1, 0), &ctx),
            _ => {
                let assets = h
                    .state_mut()
                    .loaded
                    .as_mut()
                    .unwrap()
                    .plus_assets
                    .as_mut()
                    .unwrap();
                let paint = RawTileCommand::Paint {
                    tile: 0,
                    points: vec![Point { x: 0, y: 0 }],
                    hue: 3,
                };
                match domain {
                    "tiles" => assets.tiles.apply_batch(&[paint]).unwrap(),
                    "roof" => assets.roof.as_mut().unwrap().apply_batch(&[paint]).unwrap(),
                    "blocks" => assets
                        .blocks
                        .blockset_mut()
                        .apply(&AttributeCommand::Set {
                            block: 0,
                            offsets: [1; 16],
                            attributes: [0xe7; 16],
                        })
                        .unwrap(),
                    _ => unreachable!(),
                }
            }
        }
        h.run();
        assert!(h.state().is_modified(), "{domain}");
        // Cancel each destructive continuation through the real keyboard route.
        for (modifiers, key) in [
            (egui::Modifiers::COMMAND, egui::Key::O),
            (egui::Modifiers::COMMAND, egui::Key::W),
            (egui::Modifiers::ALT, egui::Key::F4),
        ] {
            h.key_press_modifiers(modifiers, key);
            h.run();
            assert!(h.state().confirm_discard, "{domain}");
            h.get_by_label("Cancel").click();
            h.run();
            assert!(!h.state().confirm_discard && !h.state().exit_approved);
            assert!(h.state().loaded.is_some() && h.state().is_modified());
            assert_eq!(bytes(dir.path()), before, "{domain}");
        }
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
        h.run();
        h.get_by_label("OK").click();
        h.run();
        assert!(h.state().loaded.is_none());
        assert!(h.state().profile() == documents::Profile::PlusPlus);
        assert_eq!(
            h.state().palette_document().colors(PaletteSet::Day).len(),
            8
        );
        assert_eq!(
            h.state().last_window_title.as_deref(),
            Some("Polished Map NG++ — development")
        );
        assert_eq!(bytes(dir.path()), before, "{domain}");
    }
}

#[test]
fn plusplus_new_map_create_save_close_and_reopen_reuses_shared_workflow() {
    let (dir, mut h) = fixture();
    let before = bytes(dir.path());
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::N);
    h.run();
    let form = h.state().open_form.as_ref().unwrap();
    assert!(form.new_map && form.profile == documents::Profile::PlusPlus);
    assert_eq!((form.width, form.height), (10, 9));
    assert!(form.path.ends_with("NewMap.ablk"));
    h.get_by_label("Width").set_value_accesskit(3.0);
    h.get_by_label("Height").set_value_accesskit(2.0);
    h.get_by_label("Create map").click();
    h.step();
    wait(&mut h);
    let loaded = h.state().loaded.as_ref().unwrap();
    assert_eq!(loaded.document.map().blocks(), [0; 6]);
    assert!(!loaded.document.has_backing_file() && loaded.document.map().is_modified());
    assert!(loaded.plus_assets.is_some());
    assert_eq!(bytes(dir.path()), before);
    // The native save chooser is injected at its returned-path boundary.
    let destination = dir.path().join("New.3x2.demo.ablk");
    h.state_mut().save_as_path(destination);
    crate::save_feedback::tests::dismiss(&mut h);
    assert!(h.state().error.is_none(), "{:?}", h.state().error);
    let saved = dir.path().join("New.3x2.demo.ablk");
    assert_eq!(fs::read(&saved).unwrap(), [0; 6]);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
    h.run();
    assert!(h.state().loaded.is_none() && !h.state().confirm_discard);
    open(&mut h, &saved, false);
    let loaded = h.state().loaded.as_ref().unwrap();
    assert_eq!(
        (
            loaded.document.map().width(),
            loaded.document.map().height()
        ),
        (3, 2)
    );
    assert_eq!(loaded.document.map().blocks(), [0; 6]);
    assert!(!loaded.document.map().is_modified());
    for (path, original) in before {
        assert_eq!(fs::read(path).unwrap(), original);
    }
}
