# Master menu inventory

Reference: `Main_Window::initialize` at pinned master
`8b9bdc2b8aac8c865809516c2123af69c498da0b`, principally
[`src/main-window.cpp`](https://github.com/Rangi42/polished-map/blob/8b9bdc2b8aac8c865809516c2123af69c498da0b/src/main-window.cpp).
This is an implementation checklist, not a completed parity claim.

The top-level order is **File, Data, Edit, View, Mode, Tools, Options, Help**.
`crates/gui/src/menus.rs` now presents the implemented commands in those groups.
Existing editing/persistence operations remain shared with their toolbar,
keyboard and headless counterparts where applicable.

| Group | Present in this port | Remaining source items or differences |
|---|---|---|
| File | New, Open, Open Recent/Clear Recent, Close, Save, Save As, Save Map, Save Blockset, Save Tileset, Save Roof, Print, Exit | Print's complete options/overlays remain pending. Native close delivery still needs platform verification. |
| Data | Load/Save/View/Reload/Unload Event Script; Load/Export Palettes; Load Roof Colors | Native dialogs and complete popup geometry remain pending. |
| Edit | Undo, Redo, Copy/Paste/Swap Block | The extra Edit Block entry is temporary access to the existing editor; upstream opens it through block interactions. Complete context-menu/gesture equivalence is pending. |
| View | Grid, Rulers, Zoom, Block IDs, Hexadecimal, Game Boy Screen, Show Events, Show Warp IDs, six palette sets | Theme submenu, Show Priority, Transparent, Full Screen. The existing Dark appearance checkbox remains a portable interim control; native theme/font matching remains incomplete. |
| Mode | Blocks, Events, Switch Mode | Native menu keyboard traversal/mnemonics still need verification. Radio items and Tab/shift-command shortcuts use the same mode transition and clear active gestures. |
| Tools | Resize Blockset, Resize Map, Change/Edit Tileset, Change/Edit Roof, Edit Current Palettes | Native child-window fidelity and complete editor parity remain separate requirements. Resize and editor actions moved here from Edit. |
| Options | Five Roof Palettes choices; Auto-Load Events, Auto-Load Special Palettes, Auto-Load Roof Colors | Monochrome, Tile Priority, 256 Tiles, Drag and Drop, Overworld Map Size. Remaining native/persistence gaps are tracked in `preferences.md`. |
| Help | Inline development status, project attribution and icon licensing links | Original Help and About actions/dialogs, including keyboard shortcuts. Attribution remains available throughout the port. |

## Geometry and interaction gates

Open Recent shows up to ten basenames with Alt+1 through Alt+0. Full paths are
available on hover; duplicate basenames remain separate entries. Completed opens
and Save As promote the exact path spelling, removing an existing exact match
and evicting the oldest entry when full. New unsaved maps, canceled options,
failed opens/saves and ordinary Save do not add entries. Selection uses the
normal unsaved warning followed by Map Options. Clear Recent changes only the
list, persists on exit, and remains available when empty. History survives Close.

Load Roof Colors requires a known map group and resolves the modern project
roof table before the legacy path. Manual loading is available for indoor and
monochrome maps too. It asks OK/Cancel before replacing edited palette colors,
preserves the edited flag and selected set, updates previews after success and
shows the original success wording. Failed reads retain the whole session.
Roof Palettes changes only the interpretation of subsequent manual/automatic
imports; it does not immediately reload colors. All five choices survive Close
within the session and are captured on exit for the next launch. See
`preferences.md` for the implemented settings, defaults and native checks still
required. Auto-Load Special Palettes and Auto-Load Roof Colors affect subsequent
opens without immediately replacing colors or disabling manual imports.

Close clears loaded map/events/assets, textures, map history, block clipboard,
selection, gestures and scroll state. It retains palette colors, the selected
palette set, palette-only dirty state, monochrome and view/mode settings.
Save Map writes only the map; Save retains the original independent-domain
ordering. Map conflicts preserve the loaded document and external bytes.
Close (Ctrl/Cmd+W) and Exit (Alt+F4) share upstream's unsaved warning semantics:
OK discards/continues, Cancel or Escape keeps the document, Enter accepts.
Warnings name the first dirty domain in map/events/tiles/roof/blockset order;
palette-only edits do not trigger this source guard. Shift+Ctrl/Cmd+W continues
to unload only the event script. Native window close uses the same exit guard.
The warning's native window, shield icon and measured client geometry remain
pending; the current egui modal reproduces wording, button order and actions.

- Preserve the 21-point menu-bar band and source top-level/command order. Tests
  verify bounds, command placement and disabled state when data is unavailable.
- Reproduce the popup image/check column, row metrics, separators, shortcut
  captions and mnemonics. Upstream's `BLANK_ICON` reserves an image column for
  ordinary commands; it does not decorate all commands with toolbar artwork.
  That alignment and the complete popup styling remain pending.
- Verify source enablement for every data/profile condition. Current checks
  cover unavailable map/blockset/tileset/roof/events, history controls, existing
  workflow dialogs and modal guards; this is not the full profile matrix.
- Test every command through its final menu route as it is ported. Never add
  inert placeholders to imply that a missing source command works.
- Keep native OS behavior separate from egui headless evidence. Screenshots and
  synthetic interaction tests do not prove native font, menu, DPI or dialog
  equivalence on any platform.
