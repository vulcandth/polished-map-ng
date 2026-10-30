//! master DnD_Receiver and Main_Window::drag_and_drop_cb (LGPL-3.0-only).
//! Native transfer coordinates are supplied by the pinned egui-winit bridge.
use super::*;

impl Editor {
    pub(super) fn external_file_drops(&mut self, ctx: &egui::Context, bounds: Rect) {
        let drops = egui_winit::file_drop::take(ctx, ctx.viewport_id());
        if ctx.viewport_id() != egui::ViewportId::ROOT || self.interaction_blocked() {
            return;
        }
        let layout = geometry::Layout::new(
            bounds.width(),
            bounds.height(),
            self.zoom,
            self.rulers,
            16.0,
        );
        let workspace = Rect::from_min_size(
            bounds.min + Vec2::new(layout.canvas.x, layout.canvas.y),
            Vec2::new(layout.canvas.width, layout.canvas.height),
        );
        for drop in drops {
            let point = drop.position;
            if point.x >= workspace.min.x
                && point.x < workspace.max.x
                && point.y >= workspace.min.y
                && point.y < workspace.max.y
                && let Some(path) = drop.paths.into_iter().next()
            {
                self.request_open_path(path);
                // The first accepted transfer starts a modal workflow. Further
                // transfers in the same frame must not replace its pending path.
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    fn drop_at(
        ctx: &egui::Context,
        viewport: egui::ViewportId,
        id: u64,
        paths: Vec<PathBuf>,
        point: [f64; 2],
        scale: f32,
    ) {
        egui_winit::file_drop::enqueue(ctx, viewport, id, paths, point, scale);
    }

    #[test]
    fn external_drop_bridge_scales_once_preserves_order_and_isolates_viewports() {
        let ctx = egui::Context::default();
        let child = egui::ViewportId::from_hash_of("child");
        let paths = vec![PathBuf::from("目录/a b.blk"), PathBuf::from("other.blk")];
        drop_at(&ctx, child, 12, paths.clone(), [900.0, 600.0], 3.0);
        assert!(egui_winit::file_drop::take(&ctx, egui::ViewportId::ROOT).is_empty());
        let got = egui_winit::file_drop::take(&ctx, child);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].paths, paths);
        assert_eq!(got[0].position, egui::pos2(300.0, 200.0));
        assert!(egui_winit::file_drop::take(&ctx, child).is_empty());
        for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            drop_at(&ctx, child, 13, paths.clone(), [1.0, 2.0], scale);
        }
        drop_at(&ctx, child, 14, paths.clone(), [f64::NAN, 2.0], 1.0);
        assert!(egui_winit::file_drop::take(&ctx, child).is_empty());
        for id in 0..100 {
            drop_at(&ctx, child, id, paths.clone(), [1.0, 2.0], 1.0);
        }
        assert_eq!(egui_winit::file_drop::take(&ctx, child).len(), 16);
    }

    #[test]
    fn external_drop_targets_workspace_including_blank_space_and_ignores_pointer() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let first = dir.path().join("First.2x1.blk");
        let second = dir.path().join("Second.2x1.blk");
        std::fs::write(&first, [0, 1]).unwrap();
        std::fs::write(&second, [1, 0]).unwrap();
        let bounds = h.ctx.content_rect();
        let layout = geometry::Layout::new(
            bounds.width(),
            bounds.height(),
            h.state().zoom,
            h.state().rulers,
            16.0,
        );
        let x = f64::from(bounds.left() + layout.canvas.x);
        let y = f64::from(bounds.top() + layout.canvas.y);
        for point in [
            [x - 1.0, y + 10.0],
            [x + 10.0, y - 1.0],
            [x + f64::from(layout.canvas.width), y + 10.0],
        ] {
            drop_at(
                &h.ctx,
                egui::ViewportId::ROOT,
                1,
                vec![first.clone()],
                point,
                1.0,
            );
            h.run();
            assert!(h.state().open_form.is_none());
        }
        // Near the workspace's lower-right corner, outside the tiny map image.
        let point = [
            x + f64::from(layout.canvas.width) - 2.0,
            y + f64::from(layout.canvas.height) - 2.0,
        ];
        drop_at(
            &h.ctx,
            egui::ViewportId::ROOT,
            2,
            vec![first.clone(), second],
            point.map(|v| v * 2.5),
            2.5,
        );
        h.run();
        assert_eq!(h.state().open_form.as_ref().unwrap().path, first);
        h.get_by_label("Cancel").click();
        h.run();
        assert!(
            h.state().open_form.is_none(),
            "consumed drops cannot replay after cancellation"
        );
    }

    #[test]
    fn external_drop_dirty_guard_and_modal_rejection_preserve_original_document() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let first = dir.path().join("Drop.2x1.blk");
        std::fs::write(&first, [1, 0]).unwrap();
        let original = h
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .path()
            .to_path_buf();
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 1,
        });
        drop_at(
            &h.ctx,
            egui::ViewportId::ROOT,
            3,
            vec![first.clone()],
            [300.0, 200.0],
            1.0,
        );
        h.run();
        assert!(h.state().confirm_discard);
        assert_eq!(h.state().pending_recent.as_ref(), Some(&first));
        drop_at(
            &h.ctx,
            egui::ViewportId::ROOT,
            4,
            vec![PathBuf::from("must-not-replace.blk")],
            [300.0, 200.0],
            1.0,
        );
        h.run();
        assert_eq!(h.state().pending_recent.as_ref(), Some(&first));
        h.get_by_label("Cancel").click();
        h.run();
        assert_eq!(h.state().loaded.as_ref().unwrap().document.path(), original);
        assert!(h.state().is_modified());
        assert!(h.state().pending_recent.is_none());
    }

    #[test]
    fn external_drop_failure_does_not_try_later_files_or_change_history() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        let original = h
            .state()
            .loaded
            .as_ref()
            .unwrap()
            .document
            .path()
            .to_path_buf();
        let history = h.state().recent_maps.clone();
        let missing = dir.path().join("missing-folder/Map.blk");
        drop_at(
            &h.ctx,
            egui::ViewportId::ROOT,
            5,
            vec![missing, original.clone()],
            [300.0, 200.0],
            1.0,
        );
        h.run();
        assert!(h.state().error.is_some());
        assert!(h.state().open_form.is_none());
        assert_eq!(h.state().loaded.as_ref().unwrap().document.path(), original);
        assert_eq!(h.state().recent_maps, history);
        h.get_by_label("OK").click();
        h.run();
        assert!(h.state().error.is_none());
        assert!(h.state().open_form.is_none());
    }
}
