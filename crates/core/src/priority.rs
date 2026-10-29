//! Priority zigzags from Rangi/Polished Map's tile.cpp. LGPL-3.0-only.
//! Original artwork: plusplus d090450b; master adaptation 0a627967.
//! Periodic color-index masks preserve the original 8/16/24-pixel PNG samples.

/// The three source sizes are different patterns, not scaled copies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PriorityPattern {
    Small,
    Large,
    Chip,
}

impl PriorityPattern {
    pub const ALL: [Self; 3] = [Self::Small, Self::Large, Self::Chip];

    pub fn size(self) -> usize {
        match self {
            Self::Small => 8,
            Self::Large => 16,
            Self::Chip => 24,
        }
    }

    /// Row-major, straight-alpha RGBA8; independent of GUI/GPU compositing.
    pub fn rgba(self) -> Vec<u8> {
        let mask: &[u8] = match self {
            Self::Small => b"3213200101201332",
            Self::Large => concat!(
                "33321333", "33200133", "32000013", "20000001", "00012000", "00133200", "01333320",
                "13333332"
            )
            .as_bytes(),
            Self::Chip => concat!(
                "333332133333",
                "333320013333",
                "333200001333",
                "332000000133",
                "320000000013",
                "200000000001",
                "000001200000",
                "000013320000",
                "000133332000",
                "001333333200",
                "013333333320",
                "133333333332"
            )
            .as_bytes(),
        };
        let colors = [
            [32, 96, 96, 96],
            [50, 126, 126, 96],
            [86, 186, 186, 96],
            [104, 216, 216, 96],
        ];
        let size = self.size();
        let period = size / 2;
        (0..size * size)
            .flat_map(|i| {
                colors[usize::from(mask[(i / size % period) * period + i % period] - b'0')]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Blockset, Graphics, RasterError, Tile};

    #[test]
    fn priority_flags_follow_block_replacement_without_changing_rgb_exports() {
        let plain = Graphics::new(
            vec![Tile::new([0; 64]).unwrap(); 2],
            vec![[[12, 34, 56]; 4]; 2],
            vec![[0; 16]],
        )
        .unwrap();
        assert_eq!(plain.block_priorities(0), Some([false; 16]));
        assert!(matches!(
            plain.clone().with_tile_priorities(vec![true]),
            Err(RasterError::PriorityCount)
        ));
        let mut marked = plain
            .clone()
            .with_tile_priorities(vec![false, true])
            .unwrap();
        assert_eq!(plain.render_block(0), marked.render_block(0));
        marked
            .replace_blocks(&Blockset::from_bytes(&[1; 16]).unwrap())
            .unwrap();
        assert_eq!(marked.block_priorities(0), Some([true; 16]));
        assert_eq!(marked.block_priorities(1), None);
        assert!(!marked.tile_priority(255));
    }
}
