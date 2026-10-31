use polished_map_core::{
    Map, Palette, Point, PrintOptions, PrintStyle, Tile, UNDEFINED_COLORS,
    plusplus::{AttributeBlockset, BankOrder, Graphics, RawTileCommand as C, RawTileset},
};

fn raw() -> RawTileset {
    RawTileset::new(vec![Tile::new([0; 64]).unwrap(); 512], vec![false; 512]).unwrap()
}
fn pixel(image: &polished_map_core::RgbImage, x: usize, y: usize) -> [u8; 3] {
    image.pixels[(y * image.width as usize + x) * 3..(y * image.width as usize + x) * 3 + 3]
        .try_into()
        .unwrap()
}

#[test]
fn high_tile_edits_define_raw_pixels_without_block_attributes_and_reject_partial_batches() {
    let mut r = raw();
    r.apply(&C::Paint {
        tile: 511,
        points: vec![Point { x: 1, y: 2 }],
        hue: 3,
    })
    .unwrap();
    assert!(r.defined()[511]);
    assert_eq!(r.tiles()[511].hues()[17], 3);
    r.apply(&C::Fill {
        tile: 511,
        at: Point { x: 0, y: 0 },
        hue: 1,
    })
    .unwrap();
    assert_eq!(
        r.tiles()[511].hues().iter().filter(|&&v| v == 1).count(),
        63
    );
    r.apply(&C::SwapHues {
        tile: 511,
        a: 1,
        b: 3,
    })
    .unwrap();
    assert_eq!(r.tiles()[511].hues()[17], 1);
    r.apply(&C::Copy { from: 511, to: 256 }).unwrap();
    assert_eq!(r.snapshot(511).unwrap(), r.snapshot(256).unwrap());
    r.apply(&C::Swap { a: 256, b: 0 }).unwrap();
    assert!(!r.defined()[256]);
    assert!(r.defined()[0]);
    r.mark_saved();
    let before = r.clone();
    assert!(
        r.apply_batch(&[
            C::Delete { tile: 511 },
            C::Paint {
                tile: 512,
                points: vec![],
                hue: 0
            }
        ])
        .is_err()
    );
    assert_eq!(r, before);
    assert!(
        r.apply(&C::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }, Point { x: 8, y: 0 }],
            hue: 2
        })
        .is_err()
    );
    assert_eq!(r, before);
    r.apply(&C::Delete { tile: 511 }).unwrap();
    assert!(!r.defined()[511]);
    assert_eq!(*r.tiles()[511].hues(), [0; 64]);
}

#[test]
fn rgb_paste_clips_at_256_or_512_and_preserves_uncovered_pixels() {
    for (allow_512, start, count) in [
        (false, 255, 1),
        (true, 255, 2),
        (true, 511, 1),
        (false, 256, 0),
    ] {
        let mut r = raw();
        r.apply(&C::ImportRgb {
            tile: start,
            width: 9,
            height: 9,
            pixels: vec![0; 9 * 9 * 3],
            allow_512,
        })
        .unwrap();
        assert_eq!(r.defined().iter().filter(|&&v| v).count(), count);
        if count > 0 {
            assert_eq!(*r.tiles()[usize::from(start)].hues(), [3; 64]);
            if count == 2 {
                assert_eq!(&r.tiles()[271].hues()[0..8], &[3; 8]);
                assert_eq!(r.tiles()[271].hues()[8], 0);
            }
        }
    }
    let mut r = raw();
    let before = r.clone();
    assert!(
        r.apply(&C::ImportRgb {
            tile: 0,
            width: 2049,
            height: 1,
            pixels: vec![],
            allow_512: true
        })
        .is_err()
    );
    assert_eq!(r, before);
}

#[test]
fn trimming_clears_interior_blanks_and_export_uses_linear_offsets() {
    let mut r = RawTileset::new(vec![Tile::new([0; 64]).unwrap(); 512], vec![true; 512]).unwrap();
    r.apply(&C::Paint {
        tile: 511,
        points: vec![Point { x: 0, y: 0 }],
        hue: 3,
    })
    .unwrap();
    r.mark_saved();
    let b = AttributeBlockset::from_bytes(&[0x81; 16], &[0; 16]).unwrap(); // default ordinal $181.
    r.trim_unused(&b, BankOrder::Bank1First);
    assert!(!r.defined()[0]);
    assert!(!r.defined()[384]);
    assert!(r.defined()[385]);
    assert!(r.defined()[511]);
    assert!(!r.is_modified());
    let image = r.export_region(511, None).unwrap();
    assert_eq!((image.width, image.height), (128, 8));
    assert_eq!(pixel(&image, 0, 0), [0; 3]);
    assert_eq!(pixel(&image, 8, 0), [255; 3]);
    assert!(r.export_region(511, Some(2)).is_err());
    assert!(r.export_region(512, None).is_err());
    assert!(raw().export_region(0, None).is_err());
}

#[test]
fn occurrence_palette_bank_flips_roof_and_print_share_a_single_compositor() {
    let mut r = raw();
    let mut hues = [0; 64];
    hues[0] = 1;
    hues[7] = 2;
    hues[56] = 3;
    r.apply(&C::Set {
        tile: 256,
        value: polished_map_core::plusplus::RawTileSnapshot {
            pixels: hues.to_vec(),
            defined: true,
        },
    })
    .unwrap();
    let mut offsets = [0x80; 16];
    offsets[4] = 0x0a;
    let mut attrs = [8; 16];
    attrs[1] = 0x29;
    attrs[2] = 0x4a;
    attrs[3] = 0xeb;
    attrs[4] = 4;
    let b = AttributeBlockset::from_bytes(&offsets, &attrs).unwrap();
    let palettes: [Palette; 8] =
        std::array::from_fn(|p| std::array::from_fn(|h| [p as u8, h as u8, 77]));
    let g = Graphics::new(r, b, palettes, BankOrder::Bank1First)
        .with_roof(std::array::from_fn(|_| Tile::new([2; 64]).unwrap()));
    let image = g.render_block(0).unwrap();
    assert_eq!(pixel(&image, 0, 0), [0, 1, 77]);
    assert_eq!(pixel(&image, 8, 0), [1, 2, 77]);
    assert_eq!(pixel(&image, 16, 0), [2, 3, 77]);
    assert_eq!(pixel(&image, 24, 0), [3, 0, 77]);
    assert_eq!(pixel(&image, 0, 8), [4, 2, 77]);
    assert_eq!(
        g.block_priorities(0).unwrap(),
        std::array::from_fn(|i| i == 3)
    );
    assert!(g.block_priorities(255).is_none());
    let map = Map::from_bytes(2, 1, vec![0, 255]).unwrap();
    let rendered = g.render_map(&map).unwrap();
    assert_eq!(pixel(&rendered, 32, 0), UNDEFINED_COLORS[0]);
    let printed = g
        .render_print(
            &map,
            None,
            PrintOptions::default(),
            PrintStyle::default(),
            |_, _| panic!("no text requested"),
        )
        .unwrap();
    assert_eq!(rendered, printed);
    let overlay = g
        .render_print(
            &map,
            None,
            PrintOptions {
                priority: true,
                ..Default::default()
            },
            PrintStyle::default(),
            |_, _| panic!("no text requested"),
        )
        .unwrap();
    assert_ne!(overlay, printed);
    assert_eq!(pixel(&overlay, 0, 0), pixel(&printed, 0, 0));
}
