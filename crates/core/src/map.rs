use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const HISTORY_LIMIT: usize = 100;

/// Coordinates in map blocks, not tiles or event coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

/// A single undoable edit. Painting an entire drag uses one command.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Paint { points: Vec<Point>, block: u8 },
    Fill { at: Point, block: u8 },
    Replace { from: u8, to: u8 },
    Swap { a: u8, b: u8 },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MapError {
    #[error("map dimensions must be between 1 and 255; got {width}×{height}")]
    Dimensions { width: u16, height: u16 },
    #[error("expected {expected} map bytes, found {actual}")]
    Length { expected: usize, actual: usize },
    #[error("block coordinate ({0}, {1}) is outside the map")]
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

    pub fn width(&self) -> u8 {
        self.width
    }
    pub fn height(&self) -> u8 {
        self.height
    }
    pub fn blocks(&self) -> &[u8] {
        &self.blocks
    }
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    /// Call only after persistence succeeds.
    pub fn mark_saved(&mut self) {
        self.saved.clone_from(&self.blocks);
        self.modified = false;
    }
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    fn index(&self, point: Point) -> Result<usize, MapError> {
        if point.x >= u16::from(self.width) || point.y >= u16::from(self.height) {
            return Err(MapError::Coordinate(point.x, point.y));
        }
        Ok(usize::from(point.y) * usize::from(self.width) + usize::from(point.x))
    }

    pub fn block(&self, point: Point) -> Result<u8, MapError> {
        Ok(self.blocks[self.index(point)?])
    }

    /// Validate the complete command before changing state. Returns whether it changed pixels.
    pub fn apply(&mut self, command: &Command) -> Result<bool, MapError> {
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
        }
        Ok(self.record(next))
    }

    // Upstream remembers even a no-op click and keeps its modified flag until save.
    fn record(&mut self, next: Vec<u8>) -> bool {
        let changed = next != self.blocks;
        self.future.clear();
        push_history(&mut self.past, std::mem::replace(&mut self.blocks, next));
        self.modified = true;
        changed
    }

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
