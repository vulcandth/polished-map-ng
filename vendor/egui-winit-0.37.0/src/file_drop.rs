//! Polished Map NG extension, under egui-winit's MIT OR Apache-2.0 license.
use std::path::PathBuf;

/// A completed native transfer. Coordinates are egui points in this viewport.
#[derive(Clone, Debug)]
pub struct FileDrop {
    pub transfer_id: u64,
    pub paths: Vec<PathBuf>,
    pub position: egui::Pos2,
}

fn key(viewport: egui::ViewportId) -> egui::Id {
    egui::Id::unique(("polished-map-native-file-drops-v1", viewport))
}

/// Consume this viewport's completed transfers exactly once, including rejected
/// transfers. Other viewports' transfers are never returned.
pub fn take(ctx: &egui::Context, viewport: egui::ViewportId) -> Vec<FileDrop> {
    ctx.data_mut(|data| {
        data.remove_temp::<Vec<FileDrop>>(key(viewport))
            .unwrap_or_default()
    })
}

/// Native event bridge. Invalid scale/positions are rejected, never inferred
/// from the ordinary pointer. The bounded queue retains the oldest transfers.
pub fn enqueue(
    ctx: &egui::Context,
    viewport: egui::ViewportId,
    transfer_id: u64,
    paths: Vec<PathBuf>,
    position: [f64; 2],
    pixels_per_point: f32,
) {
    if !pixels_per_point.is_finite()
        || pixels_per_point <= 0.0
        || paths.is_empty()
        || paths.len() > 4096
    {
        return;
    }
    let position = egui::pos2(
        position[0] as f32 / pixels_per_point,
        position[1] as f32 / pixels_per_point,
    );
    if !position.x.is_finite() || !position.y.is_finite() {
        return;
    }
    ctx.data_mut(|data| {
        let queue = data.get_temp_mut_or_default::<Vec<FileDrop>>(key(viewport));
        if queue.len() < 16 && !queue.iter().any(|drop| drop.transfer_id == transfer_id) {
            queue.push(FileDrop {
                transfer_id,
                paths,
                position,
            });
        }
    });
}
