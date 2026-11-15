//! Pinned plusplus map-events.cpp and Main_Window::{open_map,warp_to_map}.
//! LGPL-3.0-only. Reuses the shared parser, edits, source spans and guarded save;
//! the catalog and script discovery remain explicit plusplus entry points.
use super::MapDocument;
use crate::{
    EventScript, EventScriptError, ProjectError, SaveTarget, TilesetOutput, discovery::Rules,
};
use polished_map_core::EventCommand;
use std::path::{Path, PathBuf};

/// Parse the extended event catalog while preserving unrelated assembly.
///
/// # Errors
/// Returns bounded-source and recognized-event syntax errors.
pub fn parse_event_script(source: &str) -> Result<EventScript, EventScriptError> {
    EventScript::parse_with_rules(source, Rules::PlusPlus)
}

/// Config::event_script_path has no SXY/pokered directory fallbacks in plusplus.
pub fn event_script_path(root: &Path, map_name: &str) -> Result<PathBuf, ProjectError> {
    crate::events::validate_map_name(root, map_name)?;
    Ok(root.join("maps").join(format!("{map_name}.asm")))
}

/// Prefer the root's map script, then the current map's adjacent .asm file.
/// Missing scripts and unsaved maps return None; no file is created.
pub fn event_script_for_map(map: &MapDocument) -> Result<Option<PathBuf>, ProjectError> {
    if !map.has_backing_file() {
        return Ok(None);
    }
    let discovered = map
        .metadata()
        .root
        .as_deref()
        .map(|root| event_script_path(root, &map.metadata().name))
        .transpose()?;
    let path = discovered
        .filter(|p| p.is_file())
        .unwrap_or_else(|| map.path().with_extension("asm"));
    Ok(path.is_file().then_some(path))
}

/// Resolve an external warp in the same map directory and with the current
/// extension. Callers own existence/same-map checks and dirty-document consent.
pub fn warp_map_path(current: &Path, destination: &str) -> Result<PathBuf, ProjectError> {
    crate::events::validate_map_name(current, destination)?;
    let extension = current
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("ablk");
    Ok(current.with_file_name(format!("{destination}.{extension}")))
}

#[derive(Clone, Debug)]
/// Compatibility facade selecting the extended event catalog at open time.
///
/// Convert into [`crate::EventDocument`] to use the shared persistence API;
/// the parsed macro kinds and source-rewrite state are retained.
pub struct EventDocument {
    inner: crate::EventDocument,
}

impl EventDocument {
    /// Open a bounded UTF-8 script with the extended event catalog.
    ///
    /// # Errors
    /// Read failures, malformed source, or concurrent changes are rejected.
    pub fn open(path: &Path) -> Result<Self, ProjectError> {
        Ok(Self {
            inner: crate::EventDocument::open_with_rules(path, Rules::PlusPlus)?,
        })
    }
    /// Discover and open a map’s event script, or return `None` when absent.
    ///
    /// # Errors
    /// Invalid map names and script read/parse failures are returned.
    pub fn open_for_map(map: &MapDocument) -> Result<Option<Self>, ProjectError> {
        event_script_for_map(map)?
            .as_deref()
            .map(Self::open)
            .transpose()
    }
    /// Observed script path and hash.
    pub fn source(&self) -> &SaveTarget {
        self.inner.source()
    }
    /// Shared source-preserving script model.
    pub fn script(&self) -> &EventScript {
        self.inner.script()
    }
    /// Apply an atomic event batch; see [`crate::EventScript::apply`] for errors.
    pub fn apply(&mut self, commands: &[EventCommand]) -> Result<(), ProjectError> {
        self.inner.apply(commands)
    }
    /// Preview a guarded rewrite; see [`crate::EventDocument::preview_save`].
    pub fn preview_save(&self) -> Result<Option<TilesetOutput>, ProjectError> {
        self.inner.preview_save()
    }
    /// Persist changed spans; see [`crate::EventDocument::save`] for conflict handling.
    pub fn save(&mut self) -> Result<(), ProjectError> {
        self.inner.save()
    }
}

/// Recover the shared event document after selecting the extended parser.
/// The parsed macro kinds, source spans, dirty state and observed hash are retained.
impl From<EventDocument> for crate::EventDocument {
    fn from(document: EventDocument) -> Self {
        document.inner
    }
}
