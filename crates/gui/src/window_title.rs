//! master Main_Window::{open_map,save_as_cb,close_cb}, option-dialogs.h::NEW_MAP_NAME.
//! LGPL-3.0-only. Keep project filenames out of the fixed application identity.
use super::*;
use std::ffi::{OsStr, OsString};

impl Loaded {
    pub(super) fn display_name(&self) -> &OsStr {
        if !self.document.has_backing_file() {
            return OsStr::new("New Map");
        }
        let path = self.recent_path.as_deref().unwrap_or(self.document.path());
        path.file_name().unwrap_or(path.as_os_str())
    }

    pub(super) fn print_filename(&self) -> OsString {
        let name = self.display_name();
        let path = std::path::Path::new(name);
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("blk"))
        {
            path.with_extension("png").into_os_string()
        } else {
            let mut result = name.to_os_string();
            result.push(".png");
            result
        }
    }
}

impl Editor {
    fn window_title(&self) -> String {
        match &self.loaded {
            Some(loaded) => format!(
                "{WINDOW_TITLE} - {}",
                loaded.display_name().to_string_lossy()
            ),
            None => WINDOW_TITLE.into(),
        }
    }

    pub(super) fn sync_window_title(&mut self, ctx: &egui::Context) {
        let title = self.window_title();
        if self.last_window_title.as_ref() != Some(&title) {
            ctx.send_viewport_cmd_to(
                egui::ViewportId::ROOT,
                egui::ViewportCommand::Title(title.clone()),
            );
            self.last_window_title = Some(title);
        }
    }
}

#[cfg(test)]
mod tests;
