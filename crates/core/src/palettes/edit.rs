//! master palette-window.cpp channel/hex/copy/swap and colors.cpp export.
use super::{PaletteBank, PaletteSet};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Address of one hue in a palette set; validated when read or edited.
pub struct ColorSlot {
    /// Time-of-day or environment palette set.
    pub set: PaletteSet,
    /// 0..7 game palettes, 8 monochrome.
    pub palette: u8,
    /// Internal order: white=0, dark=1, light=2, black=3.
    pub hue: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
/// Color edits applied atomically through [`PaletteBank::edit`].
pub enum PaletteCommand {
    /// Set an exact RGB8 color without quantization.
    SetRgb {
        /// Destination color address.
        slot: ColorSlot,
        /// Red, green and blue channels in 0..=255.
        rgb: [u8; 3],
    },
    /// Set one RGB5 channel, expanded with `value * 33 / 4`.
    Channel {
        /// Destination color address.
        slot: ColorSlot,
        /// Channel index: red 0, green 1, blue 2.
        channel: u8,
        /// Five-bit channel value in 0..=31.
        value: u8,
    },
    /// Interpret zero to six ASCII hex digits as RGB8; empty means black.
    Hex {
        /// Destination color address.
        slot: ColorSlot,
        /// Hexadecimal digits without a leading `#`.
        value: String,
    },
    /// Copy through the original RGB5 clipboard quantization.
    Copy {
        /// Source color address.
        from: ColorSlot,
        /// Destination color address.
        to: ColorSlot,
    },
    /// Exchange exact live RGB8 values.
    Swap {
        /// First color address.
        a: ColorSlot,
        /// Second color address.
        b: ColorSlot,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
/// Invalid palette address, RGB5 channel, or hexadecimal input.
pub enum PaletteError {
    #[error("palette must be 0..8 and hue must be 0..3")]
    /// The palette or hue index is outside its range.
    Slot,
    #[error("RGB channel must be 0..2 and its five-bit value must be 0..31")]
    /// The channel is outside 0..=2 or its RGB5 value exceeds 31.
    Channel,
    #[error("hex color must contain zero to six ASCII hexadecimal digits")]
    /// The input exceeds six digits or contains a non-hexadecimal character.
    Hex,
}

impl PaletteBank {
    /// Read an RGB8 color.
    ///
    /// # Errors
    /// Returns [`PaletteError::Slot`] for an invalid palette or hue index.
    pub fn color(&self, slot: ColorSlot) -> Result<[u8; 3], PaletteError> {
        self.colors[slot.set as usize]
            .get(usize::from(slot.palette))
            .and_then(|palette| palette.get(usize::from(slot.hue)))
            .copied()
            .ok_or(PaletteError::Slot)
    }

    /// Original private clipboard stores RGB5, unlike live-source swap.
    pub fn copy_color(&self, slot: ColorSlot) -> Result<[u8; 3], PaletteError> {
        Ok(self.color(slot)?.map(|c| (u16::from(c / 8) * 33 / 4) as u8))
    }

    /// Validate and execute a whole batch on a draft before committing it.
    pub fn edit(&mut self, commands: &[PaletteCommand]) -> Result<(), PaletteError> {
        let mut draft = self.clone();
        for command in commands {
            let (slot, rgb) = match command {
                PaletteCommand::SetRgb { slot, rgb } => (*slot, *rgb),
                PaletteCommand::Channel {
                    slot,
                    channel,
                    value,
                } => {
                    let mut rgb = draft.color(*slot)?;
                    if *channel > 2 || *value > 31 {
                        return Err(PaletteError::Channel);
                    }
                    rgb[usize::from(*channel)] = (u16::from(*value) * 33 / 4) as u8;
                    (*slot, rgb)
                }
                PaletteCommand::Hex { slot, value } => {
                    if value.len() > 6 || !value.bytes().all(|c| c.is_ascii_hexdigit()) {
                        return Err(PaletteError::Hex);
                    }
                    let number = if value.is_empty() {
                        0
                    } else {
                        u32::from_str_radix(value, 16).map_err(|_| PaletteError::Hex)?
                    };
                    (
                        *slot,
                        [(number >> 16) as u8, (number >> 8) as u8, number as u8],
                    )
                }
                PaletteCommand::Copy { from, to } => (*to, draft.copy_color(*from)?),
                PaletteCommand::Swap { a, b } => {
                    let first = draft.color(*a)?;
                    let second = draft.color(*b)?;
                    draft.colors[a.set as usize][usize::from(a.palette)][usize::from(a.hue)] =
                        second;
                    (*b, first)
                }
            };
            draft.color(slot)?;
            draft.colors[slot.set as usize][usize::from(slot.palette)][usize::from(slot.hue)] = rgb;
        }
        *self = draft;
        Ok(())
    }

    /// Export six sets in source order and RGB5, with corrected set labels.
    /// The original table omits DARKNESS and passes a null CUSTOM label to %s.
    pub fn export_rgb(&self, monochrome: bool) -> String {
        let mut text = String::new();
        let names = [
            "gray", "red", "green", "water", "yellow", "brown", "roof", "text",
        ];
        for (set, label) in PaletteSet::ALL
            .into_iter()
            .zip(["morn", "day", "nite", "darkness", "indoor", "custom"])
        {
            if !monochrome {
                writeln!(text, "; {label}").unwrap();
            }
            for palette in if monochrome { 8..9 } else { 0..8 } {
                text.push_str("\tRGB ");
                for (i, hue) in [0, 2, 1, 3].into_iter().enumerate() {
                    if i > 0 {
                        text.push_str(", ");
                    }
                    let [r, g, b] = self.colors[set as usize][palette][hue].map(|c| c / 8);
                    write!(text, "{r:02},{g:02},{b:02}").unwrap();
                }
                writeln!(
                    text,
                    " ; {}",
                    if monochrome { label } else { names[palette] }
                )
                .unwrap();
            }
        }
        text
    }
}
