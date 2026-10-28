use polished_map_core::{ColorSlot, PaletteBank, PaletteCommand, PaletteError, PaletteSet};
fn slot(palette: u8, hue: u8) -> ColorSlot {
    ColorSlot {
        set: PaletteSet::Day,
        palette,
        hue,
    }
}

#[test]
fn channels_hex_copy_and_swap_preserve_original_precision_rules() {
    let mut bank = PaletteBank::default();
    let a = slot(0, 0);
    let b = slot(1, 2);
    for value in 0..=31 {
        bank.edit(&[
            PaletteCommand::SetRgb {
                slot: a,
                rgb: [3, 127, 249],
            },
            PaletteCommand::Channel {
                slot: a,
                channel: 1,
                value,
            },
        ])
        .unwrap();
        assert_eq!(
            bank.color(a).unwrap(),
            [3, (u16::from(value) * 33 / 4) as u8, 249]
        );
    }
    for (hex, rgb) in [
        ("", [0, 0, 0]),
        ("f", [0, 0, 15]),
        ("12Ab3", [1, 42, 179]),
        ("037ff9", [3, 127, 249]),
    ] {
        bank.edit(&[PaletteCommand::Hex {
            slot: a,
            value: hex.into(),
        }])
        .unwrap();
        assert_eq!(bank.color(a).unwrap(), rgb);
    }
    bank.edit(&[PaletteCommand::Copy { from: a, to: b }])
        .unwrap();
    assert_eq!(bank.color(b).unwrap(), [0, 123, 255]);
    bank.edit(&[PaletteCommand::Swap { a, b }]).unwrap();
    assert_eq!(bank.color(b).unwrap(), [3, 127, 249]);
    assert_eq!(bank.color(a).unwrap(), [0, 123, 255]);
}

#[test]
fn invalid_batches_are_atomic_and_export_keeps_order_labels_and_rgb5() {
    let mut bank = PaletteBank::default();
    let a = slot(8, 3);
    for invalid in [
        PaletteCommand::Hex {
            slot: a,
            value: "Ｇ00000".into(),
        },
        PaletteCommand::Hex {
            slot: a,
            value: "1234567".into(),
        },
        PaletteCommand::SetRgb {
            slot: slot(9, 0),
            rgb: [0; 3],
        },
        PaletteCommand::Channel {
            slot: a,
            channel: 3,
            value: 0,
        },
        PaletteCommand::Channel {
            slot: a,
            channel: 0,
            value: 32,
        },
        PaletteCommand::Swap { a, b: slot(0, 4) },
    ] {
        let before = bank.clone();
        assert!(
            bank.edit(&[
                PaletteCommand::SetRgb {
                    slot: a,
                    rgb: [123; 3]
                },
                invalid
            ])
            .is_err()
        );
        assert_eq!(bank, before);
    }
    assert_eq!(bank.color(slot(255, 255)), Err(PaletteError::Slot));
    for (hue, rgb) in [[8, 16, 24], [32, 40, 48], [56, 64, 72], [80, 88, 96]]
        .into_iter()
        .enumerate()
    {
        bank.edit(&[PaletteCommand::SetRgb {
            slot: ColorSlot {
                set: PaletteSet::Morn,
                palette: 0,
                hue: hue as u8,
            },
            rgb,
        }])
        .unwrap();
    }
    let text = bank.export_rgb(false);
    assert!(text.starts_with("; morn\n\tRGB 01,02,03, 07,08,09, 04,05,06, 10,11,12 ; gray\n"));
    assert_eq!(text.lines().count(), 54);
    assert!(text.contains("; darkness\n"));
    assert!(text.contains("; custom\n"));
    let mono = bank.export_rgb(true);
    assert_eq!(mono.lines().count(), 6);
    assert!(mono.lines().nth(3).unwrap().ends_with(" ; darkness"));
}
