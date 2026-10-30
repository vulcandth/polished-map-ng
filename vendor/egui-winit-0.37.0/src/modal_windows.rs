//! Polished Map NG native modal ownership bridge (MIT OR Apache-2.0).
//! The renderer registers weak window references. Modal leases retain live
//! windows, and registration teardown releases leases before native destruction.
//! No native call is made while the egui Context data lock is held.
use egui::{Context, ViewportId};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use winit::{modal::ModalWindow, window::Window};

#[derive(Clone, Default)]
struct Registry {
    windows: HashMap<ViewportId, Weak<Window>>,
    parents: HashMap<ViewportId, ViewportId>,
    leases: HashMap<ViewportId, Arc<ModalWindow>>,
    error: Option<&'static str>,
}
fn key() -> egui::Id {
    egui::Id::unique("polished-map-native-modal-windows-v1")
}

/// Request ownership, including before the renderer has created either window.
/// Reject cycles and simultaneous sibling modals; nested dialogs form a chain.
pub fn set_parent(
    ctx: &Context,
    child: ViewportId,
    parent: ViewportId,
) -> Result<(), &'static str> {
    ctx.data_mut(|data| {
        let registry = data.get_temp_mut_or_default::<Registry>(key());
        if registry.parents.get(&child) == Some(&parent) {
            return Ok(());
        }
        if registry.parents.contains_key(&child) {
            return Err("Detach the dialog before changing its parent");
        }
        let mut ancestor = parent;
        loop {
            if ancestor == child {
                return Err("A native dialog cannot form an ownership cycle");
            }
            match registry.parents.get(&ancestor) {
                Some(next) => ancestor = *next,
                None => break,
            }
        }
        if registry.parents.values().any(|p| *p == parent) {
            return Err("The parent already has a modal dialog");
        }
        registry.parents.insert(child, parent);
        Ok(())
    })?;
    dispatch(ctx);
    Ok(())
}

/// Whether application input to this viewport must be suppressed, regardless of
/// window-manager policy (including compositors without xdg-dialog support).
pub fn is_blocked(ctx: &Context, viewport: ViewportId) -> bool {
    ctx.data(|data| {
        data.get_temp::<Registry>(key())
            .is_some_and(|r| r.parents.values().any(|p| *p == viewport))
    })
}

/// Detach a modal and its descendants. The lease drops outside the Context lock,
/// restoring native parent input before the renderer can destroy any child.
pub fn detach(ctx: &Context, child: ViewportId) {
    let leases = ctx.data_mut(|data| {
        let registry = data.get_temp_mut_or_default::<Registry>(key());
        let mut removed = vec![child];
        let mut index = 0;
        while index < removed.len() {
            let parent = removed[index];
            removed.extend(
                registry
                    .parents
                    .iter()
                    .filter_map(|(c, p)| (*p == parent).then_some(*c)),
            );
            index += 1;
        }
        // Deepest first: restore an editor before restoring its parent.
        removed
            .into_iter()
            .rev()
            .filter_map(|id| {
                registry.parents.remove(&id);
                registry.leases.remove(&id)
            })
            .collect::<Vec<_>>()
    });
    drop(leases);
}

pub fn take_error(ctx: &Context) -> Option<&'static str> {
    ctx.data_mut(|data| data.get_temp_mut_or_default::<Registry>(key()).error.take())
}

fn dispatch(ctx: &Context) {
    let pending = ctx.data(|data| {
        let registry = data.get_temp::<Registry>(key()).unwrap_or_default();
        registry
            .parents
            .iter()
            .filter_map(|(child, parent)| {
                if registry.leases.contains_key(child) {
                    return None;
                }
                Some((
                    *child,
                    registry.windows.get(child)?.upgrade()?,
                    registry.windows.get(parent)?.upgrade()?,
                ))
            })
            .collect::<Vec<_>>()
    });
    for (id, child, parent) in pending {
        match ModalWindow::new(child, parent) {
            Ok(lease) => ctx.data_mut(|data| {
                data.get_temp_mut_or_default::<Registry>(key())
                    .leases
                    .insert(id, Arc::new(lease));
            }),
            Err(reason) => ctx.data_mut(|data| {
                data.get_temp_mut_or_default::<Registry>(key()).error = Some(reason);
            }),
        }
    }
}

pub(crate) struct Registration {
    ctx: Context,
    id: ViewportId,
    window: Weak<Window>,
}
impl Registration {
    pub(crate) fn new(ctx: &Context, id: ViewportId, window: &Arc<Window>) -> Self {
        let window = Arc::downgrade(window);
        let replacing = ctx.data(|data| {
            data.get_temp::<Registry>(key())
                .is_some_and(|r| r.windows.contains_key(&id))
        });
        if replacing {
            detach(ctx, id);
        }
        ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<Registry>(key())
                .windows
                .insert(id, window.clone());
        });
        dispatch(ctx);
        Self {
            ctx: ctx.clone(),
            id,
            window,
        }
    }
}
impl Drop for Registration {
    fn drop(&mut self) {
        // A renderer can register a replacement before dropping its old State.
        // That old registration must never detach the new window's modal lease.
        let current = self.ctx.data(|d| {
            d.get_temp::<Registry>(key())
                .and_then(|r| r.windows.get(&self.id).map(|w| w.ptr_eq(&self.window)))
                .unwrap_or(false)
        });
        if !current {
            return;
        }
        detach(&self.ctx, self.id);
        self.ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<Registry>(key())
                .windows
                .remove(&self.id);
        });
    }
}

/// Strip user input for an inactive parent while keeping timing, scale, redraw,
/// and viewport lifecycle information. Never discard its close event: the app
/// must answer CancelClose so eframe does not interpret it as an accepted exit.
pub fn filter_input(ctx: &Context, input: &mut egui::RawInput) {
    if is_blocked(ctx, input.viewport_id) {
        // Releases can occur over the child and never reach its parent. Reset
        // retained pointer state, rather than synthesizing a release/click that
        // could commit an interrupted gesture when the parent becomes active.
        ctx.input_mut_for(input.viewport_id, |state| state.pointer = Default::default());
        input.events.clear();
        input.hovered_files.clear();
        input.dropped_files.clear();
        input.events.push(egui::Event::WindowFocused(false));
        input
            .events
            .push(egui::Event::ModifiersChanged(Default::default()));
        input.focused = false;
    }
}

/// Borrow a renderer-owned window for a synchronous native dialog. The strong
/// reference stays alive for the whole operation, outside the Context lock.
pub fn with_window<T>(
    ctx: &Context,
    id: ViewportId,
    operation: impl FnOnce(&Window) -> T,
) -> Option<T> {
    let window = ctx.data(|d| d.get_temp::<Registry>(key())?.windows.get(&id)?.upgrade())?;
    Some(operation(&window))
}
