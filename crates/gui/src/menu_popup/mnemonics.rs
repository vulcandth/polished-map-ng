//! Main_Window::initialize ampersand keys at pinned master. Input only: the
//! deferred font/underline appearance is deliberately independent of this route.
use crate::egui::{Context, Event, Id, Key, Modifiers, Popup, Response, Ui, UiKind};

#[derive(Clone, Copy)]
pub(super) enum Mnemonic {
    Key(Key),
    Text(char),
}

/// FLTK label_shortcut skips escaped && and uses the first unescaped ampersand.
pub(super) fn label_key(label: &str) -> Option<Mnemonic> {
    let mut chars = label.chars();
    while let Some(c) = chars.next() {
        if c == '&' {
            let next = chars.next()?;
            if next != '&' {
                return Some(
                    Key::from_name(&next.to_string().to_uppercase())
                        .map_or(Mnemonic::Text(next), Mnemonic::Key),
                );
            }
        }
    }
    None
}

#[derive(Clone)]
struct Binding {
    popup: Id,
    widget: Id,
    key: Mnemonic,
    depth: usize,
}

const MENUS: [(&str, Key); 8] = [
    ("File", Key::F),
    ("Data", Key::D),
    ("Edit", Key::E),
    ("View", Key::V),
    ("Mode", Key::M),
    ("Tools", Key::T),
    ("Options", Key::O),
    ("Help", Key::H),
];

fn id(name: &str) -> Id {
    Id::unique(("master-menu-mnemonic", name))
}

pub(super) fn remember(ctx: &Context, name: &str, popup: Id) {
    ctx.data_mut(|data| data.insert_temp(id(name), popup));
}

pub(super) fn within<R>(ui: &mut Ui, name: &str, content: impl FnOnce(&mut Ui) -> R) -> R {
    let root = ui.ctx().data(|d| d.get_temp::<Id>(id(name)));
    ui.ctx().data_mut(|d| d.insert_temp(id("root"), root));
    let result = content(ui);
    ui.ctx().data_mut(|d| d.remove::<Option<Id>>(id("root")));
    result
}

pub(super) fn row(ui: &Ui, response: &Response, key: Option<Mnemonic>) -> bool {
    if !ui.is_enabled() {
        return false;
    }
    let Some(key) = key else {
        return false;
    };
    ui.ctx().data_mut(|d| {
        if let Some(Some(popup)) = d.get_temp::<Option<Id>>(id("root")) {
            let depth = ui
                .stack()
                .iter()
                .filter(|s| s.kind() == Some(UiKind::Menu))
                .count();
            d.get_temp_mut_or_default::<Vec<Binding>>(id("bindings"))
                .push(Binding {
                    popup,
                    widget: response.id,
                    key,
                    depth,
                });
        }
        d.get_temp::<Id>(id("activate")) == Some(response.id)
    })
}

/// Call before editor shortcuts, and only when no modal blocks the editor.
pub(crate) fn begin(ctx: &Context) {
    let mut bindings = ctx.data_mut(|d| {
        d.remove::<Id>(id("activate"));
        let bindings = d
            .get_temp::<Vec<Binding>>(id("bindings"))
            .unwrap_or_default();
        d.insert_temp(id("bindings"), Vec::<Binding>::new());
        bindings
    });
    // FLTK searches the deepest visible menu first, then its ancestors, then
    // the bar. Keep source order for duplicate keys. Closed popups cannot act.
    bindings.sort_by_key(|b| std::cmp::Reverse(b.depth));
    for binding in bindings {
        if Popup::is_id_open(ctx, binding.popup) && consume(ctx, binding.key, false) {
            ctx.data_mut(|d| d.insert_temp(id("activate"), binding.widget));
            return;
        }
    }
    let menu_open = MENUS.iter().any(|(name, _)| {
        ctx.data(|d| d.get_temp::<Id>(id(name)))
            .is_some_and(|popup| Popup::is_id_open(ctx, popup))
    });
    for (name, key) in MENUS {
        if consume(ctx, Mnemonic::Key(key), !menu_open)
            && let Some(popup) = ctx.data(|data| data.get_temp::<Id>(id(name)))
        {
            Popup::open_id(ctx, popup);
            ctx.request_repaint();
            break;
        }
    }
}

fn consume(ctx: &Context, key: Mnemonic, require_alt: bool) -> bool {
    ctx.input_mut(|i| {
        if i.modifiers.ctrl || i.modifiers.command || (require_alt && !i.modifiers.alt) {
            return false;
        }
        let consumed = match key {
            Mnemonic::Key(key) => i.consume_key(
                if i.modifiers.alt {
                    Modifiers::ALT
                } else {
                    Modifiers::NONE
                },
                key,
            ),
            Mnemonic::Text(c) => i
                .events
                .iter()
                .any(|e| matches!(e, Event::Text(t) if t.starts_with(c))),
        };
        if consumed {
            // Native backends can report both a key and text for one stroke.
            // Do not insert the mnemonic into a dialog opened by its action.
            i.events.retain(|e| {
                !matches!(e, Event::Text(t) if match key {
                    Mnemonic::Key(k) => t.eq_ignore_ascii_case(k.name()),
                    Mnemonic::Text(c) => t.starts_with(c),
                })
            });
        }
        consumed
    })
}
