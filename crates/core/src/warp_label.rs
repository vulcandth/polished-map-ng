//! Warp IDs adapted from master event.cpp::draw_warp_id{,_zoomed}.
//! Glyph rows decode its embedded warp_digits_png_buffer; see docs/assets.md.
//! Copyright Rangi42/Polished Map contributors and 2026 Polished Map NG contributors.
//! LGPL-3.0-only. Changing the storage representation does not change attribution.

// Five bits per row, leftmost pixel in bit 4. Set bits are black, others white.
const GLYPHS: [[u8; 7]; 16] = [
    [0, 0b01110, 0b01010, 0b01010, 0b01010, 0b01110, 0],
    [0, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110, 0],
    [0, 0b01110, 0b00010, 0b01110, 0b01000, 0b01110, 0],
    [0, 0b01110, 0b00010, 0b00110, 0b00010, 0b01110, 0],
    [0, 0b01010, 0b01010, 0b01110, 0b00010, 0b00010, 0],
    [0, 0b01110, 0b01000, 0b01110, 0b00010, 0b01110, 0],
    [0, 0b01110, 0b01000, 0b01110, 0b01010, 0b01110, 0],
    [0, 0b01110, 0b00010, 0b00010, 0b00100, 0b00100, 0],
    [0, 0b01110, 0b01010, 0b01110, 0b01010, 0b01110, 0],
    [0, 0b01110, 0b01010, 0b01110, 0b00010, 0b00010, 0],
    [0, 0b00100, 0b01010, 0b01110, 0b01010, 0b01010, 0],
    [0, 0b01100, 0b01010, 0b01100, 0b01010, 0b01100, 0],
    [0, 0b00110, 0b01000, 0b01000, 0b01000, 0b00110, 0],
    [0, 0b01100, 0b01010, 0b01010, 0b01010, 0b01100, 0],
    [0, 0b01110, 0b01000, 0b01100, 0b01000, 0b01110, 0],
    [0, 0b01110, 0b01000, 0b01100, 0b01000, 0b01000, 0],
];

/// One opaque 5×7 glyph, positioned relative to a 16×16 event marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarpDigit {
    value: u8,
    x: u8,
}
impl WarpDigit {
    pub fn value(self) -> u8 {
        self.value
    }
    pub fn x(self) -> u8 {
        self.x
    }
    pub fn y(self) -> u8 {
        8
    }
    /// Black pixels are set bits (leftmost = bit 4); all other pixels are white.
    pub fn rows(self) -> &'static [u8; 7] {
        &GLYPHS[usize::from(self.value)]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarpIdLabel {
    id: std::num::NonZeroUsize,
    hex: bool,
}
impl WarpIdLabel {
    /// ID zero has no label, just like the upstream draw/print guards.
    pub fn new(id: usize, hex: bool) -> Option<Self> {
        Some(Self {
            id: std::num::NonZeroUsize::new(id)?,
            hex,
        })
    }
    /// Draw in returned order: ones first, then tens/hundreds. Opaque white
    /// columns overlap by one pixel. Original overflow behavior keeps only the
    /// last two hex or three decimal digits at normal zoom.
    pub fn digits(self) -> impl Iterator<Item = WarpDigit> {
        let base = if self.hex { 16 } else { 10 };
        let id = self.id.get();
        let digit = |x, divisor| WarpDigit {
            x,
            value: ((id / divisor) % base) as u8,
        };
        [
            Some(digit(10, 1)),
            (id >= base).then(|| digit(6, base)),
            (!self.hex && id >= 100).then(|| digit(2, 100)),
        ]
        .into_iter()
        .flatten()
    }
    /// The original zoomed snprintf buffer holds three characters plus NUL.
    /// Preserve its leading-digit truncation, unlike normal-size modulo digits.
    pub fn zoomed_text(self) -> String {
        let mut text = if self.hex {
            format!("{:X}", self.id)
        } else {
            self.id.to_string()
        };
        text.truncate(3);
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn label_widths_overflow_and_zero_follow_original() {
        assert!(WarpIdLabel::new(0, false).is_none());
        for (id, hex, expected, text) in [
            (1, false, vec![(10, 1)], "1"),
            (42, false, vec![(10, 2), (6, 4)], "42"),
            (109, false, vec![(10, 9), (6, 0), (2, 1)], "109"),
            (1234, false, vec![(10, 4), (6, 3), (2, 2)], "123"),
            (0xaf, true, vec![(10, 15), (6, 10)], "AF"),
            (0x1234, true, vec![(10, 4), (6, 3)], "123"),
        ] {
            let label = WarpIdLabel::new(id, hex).unwrap();
            assert_eq!(
                label
                    .digits()
                    .map(|d| (d.x(), d.value()))
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(label.zoomed_text(), text);
            assert!(
                label
                    .digits()
                    .all(|d| d.y() == 8 && d.rows().iter().all(|r| *r < 32))
            );
        }
    }
}
