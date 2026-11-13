//! Open/New selection and asynchronous map loading; upstream Main_Window. LGPL-3.0-only.
use super::*;

impl Editor {
    pub(super) fn choose_open(&mut self) {
        self.pending_recent = None;
        self.pending_close = false;
        self.pending_new = false;
        if self.is_modified() {
            self.confirm_discard = true;
            return;
        }
        self.open_map_picker();
    }

    // Open and its accepted-discard continuation must use the same profile filter.
    pub(super) fn open_map_picker(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Map blocks",
                if self.application.is_unified() {
                    &["ablk", "blk", "map"]
                } else if self.profile() == documents::Profile::PlusPlus {
                    &["ablk", "blk"]
                } else {
                    &["blk", "map"]
                },
            )
            .pick_file()
        {
            self.prepare_open(path);
        }
    }

    pub(super) fn choose_new(&mut self) {
        self.pending_recent = None;
        self.pending_close = false;
        self.pending_new = true;
        if self.is_modified() {
            self.confirm_discard = true;
            return;
        }
        self.start_new_options();
    }

    pub(super) fn start_new_options(&mut self) {
        let root = self
            .loaded
            .as_ref()
            .and_then(|l| l.document.metadata().root.clone())
            .or_else(|| rfd::FileDialog::new().pick_folder());
        self.pending_new = false;
        if let Some(root) = root {
            self.prepare_new_in(root);
        }
    }

    pub fn prepare_new_in(&mut self, root: PathBuf) {
        let mut form = OpenForm {
            unified: self.application.is_unified(),
            roof_layout: Some(self.roof_palette_layout),
            new_map: true,
            profile: self.profile(),
            allow_512: self.allow_512,
            path: root.join(format!("NewMap.{}", self.profile().extension())),
            root: root.to_string_lossy().into_owned(),
            width: 10,
            height: 9,
            allow_256: self.allow_256,
            ..Default::default()
        };
        match form.refresh_choices() {
            Ok(()) => self.open_form = Some(form),
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub fn prepare_open(&mut self, path: PathBuf) {
        let metadata = match self.open_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        let mut form = OpenForm {
            unified: self.application.is_unified(),
            roof_layout: Some(self.roof_palette_layout),
            profile: self.profile(),
            allow_512: self.allow_512,
            root: metadata
                .root
                .as_deref()
                .or_else(|| path.parent())
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned(),
            path,
            allow_256: self.allow_256,
            ..Default::default()
        };
        match form.refresh_choices() {
            Ok(()) => self.open_form = Some(form),
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub(super) fn start_load(&mut self, mut form: OpenForm, ctx: &egui::Context) {
        if form.unified {
            form.resolve_profile();
            if let Some(error) = &form.detection_error {
                self.error = Some(error.clone());
                self.open_form = Some(form);
                return;
            }
        }
        if !self.application.accepts(form.profile) {
            self.error = Some("Open options belong to the other editor profile".into());
            return;
        }
        self.pending_map_warning = None;
        self.pending_palette_warnings.clear();
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        let palettes = self.palettes_for_open(form.profile);
        let settings = map_loading::LoadSettings {
            graphics: MasterGraphicsOptions {
                dark_theme: self.dark,
                roof_layout: form.roof_layout.unwrap_or(self.roof_palette_layout),
                special_palettes: self.auto_special_palettes,
                roof_colors: self.auto_roof_colors,
                allow_priority: self.allow_priority,
                monochrome: self.monochrome_preference(),
                ..Default::default()
            },
            bank_order: self.bank_order,
            auto_events: self.event_ui.auto_load,
        };
        std::thread::spawn(move || {
            let _ = sender.send(Loaded::open(form, palettes, settings));
            ctx.request_repaint();
        });
        self.loading = Some(receiver);
        self.status = "Loading map…".into();
    }

    pub(super) fn poll_workers(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.loading.as_ref().and_then(|rx| match rx.try_recv() {
            Ok(r) => Some(r),
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("Loading worker stopped unexpectedly".into()))
            }
            Err(_) => None,
        }) {
            self.loading = None;
            match result {
                Ok(loaded) => {
                    self.pending_open = Some(loaded);
                    self.advance_open_palettes(ctx);
                }
                Err(error) => {
                    self.status = "Open failed".into();
                    self.error = Some(error);
                }
            }
        }
        self.poll_print(ctx);
    }
}
