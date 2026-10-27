use polished_map_core::{PaletteBank, PaletteSet};

#[test]
fn palette_layouts_select_expected_sets_and_preserve_unrelated_colors() {
    use PaletteSet::*;
    for (count, current, expected) in [
        (1, Day, Custom),
        (2, Indoor, Day),
        (3, Morn, Morn),
        (4, Indoor, Indoor),
        (5, Custom, Day),
        (6, Custom, Custom),
        (8, Day, Custom),
        (16, Nite, Nite),
        (24, Indoor, Day),
        (32, Indoor, Indoor),
        (40, Custom, Day),
        (48, Custom, Custom),
    ] {
        let mut bank = PaletteBank::default();
        let input: Vec<_> = (0..count).map(|n| [[n as u8; 3]; 4]).collect();
        assert_eq!(bank.apply(&input, current), expected, "count {count}");
        let target = if count <= 6 { 8 } else { 0 };
        let first_set = match count {
            1 | 8 => Custom,
            2 | 16 => Day,
            _ => Morn,
        };
        assert_eq!(bank.colors(first_set)[target], input[0]);
    }
    let mut bank = PaletteBank::default();
    let day = *bank.colors(Day);
    assert_eq!(bank.apply(&[[[0; 3]; 4]; 7], Nite), Nite);
    assert_eq!(*bank.colors(Day), day);
}

#[test]
fn extended_water_layouts_and_roof_updates_keep_hue_order() {
    use PaletteSet::*;
    for count in [42, 43, 44] {
        let mut bank = PaletteBank::default();
        let input: Vec<_> = (0..count).map(|n| [[n as u8; 3]; 4]).collect();
        let selected = bank.apply(&input, Custom);
        assert_eq!(selected, if count == 44 { Custom } else { Day });
        assert_eq!(bank.colors(Morn)[3], input[40]);
        assert_eq!(
            bank.colors(Day)[3],
            input[if count == 42 { 40 } else { 41 }]
        );
        assert_eq!(
            bank.colors(Nite)[3],
            input[if count == 42 { 41 } else { 42 }]
        );
        if count == 44 {
            assert_eq!(bank.colors(Custom)[3], input[43]);
            assert_eq!(bank.colors(Custom)[0], input[24]);
        }
        let old_roof = bank.colors(Day)[6];
        bank.set_roof(Day, [1, 2, 3], [4, 5, 6]);
        assert_eq!(
            bank.colors(Day)[6],
            [old_roof[0], [4, 5, 6], [1, 2, 3], old_roof[3]]
        );
    }
}
