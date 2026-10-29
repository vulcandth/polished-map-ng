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
| File | New, Open, Open Recent/Clear Recent, Close, Save, Save As, Save Map, Save Blockset, Save Tileset, Save Roof, Print, Exit | Print's native fonts, clipboard delivery and chooser behavior remain verification gates. Native close delivery still needs platform verification. |
| Data | Load/Save/View/Reload/Unload Event Script; Load/Export Palettes; Load Roof Colors | Native dialogs and complete popup geometry remain pending. |
| Edit | Undo, Redo, Copy/Paste/Swap Block | The extra Edit Block entry is temporary access to the existing editor; upstream opens it through block interactions. Complete context-menu/gesture equivalence is pending. |
| View | Grid, Rulers, Zoom, Block IDs, Hexadecimal, Show Priority, Game Boy Screen, Show Events, Show Warp IDs, six palette sets, Transparent, Full Screen | Theme submenu; Wayland whole-window opacity integration. The existing Dark appearance checkbox remains a portable interim control; native theme/font matching remains incomplete. |
| Mode | Blocks, Events, Switch Mode | Native menu keyboard traversal/mnemonics still need verification. Radio items and Tab/shift-command shortcuts use the same mode transition and clear active gestures. |
| Tools | Resize Blockset, Resize Map, Change/Edit Tileset, Change/Edit Roof, Edit Current Palettes | Native child-window fidelity and complete editor parity remain separate requirements. Resize and editor actions moved here from Edit. |
| Options | Monochrome, Tile Priority, 256 Tiles; five Roof Palettes choices; Auto-Load Events, Auto-Load Special Palettes, Auto-Load Roof Colors; Drag and Drop; Overworld Map Size | Remaining native/persistence gaps are tracked in `preferences.md`. |
| Help | Help/F1 with bundled source documentation; About/command-/ with project and icon attribution | About's application icon remains excluded pending provenance review. Native window/frame/font appearance still needs verification. |

## Geometry and interaction gates

Transparent precedes Full Screen and shares the source F10 shortcut. It requests
75% whole-window opacity (Win32 byte 191, X11 CARDINAL `0xBFFFFFFF`, AppKit 0.75).
The platform adapter changes the root window, including its content; no image,
palette, export or widget alpha is modified. Native errors preserve the previous
accepted menu state and are reported. Wayland currently returns an explicit
unsupported-integration error. The Windows API has a hidden-window round-trip
test; X11 has an unmapped-window property test under Xvfb. These do not prove
GPU/compositor appearance or actual F10 delivery. See `preferences.md` for
startup intent and `architecture.md` for the remaining native acceptance gates.

All eight current groups and their recent/palette/roof submenus now use one
measured popup renderer. `menu_popup/specs.rs` records each implemented row's
mark, shortcut and trailing divider. Ordinary commands reserve the source's
transparent 16×16 column without importing an empty bitmap. Visible command
ellipses use the source's three periods; accessible names remain stable and
shortcuts have their own accessibility property. Recent-file names remain
literal, including Unicode ellipses; duplicate basenames use distinct indices.

Master `widgets.h` supplies one leading and nine trailing label spaces and
12/13-point platform sizes. FLTK 1.4.5, required by master `INSTALL.md`, supplies
the remaining calculations: a row is the largest measured label/image height
plus four; check/radio measurement reserves 18 points, submenus add 14; shortcut
modifier and key maxima are measured separately, with four points after keys.
Popup width is the largest item width plus both shortcut columns plus nine,
bounded below by title width plus twelve. With a one-point panel border, height
is `row_height * row_count + 1`. Dividers occupy the bottom of their preceding
row, not a separate item. The menu bar remains in its original 21-point band.

The source-derived backgrounds, square frame, row bounds, check/radio positions,
image space and shortcut alignment are implemented; theme-specific beveled
frames/checkmarks/radio rasterization, inactive-color arithmetic and native
fonts remain incomplete. Keyboard submenu opening/activation and modal guards
are tested headlessly. Activating Enter is consumed before a command can open
a modal, preventing accidental acceptance/dismissal in that same frame.
Mnemonic underlines/Alt navigation, screen-edge/native-popup behavior and the
complete Theme controls remain separate parity gates. Transparent's remaining
native verification and Wayland work are described above.

Mac Shift/Option/Command captions use an unchanged, separately licensed Noto
Sans Symbols 2 font because the existing bundled faces lack some glyphs. Its
dedicated family does not alter ordinary UI text or claim native macOS metrics.
See `assets.md` and `crates/gui/assets/fonts/NOTICE.md` for provenance.

About uses the source Help-menu order/separator and command-/ shortcut. OK,
Enter and Escape close its modal without modifying the loaded project or
history. Parent commands and root-window closing are guarded while it is open.
It remains available before loading a map. The text identifies NG's own version,
source and copyright separately from Polished Map 4.7.1/Rangi's original notice,
and retains Fugue creator, source, CC BY 3.0 and adaptation attribution.

The source `Modal_Dialog` APP_ICON layout measures at a 620-point wrap width,
adds 80 plus OS_FONT_SIZE to the larger text width, and enforces a 280-point
minimum. OS_FONT_SIZE is 12 on Windows and 13 elsewhere, with heading +4.
The heading has a 25-point slot at (70,10), body starts at (70,45), and the
80×22 OK button stays ten points from the bottom/right. The 50×50 image slot
at (10,10) is reserved but blank until the application artwork is cleared.
The native frame/system font and missing art remain explicit fidelity gaps.

Help opens a nonmodal, resizable child viewport with an initial 700×500 client
size and a 48-point offset from the parent. Its body uses `(10,10,w-20,h-52)`;
the 80×22 OK button remains ten points from the bottom/right. Reopening resets
document scrolling and position while retaining the resized native client size.
Menu Help and F1 share the same action, including before a map is loaded.
OK, Enter, Escape and the window close button dismiss Help without closing or
editing the map. Main-window controls remain available while Help is open;
existing application modals block opening/using it.

The complete pinned master text is retained as reference documentation, with
95 display blocks and 16 linked spans. Its source product/version, copyright,
platform paths and URLs are preserved. A separate NG notice avoids presenting
upstream instructions as a completed parity claim. The original HTML subset is
converted reproducibly to static blocks; links open only on explicit activation.
Native child-window placement/focus/resize/close and FLTK HTML/font equivalence
remain acceptance gates. Headless tests use an embedded nonmodal fallback.

Print now connects its CPU compositor to the options, clipboard and PNG export
workflow. `Print_Options_Dialog::refresh` uses 10-point outer
margins, 22-point controls, 4-point checkbox gaps and 80-point buttons. The Show
label width is `text_width("Show:", 4)`; each checkbox gets
`text_width(label, 2) + 22`. The client width comes from the final checkbox plus
the trailing margin, with a minimum of 288. The button row starts at y=48 and
the client height is 80. Button order is Export/Copy/Cancel on Windows and
Cancel/Copy/Export on Unix. Enter, command-C, Escape and initial Export focus
are implemented in a centered egui modal. Its heading/frame and font metrics
remain different from the original native child window.

Grid, IDs, Priority, Events and Warp IDs are independent controls, default off.
The source copies their values into preferences even when the options dialog is
canceled. Warp IDs only affect output when Events is on; the checkbox itself
stays enabled. Export opens `Print Screenshot` with `screenshot.png`, PNG filter
and overwrite confirmation. Copy sends an image without writing a file. Both
use native map dimensions regardless of zoom or current canvas visibility.
File/Print, the toolbar and command-P share this workflow. Command-Shift-P remains
Show Priority. Rendering captures live unsaved map, graphics and event data,
then runs off the UI thread. The image clipboard request is emitted on the UI
thread after completion. Export retains any existing filename suffix, adding
`.png` only when absent, and confirms a newly suffixed existing destination.
The save target is hashed and rechecked before installation. Native font metrics, clipboard payloads,
file chooser behavior and dialog frame/DPI remain targeted platform checks.

Full Screen follows the source View entry and F11 shortcut, including use before
opening a map. Both request native fullscreen through eframe. The checkmark reads
the native viewport state, so restored or OS-initiated changes are reflected.
It does not change map bytes, history, palettes or dirty flags. Existing modal
and text-input guards apply; Escape does not exit fullscreen or close the main
window. Normal geometry, maximized state and fullscreen persist together in
eframe's native window record. Actual OS transitions and restoration still need
platform verification; headless tests cover the requests and state feedback.

Drag and Drop defaults on and gates left-button internal transfers in the block
sidebar, tileset/roof editors and palette editor. Drag swaps; Ctrl-drag copies.
Palette transfers copy live RGB8 colors, while keyboard Copy keeps its separate
RGB5 quantization. Selection, clipboard commands, map/pixel painting and event
movement remain available when transfers are disabled. The option survives
Close and exit. External file dropping onto the map is independent in upstream
and has not yet been ported.

Overworld Map Size uses the original 380-point client width, text-derived content
height, 22-point controls, 10-point margins, 16-point button gap and platform button
order. It accepts 1–4096 bytes (default 1300). The save warning uses
`(width + 6) * (height + 6) > capacity`, after a successful dirty-map write or
forced Save As. Equality is permitted. Clean ordinary saves and failed writes
do not warn. Dismissing the warning keeps the completed save and history intact.
The warning's native frame/icon and font rasterization still need verification;
the port uses its existing portable warning modal without importing unaudited art.

Monochrome and 256 Tiles default off. Each change reloads the current tileset,
blocks/collisions and roof without opening the tileset chooser. Unsaved assets
use the same warning as Change Tileset; map history and session palette edits
survive. Cancel retains the changed preference, matching upstream, while the
live document retains its original interpretation and save layout until a
successful reload. Failed reads retain the whole document. Missing palette maps
enable Monochrome again. Both options survive Close and exit and seed subsequent
opens; the existing Map Options 256-tile checkbox starts from the session value.
See `deviations.md` for the canceled-mode safety boundary.

Tile Priority controls the tile editor's priority checkbox and palette-map gap/
padding output. Toggling it preserves existing attribute bits, pixels, dirty flags
and map history. It survives Close and exit; successful reads of PRIORITY_* colors
enable it again, while ordinary palettes preserve the current setting. It is
independent of View > Show Priority. Disabling the option must not silently enable
it again during Save. Show Priority defaults on and has a synchronized toolbar
toggle and Ctrl/Cmd+Shift+P shortcut. Its controls are disabled while Tile Priority
is off, but the stored display value and existing overlays are retained. It draws
before grids, IDs, selection borders and event markers. Main/sidebar tiles use
8/16-pixel patterns; block sheets and tileset thumbnails use 16, block chips 24.
Pixel-editing grids, tile-image clipboard and plain RGB export omit the overlay.
The original roof-dialog invalid cast is handled as documented in `deviations.md`.

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
- The popup image/check column, row metrics, dividers and shortcut columns are
  implemented. Complete theme painting, mnemonics and native behavior remain.
  Upstream's `BLANK_ICON` reserves an image column for ordinary commands;
  it does not decorate all commands with toolbar artwork.
- Verify source enablement for every data/profile condition. Current checks
  cover unavailable map/blockset/tileset/roof/events, history controls, existing
  workflow dialogs and modal guards; this is not the full profile matrix.
- Test every command through its final menu route as it is ported. Never add
  inert placeholders to imply that a missing source command works.
- Keep native OS behavior separate from egui headless evidence. Screenshots and
  synthetic interaction tests do not prove native font, menu, DPI or dialog
  equivalence on any platform.
