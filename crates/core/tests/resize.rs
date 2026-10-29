use polished_map_core::{Command, Map, Point, ResizeAnchor};

#[test]
fn overworld_buffer_accounts_for_border_and_widens_maximum_dimensions() {
    for (w, h, bytes) in [
        (1, 1, 49),
        (20, 43, 1274),
        (20, 44, 1300),
        (21, 44, 1350),
        (255, 255, 68121),
    ] {
        let map = Map::from_bytes(w, h, vec![0; usize::from(w) * usize::from(h)]).unwrap();
        assert_eq!(map.overworld_buffer_bytes(), bytes);
    }
    let mut map = Map::from_bytes(1, 1, vec![0]).unwrap();
    map.resize(20, 44, ResizeAnchor::Center).unwrap();
    assert_eq!(map.overworld_buffer_bytes(), 1300);
}

#[test]
fn all_nine_anchors_place_old_pixels_and_zero_fill_the_remainder() {
    // Source-derived acceptance vectors: a 2×2 map inside a 4×4 destination.
    for (anchor, (x, y)) in ResizeAnchor::ALL.into_iter().zip([
        (0, 0),
        (1, 0),
        (2, 0),
        (0, 1),
        (1, 1),
        (2, 1),
        (0, 2),
        (1, 2),
        (2, 2),
    ]) {
        let mut map = Map::from_bytes(2, 2, vec![1, 2, 3, 4]).unwrap();
        map.resize(4, 4, anchor).unwrap();
        let mut expected = vec![0; 16];
        expected[y * 4 + x] = 1;
        expected[y * 4 + x + 1] = 2;
        expected[(y + 1) * 4 + x] = 3;
        expected[(y + 1) * 4 + x + 1] = 4;
        assert_eq!(map.blocks(), expected, "{anchor:?}");
    }
}

#[test]
fn shrinking_center_truncates_toward_zero_and_resize_clears_both_histories() {
    let mut map = Map::from_bytes(4, 3, (1..=12).collect()).unwrap();
    map.apply(&Command::Paint {
        points: vec![Point { x: 0, y: 0 }],
        block: 42,
    })
    .unwrap();
    map.undo();
    assert!(map.can_redo());
    // -1 / 2 == 0: centered 4×3 -> 3×2 drops the right and bottom edges.
    map.resize(3, 2, ResizeAnchor::Center).unwrap();
    assert_eq!(map.blocks(), [1, 2, 3, 5, 6, 7]);
    assert!(!map.can_undo());
    assert!(!map.can_redo());
    assert!(map.is_modified());
    map.resize(2, 1, ResizeAnchor::BottomRight).unwrap();
    assert_eq!(map.blocks(), [6, 7]);
}

#[test]
fn invalid_and_unchanged_dimensions_leave_data_and_history_intact() {
    let mut map = Map::from_bytes(2, 2, vec![1, 2, 3, 4]).unwrap();
    map.apply(&Command::Replace { from: 1, to: 5 }).unwrap();
    for (w, h) in [(0, 2), (2, 0), (256, 2), (2, 256), (u16::MAX, u16::MAX)] {
        assert!(map.resize(w, h, ResizeAnchor::Center).is_err());
        assert_eq!(map.blocks(), [5, 2, 3, 4]);
        assert!(map.can_undo());
    }
    assert!(!map.resize(2, 2, ResizeAnchor::TopLeft).unwrap());
    assert!(map.can_undo());
    map.undo();
    assert_eq!(map.blocks(), [1, 2, 3, 4]);
}
