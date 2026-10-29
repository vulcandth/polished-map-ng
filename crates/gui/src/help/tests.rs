use super::*;
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn complete_bundled_documents_preserve_upstream_text_links_and_platform_paths() {
    for (source, executable, path) in [
        (
            include_str!("../../assets/help/windows.json"),
            "polishedmap.exe",
            "pokecrystal\\maps\\SomeMap.blk",
        ),
        (
            include_str!("../../assets/help/unix.json"),
            "polishedmap",
            "pokecrystal/maps/SomeMap.blk",
        ),
    ] {
        let blocks: Vec<Block> = serde_json::from_str(source).unwrap();
        assert_eq!(blocks.len(), 95);
        let text = blocks
            .iter()
            .flat_map(|b| &b.spans)
            .map(|s| s.text.as_str())
            .collect::<String>();
        assert!(text.contains("Copyright © 2021 Rangi"));
        assert!(text.contains(executable) && text.contains(path));
        assert!(text.ends_with("ice_path.pal defines a CUSTOM palette."));
        assert!(!text.contains("PROGRAM_") && !text.contains("DIR_SEP"));
        let links: Vec<_> = blocks
            .iter()
            .flat_map(|b| &b.spans)
            .filter_map(|s| s.link.as_deref())
            .collect();
        assert_eq!(links.len(), 16);
        assert!(links.contains(&"https://github.com/Rangi42/polished-map"));
        assert!(
            links
                .iter()
                .all(|url| url.starts_with("https://") || url.starts_with("http://"))
        );
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
    // Stable document text for the cross-platform visual baseline; both native
    // platform expansions are independently checked above and by the generator.
    let mut help = HelpWindow {
        windows_paths: true,
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
    h.get_by_role_and_label(egui::Role::Link, "crowdmap")
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
    assert_eq!(links, ["https://github.com/yenatch/crowdmap"]);
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
    let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
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
