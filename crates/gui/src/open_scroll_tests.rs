//! master Main_Window::open_map initial scroll and publication boundaries.
use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn options(h: &mut Harness<'_, Editor>, root: &Path, width: u8, height: u8, new: bool) {
    if new {
        h.state_mut().prepare_new_in(root.to_owned());
    } else {
        let path = root.join("Next.blk");
        fs::write(&path, vec![0; usize::from(width) * usize::from(height)]).unwrap();
        h.state_mut().prepare_open(path);
    }
    let form = h.state_mut().open_form.as_mut().unwrap();
    form.root = root.to_string_lossy().into_owned();
    form.tileset = "demo".into();
    form.width = u16::from(width);
    form.height = u16::from(height);
    h.run();
}

fn accept(h: &mut Harness<'_, Editor>, new: bool) {
    h.get_by_label(if new { "Create map" } else { "Open map" })
        .click();
    h.step();
    let deadline = Instant::now() + Duration::from_secs(5);
    while h.state().loading.is_some() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
        h.step();
    }
    assert!(h.state().loading.is_none(), "map worker timed out");
    h.run();
}

fn wheel(h: &mut Harness<'_, Editor>, delta: Vec2) {
    h.hover_at(Pos2::new(400.0, 300.0));
    h.run();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
}

#[test]
fn source_open_scroll_preserves_integer_halves_and_the_strict_margin_branch() {
    for size in [32.0, 64.0] {
        let margin = 3.0 * size;
        for (surplus, expected) in [
            (-1.0, 0.0),
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 1.0),
            (margin - 1.0, margin / 2.0 - 1.0),
            (margin, margin / 2.0),
            (margin + 1.0, margin),
            (10_000.0, margin),
        ] {
            assert_eq!(
                geometry::open_scroll(20_000.0, 20_016.0 - surplus, size, 16.0),
                expected
            );
        }
    }
}

#[test]
fn open_and_new_position_each_map_from_its_size_zoom_rulers_and_current_viewport() {
    #[cfg(feature = "visual-tests")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for scale in [1.0, 2.0] {
        for new in [false, true] {
            for (width, height, zoom, rulers, client, expected) in [
                (2, 1, false, false, [800.0, 600.0], [0.0, 0.0]),
                (2, 20, false, false, [800.0, 600.0], [0.0, 96.0]),
                (20, 1, false, false, [800.0, 600.0], [96.0, 0.0]),
                (15, 12, false, false, [800.0, 600.0], [16.0, 31.0]),
                (15, 12, false, true, [800.0, 600.0], [24.0, 39.0]),
                (17, 13, false, false, [800.0, 600.0], [48.0, 47.0]),
                (18, 14, false, false, [800.0, 600.0], [96.0, 96.0]),
                (20, 20, true, true, [800.0, 600.0], [192.0, 192.0]),
                // The source allowance is unconditional, even for an exact fit.
                (2, 1, true, true, [800.0, 600.0], [8.0, 0.0]),
                (2, 1, true, false, [335.0, 262.0], [192.0, 192.0]),
                (2, 1, true, true, [335.0, 262.0], [192.0, 192.0]),
            ] {
                let (dir, mut h) = tileset_editor_tests::fixture(false, false);
                h.set_pixels_per_point(scale);
                h.run();
                h.set_size(Vec2::from(client));
                h.state_mut().zoom = zoom;
                h.state_mut().rulers = rulers;
                h.run();
                options(&mut h, dir.path(), width, height, new);
                accept(&mut h, new);
                assert!(h.state().error.is_none(), "{:?}", h.state().error);
                assert_eq!(
                    h.state().canvas_scroll,
                    Vec2::from(expected),
                    "{width}x{height}, zoom={zoom}, rulers={rulers}, new={new}, dpi={scale}"
                );
                assert_eq!(h.state().ruler_scroll, h.state().canvas_scroll);
                assert!(!h.state().initial_canvas_scroll);
                assert_eq!(h.state().sidebar_scroll, 0.0);
                let map = h.state().loaded.as_ref().unwrap().document.map();
                assert_eq!((map.width(), map.height()), (width, height));
                assert_eq!(map.is_modified(), new);
                assert!(!map.can_undo() && !map.can_redo());
                assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [0, 1]);
                if !new {
                    assert_eq!(
                        fs::read(dir.path().join("Next.blk")).unwrap(),
                        vec![0; usize::from(width) * usize::from(height)]
                    );
                }
                #[cfg(feature = "visual-tests")]
                if !new && scale == 1.0 {
                    let name = match (width, height, zoom, rulers, client[0] as u16) {
                        (15, 12, false, false, 800) => Some("open-scroll-centered"),
                        (20, 20, true, true, 800) => Some("open-scroll-large-zoomed"),
                        (2, 1, true, true, 335) => Some("open-scroll-minimum-client"),
                        _ => None,
                    };
                    if let Some(name) = name {
                        h.remove_cursor();
                        h.run();
                        snapshots.add(h.try_snapshot(name));
                    }
                }
            }
        }
    }
    #[cfg(feature = "visual-tests")]
    snapshots.unwrap();
}

#[test]
fn oversized_warning_does_not_reposition_until_acknowledgment() {
    let (dir, mut h) = tileset_editor_tests::fixture(false, false);
    options(&mut h, dir.path(), 20, 20, false);
    accept(&mut h, false);
    wheel(&mut h, Vec2::splat(-20.0));
    options(&mut h, dir.path(), 15, 12, false);
    fs::write(dir.path().join("Next.blk"), vec![0; 181]).unwrap();
    accept(&mut h, false);
    assert!(h.state().pending_map_warning.is_some());
    assert_eq!(h.state().canvas_scroll, Vec2::splat(116.0));
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().width(),
        20
    );
    assert!(!h.state().initial_canvas_scroll);
    h.get_by_label("OK").click();
    h.run();
    assert!(h.state().pending_map_warning.is_none());
    assert_eq!(h.state().canvas_scroll, Vec2::new(16.0, 31.0));
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().width(),
        15
    );
    assert_eq!(fs::read(dir.path().join("Next.blk")).unwrap().len(), 181);
}

#[test]
fn opening_repositions_once_and_failed_or_canceled_options_preserve_the_old_view() {
    let (dir, mut h) = tileset_editor_tests::fixture(false, false);
    options(&mut h, dir.path(), 20, 20, false);
    accept(&mut h, false);
    wheel(&mut h, Vec2::splat(-20.0));
    assert_eq!(h.state().canvas_scroll, Vec2::splat(116.0));
    for new in [false, true] {
        options(&mut h, dir.path(), 15, 12, new);
        h.get_by_label("Cancel").click();
        h.run();
        assert_eq!(h.state().canvas_scroll, Vec2::splat(116.0));
    }
    // A listed file can become unreadable or invalid after discovery; retain
    // the load-failure check without bypassing the selector's membership guard.
    fs::write(dir.path().join("gfx/tilesets/broken.2bpp"), [0]).unwrap();
    options(&mut h, dir.path(), 15, 12, false);
    h.state_mut().open_form.as_mut().unwrap().tileset = "broken".into();
    accept(&mut h, false);
    assert!(h.state().error.is_some());
    assert_eq!(h.state().canvas_scroll, Vec2::splat(116.0));
    assert_eq!(
        h.state().loaded.as_ref().unwrap().document.map().width(),
        20
    );
    h.get_by_label("OK").click();
    h.run();
    options(&mut h, dir.path(), 15, 12, false);
    accept(&mut h, false);
    assert_eq!(h.state().canvas_scroll, Vec2::new(16.0, 31.0));
    wheel(&mut h, Vec2::splat(5.0));
    assert_eq!(h.state().canvas_scroll, Vec2::new(11.0, 26.0));
    h.set_size(Vec2::new(900.0, 700.0));
    h.run();
    assert_eq!(
        h.state().canvas_scroll,
        Vec2::new(11.0, 26.0),
        "resize must not repeat the opening position"
    );
    h.state_mut().close_map();
    h.run();
    assert!(!h.state().initial_canvas_scroll);
    options(&mut h, dir.path(), 20, 20, true);
    accept(&mut h, true);
    assert_eq!(
        h.state().canvas_scroll,
        Vec2::new(46.0, 96.0),
        "Close must not suppress the next opening position"
    );
}
