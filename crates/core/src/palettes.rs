//! Palette layouts adapted from master colors.cpp; see upstream.lock.toml. LGPL-3.0-only.
use serde::{Deserialize, Serialize};

use crate::Palette;

#[path = "palette_edit.rs"]
mod edit;
pub use edit::{ColorSlot, PaletteCommand, PaletteError};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(usize)]
pub enum PaletteSet {
    Morn,
    #[default]
    Day,
    Nite,
    Darkness,
    Indoor,
    Custom,
}

impl PaletteSet {
    pub const ALL: [Self; 6] = [
        Self::Morn,
        Self::Day,
        Self::Nite,
        Self::Darkness,
        Self::Indoor,
        Self::Custom,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteBank {
    colors: [[Palette; 9]; 6],
}

impl Default for PaletteBank {
    fn default() -> Self {
        Self {
            colors: DEFAULT_COLORS.map(|set| {
                set.map(|palette| palette.map(|rgb| rgb.map(|c| (u16::from(c) * 33 / 4) as u8)))
            }),
        }
    }
}

impl PaletteBank {
    pub fn colors(&self, set: PaletteSet) -> &[Palette; 9] {
        &self.colors[set as usize]
    }

    /// Apply one of the upstream palette-file layouts, returning its selected set.
    /// Colors are already expanded to 8-bit RGB in internal hue order.
    /// Unrecognized palette counts leave the bank and selection unchanged.
    pub fn apply(&mut self, palettes: &[Palette], selected: PaletteSet) -> PaletteSet {
        use PaletteSet::*;
        let count = palettes.len();
        let mono = (1..=6).contains(&count);
        let groups = if mono { count } else { count / 8 };
        let sets: &[PaletteSet] = match count {
            1 | 8 => &[Custom],
            2 | 16 => &[Day, Nite],
            3 | 24 => &[Morn, Day, Nite],
            4 | 32 => &[Morn, Day, Nite, Indoor],
            5 | 40 | 42 | 43 => &[Morn, Day, Nite, Darkness, Indoor],
            44 => &[Morn, Day, Nite, Custom, Indoor],
            6 | 48 => &PaletteSet::ALL,
            _ => return selected,
        };
        for (index, &set) in sets.iter().enumerate() {
            if mono {
                self.colors[set as usize][8] = palettes[index];
            } else {
                self.colors[set as usize][..8]
                    .copy_from_slice(&palettes[index * 8..(index + 1) * 8]);
            }
        }
        if matches!(count, 42..=44) {
            self.colors[Morn as usize][3] = palettes[40];
            self.colors[Day as usize][3] = palettes[if count == 42 { 40 } else { 41 }];
            self.colors[Nite as usize][3] = palettes[if count == 42 { 41 } else { 42 }];
            if count == 44 {
                self.colors[Custom as usize][3] = palettes[43];
            }
        }
        if groups == 1 {
            Custom
        } else if count == 44 || sets.contains(&selected) {
            selected
        } else {
            Day
        }
    }

    /// Replace only the two middle roof hues; white/black remain from the base palette.
    pub fn set_roof(&mut self, set: PaletteSet, light: [u8; 3], dark: [u8; 3]) {
        self.colors[set as usize][6][2] = light;
        self.colors[set as usize][6][1] = dark;
    }
}

// Defaults below preserve every RGB5 triple from upstream colors.cpp.
const DEFAULT_COLORS: [[[[u8; 3]; 4]; 9]; 6] = [
    [
        [[28, 31, 16], [13, 13, 13], [21, 21, 21], [7, 7, 7]],
        [[28, 31, 16], [30, 10, 6], [31, 19, 24], [7, 7, 7]],
        [[22, 31, 10], [5, 14, 0], [12, 25, 1], [7, 7, 7]],
        [[28, 31, 16], [13, 12, 31], [18, 19, 31], [7, 7, 7]],
        [[28, 31, 16], [31, 16, 1], [31, 31, 7], [7, 7, 7]],
        [[28, 31, 16], [20, 15, 3], [24, 18, 7], [7, 7, 7]],
        [[28, 31, 16], [5, 17, 31], [15, 31, 31], [7, 7, 7]],
        [[31, 0, 31], [16, 0, 16], [31, 0, 31], [0, 0, 0]],
        [[31, 29, 31], [20, 26, 31], [21, 28, 11], [3, 2, 2]],
    ],
    [
        [[27, 31, 27], [13, 13, 13], [21, 21, 21], [7, 7, 7]],
        [[27, 31, 27], [30, 10, 6], [31, 19, 24], [7, 7, 7]],
        [[22, 31, 10], [5, 14, 0], [12, 25, 1], [7, 7, 7]],
        [[27, 31, 27], [13, 12, 31], [18, 19, 31], [7, 7, 7]],
        [[27, 31, 27], [31, 16, 1], [31, 31, 7], [7, 7, 7]],
        [[27, 31, 27], [20, 15, 3], [24, 18, 7], [7, 7, 7]],
        [[27, 31, 27], [5, 17, 31], [15, 31, 31], [7, 7, 7]],
        [[31, 0, 31], [16, 0, 16], [31, 0, 31], [0, 0, 0]],
        [[28, 31, 26], [6, 13, 10], [17, 24, 14], [1, 3, 4]],
    ],
    [
        [[15, 14, 24], [7, 7, 12], [11, 11, 19], [0, 0, 0]],
        [[15, 14, 24], [13, 0, 8], [14, 7, 17], [0, 0, 0]],
        [[15, 14, 24], [0, 11, 13], [8, 13, 19], [0, 0, 0]],
        [[15, 14, 24], [4, 3, 18], [10, 9, 20], [0, 0, 0]],
        [[30, 30, 11], [16, 14, 10], [16, 14, 18], [0, 0, 0]],
        [[15, 14, 24], [8, 4, 5], [12, 9, 15], [0, 0, 0]],
        [[15, 14, 24], [11, 9, 20], [13, 12, 23], [0, 0, 0]],
        [[31, 0, 31], [16, 0, 16], [31, 0, 31], [0, 0, 0]],
        [[31, 29, 31], [18, 24, 22], [21, 14, 9], [3, 2, 2]],
    ],
    [
        [[1, 1, 2], [0, 0, 0], [0, 0, 0], [0, 0, 0]],
        [[1, 1, 2], [0, 0, 0], [0, 0, 0], [0, 0, 0]],
        [[1, 1, 2], [0, 0, 0], [0, 0, 0], [0, 0, 0]],
        [[1, 1, 2], [0, 0, 0], [0, 0, 0], [0, 0, 0]],
        [[30, 30, 11], [0, 0, 0], [0, 0, 0], [0, 0, 0]],
        [[1, 1, 2], [0, 0, 0], [0, 0, 0], [0, 0, 0]],
        [[1, 1, 2], [0, 0, 0], [0, 0, 0], [0, 0, 0]],
        [[31, 0, 31], [16, 0, 16], [31, 0, 31], [0, 0, 0]],
        [[0, 0, 0], [31, 27, 0], [0, 16, 16], [31, 31, 31]],
    ],
    [
        [[30, 28, 26], [13, 13, 13], [19, 19, 19], [7, 7, 7]],
        [[30, 28, 26], [30, 10, 6], [31, 19, 24], [7, 7, 7]],
        [[18, 24, 9], [9, 13, 0], [15, 20, 1], [7, 7, 7]],
        [[30, 28, 26], [9, 9, 31], [15, 16, 31], [7, 7, 7]],
        [[30, 28, 26], [31, 16, 1], [31, 31, 7], [7, 7, 7]],
        [[26, 24, 17], [16, 13, 3], [21, 17, 7], [7, 7, 7]],
        [[30, 28, 26], [14, 16, 31], [17, 19, 31], [7, 7, 7]],
        [[31, 0, 31], [16, 0, 16], [31, 0, 31], [0, 0, 0]],
        [[31, 31, 31], [18, 7, 7], [31, 16, 16], [3, 2, 2]],
    ],
    [
        [[28, 31, 28], [13, 13, 13], [21, 21, 21], [7, 7, 7]],
        [[28, 31, 28], [24, 6, 8], [29, 16, 13], [7, 7, 7]],
        [[14, 25, 20], [5, 14, 10], [9, 19, 5], [7, 7, 7]],
        [[28, 31, 28], [1, 12, 19], [8, 20, 27], [7, 7, 7]],
        [[28, 31, 28], [12, 20, 27], [19, 26, 31], [7, 7, 7]],
        [[28, 31, 28], [17, 13, 10], [22, 18, 15], [7, 7, 7]],
        [[28, 31, 28], [17, 13, 23], [22, 18, 27], [7, 7, 7]],
        [[28, 31, 28], [26, 8, 5], [31, 31, 31], [0, 0, 0]],
        [[31, 31, 31], [10, 10, 10], [20, 20, 20], [0, 0, 0]],
    ],
];
