//! master Main_Window::{store_recent_map,update_recent_maps,open_recent}.
//! LGPL-3.0-only. Keep original path spelling; never canonicalize or probe on startup.
use super::*;
use serde::{Deserialize, Serialize};

const LIMIT: usize = 10;
const KEYS: [egui::Key; LIMIT] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
    egui::Key::Num9,
    egui::Key::Num0,
];

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<StoredPath>", into = "Vec<StoredPath>")]
pub(super) struct RecentMaps(Vec<PathBuf>);

// UTF-8 paths remain readable and portable. Native fallbacks are lossless and
// reject incompatible hosts rather than silently changing a file name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StoredPath {
    Utf8(String),
    UnixBytes(Vec<u8>),
    WindowsWide(Vec<u16>),
}

impl From<RecentMaps> for Vec<StoredPath> {
    fn from(recent: RecentMaps) -> Self {
        recent
            .0
            .into_iter()
            .map(|path| {
                if let Some(text) = path.to_str() {
                    return StoredPath::Utf8(text.into());
                }
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStringExt;
                    StoredPath::UnixBytes(path.into_os_string().into_vec())
                }
                #[cfg(windows)]
                {
                    use std::os::windows::ffi::OsStrExt;
                    StoredPath::WindowsWide(path.as_os_str().encode_wide().collect())
                }
            })
            .collect()
    }
}

impl TryFrom<Vec<StoredPath>> for RecentMaps {
    type Error = &'static str;
    fn try_from(stored: Vec<StoredPath>) -> Result<Self, Self::Error> {
        if stored.len() > LIMIT {
            return Err("too many recent maps");
        }
        let mut paths: Vec<PathBuf> = Vec::with_capacity(stored.len());
        for item in stored {
            let path = match item {
                StoredPath::Utf8(text) => PathBuf::from(text),
                #[cfg(unix)]
                StoredPath::UnixBytes(bytes) => {
                    use std::os::unix::ffi::OsStringExt;
                    std::ffi::OsString::from_vec(bytes).into()
                }
                #[cfg(windows)]
                StoredPath::WindowsWide(wide) => {
                    use std::os::windows::ffi::OsStringExt;
                    std::ffi::OsString::from_wide(&wide).into()
                }
                _ => return Err("recent map uses another platform's native path encoding"),
            };
            if path.as_os_str().is_empty()
                || paths.iter().any(|p| p.as_os_str() == path.as_os_str())
            {
                return Err("empty or duplicate recent map");
            }
            paths.push(path);
        }
        Ok(Self(paths))
    }
}

impl RecentMaps {
    pub(super) fn remember(&mut self, path: PathBuf) {
        if path.as_os_str().is_empty() {
            return;
        }
        self.0.retain(|p| p.as_os_str() != path.as_os_str());
        self.0.insert(0, path);
        self.0.truncate(LIMIT);
    }
}

impl Editor {
    fn request_recent(&mut self, index: usize) {
        let Some(path) = self.recent_maps.0.get(index).cloned() else {
            return;
        };
        self.pending_close = false;
        self.pending_exit = false;
        self.pending_new = false;
        self.event_ui.warp_path = None;
        if self.is_modified() {
            self.pending_recent = Some(path);
            self.confirm_discard = true;
        } else {
            self.pending_recent = None;
            self.prepare_open(path);
        }
    }

    pub(super) fn recent_shortcuts(&mut self, ctx: &egui::Context) {
        for (index, key) in KEYS.into_iter().enumerate() {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, key)) {
                self.request_recent(index);
                break;
            }
        }
    }

    pub(super) fn recent_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Open Recent", |ui| {
            let mut chosen = None;
            for (index, path) in self.recent_maps.0.iter().enumerate() {
                let label = path
                    .file_name()
                    .unwrap_or(path.as_os_str())
                    .to_string_lossy();
                let response = ui
                    .push_id(index, |ui| {
                        ui.add(egui::Button::new(label.as_ref()).shortcut_text(
                            ui.ctx().format_shortcut(&egui::KeyboardShortcut::new(
                                egui::Modifiers::ALT,
                                KEYS[index],
                            )),
                        ))
                    })
                    .inner;
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::MenuItem, ui.is_enabled(), label.as_ref())
                });
                if response.on_hover_text(path.to_string_lossy()).clicked() {
                    chosen = Some(index);
                    ui.close();
                }
            }
            if !self.recent_maps.0.is_empty() {
                ui.separator();
            }
            if ui.button("Clear Recent").clicked() {
                self.recent_maps.0.clear();
                ui.close();
            }
            if let Some(index) = chosen {
                self.request_recent(index);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;
    use std::{
        fs,
        time::{Duration, Instant},
    };

    #[test]
    fn history_keeps_ten_paths_promotes_exact_duplicates_and_round_trips_native_names() {
        let mut recent = RecentMaps::default();
        for i in 0..12 {
            recent.remember(format!("{i}/Map.blk").into());
        }
        assert_eq!(
            recent.0,
            (2..12)
                .rev()
                .map(|i| PathBuf::from(format!("{i}/Map.blk")))
                .collect::<Vec<_>>()
        );
        recent.remember("5/Map.blk".into());
        assert_eq!(recent.0[0], PathBuf::from("5/Map.blk"));
        assert_eq!(recent.0[1], PathBuf::from("11/Map.blk"));
        assert_eq!(recent.0[9], PathBuf::from("2/Map.blk"));
        recent.remember("5/./Map.blk".into());
        assert_ne!(recent.0[0].as_os_str(), recent.0[1].as_os_str());
        recent.remember("目录/é & map.blk".into());
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            recent.remember(std::ffi::OsString::from_vec(b"bad-\xff.blk".to_vec()).into());
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt;
            recent.remember(std::ffi::OsString::from_wide(&[0xd800, 46, 98, 108, 107]).into());
        }
        let json = serde_json::to_string(&recent).unwrap();
        let restored: RecentMaps = serde_json::from_str(&json).unwrap();
        for (a, b) in recent.0.iter().zip(&restored.0) {
            assert_eq!(a.as_os_str(), b.as_os_str());
        }
        assert!(serde_json::from_str::<RecentMaps>(r#"[{"utf8":""}]"#).is_err());
        assert!(serde_json::from_str::<RecentMaps>(r#"[{"utf8":"a"},{"utf8":"a"}]"#).is_err());
        assert!(
            RecentMaps::try_from(
                (0..11)
                    .map(|i| StoredPath::Utf8(i.to_string()))
                    .collect::<Vec<_>>()
            )
            .is_err()
        );
    }

    #[test]
    fn recent_shortcuts_open_options_guard_edits_and_clear_without_touching_files() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let original = fs::canonicalize(dir.path().join("Map.2x1.blk")).unwrap();
        assert_eq!(h.state().recent_maps.0, std::slice::from_ref(&original));
        for i in 0..10 {
            let path = dir.path().join(format!("Recent{i}.2x1.blk"));
            fs::write(&path, [1, 0]).unwrap();
            h.state_mut().recent_maps.remember(path);
        }
        let expected = h.state().recent_maps.clone();
        for (index, key) in KEYS.into_iter().enumerate() {
            h.key_press_modifiers(egui::Modifiers::ALT, key);
            h.run();
            assert_eq!(
                h.state().open_form.as_ref().unwrap().path,
                expected.0[index]
            );
            assert_eq!(
                h.state().recent_maps,
                expected,
                "selecting is not a completed open"
            );
            h.get_by_label("Cancel").click();
            h.run();
        }
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num1);
        h.run();
        assert!(h.state().confirm_discard);
        assert!(
            h.query_by_label("Map.2x1.blk has unsaved changes!\n\nOpen another map anyway?")
                .is_some()
        );
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num0);
        h.run();
        assert_eq!(
            h.state().pending_recent.as_ref(),
            Some(&expected.0[0]),
            "modal blocks shortcuts"
        );
        h.get_by_label("Cancel").click();
        h.run();
        assert!(h.state().pending_recent.is_none());
        assert!(h.state().is_modified());
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num0);
        h.run();
        h.get_by_label("OK").click();
        h.run();
        assert_eq!(h.state().open_form.as_ref().unwrap().path, expected.0[9]);
        h.get_by_label("Cancel").click();
        h.run();
        assert!(
            h.state().is_modified(),
            "canceling options retains the document"
        );
        h.get_by_label("File").click();
        h.run();
        h.get_by_label("Open Recent").click();
        h.run();
        #[cfg(feature = "visual-tests")]
        {
            h.remove_cursor();
            h.run();
            h.snapshot("open-recent-ten");
        }
        h.get_by_role_and_label(egui::Role::MenuItem, "Recent9.2x1.blk")
            .click();
        h.run();
        assert_eq!(h.state().pending_recent.as_ref(), Some(&expected.0[0]));
        h.get_by_label("Cancel").click();
        h.run();
        h.get_by_label("File").click();
        h.run();
        h.get_by_label("Open Recent").click();
        h.run();
        h.get_by_label("Clear Recent").click();
        h.run();
        assert!(h.state().recent_maps.0.is_empty());
        assert!(h.state().is_modified());
        assert_eq!(fs::read(&original).unwrap(), [0, 1]);
        for path in expected.0 {
            assert_eq!(fs::read(path).unwrap(), [1, 0]);
        }
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Num1);
        h.run();
        assert!(!h.state().confirm_discard && h.state().open_form.is_none());
    }

    #[test]
    fn completed_opens_and_save_as_record_paths_but_new_maps_failures_and_regular_saves_do_not() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let original = fs::canonicalize(dir.path().join("Map.2x1.blk")).unwrap();
        let next = dir.path().join("Next.2x1.blk");
        fs::write(&next, [1, 0]).unwrap();
        let ctx = h.ctx.clone();
        for (path, new_map, success) in [
            (next.clone(), false, true),
            (dir.path().join("Missing.blk"), false, false),
            (dir.path().join("NewMap.blk"), true, true),
        ] {
            h.state_mut().start_load(
                OpenForm {
                    path,
                    new_map,
                    root: dir.path().to_string_lossy().into_owned(),
                    tileset: "demo".into(),
                    width: 2,
                    height: 1,
                    ..Default::default()
                },
                &ctx,
            );
            let deadline = Instant::now() + Duration::from_secs(5);
            while h.state().loading.is_some() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(5));
                h.step();
            }
            assert!(h.state().loading.is_none());
            h.run();
            assert_eq!(h.state().error.is_none(), success);
            assert_eq!(h.state().recent_maps.0, [next.clone(), original.clone()]);
            h.state_mut().error = None;
        }
        h.state_mut().save_as_path(dir.path().join("Copy"));
        assert!(h.state().error.is_none());
        let copy = dir.path().join("Copy.blk");
        assert_eq!(copy.extension().unwrap(), "blk");
        assert_eq!(h.state().recent_maps.0, [copy.clone(), next, original]);
        h.state_mut().save_as_path(dir.path().join("invalid.map"));
        assert!(h.state().error.is_some());
        assert_eq!(h.state().recent_maps.0[0], copy);
        let history = h.state().recent_maps.clone();
        h.state_mut().recent_maps.0.clear();
        h.state_mut().save_map();
        assert!(
            h.state().recent_maps.0.is_empty(),
            "ordinary Save must not repopulate cleared history"
        );
        h.state_mut().recent_maps = history.clone();
        h.state_mut().close_map();
        assert_eq!(h.state().recent_maps, history);
    }
}
