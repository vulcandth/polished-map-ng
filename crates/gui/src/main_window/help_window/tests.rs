use super::*;
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn rules_and_list_boundaries_have_readable_spacing_even_when_wrapped() {
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    let block = |kind, text: &str| Block {
        kind,
        spans: vec![Span {
            text: text.into(),
            bold: false,
            italic: false,
            mono: false,
            small: false,
            link: None,
        }],
    };
    let blocks = [
        block(Kind::Paragraph, "Before rule"),
        block(Kind::Rule, ""),
        block(Kind::Paragraph, "After rule"),
        block(Kind::Bullet, "First item"),
        block(Kind::Bullet, "Second item"),
        block(
            Kind::Paragraph,
            "Following paragraph wraps in a narrow window without overlapping the list above it.",
        ),
    ];
    for width in [220.0, 540.0] {
        for dark in [false, true] {
            let mut h = Harness::builder()
                .with_size(Vec2::new(width, 440.0))
                .with_theme(if dark {
                    egui::Theme::Dark
                } else {
                    egui::Theme::Light
                })
                .build_ui(|ui| render_document(ui, &blocks));
            h.run();
            let before = h.get_by_label("Before rule").rect();
            let after = h.get_by_label("After rule").rect();
            assert!(after.top() - before.bottom() >= 24.0);
            let first = h.get_by_label("First item").rect();
            let second = h.get_by_label("Second item").rect();
            let following = h.get_by_label(&blocks[5].spans[0].text).rect();
            assert!(following.top() - second.bottom() >= second.top() - first.bottom() + 8.0);
            assert!(following.bottom() < 440.0);
            #[cfg(feature = "visual-tests")]
            snapshots.add(h.try_snapshot(format!(
                "help-spacing-{width}-{}",
                if dark { "dark" } else { "light" }
            )));
        }
    }
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}

#[test]
fn current_guide_covers_both_storage_models_and_retains_attribution() {
    let text = document()
        .iter()
        .flat_map(|b| &b.spans)
        .map(|s| s.text.as_str())
        .collect::<String>();
    for phrase in [
        "Polished Map NG",
        "vulcandth",
        "Copyright © 2021 Rangi",
        "Rust and egui/eframe",
        ".blk and .ablk",
        "classic",
        "attributed",
        "Alt+X",
        "Ctrl+Shift+E",
    ] {
        assert!(text.contains(phrase), "missing {phrase}");
    }
    for obsolete in [
        "implemented with C++",
        "Run install.bat",
        "unified development is in progress",
        "polishedmap-plusplus",
        "PROGRAM_",
        "DIR_SEP",
    ] {
        assert!(!text.contains(obsolete), "obsolete {obsolete}");
    }
    let links: Vec<_> = document()
        .iter()
        .flat_map(|b| &b.spans)
        .filter_map(|s| s.link.as_deref())
        .collect();
    assert!(links.contains(&"https://github.com/Rangi42/polished-map"));
    assert!(links.contains(&"https://github.com/vulcandth/polished-map-ng"));
    assert!(links.iter().all(|url| url.starts_with("https://")));
}

#[test]
fn all_launchers_use_current_help_with_stable_window_identities() {
    for application in [
        Application::Ng,
        Application::Parity(documents::Storage::Classic),
        Application::Parity(documents::Storage::Attributed),
    ] {
        let mut help = HelpWindow {
            application,
            storage: application.initial_storage(),
            ..Default::default()
        };
        help.open();
        let expected = if application == Application::Parity(documents::Storage::Attributed) {
            "Polished Map NG++ Help"
        } else {
            "Polished Map NG Help"
        };
        assert_eq!(help.builder(Pos2::ZERO).title.as_deref(), Some(expected));
    }
}

#[test]
fn native_window_builder_uses_source_size_offset_and_reopen_position() {
    let mut help = HelpWindow::default();
    help.open();
    let builder = help.builder(Pos2::new(100.0, 200.0));
    assert_eq!(builder.inner_size, Some(Vec2::new(700.0, 500.0)));
    assert_eq!(builder.position, Some(Pos2::new(148.0, 248.0)));
    assert_eq!(builder.active, Some(true));
    assert_eq!(builder.resizable, Some(true));
    help.reposition = false;
    help.native_size = Vec2::new(820.0, 620.0);
    assert!(help.builder(Pos2::ZERO).position.is_none());
    help.open();
    assert!(help.reposition && help.reset_scroll);
    assert_eq!(
        help.builder(Pos2::ZERO).inner_size,
        Some(Vec2::new(820.0, 620.0))
    );
}

#[test]
fn help_geometry_scrolling_links_and_close_without_a_native_window() {
    // Every launcher uses the same current guide; the generator separately
    // preserves and verifies the original manuals as provenance evidence.
    let mut help = HelpWindow {
        application: Application::Ng,
        ..Default::default()
    };
    help.open();
    let mut h = Harness::builder()
        .with_size(Vec2::new(700.0, 500.0))
        .with_context_setup(fonts::install_bundled)
        .with_theme(egui::Theme::Light)
        .build_ui_state(|ui, state: &mut HelpWindow| state.content(ui), help);
    h.run();
    let area = h.state().bounds;
    let ok = h.get_by_label("OK").rect();
    assert_eq!(ok.size(), Vec2::new(80.0, 22.0));
    assert_eq!(area.right() - ok.right(), 10.0);
    assert_eq!(area.bottom() - ok.bottom(), 10.0);
    assert_eq!(h.state().scroll, 0.0);
    h.set_size(Vec2::new(820.0, 620.0));
    h.run();
    let area = h.state().bounds;
    let ok = h.get_by_label("OK").rect();
    assert_eq!(area.right() - ok.right(), 10.0);
    assert_eq!(area.bottom() - ok.bottom(), 10.0);
    h.set_size(Vec2::new(700.0, 500.0));
    h.run();
    h.remove_cursor();
    h.run();
    #[cfg(feature = "visual-tests")]
    h.snapshot("help-document-top");
    h.get_by_role_and_label(
        egui::Role::Link,
        "https://github.com/vulcandth/polished-map-ng",
    )
    .click();
    let mut links = Vec::new();
    for _ in 0..4 {
        h.step();
        for command in &h.output().platform_output.commands {
            if let egui::OutputCommand::OpenUrl(url) = command {
                links.push(url.url.clone());
            }
        }
    }
    assert_eq!(links, ["https://github.com/vulcandth/polished-map-ng"]);
    h.hover_at(area.center());
    h.run();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: Vec2::new(0.0, -100000.0),
        modifiers: egui::Modifiers::NONE,
    });
    // Egui's smooth wheel animation needs more frames for this end-of-document jump.
    h.run_steps(8);
    h.run();
    assert!(h.state().scroll > 500.0);
    h.remove_cursor();
    h.run();
    #[cfg(feature = "visual-tests")]
    h.snapshot("help-document-bottom");
    h.get_by_label("OK").click();
    h.run();
    assert!(!h.state().open);
    h.state_mut().open();
    h.run();
    assert_eq!(h.state().scroll, 0.0);
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(!h.state().open);
}

#[test]
fn menu_f1_and_nonmodal_help_preserve_editor_state_and_modal_guards() {
    let (dir, mut h) = crate::main_window::tileset_editor_tests::fixture(false, false);
    h.get_by_label("Help").click();
    h.run();
    h.get_by_role_and_label(egui::Role::MenuItem, "Help")
        .click();
    h.run();
    assert!(h.state().help_window.open);
    // Help is not included in the editor's modal guard: controls outside its
    // embedded test window and the native parent remain usable.
    h.get_by_label("View").click();
    h.run();
    h.get_by_role_and_label(egui::Role::CheckBox, "Grid")
        .click();
    h.run();
    assert!(!h.state().grid && h.state().help_window.open);
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(!h.state().help_window.open);
    h.key_press(egui::Key::F1);
    h.run();
    assert!(h.state().help_window.open);
    h.get_by_label("Close window").click();
    h.run();
    assert!(!h.state().help_window.open);
    h.state_mut().prepare_resize();
    h.run();
    h.key_press(egui::Key::F1);
    h.run();
    assert!(!h.state().help_window.open);
    h.get_by_label("Cancel").click();
    h.run();
    assert!(!h.state().is_modified());
    assert_eq!(
        std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
        [0, 1]
    );
    h.state_mut().close_map();
    h.key_press(egui::Key::F1);
    h.run();
    assert!(h.state().help_window.open);
}
