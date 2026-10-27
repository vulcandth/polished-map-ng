use polished_map_core::{Command, Map, Point};

#[test]
fn fill_is_four_connected_and_undo_redo_is_reversible() {
    let original = vec![1, 0, 1, 1, 0, 1, 0, 1, 1];
    let mut map = Map::from_bytes(3, 3, original.clone()).unwrap();
    map.apply(&Command::Fill {
        at: Point { x: 0, y: 0 },
        block: 2,
    })
    .unwrap();
    assert_eq!(map.blocks(), &[2, 0, 1, 2, 0, 1, 0, 1, 1]);
    assert!(map.is_modified());
    assert!(map.undo());
    assert_eq!(map.blocks(), original);
    assert!(map.is_modified()); // Upstream keeps dirty status until save.
    assert!(map.redo());
    assert_eq!(map.blocks()[0], 2);
}

#[test]
fn rejected_paint_does_not_apply_its_valid_prefix() {
    let mut map = Map::from_bytes(2, 1, vec![0, 0]).unwrap();
    assert!(
        map.apply(&Command::Paint {
            points: vec![Point { x: 0, y: 0 }, Point { x: 2, y: 0 }],
            block: 8,
        })
        .is_err()
    );
    assert_eq!(map.blocks(), &[0, 0]);
    assert!(!map.can_undo());
}

#[test]
fn history_caps_at_100_and_new_edits_invalidate_redo() {
    let mut map = Map::from_bytes(1, 1, vec![0]).unwrap();
    for block in 1..=110 {
        map.apply(&Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block,
        })
        .unwrap();
    }
    for _ in 0..100 {
        assert!(map.undo());
    }
    assert!(!map.undo());
    assert_eq!(map.blocks(), &[10]);
    map.apply(&Command::Replace { from: 10, to: 7 }).unwrap();
    assert!(!map.can_redo());
}

#[test]
fn swap_is_simultaneous_and_no_op_click_has_its_own_history_entry() {
    let mut map = Map::from_bytes(3, 1, vec![1, 2, 3]).unwrap();
    map.apply(&Command::Swap { a: 1, b: 2 }).unwrap();
    assert_eq!(map.blocks(), &[2, 1, 3]);
    assert!(!map.apply(&Command::Replace { from: 7, to: 8 }).unwrap());
    map.undo();
    assert_eq!(map.blocks(), &[2, 1, 3]);
    map.undo();
    assert_eq!(map.blocks(), &[1, 2, 3]);
}

#[test]
fn undo_after_saving_must_not_lose_unsaved_changes() {
    let mut map = Map::from_bytes(1, 1, vec![0]).unwrap();
    map.apply(&Command::Replace { from: 0, to: 1 }).unwrap();
    map.mark_saved();
    assert!(!map.is_modified());
    map.undo();
    assert!(map.is_modified());
    assert_eq!(map.blocks(), [0]);
}

#[test]
fn map_boundaries_are_enforced() {
    assert!(Map::from_bytes(0, 1, vec![]).is_err());
    assert!(Map::from_bytes(256, 1, vec![0; 256]).is_err());
    assert!(Map::from_bytes(1, 1, vec![0, 0]).is_err());
    assert!(Map::from_bytes(255, 255, vec![255; 255 * 255]).is_ok());
}
