//! Shared project errors and stable CLI error-code mapping. LGPL-3.0-only.
use crate::codecs::CodecError;
use polished_map_core::{BlocksetError, MapError, RasterError};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
/// Project I/O, parsing, model-validation, or guarded-save failure.
///
/// Use [`Self::code`] for the existing CLI/JSON category; display text adds
/// context for humans. A conflict requires reloading or observing a new target.
pub enum ProjectError {
    #[error("{path}: {source}")]
    /// A filesystem operation failed.
    Io {
        /// Affected file, or recovery directory for `RecoveryRequired`.
        path: PathBuf,
        #[source]
        /// Underlying operating-system error.
        source: std::io::Error,
    },
    #[error("{path}: {message}")]
    /// A file, path, or requested format is unsupported or malformed.
    Format {
        /// Affected file, or recovery directory for `RecoveryRequired`.
        path: PathBuf,
        /// Human-readable explanation of the failure.
        message: String,
    },
    #[error("{0} changed on disk since it was read; reload or preview again before saving")]
    /// Observed file content changed before a guarded operation.
    Conflict(PathBuf),
    #[error(
        "paired save failed and could not fully roll back; recovery files are in {path}: {message}"
    )]
    /// A multi-file save failed and rollback left recovery files.
    RecoveryRequired {
        /// Affected file, or recovery directory for `RecoveryRequired`.
        path: PathBuf,
        /// Human-readable explanation of the failure.
        message: String,
    },
    #[error("map dimensions are unknown; supply width/height or use Name.WxH.tileset.blk")]
    /// Neither explicit options nor metadata supplied map dimensions.
    DimensionsRequired,
    #[error("this map has no backing file; use Save As")]
    /// An unsaved or context-changed map requires Save As.
    DestinationRequired,
    #[error(transparent)]
    /// The byte-map model rejected an operation.
    Map(#[from] MapError),
    #[error(transparent)]
    /// Classic graphics validation or rendering failed.
    Raster(#[from] RasterError),
    #[error(transparent)]
    /// Classic block or collision validation failed.
    Blockset(#[from] BlocksetError),
    #[error(transparent)]
    /// Attributed model validation failed; the variant name is retained for API compatibility.
    PlusPlus(#[from] polished_map_core::attributed::Error),
    #[error(transparent)]
    /// Tile pixels, attributes, or an export layout are invalid.
    Tileset(#[from] polished_map_core::TilesetError),
    #[error(transparent)]
    /// A palette address or color edit is invalid.
    Palette(#[from] polished_map_core::PaletteError),
    #[error(transparent)]
    /// An event edit violates model constraints.
    Event(#[from] polished_map_core::EventError),
}

impl ProjectError {
    pub(crate) fn io(path: &Path, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
    pub(crate) fn codec(path: &Path, error: CodecError) -> Self {
        Self::Format {
            path: path.into(),
            message: error.to_string(),
        }
    }
    /// Stable machine-readable category used in CLI JSON errors.
    ///
    /// Historical category spellings remain unchanged for existing consumers.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Io { .. } => "io",
            Self::Format { .. } => "format",
            Self::Conflict(_) => "conflict",
            Self::RecoveryRequired { .. } => "recovery_required",
            Self::DimensionsRequired => "dimensions_required",
            Self::DestinationRequired => "destination_required",
            Self::Map(_) => "invalid_map",
            Self::Raster(_) => "invalid_graphics",
            Self::Blockset(_) => "invalid_blockset",
            Self::PlusPlus(_) => "invalid_plusplus_data",
            Self::Tileset(_) => "invalid_tileset",
            Self::Palette(_) => "invalid_palette",
            Self::Event(_) => "invalid_event",
        }
    }
}
