//! Master option-dialogs.cpp: add_valid_sizes/add_map_size/guess_map_size.
//! LGPL-3.0-only; the GUI chooses a size, while headless opens stay explicit.
use std::path::Path;

use polished_map_core::MapError;

use crate::{ProjectError, document::decode_map_blocks, read_bounded};

#[derive(Clone, Debug)]
pub struct MapSizeOptions {
    block_count: usize,
    choices: Vec<(u16, u16)>,
    selected: Option<usize>,
}

impl MapSizeOptions {
    /// Descending-width factor pairs, preferring 10x9 multiples on the wide
    /// half of the list, exactly as master's add_valid_sizes does.
    pub fn from_block_count(block_count: usize) -> Self {
        let choices: Vec<_> = (1_u16..=255)
            .rev()
            .filter_map(|width| {
                let w = usize::from(width);
                let height = block_count / w;
                (block_count.is_multiple_of(w) && (1..=255).contains(&height))
                    .then_some((width, height as u16))
            })
            .collect();
        let selected = choices.len().checked_sub(1).map(|last| {
            let middle = last / 2;
            (0..=middle)
                .rev()
                .find(|&i| choices[i].0.is_multiple_of(10) && choices[i].1.is_multiple_of(9))
                .unwrap_or(middle)
        });
        Self {
            block_count,
            choices,
            selected,
        }
    }

    /// Filename/metadata dimensions select an existing pair or append it even
    /// when its area differs from the file. The loader owns size warnings/errors.
    pub fn with_preferred(mut self, width: u16, height: u16) -> Result<Self, MapError> {
        if !(1..=255).contains(&width) || !(1..=255).contains(&height) {
            return Err(MapError::Dimensions { width, height });
        }
        let pair = (width, height);
        self.selected = Some(
            self.choices
                .iter()
                .position(|&p| p == pair)
                .unwrap_or_else(|| {
                    self.choices.push(pair);
                    self.choices.len() - 1
                }),
        );
        Ok(self)
    }

    pub fn block_count(&self) -> usize {
        self.block_count
    }

    pub fn choices(&self) -> &[(u16, u16)] {
        &self.choices
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// New/unfactorable maps use the source's 10x9 fallback and free spinners.
    pub fn dimensions(&self) -> (u16, u16) {
        self.selected.map_or((10, 9), |i| self.choices[i])
    }
}

/// Read the same bounded binary/ASM bytes as map opening, without modifying the
/// project. For ASM maps the displayed byte count is decoded data, not text size.
pub fn inspect_map_sizes(
    path: &Path,
    preferred: Option<(u16, u16)>,
) -> Result<MapSizeOptions, ProjectError> {
    let bytes = read_bounded(path, 4 * 1024 * 1024)?;
    let blocks = decode_map_blocks(path, &bytes)?;
    let options = MapSizeOptions::from_block_count(blocks.len());
    Ok(match preferred {
        Some((width, height)) => options.with_preferred(width, height)?,
        None => options,
    })
}
