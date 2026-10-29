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
| File | New, Open, Save, Save As, Save Blockset, Save Tileset, Save Roof, Print | Open Recent (ten entries and Clear Recent), Close, separate Save Map, Exit menu item. Native window close already has a dirty guard. Print's complete options/overlays remain pending. |
| Data | Load/Save/View/Reload/Unload Event Script; Load/Export Palettes | Manual Load Roof Colors. Event and palette actions moved here from File. |
| Edit | Undo, Redo, Copy/Paste/Swap Block | The extra Edit Block entry is temporary access to the existing editor; upstream opens it through block interactions. Complete context-menu/gesture equivalence is pending. |
| View | Grid, Rulers, Zoom, Block IDs, Hexadecimal, Game Boy Screen, Show Events, Show Warp IDs, six palette sets | Theme submenu, Show Priority, Transparent, Full Screen. The existing Dark appearance checkbox remains a portable interim control; native theme/font matching remains incomplete. |
| Mode | Blocks, Events, Switch Mode | Native menu keyboard traversal/mnemonics still need verification. Radio items and Tab/shift-command shortcuts use the same mode transition and clear active gestures. |
| Tools | Resize Blockset, Resize Map, Change/Edit Tileset, Change/Edit Roof, Edit Current Palettes | Native child-window fidelity and complete editor parity remain separate requirements. Resize and editor actions moved here from Edit. |
| Options | Auto-Load Events | Monochrome, Tile Priority, 256 Tiles, five Roof Palettes choices, Auto-Load Special Palettes, Auto-Load Roof Colors, Drag and Drop, Overworld Map Size. Some underlying options exist in project/open APIs; that does not establish menu or persistent-preference parity. |
| Help | Inline development status, project attribution and icon licensing links | Original Help and About actions/dialogs, including keyboard shortcuts. Attribution remains available throughout the port. |

## Geometry and interaction gates

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
