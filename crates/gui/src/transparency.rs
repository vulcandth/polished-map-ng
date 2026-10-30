//! master initialize/transparent_cb/apply_transparency/exit_cb and main.cpp.
//! LGPL-3.0-only. Opacity belongs to the native window, never rendered map data.
use super::*;
use polished_map_platform::{OpacityError, WindowOpacity};

#[derive(Default)]
pub(super) struct Transparency {
    // Preserve startup intent if this desktop cannot apply it. The menu reflects
    // accepted native requests, so a failed restore does not claim transparency.
    preferred: bool,
    applied: bool,
    pending: Option<bool>,
    in_flight: Option<(u64, bool)>,
}

impl Transparency {
    pub(super) fn preferred(&self) -> bool {
        self.preferred
    }

    pub(super) fn restore(&mut self, enabled: bool) {
        self.preferred = enabled;
        // Source main.cpp applies only enabled transparency at startup.
        self.pending = enabled.then_some(true);
    }

    pub(super) fn toggle(&mut self) {
        if !self.busy() {
            self.pending = Some(!self.applied);
        }
    }

    pub(super) fn menu(&mut self, ui: &mut egui::Ui) {
        let mut enabled = self.applied;
        if menu_popup::checkbox(ui, !self.busy(), &mut enabled, "Transparent")
            .on_hover_text("Whole-window opacity: 75% (F10). Requires native desktop support.")
            .changed()
        {
            self.toggle();
        }
    }

    fn busy(&self) -> bool {
        self.pending.is_some() || self.in_flight.is_some()
    }
}

impl Editor {
    #[cfg(test)]
    pub(super) fn dispatch_transparency(
        &mut self,
        apply: impl FnOnce(WindowOpacity) -> Result<(), OpacityError>,
    ) {
        self.dispatch_transparency_inner(None, apply);
    }

    pub(super) fn dispatch_native_transparency(
        &mut self,
        ctx: &egui::Context,
        apply: impl FnOnce(WindowOpacity) -> Result<(), OpacityError>,
    ) {
        self.dispatch_transparency_inner(Some(ctx), apply);
    }

    fn dispatch_transparency_inner(
        &mut self,
        ctx: Option<&egui::Context>,
        apply: impl FnOnce(WindowOpacity) -> Result<(), OpacityError>,
    ) {
        if let Some(ctx) = ctx {
            let viewport = ctx.viewport_id();
            if let Some((id, enabled)) = self.transparency.in_flight
                && let Some(result) = egui_winit::window_opacity::take_reply(ctx, viewport, id)
            {
                self.transparency.in_flight = None;
                self.complete_transparency(enabled, result.map_err(OpacityError::Unsupported));
            }
            if let Some(reason) = egui_winit::window_opacity::take_unavailable(ctx, viewport) {
                // The native adapter reset the root AND frame to opaque. Retain
                // startup intent, but never leave the menu claiming transparency.
                self.transparency.applied = false;
                self.error = Some(reason.to_owned());
            }
        }
        if self.transparency.in_flight.is_some() {
            return;
        }
        let Some(enabled) = self.transparency.pending.take() else {
            return;
        };
        let opacity = if enabled {
            WindowOpacity::Transparent
        } else {
            WindowOpacity::Opaque
        };
        let result = apply(opacity);
        if matches!(result, Err(OpacityError::WaylandProtocolRequired))
            && let Some(ctx) = ctx
        {
            match egui_winit::window_opacity::request(ctx, ctx.viewport_id(), opacity.alpha32()) {
                Ok(id) => self.transparency.in_flight = Some((id, enabled)),
                Err(reason) => self.error = Some(reason.to_owned()),
            }
            return;
        }
        self.complete_transparency(enabled, result);
    }

    fn complete_transparency(&mut self, enabled: bool, result: Result<(), OpacityError>) {
        match result {
            Ok(()) => {
                self.transparency.applied = enabled;
                self.transparency.preferred = enabled;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    fn complete(h: &mut Harness<'_, Editor>, expected: WindowOpacity) {
        h.state_mut().dispatch_transparency(|actual| {
            assert_eq!(actual, expected);
            Ok(())
        });
        h.run();
        h.state_mut()
            .dispatch_transparency(|_| panic!("request replayed on idle"));
    }

    #[test]
    fn menu_and_f10_request_whole_window_opacity_without_a_map() {
        let mut h = Harness::builder()
            .with_size(Vec2::new(800.0, 600.0))
            .with_theme(egui::Theme::Light)
            .build_eframe(|cc| Editor::new(cc));
        crate::menu_popup::test_font_size(&h.ctx, 12.0);
        h.key_press(egui::Key::F10);
        h.run();
        assert_eq!(h.state().transparency.pending, Some(true));
        complete(&mut h, WindowOpacity::Transparent);
        h.get_by_label("View").click();
        h.run();
        let node = h.get_by_label("Transparent");
        assert_eq!(
            node.accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        assert_eq!(
            node.accesskit_node().data().keyboard_shortcut(),
            Some("F10")
        );
        assert!(node.rect().top() > h.get_by_label("Palettes").rect().top());
        assert!(node.rect().bottom() <= h.get_by_label("Full Screen").rect().top());
        #[cfg(feature = "visual-tests")]
        {
            h.remove_cursor();
            h.run();
            h.snapshot("transparency-menu");
        }
        h.get_by_label("Transparent").click();
        h.run();
        complete(&mut h, WindowOpacity::Opaque);
        assert!(!h.state().transparency.preferred());
        assert!(!h.state().is_modified() && h.state().loaded.is_none());
        h.key_down(egui::Key::F10);
        h.step();
        h.key_up(egui::Key::F10);
        h.run();
        complete(&mut h, WindowOpacity::Transparent);
    }

    #[test]
    fn failures_keep_previous_state_and_preserve_restored_intent_without_retry_loops() {
        let mut editor = Editor::default();
        editor.transparency.restore(true);
        assert!(!editor.transparency.applied);
        editor.dispatch_transparency(|_| Err(OpacityError::Unsupported("unsupported desktop")));
        assert_eq!(editor.error.as_deref(), Some("unsupported desktop"));
        assert!(editor.transparency.preferred());
        assert!(!editor.transparency.applied);
        editor.dispatch_transparency(|_| panic!("failed request retried automatically"));
        editor.error = None;
        editor.transparency.toggle();
        editor.dispatch_transparency(|_| Ok(()));
        assert!(editor.transparency.applied);
        editor.transparency.toggle();
        editor.dispatch_transparency(|_| Err(OpacityError::Native("failure".into())));
        assert!(editor.transparency.preferred() && editor.transparency.applied);
    }

    #[test]
    fn wayland_transparency_waits_for_native_reply_and_preserves_intent_on_capability_loss() {
        let ctx = egui::Context::default();
        let mut editor = Editor::default();
        editor.transparency.restore(true);
        editor.dispatch_native_transparency(&ctx, |_| Err(OpacityError::WaylandProtocolRequired));
        assert!(!editor.transparency.applied);
        assert!(editor.transparency.preferred());
        assert!(editor.transparency.busy());
        editor.transparency.toggle();
        assert!(
            editor.transparency.pending.is_none(),
            "no duplicate request while awaiting native output"
        );
        editor.dispatch_native_transparency(&ctx, |_| panic!("must await reply"));
        egui_winit::window_opacity::dispatch_with(&ctx, egui::ViewportId::ROOT, |factor| {
            assert_eq!(factor, 0xbfff_ffff);
            Ok(())
        });
        assert!(
            !editor.transparency.applied,
            "queueing output does not update the editor itself"
        );
        editor.dispatch_native_transparency(&ctx, |_| panic!("must not replay native request"));
        assert!(editor.transparency.applied && !editor.transparency.busy());
        egui_winit::window_opacity::unavailable(
            &ctx,
            egui::ViewportId::ROOT,
            "Compositor took ownership of the frame",
        );
        editor.dispatch_native_transparency(&ctx, |_| {
            panic!("must not automatically retry after capability loss")
        });
        assert!(!editor.transparency.applied);
        assert!(editor.transparency.preferred());
        assert_eq!(
            editor.error.as_deref(),
            Some("Compositor took ownership of the frame")
        );
        editor.error = None;
        editor.dispatch_native_transparency(&ctx, |_| panic!("idle after dismissal"));
        assert!(editor.error.is_none());
        editor.transparency.toggle();
        editor.dispatch_native_transparency(&ctx, |_| Err(OpacityError::WaylandProtocolRequired));
        egui_winit::window_opacity::dispatch_with(&ctx, egui::ViewportId::ROOT, |_| {
            Err("No alpha-modifier protocol")
        });
        editor.dispatch_native_transparency(&ctx, |_| panic!("failure must resolve without retry"));
        assert!(!editor.transparency.applied && !editor.transparency.busy());
        assert!(editor.transparency.preferred());
        assert_eq!(editor.error.as_deref(), Some("No alpha-modifier protocol"));
    }

    #[test]
    fn native_opacity_output_is_scoped_and_consumed_once_including_failures() {
        use egui_winit::window_opacity as bridge;
        let ctx = egui::Context::default();
        let root = egui::ViewportId::ROOT;
        let child = egui::ViewportId::from_hash_of("child");
        let first = bridge::request(&ctx, root, 0xbfff_ffff).unwrap();
        let other = bridge::request(&ctx, child, u32::MAX).unwrap();
        assert!(bridge::request(&ctx, root, u32::MAX).is_err());
        assert!(bridge::take_reply(&ctx, root, first).is_none());
        bridge::dispatch_with(&ctx, child, |factor| {
            assert_eq!(factor, u32::MAX);
            Ok(())
        });
        assert!(bridge::take_reply(&ctx, root, first).is_none());
        assert_eq!(bridge::take_reply(&ctx, child, other), Some(Ok(())));
        bridge::dispatch_with(&ctx, root, |_| Err("Unsupported frame"));
        bridge::dispatch_with(&ctx, root, |_| panic!("request replayed"));
        assert!(bridge::take_reply(&ctx, root, first.wrapping_add(1)).is_none());
        assert!(
            bridge::request(&ctx, root, u32::MAX).is_err(),
            "unread reply cannot be overwritten"
        );
        assert_eq!(
            bridge::take_reply(&ctx, root, first),
            Some(Err("Unsupported frame"))
        );
        assert!(bridge::take_reply(&ctx, root, first).is_none());
        let second = bridge::request(&ctx, root, u32::MAX).unwrap();
        assert_ne!(first, second);
        bridge::dispatch_with(&ctx, root, |_| Ok(()));
        assert_eq!(bridge::take_reply(&ctx, root, second), Some(Ok(())));
    }

    #[test]
    fn transparency_preserves_dirty_map_history_and_obeys_modal_and_modifier_guards() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        h.run();
        h.key_press(egui::Key::F10);
        h.run();
        complete(&mut h, WindowOpacity::Transparent);
        assert!(h.state().loaded.as_ref().unwrap().document.map().can_undo());
        assert_eq!(
            h.state().loaded.as_ref().unwrap().document.map().blocks(),
            [1, 1]
        );
        h.state_mut().prepare_resize();
        h.run();
        h.key_press(egui::Key::F10);
        h.run();
        assert!(h.state().transparency.pending.is_none());
        h.get_by_label("Cancel").click();
        h.run();
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
        h.run();
        assert!(h.state().transparency.pending.is_none());
        h.state_mut().close_map();
        h.run();
        assert!(h.state().transparency.applied);
        assert_eq!(
            std::fs::read(dir.path().join("Map.2x1.blk")).unwrap(),
            [0, 1]
        );
    }
}
