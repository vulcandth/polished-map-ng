//! November 7 review: choose a native first-launch default without replacing
//! saved NG/legacy choices or blocking the UI on a Linux desktop portal.
use super::*;

impl Editor {
    pub(super) fn detect_startup_theme(&mut self, ctx: &egui::Context) {
        if let Some(theme) = ctx.system_theme() {
            self.dark = theme == egui::Theme::Dark;
            ctx.set_theme(theme);
            return;
        }
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let dark = match dark_light::detect() {
                Ok(dark_light::Mode::Dark) => Some(true),
                Ok(dark_light::Mode::Light) => Some(false),
                _ => None,
            };
            let _ = tx.send(dark);
            ctx.request_repaint();
        });
        self.startup_theme = Some(rx);
    }

    pub(super) fn poll_startup_theme(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.startup_theme else {
            return;
        };
        match rx.try_recv() {
            Ok(dark) => {
                self.startup_theme = None;
                if let Some(dark) = dark {
                    self.dark = dark;
                    ctx.set_theme(if dark {
                        egui::Theme::Dark
                    } else {
                        egui::Theme::Light
                    });
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => self.startup_theme = None,
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delayed_detection_is_once_only_and_manual_choice_cancels_it() {
        for detected in [None, Some(false), Some(true)] {
            for cancelled in [false, true] {
                let ctx = egui::Context::default();
                let (tx, rx) = mpsc::channel();
                let mut editor = Editor {
                    startup_theme: Some(rx),
                    ..Default::default()
                };
                editor.poll_startup_theme(&ctx);
                assert!(editor.startup_theme.is_some());
                tx.send(detected).unwrap();
                if cancelled {
                    editor.startup_theme = None;
                }
                editor.poll_startup_theme(&ctx);
                assert_eq!(editor.dark, !cancelled && detected == Some(true));
                assert!(editor.startup_theme.is_none());
            }
        }
    }
}
