use polished_map_core::{
    BlockBrush, CollisionRow, Point,
    plusplus::{AttributeBlockset, AttributeCommand, BankOrder, TileReference},
};

#[test]
fn bank_boundaries_are_not_contiguous_vram_banks() {
    for (order, cases) in [
        (
            BankOrder::Bank1First,
            [(0x00, 0), (0x00, 8), (0x80, 8), (0x80, 0)],
        ),
        (
            BankOrder::Bank0First,
            [(0x00, 0), (0x00, 8), (0x80, 0), (0x80, 8)],
        ),
    ] {
        for (i, (offset, attribute)) in cases.into_iter().enumerate() {
            let t = TileReference::from_index((i * 128) as u16, order).unwrap();
            assert_eq!((t.offset(), t.attribute()), (offset, attribute));
        }
        for i in 0..512 {
            assert_eq!(TileReference::from_index(i, order).unwrap().index(order), i);
        }
    }
    assert!(TileReference::from_index(512, BankOrder::default()).is_err());
    let t = TileReference::from_bytes(255, 255);
    assert_eq!(t.attribute(), 0xef);
    assert_eq!(t.palette(), 7);
    assert!(t.bank1() && t.x_flip() && t.y_flip() && t.priority());
}

#[test]
fn attribute_lengths_padding_and_automatic_512_threshold_match_source() {
    let b = AttributeBlockset::from_bytes(&[0x80; 32], &[0xff; 16]).unwrap();
    assert_eq!(b.attribute_bytes(), [[0xef; 16], [0; 16]].concat());
    assert!(!b.is_modified());
    for size in [1, 15, 17, 33, 48] {
        assert!(AttributeBlockset::from_bytes(&[0; 32], &vec![0; size]).is_err());
    }
    for size in [0, 1, 15, 17, 4112] {
        assert!(AttributeBlockset::from_bytes(&vec![0; size], &[]).is_err());
    }
    let b = AttributeBlockset::from_bytes(&[0x80; 16], &[8; 16]).unwrap();
    assert!(!b.enables_512_tiles(BankOrder::Bank1First)); // Exactly $100.
    assert!(b.enables_512_tiles(BankOrder::Bank0First)); // $180 in alternate order.
}

#[test]
fn copy_swap_resize_keep_three_domains_in_step_and_failures_are_atomic() {
    let mut b = AttributeBlockset::from_bytes(
        &[[1; 16], [2; 16]].concat(),
        &[[0xa5; 16], [0x48; 16]].concat(),
    )
    .unwrap()
    .with_collisions(vec![
        CollisionRow::Binary([1; 4]),
        CollisionRow::Binary([2; 4]),
    ])
    .unwrap();
    b.apply(&AttributeCommand::Swap { a: 0, b: 1 }).unwrap();
    assert_eq!(b.offsets().blocks()[0], [2; 16]);
    assert_eq!(b.attributes()[0], [0x48; 16]);
    assert_eq!(
        b.offsets().collisions().unwrap()[0],
        CollisionRow::Binary([2; 4])
    );
    b.apply(&AttributeCommand::Copy { from: 0, to: 1 }).unwrap();
    assert_eq!(b.block(0).unwrap(), b.block(1).unwrap());
    b.apply(&AttributeCommand::Resize { count: 1 }).unwrap();
    b.apply(&AttributeCommand::Resize { count: 256 }).unwrap();
    assert_eq!(b.offsets().blocks()[255], [0; 16]);
    assert_eq!(b.attributes()[255], [0; 16]);
    assert_eq!(
        b.offsets().collisions().unwrap()[255],
        CollisionRow::Binary([0; 4])
    );
    b.mark_saved();
    let before = (b.offsets().bytes(), b.attribute_bytes());
    assert!(
        b.apply_batch(&[
            AttributeCommand::Set {
                block: 0,
                offsets: [7; 16],
                attributes: [7; 16]
            },
            AttributeCommand::Resize { count: 257 },
        ])
        .is_err()
    );
    assert_eq!((b.offsets().bytes(), b.attribute_bytes()), before);
    assert!(!b.is_modified());
    assert!(
        b.apply(&AttributeCommand::Paint {
            block: 0,
            at: Point { x: 0, y: 0 },
            tile: 512,
            palette: 0,
            x_flip: false,
            y_flip: false,
            priority: false,
            brush: BlockBrush::default(),
            order: BankOrder::default()
        })
        .is_err()
    );
    assert_eq!((b.offsets().bytes(), b.attribute_bytes()), before);
    assert!(!b.is_modified());
}

#[test]
fn flipped_brush_reverses_footprint_and_keeps_occurrence_flags() {
    let mut b = AttributeBlockset::from_bytes(&[0; 16], &[]).unwrap();
    b.apply(&AttributeCommand::Paint {
        block: 0,
        at: Point { x: 0, y: 0 },
        tile: 0x100,
        palette: 6,
        x_flip: true,
        y_flip: true,
        priority: true,
        brush: BlockBrush {
            ctrl: true,
            ..Default::default()
        },
        order: BankOrder::Bank0First,
    })
    .unwrap();
    assert_eq!(
        b.offsets().blocks()[0],
        [0x91, 0x90, 0, 0, 0x81, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        b.attributes()[0],
        [0xe6, 0xe6, 0, 0, 0xe6, 0xe6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}
