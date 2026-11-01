//! Opt-in native shutdown/restart verification with caller-owned temporary paths.
//! No default user directories are read. This does not test OS home discovery.
use super::*;
use eframe::App;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

const FUTURE: &str = "{\"version\":999,\"future-setting\":true}";

struct Probe {
    editor: Editor,
    phase: String,
    frames: usize,
    saved: Arc<AtomicBool>,
    started: Instant,
    resized: bool,
}

impl App for Probe {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.editor.ui(ui, frame);
        self.frames += 1;
        let ctx = ui.ctx();
        // Let the native geometry tracker receive the normal window events.
        if self.frames >= 4 && self.started.elapsed() >= Duration::from_millis(250) {
            let rect = ctx
                .input(|i| i.viewport().inner_rect)
                .expect("native inner rectangle");
            if self.phase == "restore" && !self.resized {
                assert!((rect.width() - 640.0).abs() <= 1.0, "{rect:?}");
                assert!((rect.height() - 480.0).abs() <= 1.0, "{rect:?}");
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(720.0, 520.0)));
                self.resized = true;
                ctx.request_repaint_after(Duration::from_millis(25));
                return;
            }
            let expected = if self.phase == "import" {
                egui::vec2(640.0, 480.0)
            } else {
                egui::vec2(720.0, 520.0)
            };
            if (rect.size() - expected).abs().max_elem() > 1.0 {
                assert!(
                    self.phase == "restore" && self.started.elapsed() < Duration::from_secs(5),
                    "native size {rect:?}, expected {expected:?}"
                );
                ctx.request_repaint_after(Duration::from_millis(25));
                return;
            }
            self.editor.request_exit(ctx);
            assert!(self.editor.exit_approved && !self.editor.confirm_discard);
        } else {
            ctx.request_repaint_after(Duration::from_millis(25));
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        assert!(
            self.editor.exit_approved,
            "settings must not autosave before exit"
        );
        App::save(&mut self.editor, storage);
        let own_key = key(self.editor.profile);
        if self.phase == "future" {
            // Simulate a later NG version's record using the real storage writer.
            storage.set_string(own_key, FUTURE.into());
        } else if self.phase == "preserve" {
            assert_eq!(storage.get_string(own_key).as_deref(), Some(FUTURE));
        } else {
            let actual: Preferences =
                serde_json::from_str(&storage.get_string(own_key).unwrap()).unwrap();
            assert_eq!(actual, Preferences::capture(&self.editor));
        }
        let other = match self.editor.profile {
            documents::Profile::Master => documents::Profile::PlusPlus,
            documents::Profile::PlusPlus => documents::Profile::Master,
        };
        assert!(storage.get_string(key(other)).is_none());
        assert!(
            storage.get_string("window").is_some(),
            "eframe native window record"
        );
        assert!(
            !self.saved.swap(true, Ordering::SeqCst),
            "exactly one shutdown save"
        );
    }

    fn auto_save_interval(&self) -> Duration {
        self.editor.auto_save_interval()
    }
    fn persist_egui_memory(&self) -> bool {
        self.editor.persist_egui_memory()
    }
}

/// Run one child of the `native_preferences` example. Requires the opt-in
/// feature and environment flag; `directory` must be a disposable fixture root.
/// The explicit persistence path intentionally avoids the user's actual store.
pub fn run(directory: &Path, profile: &str, phase: &str) -> eframe::Result {
    assert_eq!(
        std::env::var("POLISHED_MAP_NATIVE_PREFERENCES_PROBE").as_deref(),
        Ok("1")
    );
    assert!(directory.join("probe-marker").is_file());
    let profile = match profile {
        "master" => documents::Profile::Master,
        "plusplus" => documents::Profile::PlusPlus,
        _ => panic!("unknown probe profile"),
    };
    assert!(matches!(
        phase,
        "import" | "restore" | "future" | "preserve"
    ));
    let folder = directory.join(profile.app_id());
    let file = folder.join("app.ron");
    std::fs::create_dir_all(&folder).unwrap();
    let startup = NativeStartup::from_result(
        profile,
        NativeStartup::load(
            profile,
            directory,
            &LegacyUserLocation::Windows {
                roaming_appdata: None,
            },
            Some(&file),
        ),
    );
    let viewport = startup.viewport();
    assert_eq!(viewport.app_id.as_deref(), Some(profile.app_id()));
    let saved = Arc::new(AtomicBool::new(false));
    let saved_in_app = saved.clone();
    let phase = phase.to_owned();
    eframe::run_native(
        profile.title(),
        eframe::NativeOptions {
            viewport,
            persistence_path: Some(file.clone()),
            persist_window: true,
            ..Default::default()
        },
        Box::new(move |cc| {
            let storage = cc.storage.expect("native file storage enabled");
            if phase != "import" {
                assert!(
                    storage.get_string("window").is_some(),
                    "previous native window record restored"
                );
            }
            let mut editor = startup.create_editor(cc);
            assert!(editor.profile == profile);
            if phase == "preserve" {
                assert!(!editor.preferences_writable);
                assert!(editor.grid && !editor.rulers && !editor.allow_512);
                assert_eq!(storage.get_string(key(profile)).as_deref(), Some(FUTURE));
            } else {
                assert!(editor.preferences_writable);
                assert!(!editor.grid && editor.rulers);
                assert_eq!(editor.allow_512, profile == documents::Profile::PlusPlus);
                assert_eq!(
                    editor.bank_order == polished_map_core::plusplus::BankOrder::Bank0First,
                    profile == documents::Profile::PlusPlus
                );
                if phase == "import" {
                    assert!(editor.status.starts_with("Imported settings"));
                    assert!(!editor.dark);
                    // Persist a setting different from the source import to prove
                    // subsequent processes restore NG rather than re-importing it.
                    editor.dark = true;
                } else {
                    assert!(editor.dark);
                    assert!(!editor.status.starts_with("Imported settings"));
                }
            }
            assert!(editor.loaded.is_none() && editor.open_form.is_none());
            Ok(Box::new(Probe {
                editor,
                phase,
                frames: 0,
                saved: saved_in_app,
                started: Instant::now(),
                resized: false,
            }))
        }),
    )?;
    assert!(saved.load(Ordering::SeqCst), "native shutdown did not save");
    assert!(
        std::fs::metadata(file).unwrap().len() > 0,
        "storage was not flushed to disk"
    );
    Ok(())
}
