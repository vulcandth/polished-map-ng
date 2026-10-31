//! Common master/plusplus tileset-window.cpp hue operations. LGPL-3.0-only.
use crate::{Point, Tile, TilesetError};

pub(crate) fn hue(value: u8) -> Result<(), TilesetError> {
    if value <= 3 {
        Ok(())
    } else {
        Err(TilesetError::Hue(value))
    }
}
fn coordinate(at: Point) -> Result<usize, TilesetError> {
    if at.x < 8 && at.y < 8 {
        Ok(usize::from(at.y * 8 + at.x))
    } else {
        Err(TilesetError::Coordinate(at.x, at.y))
    }
}
pub(crate) fn from_pixels(pixels: &[u8]) -> Result<Tile, TilesetError> {
    let pixels: [u8; 64] = pixels.try_into().map_err(|_| TilesetError::Pixels)?;
    for &p in &pixels {
        hue(p)?;
    }
    Tile::new(pixels).map_err(|_| TilesetError::Pixels)
}
pub(crate) fn paint(tile: &Tile, points: &[Point], value: u8) -> Result<Tile, TilesetError> {
    hue(value)?;
    let mut pixels = *tile.hues();
    for &at in points {
        pixels[coordinate(at)?] = value;
    }
    from_pixels(&pixels)
}
pub(crate) fn fill(tile: &Tile, at: Point, value: u8) -> Result<Tile, TilesetError> {
    hue(value)?;
    let start = coordinate(at)?;
    let mut pixels = *tile.hues();
    let old = pixels[start];
    if old != value {
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(p) = queue.pop_front() {
            if pixels[p] != old {
                continue;
            }
            pixels[p] = value;
            if p % 8 > 0 {
                queue.push_back(p - 1);
            }
            if p % 8 < 7 {
                queue.push_back(p + 1);
            }
            if p >= 8 {
                queue.push_back(p - 8);
            }
            if p < 56 {
                queue.push_back(p + 8);
            }
        }
    }
    from_pixels(&pixels)
}
pub(crate) fn replace(tile: &Tile, from: u8, to: u8, swap: bool) -> Result<Tile, TilesetError> {
    hue(from)?;
    hue(to)?;
    from_pixels(&tile.hues().map(|p| {
        if p == from {
            to
        } else if swap && p == to {
            from
        } else {
            p
        }
    }))
}
pub(crate) fn rgb_hue(rgb: &[u8]) -> u8 {
    let gray = (u16::from(rgb[0]) * 31 + u16::from(rgb[1]) * 61 + u16::from(rgb[2]) * 8) / 100;
    [3, 1, 2, 0][usize::from(gray / 64)]
}
