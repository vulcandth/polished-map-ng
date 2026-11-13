//! Map and blockset save orchestration; upstream Main_Window save callbacks. LGPL-3.0-only.
use super::*;

impl Editor {
    pub(super) fn save(&mut self) {
        if self.save_ui.busy() || self.error.is_some() {
            return;
        }
        let Some(loaded) = &self.loaded else {
            return;
        };
        let tiles = loaded
            .tileset
            .as_ref()
            .is_some_and(|t| t.tileset().is_modified());
        let roof = loaded
            .tileset
            .as_ref()
            .and_then(|t| t.roof())
            .is_some_and(|r| r.roof().tileset().is_modified());
        let blocks = loaded
            .blockset
            .as_ref()
            .is_some_and(|b| b.blockset().is_modified());
        let events = loaded
            .events
            .as_ref()
            .is_some_and(|e| e.script().events().is_modified());
        let tiles = tiles
            || loaded
                .plus_assets
                .as_ref()
                .is_some_and(|a| a.tiles.model().is_modified());
        let roof = roof
            || loaded
                .plus_assets
                .as_ref()
                .and_then(|a| a.roof.as_ref())
                .is_some_and(|r| r.is_modified());
        let blocks = blocks
            || loaded
                .plus_assets
                .as_ref()
                .is_some_and(|a| a.blocks.blockset().is_modified());
        use save_feedback::SaveStep;
        let map = loaded.document.map().is_modified() || !(tiles || roof || blocks || events);
        for (needed, step) in [
            (tiles, SaveStep::Tileset),
            (roof, SaveStep::Roof),
            (blocks, SaveStep::Blocks),
            (events, SaveStep::Events),
            (map, SaveStep::Map),
        ] {
            if needed {
                self.save_ui.steps.push_back(step);
            }
        }
        self.advance_save();
    }

    pub(super) fn save_map(&mut self) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        if loaded.document.needs_save_as() {
            self.save_as();
            return;
        }
        if let Some(loaded) = &mut self.loaded {
            let modified = loaded.document.map().is_modified();
            // master save_map(false): acknowledging an oversized input does
            // not dirty the map. A clean Save Map must not truncate its source.
            if !modified {
                let name = save_feedback::basename(loaded.document.path());
                self.status = "Map saved".into();
                self.saved_message(name);
                return;
            }
            match loaded.document.save() {
                Ok(()) => {
                    let name = save_feedback::basename(loaded.document.path());
                    self.status = "Map saved".into();
                    self.check_overworld_buffer();
                    self.saved_message(name);
                }
                Err(e) => self.error = Some(e.to_string()),
            }
        }
    }

    pub(super) fn save_blockset(&mut self) {
        if self.save_plusplus_blocks() {
            return;
        }
        let Some(doc) = self.loaded.as_mut().and_then(|l| l.blockset.as_mut()) else {
            return;
        };
        if !doc.blockset().is_modified() {
            self.blockset_saved_message();
            return;
        }
        if doc.needs_save_as() {
            let path = rfd::FileDialog::new()
                .add_filter("Binary blockset", &["bin", "bst"])
                .set_file_name(
                    doc.path()
                        .with_extension("bin")
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                )
                .save_file();
            let choice = path.and_then(|path| {
                if let Some(collision) = doc.collision_path() {
                    rfd::FileDialog::new()
                        .set_title("Save Collision Copy")
                        .set_file_name(collision.file_name().unwrap_or_default().to_string_lossy())
                        .save_file()
                        .map(|collision| (path, Some(collision)))
                } else {
                    Some((path, None))
                }
            });
            self.save_blockset_choice(choice);
            return;
        }
        let result = doc.save();
        self.finish_blockset_save(result);
    }

    /// Both native destinations must be chosen before preparing or writing either file.
    pub(super) fn save_blockset_choice(&mut self, choice: Option<(PathBuf, Option<PathBuf>)>) {
        let Some((path, collision)) = choice else {
            self.save_ui.steps.clear();
            return;
        };
        let Some(doc) = self.loaded.as_mut().and_then(|l| l.blockset.as_mut()) else {
            return;
        };
        let result = prepare_blockset_save_as(&path).and_then(|target| {
            if let Some(collision) = collision {
                prepare_collision_save_as(&collision, doc.binary_collisions())
                    .and_then(|collision| doc.save_as_pair(&target, &collision))
            } else {
                doc.save_as(&target)
            }
        });
        self.finish_blockset_save(result);
    }

    pub(super) fn finish_blockset_save(
        &mut self,
        result: Result<(), polished_map_project::ProjectError>,
    ) {
        match result {
            Ok(()) => {
                self.status = "Blockset saved".into();
                self.blockset_saved_message();
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    pub(super) fn save_as(&mut self) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let source = loaded.document.path();
        let name = loaded
            .document
            .save_path_suggestion(self.application.is_unified());
        let path = rfd::FileDialog::new()
            .add_filter("Map blocks", loaded.document.save_extensions())
            .set_directory(source.parent().unwrap_or(source))
            .set_file_name(name.file_name().unwrap_or_default().to_string_lossy())
            .save_file();
        // The native Save dialog supplies the overwrite decision. Observe the target
        // immediately after it returns, then recheck before atomic replacement.
        self.save_map_choice(path);
    }

    pub(super) fn save_map_choice(&mut self, path: Option<PathBuf>) {
        if let Some(path) = path {
            self.save_as_path(path);
        } else {
            self.save_ui.steps.clear();
        }
    }

    pub(super) fn save_as_path(&mut self, path: PathBuf) {
        if self.loaded.is_none() {
            return;
        }
        match self.loaded.as_mut().unwrap().document.save_as(&path) {
            Ok(()) => {
                let mut recent_path = path.clone();
                if recent_path.extension().is_none() {
                    recent_path
                        .set_extension(self.loaded.as_ref().unwrap().document.default_extension());
                }
                self.loaded.as_mut().unwrap().recent_path = Some(recent_path.clone());
                self.recent_maps.remember(recent_path);
                self.status = format!("Saved {}", path.display());
                self.check_overworld_buffer();
                self.saved_message(save_feedback::basename(
                    self.loaded.as_ref().unwrap().document.path(),
                ));
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}
