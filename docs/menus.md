# Menus and shortcuts

The unified editor and both parity launchers use **File, Data, Edit, View,
Mode, Tools, Options, Help**. Commands are disabled when their required document
or asset is unavailable. See the [editing guide](usage.md) for workflows and
[preferences](preferences.md) for defaults and persistence.

| Menu | Commands |
|---|---|
| File | New, Open, Open Recent/Clear Recent, Close, Save, Save As, Save Map, Save Blockset, Save Tileset, Save Roof, Print, Exit |
| Data | Load, Save, View, Reload and Unload Event Script; Load/Export Palettes; Load Roof Colors |
| Edit | Undo, Redo, Copy/Paste/Swap Block |
| View | Grid, Rulers, Zoom, Block IDs, Hexadecimal, Show Priority, Game Boy Screen, Show Events, Show Warp IDs, Palettes, Dark Theme, Transparent, Full Screen |
| Mode | Blocks, Events, Switch Mode |
| Tools | Resize Blockset, Resize Map, Change/Edit Tileset, Change/Edit Roof, Edit Current Palettes |
| Options | Format-specific tile controls, Roof Palettes, Auto-Load Events/Special Palettes/Roof Colors, Drag and Drop, Overworld Map Size |
| Help | Help, About |

## Keyboard navigation

Use Ctrl on Windows/Linux and Command on macOS for the shortcuts below unless
noted otherwise. Text entry and modal dialogs take precedence over main-window
commands. Alt+letter opens a menu; arrow keys navigate, Enter activates, and
Escape closes the current menu or cancels a dialog. A number field consumes
Enter to commit its value before the dialog can be accepted.

| Action | Shortcut |
|---|---|
| New / Open / Close | Ctrl/Cmd+N / O / W |
| Save / Save As | Ctrl/Cmd+S / Shift+S |
| Undo / Redo | Ctrl/Cmd+Z / Y |
| Copy / Paste / Swap Block | Ctrl/Cmd+C / V / X |
| Print | Ctrl/Cmd+P |
| Resize Blockset / Map | Ctrl/Cmd+B / E |
| Change / Edit Tileset | Ctrl/Cmd+H / T |
| Change / Edit Roof | Ctrl/Cmd+Shift+H / Ctrl/Cmd+F |
| Edit Current Palettes | Ctrl/Cmd+Shift+L |
| Load / Save Event Script | Ctrl/Cmd+A / Shift+A |
| View / Reload / Unload Event Script | Ctrl/Cmd+U / R / Shift+W |
| Load Palettes | Ctrl/Cmd+L |
| Grid / Zoom / Block IDs | Ctrl/Cmd+G / = / I |
| Rulers / Hexadecimal / Show Priority | Ctrl/Cmd+Shift+R / Shift+4 / Shift+P |
| Game Boy Screen / Show Events / Show Warp IDs | Ctrl/Cmd+Shift+M / Shift+V / Shift+3 |
| Blocks / Events / Switch Mode | Ctrl/Cmd+Shift+B / Shift+E / Tab |
| Dark Theme / Transparent / Full Screen | F9 / F10 / F11 |
| Help / About | F1 / Ctrl/Cmd+/ |

Block digit bindings use **physical Ctrl on every platform**: Ctrl+digit assigns
one digit to the selected block, Ctrl+Shift+digit clears it, and the digit selects
and reveals its bound block. Bindings last only for the session. Assignment and
clearing do not suppress overlapping View shortcuts. Bound Alt+digits select a
block before Open Recent; unbound Alt+1 through Alt+0 open the recent entries.

## Options by document format

The unified Options menu always shows both tile-control groups. The inactive
group is disabled; both are disabled before opening a map.

- Classic assets: Monochrome, Tile Priority and 256 Tiles. Tile Priority enables
  editing priority attributes; View → Show Priority only controls their overlay.
- Attributed assets: 512 Tiles and `$0:80-FF Before $1:80-FF`. Bank ordering is
  disabled when 512 Tiles is off; priority belongs to each tile occurrence.

Roof Palettes chooses how later roof-color imports interpret their table:
Custom; Morn + Day, Night; Morn, Day, Night; Morn + Day, Night, Custom; or
Morn, Day, Night, Custom. It does not immediately reload colors. Unified Open/New
can infer this layout or accept an explicit override under Project options.

Drag and Drop controls internal block/tile/roof/palette transfers (drag swaps,
Ctrl-drag copies). It does not disable painting, event movement or external file
opening. External drops open the first file dropped over the map workspace,
using the normal Open options and unsaved-change guard.

Overworld Map Size sets the post-save warning threshold, not the map dimensions
or the game's actual capacity. See [saving](usage.md#saving).

## Separate plusplus menu differences

The fixed master launcher shows only classic tile options; fixed plusplus shows
only attributed options. Neither switches formats automatically. All launchers
use the current NG Help guide; fixed launchers identify their format restriction.

## Window and help commands

Dark Theme switches the supported light/dark styles without changing map colors.
Transparent requests 75% opacity; unsupported desktop capabilities produce an
error and preserve the last accepted setting. Full Screen reflects the native
window state. Escape does not exit fullscreen or close the main window.

Help opens a resizable nonmodal window. About is modal and retains project and
asset attribution. Both work before opening a map. Their OK buttons, Enter and
Escape dismiss them without editing the project.

## Geometry and interaction gates

Source-derived sizing, popup behavior, native adapters and their verification
limits belong in [compatibility evidence](compatibility.md). Current definitions
are in [menu specifications](../crates/gui/src/editor/menu_popup/specs.rs) and
[command dispatch](../crates/gui/src/editor/menus.rs). Native acceptance and deferred
appearance matching are tracked in the [roadmap](roadmap.md).
