//! master Main_Window::print_map, Block::print and Event::print.
//! LGPL-3.0-only. Pixels and geometry are independent of a window, GPU or font API.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Events, Graphics, Map, PriorityPattern, RasterError, RgbImage, WarpIdLabel};

/// Independent Print Options flags, all off in the original default preferences.
/// Warp IDs have no effect unless events are printed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PrintOptions {
    pub grid: bool,
    pub ids: bool,
    pub priority: bool,
    pub events: bool,
    pub warp_ids: bool,
}

/// Print uses the map's display base and theme grid color, but not view toggles,
/// zoom, rulers, selection rings, scroll positions or the Game Boy guide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PrintStyle {
    pub hexadecimal_ids: bool,
    pub grid_rgb: [u8; 3],
}
impl Default for PrintStyle {
    fn default() -> Self {
        Self {
            hexadecimal_ids: false,
            grid_rgb: [85; 3],
        }
    }
}

/// Native font requests in the original, both at logical size 14.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrintFont {
    CourierBold14,
    HelveticaBold14,
}

/// Coverage of a laid-out text line, including its font's ascent/descent space.
/// Coordinates start at the line's top left; values are straight 8-bit coverage.
/// The font adapter owns native/bundled typeface choice and baseline placement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintTextMask {
    width: u16,
    height: u16,
    coverage: Vec<u8>,
}
impl PrintTextMask {
    /// Bound font-adapter output: source labels are at most three ASCII digits
    /// or one event symbol at size 14. The limits also bound cached mask memory.
    pub fn new(width: u16, height: u16, coverage: Vec<u8>) -> Result<Self, PrintError> {
        if !(1..=128).contains(&width)
            || !(1..=64).contains(&height)
            || coverage.len() != usize::from(width) * usize::from(height)
        {
            return Err(PrintError::TextMask);
        }
        Ok(Self {
            width,
            height,
            coverage,
        })
    }
    pub fn width(&self) -> u16 {
        self.width
    }
    pub fn height(&self) -> u16 {
        self.height
    }
    pub fn coverage(&self) -> &[u8] {
        &self.coverage
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PrintError {
    #[error(transparent)]
    Raster(#[from] RasterError),
    #[error("print text mask must be 1..=128 by 1..=64 with one coverage byte per pixel")]
    TextMask,
    #[error("could not rasterize print text: {0}")]
    Text(String),
}

impl Graphics {
    /// Render the original native-resolution Print composition (32px/block,
    /// 16px/event), clipped to the map with no surrounding editor margins.
    ///
    /// The callback supplies font coverage without introducing a platform/font
    /// dependency in core. It is called once per distinct (font, text) pair and
    /// is not used when the corresponding text option is off. A failed callback returns no image
    /// and cannot modify the input map, events or graphics.
    ///
    /// Alpha uses FLTK 1.4.5's Xlib software-compositing integer rule. Native
    /// XRender/GDI/Quartz and font rasterization still require separate parity
    /// checks; this method does not claim identical output from every driver.
    pub fn render_print<F>(
        &self,
        map: &Map,
        events: Option<&Events>,
        options: PrintOptions,
        style: PrintStyle,
        mut rasterize: F,
    ) -> Result<RgbImage, PrintError>
    where
        F: FnMut(PrintFont, &str) -> Result<PrintTextMask, PrintError>,
    {
        let width = usize::from(map.width()) * 32;
        let height = usize::from(map.height()) * 32;
        let mut image = RgbImage {
            width: width as u32,
            height: height as u32,
            pixels: vec![0; width * height * 3],
        };
        let mut blocks = BTreeMap::new();
        let mut masks = BTreeMap::new();
        let priority = options.priority.then(|| PriorityPattern::Small.rgba());
        for (index, &id) in map.blocks().iter().enumerate() {
            let x = (index % usize::from(map.width()) * 32) as i32;
            let y = (index / usize::from(map.width()) * 32) as i32;
            if let std::collections::btree_map::Entry::Vacant(entry) = blocks.entry(id) {
                entry.insert(self.render_block(id)?);
            }
            let block = &blocks[&id];
            // Complete each block before the next, matching Block::print even
            // if a supplied font mask has ink extending into a neighboring block.
            for row in 0..32 {
                let target = ((y as usize + row) * width + x as usize) * 3;
                image.pixels[target..target + 96]
                    .copy_from_slice(&block.pixels[row * 96..row * 96 + 96]);
            }
            if let Some(pattern) = &priority
                && let Some(flags) = self.block_priorities(id)
            {
                for (slot, enabled) in flags.into_iter().enumerate() {
                    if enabled {
                        let ox = x + (slot % 4 * 8) as i32;
                        let oy = y + (slot / 4 * 8) as i32;
                        for (i, rgba) in pattern.as_chunks::<4>().0.iter().enumerate() {
                            pixel(
                                &mut image,
                                ox + (i % 8) as i32,
                                oy + (i / 8) as i32,
                                [rgba[0], rgba[1], rgba[2]],
                                rgba[3],
                            );
                        }
                    }
                }
            }
            if options.grid {
                for offset in 0..32 {
                    pixel(&mut image, x + offset, y + 31, style.grid_rgb, 255);
                    pixel(&mut image, x + 31, y + offset, style.grid_rgb, 255);
                }
            }
            if options.ids {
                let text = if style.hexadecimal_ids {
                    format!("{id:02X}")
                } else {
                    id.to_string()
                };
                let key = (PrintFont::CourierBold14, text);
                if let std::collections::btree_map::Entry::Vacant(entry) = masks.entry(key.clone())
                {
                    entry.insert(rasterize(key.0, &key.1)?);
                }
                let mask = &masks[&key];
                for (dx, dy) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                    text_mask(&mut image, mask, x + 3 + dx, y + 1 + dy, [0; 3]);
                }
                text_mask(&mut image, mask, x + 3, y + 1, [255; 3]);
            }
        }
        if options.events
            && let Some(events) = events
        {
            for event in events.entries() {
                let definition = event.definition();
                let x = i32::from(definition.x) * 16;
                let y = i32::from(definition.y) * 16;
                // Clip actual ink, not the event's nominal rectangle: a text
                // mask can overhang an off-map event, as in the original draw.
                let (inside, border) = definition.kind.marker_rgb();
                for dy in 0..16 {
                    for dx in 0..16 {
                        let color = if dx == 0 || dy == 0 || dx == 15 || dy == 15 {
                            border
                        } else {
                            inside
                        };
                        pixel(&mut image, x + dx, y + dy, color, 128);
                    }
                }
                let key = (
                    PrintFont::HelveticaBold14,
                    definition.kind.symbol().to_string(),
                );
                if let std::collections::btree_map::Entry::Vacant(entry) = masks.entry(key.clone())
                {
                    entry.insert(rasterize(key.0, &key.1)?);
                }
                let mask = &masks[&key];
                text_mask(
                    &mut image,
                    mask,
                    x + (16 - i32::from(mask.width)) / 2,
                    y + (16 - i32::from(mask.height)) / 2,
                    [255; 3],
                );
                if options.warp_ids
                    && let Some(label) = event
                        .warp_id()
                        .and_then(|id| WarpIdLabel::new(id, definition.hex_coordinates))
                {
                    for digit in label.digits() {
                        for (dy, row) in digit.rows().iter().enumerate() {
                            for dx in 0..5 {
                                let color = if row & (1 << (4 - dx)) != 0 {
                                    [0; 3]
                                } else {
                                    [255; 3]
                                };
                                pixel(
                                    &mut image,
                                    x + i32::from(digit.x()) + dx,
                                    y + i32::from(digit.y()) + dy as i32,
                                    color,
                                    255,
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(image)
    }
}

fn text_mask(image: &mut RgbImage, mask: &PrintTextMask, x: i32, y: i32, rgb: [u8; 3]) {
    for (index, &alpha) in mask.coverage.iter().enumerate() {
        pixel(
            image,
            x + (index % usize::from(mask.width)) as i32,
            y + (index / usize::from(mask.width)) as i32,
            rgb,
            alpha,
        );
    }
}

fn pixel(image: &mut RgbImage, x: i32, y: i32, rgb: [u8; 3], alpha: u8) {
    if x < 0 || y < 0 || x >= image.width as i32 || y >= image.height as i32 {
        return;
    }
    let offset = (y as usize * image.width as usize + x as usize) * 3;
    // FLTK 1.4.5 Fl_Xlib_Graphics_Driver_image.cxx alpha_blend, RGBA branch:
    // map alpha to 0..=256 with an exact copy at 255 and no change at zero.
    let a = u32::from(alpha) + (u32::from(alpha) >> 7);
    for (dst, src) in image.pixels[offset..offset + 3].iter_mut().zip(rgb) {
        *dst = ((u32::from(src) * a + u32::from(*dst) * (256 - a)) >> 8) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventDefinition, EventKind, Tile};

    fn graphics() -> Graphics {
        Graphics::new(
            vec![Tile::new([0; 64]).unwrap()],
            vec![[[20, 40, 60]; 4]],
            vec![[0; 16]],
        )
        .unwrap()
        .with_tile_priorities(vec![true])
        .unwrap()
    }

    #[test]
    fn default_print_and_warp_ids_alone_equal_plain_export_without_fonts() {
        let graphics = graphics();
        let map = Map::from_bytes(2, 2, vec![0, 255, 255, 0]).unwrap();
        for warp_ids in [false, true] {
            let print = graphics
                .render_print(
                    &map,
                    None,
                    PrintOptions {
                        warp_ids,
                        ..Default::default()
                    },
                    PrintStyle::default(),
                    |_, _| panic!("plain export must not initialize fonts"),
                )
                .unwrap();
            assert_eq!(print, graphics.render_map(&map).unwrap());
        }
    }

    #[test]
    fn font_masks_are_bounded_and_preserve_straight_coverage() {
        for (w, h, length) in [
            (0, 1, 0),
            (1, 0, 0),
            (129, 1, 129),
            (1, 65, 65),
            (2, 2, 3),
            (2, 2, 5),
        ] {
            assert_eq!(
                PrintTextMask::new(w, h, vec![255; length]),
                Err(PrintError::TextMask)
            );
        }
        let mask = PrintTextMask::new(128, 64, vec![128; 8192]).unwrap();
        assert_eq!(
            (mask.width(), mask.height(), mask.coverage().len()),
            (128, 64, 8192)
        );
        let mask = PrintTextMask::new(2, 2, vec![0, 64, 128, 255]).unwrap();
        let mut image = RgbImage {
            width: 2,
            height: 2,
            pixels: vec![100; 12],
        };
        text_mask(&mut image, &mask, 0, 0, [200; 3]);
        assert_eq!(
            image.pixels,
            [100, 100, 100, 125, 125, 125, 150, 150, 150, 200, 200, 200]
        );
        // Signed coordinates clip instead of wrapping into adjacent rows.
        let mut clipped = RgbImage {
            width: 1,
            height: 1,
            pixels: vec![100; 3],
        };
        text_mask(&mut clipped, &mask, -1, -1, [200; 3]);
        assert_eq!(clipped.pixels, [200; 3]);
    }

    #[test]
    fn text_cache_distinguishes_fonts_and_bases_and_propagates_errors() {
        let graphics = graphics();
        let map = Map::from_bytes(3, 1, vec![255, 0, 255]).unwrap();
        let events = Events::new(
            (0..3)
                .map(|x| EventDefinition {
                    kind: EventKind::WarpEvent,
                    x,
                    y: 0,
                    hex_coordinates: false,
                    prefix: None,
                    suffix: None,
                    line: 1,
                })
                .collect(),
        )
        .unwrap();
        let before = (
            map.blocks().to_vec(),
            events.clone(),
            graphics.render_map(&map).unwrap(),
        );
        let options = PrintOptions {
            ids: true,
            events: true,
            ..Default::default()
        };
        for hexadecimal_ids in [false, true] {
            let mut calls = Vec::new();
            graphics
                .render_print(
                    &map,
                    Some(&events),
                    options,
                    PrintStyle {
                        hexadecimal_ids,
                        ..Default::default()
                    },
                    |font, text| {
                        calls.push((font, text.to_owned()));
                        PrintTextMask::new(1, 1, vec![255])
                    },
                )
                .unwrap();
            assert_eq!(
                calls,
                vec![
                    (
                        PrintFont::CourierBold14,
                        if hexadecimal_ids { "FF" } else { "255" }.into()
                    ),
                    (
                        PrintFont::CourierBold14,
                        if hexadecimal_ids { "00" } else { "0" }.into()
                    ),
                    (PrintFont::HelveticaBold14, "W".into()),
                ]
            );
        }
        let result = graphics.render_print(
            &map,
            Some(&events),
            options,
            PrintStyle::default(),
            |_, _| Err(PrintError::Text("font unavailable".into())),
        );
        assert_eq!(result, Err(PrintError::Text("font unavailable".into())));
        assert_eq!(
            before,
            (
                map.blocks().to_vec(),
                events,
                graphics.render_map(&map).unwrap()
            )
        );
    }

    #[test]
    fn grid_replaces_bottom_and_right_edges_after_priority() {
        let graphics = graphics();
        let map = Map::from_bytes(2, 1, vec![0, 255]).unwrap();
        let image = graphics
            .render_print(
                &map,
                None,
                PrintOptions {
                    grid: true,
                    priority: true,
                    ..Default::default()
                },
                PrintStyle {
                    grid_rgb: [38; 3],
                    ..Default::default()
                },
                |_, _| unreachable!(),
            )
            .unwrap();
        let rgb = |x: usize, y: usize| &image.pixels[(y * 64 + x) * 3..(y * 64 + x) * 3 + 3];
        for x in 0..64 {
            assert_eq!(rgb(x, 31), [38; 3]);
        }
        for y in 0..32 {
            for x in [31, 63] {
                assert_eq!(rgb(x, y), [38; 3]);
            }
        }
        // Small priority PNG's top-left RGBA is (104,216,216,96).
        assert_eq!(rgb(0, 0), [51, 106, 118]);
        assert_eq!(rgb(32, 0), [171, 205, 239]);
    }
}
