use polished_map_core::{BlockBrush, BlockCommand, Blockset, Point};

#[test]
fn collisions_follow_copy_swap_resize_and_invalid_drafts_are_atomic() {
    use polished_map_core::CollisionRow;
    let mut set = Blockset::from_bytes(&[0; 32])
        .unwrap()
        .with_collisions(vec![
            CollisionRow::Binary([1, 2, 3, 4]),
            CollisionRow::Binary([5, 6, 7, 8]),
        ])
        .unwrap();
    set.apply(&BlockCommand::Swap { a: 0, b: 1 }).unwrap();
    assert_eq!(
        set.collisions().unwrap()[0],
        CollisionRow::Binary([5, 6, 7, 8])
    );
    set.apply(&BlockCommand::Copy { from: 0, to: 1 }).unwrap();
    assert_eq!(set.collisions().unwrap()[0], set.collisions().unwrap()[1]);
    set.apply(&BlockCommand::Resize { count: 3 }).unwrap();
    assert_eq!(set.collisions().unwrap()[2], CollisionRow::Binary([0; 4]));
    set.mark_saved();
    let before = set.bytes();
    let collisions = set.collisions().unwrap().to_vec();
    assert!(
        set.apply_batch(&[
            BlockCommand::Set {
                block: 0,
                tiles: [9; 16]
            },
            BlockCommand::SetCollisions {
                block: 0,
                collision: CollisionRow::Asm(["FLOOR".into(), "".into(), "".into(), "".into()])
            }
        ])
        .is_err()
    );
    assert_eq!(set.bytes(), before);
    assert_eq!(set.collisions().unwrap(), collisions);
    assert!(!set.is_modified());
    assert!(
        Blockset::from_bytes(&[0; 16])
            .unwrap()
            .with_collisions(vec![])
            .is_err()
    );
    let mut asm = Blockset::from_bytes(&[0; 16])
        .unwrap()
        .with_collisions(vec![CollisionRow::Asm(std::array::from_fn(|_| {
            "FLOOR".into()
        }))])
        .unwrap();
    for bad in ["A,B", "A;comment", "A\nB", "A\rB", "A\0B"] {
        assert!(
            asm.apply(&BlockCommand::SetCollisions {
                block: 0,
                collision: CollisionRow::Asm(std::array::from_fn(|_| bad.into()))
            })
            .is_err()
        );
        assert!(!asm.is_modified());
    }
    assert!(
        asm.apply(&BlockCommand::SetCollisions {
            block: 0,
            collision: CollisionRow::Asm(std::array::from_fn(|_| "A".repeat(4097)))
        })
        .is_err()
    );
}

#[test]
fn all_modifier_patterns_match_independent_vectors() {
    let expected = [
        [32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [32, 33, 0, 0, 33, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [32, 33, 0, 0, 48, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            32, 33, 34, 35, 48, 49, 50, 51, 64, 65, 66, 67, 80, 81, 82, 83,
        ],
        [32, 33, 0, 0, 34, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        ],
        [32, 33, 0, 0, 30, 31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            32, 33, 34, 35, 28, 29, 30, 31, 24, 25, 26, 27, 20, 21, 22, 23,
        ],
    ];
    for (i, expected) in expected.into_iter().enumerate() {
        let brush = BlockBrush {
            shift: i & 1 != 0,
            ctrl: i & 2 != 0,
            alt: i & 4 != 0,
        };
        let mut draft = [0; 16];
        brush
            .paint(&mut draft, Point { x: 0, y: 0 }, 32, true)
            .unwrap();
        assert_eq!(draft, expected, "pattern {i}");
    }
}

#[test]
fn placement_clips_wraps_and_skips_reserved_slots_exactly() {
    let brush = BlockBrush {
        ctrl: true,
        ..Default::default()
    };
    let mut draft = [7; 16];
    brush
        .paint(&mut draft, Point { x: 3, y: 3 }, 0x5f, false)
        .unwrap();
    assert_eq!(&draft[..15], &[7; 15]);
    assert_eq!(draft[15], 0x5f);
    brush
        .paint(&mut draft, Point { x: 0, y: 0 }, 0x5f, false)
        .unwrap();
    assert_eq!(
        [draft[0], draft[1], draft[4], draft[5]],
        [0x5f, 0x80, 0x8f, 0x90]
    );
    brush
        .paint(&mut draft, Point { x: 0, y: 0 }, 0xdf, false)
        .unwrap();
    assert_eq!(
        [draft[0], draft[1], draft[4], draft[5]],
        [0xdf, 0x80, 0x8f, 0x90]
    );
    brush
        .paint(&mut draft, Point { x: 0, y: 0 }, 0xff, true)
        .unwrap();
    assert_eq!([draft[0], draft[1], draft[4], draft[5]], [255, 0, 15, 16]);
    let before = draft;
    assert!(
        brush
            .paint(&mut draft, Point { x: 4, y: 0 }, 0, true)
            .is_err()
    );
    assert_eq!(draft, before);
}

#[test]
fn block_commands_validate_atomically_and_dirty_is_independent() {
    let mut set = Blockset::from_bytes(&[1; 32]).unwrap();
    assert!(!set.is_modified());
    assert!(set.apply(&BlockCommand::Copy { from: 0, to: 2 }).is_err());
    assert!(!set.is_modified());
    set.apply(&BlockCommand::Set {
        block: 0,
        tiles: [2; 16],
    })
    .unwrap();
    set.apply(&BlockCommand::Swap { a: 0, b: 1 }).unwrap();
    assert_eq!(set.block(1).unwrap(), &[2; 16]);
    set.apply(&BlockCommand::Resize { count: 3 }).unwrap();
    assert_eq!(set.block(2).unwrap(), &[0; 16]);
    set.apply(&BlockCommand::Resize { count: 1 }).unwrap();
    set.apply(&BlockCommand::Resize { count: 2 }).unwrap();
    assert_eq!(set.block(1).unwrap(), &[0; 16]);
    set.mark_saved();
    set.apply(&BlockCommand::Copy { from: 0, to: 0 }).unwrap();
    assert!(set.is_modified());
    for len in [0, 1, 15, 17, 4097] {
        assert!(Blockset::from_bytes(&vec![0; len]).is_err());
    }
}
