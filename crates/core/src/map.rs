//! Shared byte-ID map edits and bounded history; adapted from pinned map.cpp. LGPL-3.0-only.
use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const HISTORY_LIMIT: usize = 100;

/// Integer coordinates whose units are defined by the receiving operation.
/// Map commands use blocks, block commands use tile cells, and pixel-edit
/// commands use pixels within an 8×8 tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    /// Horizontal coordinate, increasing to the right.
    pub x: u16,
    /// Vertical coordinate, increasing downwards.
    pub y: u16,
}

/// Placement of the old map inside the resized rectangle, as in Resize_Dialog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResizeAnchor {
    /// Keep the old top-left corner fixed.
    TopLeft,
    /// Keep the top edge fixed and center horizontally.
    TopCenter,
    /// Keep the old top-right corner fixed.
    TopRight,
    /// Keep the left edge fixed and center vertically.
    MiddleLeft,
    #[default]
    /// Center the old contents on both axes; odd deltas truncate toward zero.
    Center,
    /// Keep the right edge fixed and center vertically.
    MiddleRight,
    /// Keep the old bottom-left corner fixed.
    BottomLeft,
    /// Keep the bottom edge fixed and center horizontally.
    BottomCenter,
    /// Keep the old bottom-right corner fixed.
    BottomRight,
}

impl ResizeAnchor {
    /// Anchors in top-to-bottom, left-to-right UI order.
    pub const ALL: [Self; 9] = [
        Self::TopLeft,
        Self::TopCenter,
        Self::TopRight,
        Self::MiddleLeft,
        Self::Center,
        Self::MiddleRight,
        Self::BottomLeft,
        Self::BottomCenter,
        Self::BottomRight,
    ];

    /// Block-coordinate translation; event coordinates move by twice this amount.
    pub fn offset(self, old: (u8, u8), new: (u8, u8)) -> (i16, i16) {
        use ResizeAnchor::*;
        let dx = i16::from(new.0) - i16::from(old.0);
        let dy = i16::from(new.1) - i16::from(old.1);
        let x = match self {
            TopLeft | MiddleLeft | BottomLeft => 0,
            TopRight | MiddleRight | BottomRight => dx,
            _ => dx / 2,
        };
        let y = match self {
            TopLeft | TopCenter | TopRight => 0,
            BottomLeft | BottomCenter | BottomRight => dy,
            _ => dy / 2,
        };
        (x, y)
    }
}

/// An edit operation. Painting a drag is one history entry; resize clears history.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    /// Paint a complete drag as one history entry, including a no-op drag.
    Paint {
        /// Map-block coordinates to paint; all are validated before changing any cell.
        points: Vec<Point>,
        /// Replacement block ID; graphics availability is a renderer concern.
        block: u8,
    },
    /// Flood-fill connected cells sharing the seed cell's block ID.
    Fill {
        /// Flood-fill seed in map-block coordinates.
        at: Point,
        /// Replacement block ID; graphics availability is a renderer concern.
        block: u8,
    },
    /// Replace one block ID everywhere in the map.
    Replace {
        /// Block ID to replace.
        from: u8,
        /// Replacement block ID.
        to: u8,
    },
    /// Exchange two block IDs everywhere in the map.
    Swap {
        /// First block ID.
        a: u8,
        /// Second block ID.
        b: u8,
    },
    /// Resize clears both history stacks, matching master Main_Window::resize_map.
    Resize {
        /// New width in 1–255 map blocks.
        width: u16,
        /// New height in 1–255 map blocks.
        height: u16,
        #[serde(default)]
        /// Placement of old contents within the resized map.
        anchor: ResizeAnchor,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
/// Rejected dimensions, byte counts or block coordinates; edits validate before mutation.
pub enum MapError {
    #[error("map dimensions must be between 1 and 255; got {width}×{height}")]
    /// At least one axis is outside the supported 1–255 block range.
    Dimensions {
        /// Requested width in blocks.
        width: u16,
        /// Requested height in blocks.
        height: u16,
    },
    #[error("expected {expected} map bytes, found {actual}")]
    /// The supplied byte count does not equal width × height.
    Length {
        /// Width × height bytes required by the dimensions.
        expected: usize,
        /// Supplied block-byte count.
        actual: usize,
    },
    #[error("block coordinate ({0}, {1}) is outside the map")]
    /// The requested (column, row) lies outside the map.
    Coordinate(u16, u16),
}

/// A validated byte-ID map with bounded undo/redo history.
#[derive(Clone, Debug)]
pub struct Map {
    width: u8,
    height: u8,
    blocks: Vec<u8>,
    saved: Vec<u8>,
    modified: bool,
    past: VecDeque<Vec<u8>>,
    future: VecDeque<Vec<u8>>,
}

impl Map {
    /// New, unsaved map of block zero, without an initial undo entry.
    ///
    /// # Errors
    /// Returns [`MapError::Dimensions`] unless both axes are in 1–255 blocks.
    pub fn blank(width: u16, height: u16) -> Result<Self, MapError> {
        if !(1..=255).contains(&width) || !(1..=255).contains(&height) {
            return Err(MapError::Dimensions { width, height });
        }
        let mut map = Self::from_bytes(
            width,
            height,
            vec![0; usize::from(width) * usize::from(height)],
        )?;
        map.modified = true;
        Ok(map)
    }

    /// Load row-major block IDs as a clean map with empty history.
    ///
    /// # Errors
    /// Rejects axes outside 1–255 and any byte count other than width × height.
    pub fn from_bytes(width: u16, height: u16, blocks: Vec<u8>) -> Result<Self, MapError> {
        if !(1..=255).contains(&width) || !(1..=255).contains(&height) {
            return Err(MapError::Dimensions { width, height });
        }
        let expected = usize::from(width) * usize::from(height);
        if blocks.len() != expected {
            return Err(MapError::Length {
                expected,
                actual: blocks.len(),
            });
        }
        Ok(Self {
            width: width as u8,
            height: height as u8,
            saved: blocks.clone(),
            modified: false,
            blocks,
            past: VecDeque::new(),
            future: VecDeque::new(),
        })
    }

    /// Width in map blocks.
    pub fn width(&self) -> u8 {
        self.width
    }
    /// Height in map blocks.
    pub fn height(&self) -> u8 {
        self.height
    }
    /// Main_Window::save_map: WRAM includes a three-block border on every side.
    /// Widen before arithmetic: the largest valid map needs 68,121 bytes.
    pub fn overworld_buffer_bytes(&self) -> u32 {
        (u32::from(self.width) + 6) * (u32::from(self.height) + 6)
    }
    /// Row-major block IDs; immutable access keeps history and dirty state consistent.
    pub fn blocks(&self) -> &[u8] {
        &self.blocks
    }
    /// Whether editing has marked the map dirty; a no-op command also marks dirty.
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    /// Call only after persistence succeeds.
    pub fn mark_saved(&mut self) {
        self.saved.clone_from(&self.blocks);
        self.modified = false;
    }
    /// Whether an earlier pixel snapshot is available in the bounded history.
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }
    /// Whether an undone snapshot can be reapplied.
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    fn index(&self, point: Point) -> Result<usize, MapError> {
        if point.x >= u16::from(self.width) || point.y >= u16::from(self.height) {
            return Err(MapError::Coordinate(point.x, point.y));
        }
        Ok(usize::from(point.y) * usize::from(self.width) + usize::from(point.x))
    }

    /// Read a block ID at a map-block coordinate.
    ///
    /// # Errors
    /// Returns [`MapError::Coordinate`] if either coordinate is outside the map.
    pub fn block(&self, point: Point) -> Result<u8, MapError> {
        Ok(self.blocks[self.index(point)?])
    }

    /// Validate the complete command before changing state. Returns whether it changed pixels.
    ///
    /// # Errors
    /// Invalid coordinates or resize dimensions leave pixels and history unchanged.
    /// Other commands retain at most 100 undo entries; use [`Self::resize`] for
    /// the distinct resize/history boundary.
    pub fn apply(&mut self, command: &Command) -> Result<bool, MapError> {
        if let Command::Resize {
            width,
            height,
            anchor,
        } = *command
        {
            return self.resize(width, height, anchor);
        }
        let mut next = self.blocks.clone();
        match command {
            Command::Paint { points, block } => {
                for &point in points {
                    next[self.index(point)?] = *block;
                }
            }
            Command::Fill { at, block } => {
                let start = self.index(*at)?;
                let source = next[start];
                if source == *block {
                    return Ok(self.record(next));
                }
                let mut pending = vec![start];
                next[start] = *block;
                let width = usize::from(self.width);
                while let Some(i) = pending.pop() {
                    let mut visit = |j: usize| {
                        if next[j] == source {
                            next[j] = *block;
                            pending.push(j);
                        }
                    };
                    if i % width > 0 {
                        visit(i - 1);
                    }
                    if i % width + 1 < width {
                        visit(i + 1);
                    }
                    if i >= width {
                        visit(i - width);
                    }
                    if i + width < self.blocks.len() {
                        visit(i + width);
                    }
                }
            }
            Command::Replace { from, to } => {
                for id in &mut next {
                    if id == from {
                        *id = *to;
                    }
                }
            }
            Command::Swap { a, b } => {
                for id in &mut next {
                    if id == a {
                        *id = *b;
                    } else if id == b {
                        *id = *a;
                    }
                }
            }
            Command::Resize { .. } => unreachable!("handled before allocating edit snapshot"),
        }
        Ok(self.record(next))
    }

    /// Resize with zero-filled borders and clear history. An unchanged size does nothing.
    ///
    /// # Errors
    /// Returns [`MapError::Dimensions`] without mutation for axes outside 1–255.
    pub fn resize(
        &mut self,
        width: u16,
        height: u16,
        anchor: ResizeAnchor,
    ) -> Result<bool, MapError> {
        if !(1..=255).contains(&width) || !(1..=255).contains(&height) {
            return Err(MapError::Dimensions { width, height });
        }
        let (width, height) = (width as u8, height as u8);
        if (width, height) == (self.width, self.height) {
            return Ok(false);
        }
        let (dx, dy) = anchor.offset((self.width, self.height), (width, height));
        let mut blocks = vec![0; usize::from(width) * usize::from(height)];
        for y in 0..height {
            for x in 0..width {
                let (ox, oy) = (i16::from(x) - dx, i16::from(y) - dy);
                if ox >= 0 && oy >= 0 && ox < i16::from(self.width) && oy < i16::from(self.height) {
                    blocks[usize::from(y) * usize::from(width) + usize::from(x)] =
                        self.blocks[oy as usize * usize::from(self.width) + ox as usize];
                }
            }
        }
        self.blocks = blocks;
        self.width = width;
        self.height = height;
        self.modified = true;
        self.past.clear();
        self.future.clear();
        Ok(true)
    }

    // Upstream remembers even a no-op click and keeps its modified flag until save.
    fn record(&mut self, next: Vec<u8>) -> bool {
        let changed = next != self.blocks;
        self.future.clear();
        push_history(&mut self.past, std::mem::replace(&mut self.blocks, next));
        self.modified = true;
        changed
    }

    /// Restore the previous pixel snapshot; return `false` when history is empty.
    /// Dirty state remains sticky, including when undo returns to saved contents.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.past.pop_back() else {
            return false;
        };
        push_history(
            &mut self.future,
            std::mem::replace(&mut self.blocks, previous),
        );
        self.modified |= self.blocks != self.saved;
        true
    }

    /// Restore the next undone snapshot; return `false` when none exists.
    /// Like undo, this does not clear the sticky dirty flag.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.future.pop_back() else {
            return false;
        };
        push_history(&mut self.past, std::mem::replace(&mut self.blocks, next));
        self.modified |= self.blocks != self.saved;
        true
    }
}

fn push_history(history: &mut VecDeque<Vec<u8>>, state: Vec<u8>) {
    if history.len() == HISTORY_LIMIT {
        history.pop_front();
    }
    history.push_back(state);
}
