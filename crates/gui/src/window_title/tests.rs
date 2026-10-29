use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use std::{
    fs,
    time::{Duration, Instant},
};

fn titles(h: &Harness<'_, Editor>) -> Vec<String> {
    h.output()
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .unwrap()
        .commands
        .iter()
        .filter_map(|command| {
            if let egui::ViewportCommand::Title(title) = command {
                Some(title.clone())
            } else {
                None
            }
        })
        .collect()
}

fn finish_load(h: &mut Harness<'_, Editor>) -> Vec<String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut commands = Vec::new();
    while h.state().loading.is_some() || h.state().pending_open.is_some() {
        assert!(Instant::now() < deadline, "load worker did not complete");
        std::thread::sleep(Duration::from_millis(5));
        h.step();
        commands.extend(titles(h));
    }
    commands
}

#[test]
fn title_tracks_committed_new_open_save_as_and_close_without_dirty_suffix() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    // The warning includes a user filename outside the bundled font's coverage.
    // Match the runtime's tofu policy; this test checks names/commands, not glyphs.
    h.ctx
        .set_missing_glyph_policy(egui::MissingGlyphPolicy::Tofu);
    let original = format!("{WINDOW_TITLE} - Map.2x1.blk");
    assert_eq!(
        h.state().last_window_title.as_deref(),
        Some(original.as_str())
    );
    h.step();
    assert!(
        titles(&h).is_empty(),
        "unchanged frames must not repeat native commands"
    );
    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 1,
    });
    h.step();
    assert!(h.state().is_modified());
    assert!(
        titles(&h).is_empty(),
        "upstream never adds a dirty asterisk"
    );

    h.state_mut()
        .save_as_path(dir.path().join("missing/Failed.blk"));
    h.step();
    assert!(h.state().error.is_some());
    assert!(titles(&h).is_empty());
    assert_eq!(h.state().window_title(), original);
    h.state_mut().error = None;
    let saved = dir.path().join("é & 新地图");
    h.state_mut().save_as_path(saved);
    h.step();
    assert!(h.state().error.is_none());
    let renamed = format!("{WINDOW_TITLE} - é & 新地图.blk");
    assert_eq!(titles(&h), std::slice::from_ref(&renamed));
    assert_eq!(fs::read(dir.path().join("é & 新地图.blk")).unwrap(), [1, 1]);
    h.step();
    assert!(titles(&h).is_empty());

    h.state_mut().apply(Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 0,
    });
    h.state_mut().request_close_map();
    h.step();
    assert!(titles(&h).is_empty());
    h.key_press(egui::Key::Escape);
    h.run();
    assert_eq!(h.state().window_title(), renamed);
    h.state_mut().request_close_map();
    h.run();
    h.get_by_label("OK").click();
    h.step();
    assert!(h.state().loaded.is_none());
    assert_eq!(titles(&h), [WINDOW_TITLE]);

    let ctx = h.ctx.clone();
    for (name, new_map, valid, expected) in [
        ("NewMap.blk", true, true, "New Map"),
        ("Missing.blk", false, false, "New Map"),
        ("Opened.2x1.BLK", false, true, "Opened.2x1.BLK"),
    ] {
        if !new_map && valid {
            fs::write(dir.path().join(name), [0, 1]).unwrap();
        }
        let previous = h.state().window_title();
        h.state_mut().start_load(
            OpenForm {
                path: dir.path().join(name),
                root: dir.path().to_string_lossy().into(),
                tileset: "demo".into(),
                width: 2,
                height: 1,
                new_map,
                ..Default::default()
            },
            &ctx,
        );
        assert_eq!(
            h.state().window_title(),
            previous,
            "pending work must not rename the loaded map"
        );
        let commands = finish_load(&mut h);
        let expected = format!("{WINDOW_TITLE} - {expected}");
        assert_eq!(h.state().window_title(), expected);
        assert_eq!(commands, if valid { vec![expected] } else { vec![] });
        assert_eq!(h.state().error.is_none(), valid);
        h.state_mut().error = None;
    }
    h.state_mut().prepare_open(dir.path().join("Map.2x1.blk"));
    h.run();
    h.get_by_label("Cancel").click();
    h.step();
    assert!(titles(&h).is_empty());
    assert!(h.state().window_title().ends_with("Opened.2x1.BLK"));
}

#[test]
fn chosen_name_and_original_png_suffix_rules_survive_path_canonicalization() {
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
    for (name, png) in [
        ("Alias.blk", "Alias.png"),
        ("Mixed.bLk", "Mixed.png"),
        ("Town.map", "Town.map.png"),
        ("NoSuffix", "NoSuffix.png"),
        ("a.b.c.blk", "a.b.c.png"),
        ("é & 新地图.blk", "é & 新地图.png"),
    ] {
        h.state_mut().loaded.as_mut().unwrap().recent_path = Some(dir.path().join(name));
        h.step();
        assert_eq!(titles(&h), [format!("{WINDOW_TITLE} - {name}")]);
        let loaded = h.state().loaded.as_ref().unwrap();
        assert_eq!(loaded.print_filename(), OsStr::new(png));
        assert!(
            loaded.document.path().ends_with("Map.2x1.blk"),
            "display name must not change the I/O target"
        );
    }
    h.state_mut().loaded.as_mut().unwrap().document =
        MapDocument::new_map(dir.path(), 2, 1).unwrap();
    h.step();
    assert_eq!(titles(&h), [format!("{WINDOW_TITLE} - New Map")]);
    assert_eq!(
        h.state().loaded.as_ref().unwrap().print_filename(),
        "New Map.png"
    );
    // A lossy UI caption must never rewrite a native path spelling.
    #[cfg(unix)]
    let native = {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(b"bad-\xff.blk".to_vec())
    };
    #[cfg(windows)]
    let native = {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[0xd800, 46, 98, 108, 107])
    };
    #[cfg(any(unix, windows))]
    {
        let document =
            MapDocument::open(&dir.path().join("Map.2x1.blk"), &OpenOptions::default()).unwrap();
        let loaded = h.state_mut().loaded.as_mut().unwrap();
        loaded.document = document;
        loaded.recent_path = Some(PathBuf::from(&native));
        h.step();
        assert_eq!(
            h.state().window_title(),
            format!("{WINDOW_TITLE} - {}", native.to_string_lossy())
        );
        assert_eq!(
            h.state()
                .loaded
                .as_ref()
                .unwrap()
                .recent_path
                .as_ref()
                .unwrap()
                .as_os_str(),
            native
        );
    }
}

#[test]
fn empty_editor_restores_fixed_root_title_once() {
    let mut h = Harness::builder().build_eframe(|cc| Editor::new(cc));
    assert_eq!(h.state().window_title(), WINDOW_TITLE);
    h.state_mut().last_window_title = None;
    h.step();
    assert_eq!(titles(&h), [WINDOW_TITLE]);
    h.step();
    assert!(titles(&h).is_empty());
}
