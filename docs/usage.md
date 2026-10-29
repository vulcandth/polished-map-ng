# Editing guide

This guide describes the current master-oriented development build. Master
parity is still in progress; plusplus support has not started. Use project copies
while acceptance testing is incomplete. See the [roadmap](roadmap.md) for the
remaining work and [compatibility evidence](compatibility.md) for verification.

## Opening projects

The editor opens binary maps using project metadata, explicit dimensions,
or names such as `Map.20x18.johto.blk`. It finds the nearest Makefile and recognizes
master map constants/headers and numbered tileset aliases. The GUI prefills these
options for review. Shared block files still require explicit choices. The project
adapter supports master tilesets with PNG/2bpp/LZ/CHR graphics, binary
metatiles, tilepal mappings, all six palette sets, project/special palettes and
map-group roof colors, compound graphics and optional roof tile replacement.

Open/New options accept an optional roof name from `gfx/tilesets/roofs`, without
the extension. Leave it empty to use the base tileset. Compound names such as
`main.before,after` retain the full main filename and load optional companion
graphics, with upstream's 16-tile row padding. Tileset pixel editing and saving
use the shared backend in both the GUI and CLI.

## Map editing

Tools → Resize Map supports all nine anchors. Resizing fills new cells with
block zero and clears undo history,
matching upstream. Loaded events move by twice the resize anchor's block offset,
clamped to the original coordinate bounds. Dimension constants and filename
dimensions must still be updated separately.

Mouse controls implemented: paint/drag, right-click select, Shift fill, Ctrl
replace, Alt swap. One drag is one history entry. Ctrl/Cmd+O/S/Z/Y opens, saves,
undoes and redoes. Ctrl/Cmd+N creates a new map; Ctrl/Cmd+Shift+S opens Save As.
File → Print offers Grid, IDs, Priority, Events and Warp IDs independently of
the editor view settings, with Copy and PNG Export actions. Native clipboard,
font and chooser behavior still require platform acceptance testing.

## Tileset and roof editing

Tools → Edit Tileset (Ctrl/Cmd+T) opens a draft. Click/drag to paint; Shift fills,
Ctrl replaces a hue, Alt swaps hues, and right-click picks a hue. Keys 1–4 choose
white/light/dark/black; Alt+1–8 chooses a palette. Ctrl/Cmd+C/V/X copies a tile
snapshot, pastes it, or swaps with its live source. Delete clears a tile. Drag
tiles to swap; Ctrl-drag copies. The image buttons and Ctrl/Cmd+Shift+C/V use the
system image clipboard, with grayscale export and bounded RGB import. OK commits
the draft and marks the tileset dirty even if unchanged; Cancel discards it.
Tile edits have no map undo entries. File → Save Tileset saves PNG/palette files;
Save also processes the map and blockset. Native clipboard transport and child
window behavior remain unverified; the current dialog is embedded.

With a roof loaded, Tools → Edit Roof (Ctrl/Cmd+F) opens its nine-tile draft.
It shares pixel gestures and clipboard commands, with the original reversed
display rows and no palette/delete controls. File → Save Roof writes only its
24×24 two-bit grayscale PNG. Main Save processes it after the tileset and before
the blockset, events and map.
Roof dirty state is independent of base graphics. Accepting a roof draft does
not add map undo entries. Image paste affects only the selected 8×8 tile;
larger images are clipped and smaller images retain uncovered pixels.

Tools → Change Roof (Ctrl/Cmd+Shift+H) switches the loaded overlay or removes it
with `(none)`. Unsaved roof changes require confirmation; canceling either dialog
preserves them. Map, blockset, base tileset and palette edits stay intact. Selecting
the current roof reloads it from disk; a failed load retains the previous roof and
preview. Successful replacement preserves the original sticky roof dirty flag;
removing the roof clears that domain. Switching alone writes no files.

Tools → Change Tileset (Ctrl/Cmd+H) reloads graphics, block definitions, collisions
and the current roof. Numbered graphics retain their actual filenames while the
selector displays names such as `01: JOHTO`. Unsaved blockset/tileset/roof edits
require confirmation. Map bytes/history, session palette colors and their edited
state are preserved; palette files are not automatically reloaded by this action.
Choosing the same tileset reloads it. A smaller blockset leaves map IDs intact,
resets an out-of-range selected block to zero, and clears an out-of-range copied
block. Failed replacements preserve the previous assets and preview. Changing
tilesets does not rewrite map header constants or write any files.

## Block editing

Right-click and release on the same sidebar block to edit its 4×4 tile
definition. Left-click paints and right-click picks a tile. Shift/Ctrl/Alt select
the original eight placement patterns. OK applies the draft; Cancel discards it.
Block edits have separate dirty state and no map undo, matching upstream.
File → Save Blockset writes definitions and loaded collisions; Save also writes
the map. Named collision fields or binary hex spinners follow the project format.
Drag a sidebar block onto another to swap them; Ctrl-drag copies instead.
Edit → Copy Block stores a snapshot; Paste Block pastes that snapshot, while
Swap Block swaps with the copied source's current contents. Ctrl/Cmd+C/V/X
invoke these internal block operations; they do not replace the system clipboard.
Tools → Resize Blockset (Ctrl/Cmd+B) accepts 1–256 blocks. New definitions/collisions
are empty, and shrinking does not rewrite map IDs. These operations have no
map undo entries. Native clipboard shortcut verification remains pending.

## Palette colors

Tools → Edit Current Palettes (Ctrl/Cmd+Shift+L) opens the color or monochrome
draft. Select a swatch, use the 0–31 RGB controls, or enter hex and press Enter.
Ctrl/Cmd+C/V/X copy, paste, and swap swatches when a text field is not focused.
OK accepts; Cancel discards. View → Palettes selects any of the six sets.
Data → Load Palettes (Ctrl/Cmd+L) prompts before replacing edited colors;
Data → Export Palettes writes them independently. As upstream, palette edits alone
do not mark the map dirty or trigger its close warning: export them explicitly.
Windows' preview swatch opens the native color chooser. Linux and macOS have
the same RGB/hex editing controls; a shared egui graphical picker is planned
for the improvement phase after parity.

Opening or creating a map carries the current palette session forward, including
colors edited before any map was loaded. Default and special palette files only
replace the sets they contain. Each automatic import prompts while colors are
edited: **Skip** (or Escape) keeps those colors and continues opening the map.
Metadata selects a set after these imports; an active Custom set stays selected.
Roof colors have their own prompt and retain the palette edited flag. As in
upstream, accepting a missing default palette file clears that flag without
resetting colors. Missing/malformed optional roof colors warn and keep the map
open. A malformed accepted default/special import preserves the previous map.

## Events and display

The GUI automatically loads a map's discovered event script, falling back to an
adjacent `.asm` file. New maps do not auto-load events. Data → Load/Save/Reload/
Unload Event Script controls the independent script document; failed reloads
retain the previous edits. Mode → Events (Ctrl/Cmd+Shift+E) enables dragging
and right-click Event Options; Ctrl/Cmd+Shift+B returns to Blocks Mode and Tab
switches modes when no dialog or text control has focus. Shift-click
a warp to open its destination, with the normal unsaved-change prompt. Event
Options retains macro-specific prefix/suffix fields, source lines, warp IDs and
decimal/hex coordinate controls. Cancel discards only the draft.

View → Show Events and Show Warp IDs control overlays; Events Mode always shows
events. Options → Auto-Load Events controls subsequent opens and is persisted.
Ctrl/Cmd+Shift+A saves events, Ctrl/Cmd+R reloads, and Ctrl/Cmd+Shift+W unloads.
Ctrl/Cmd+Shift+3 toggles warp IDs. Data → View Event Script (Ctrl/Cmd+U) opens the
saved script in an external editor; double-click an event to request its source
line. Windows supports line navigation with associated Notepad2/3, Notepad++,
VS Code and Sublime Text, falling back to the file's `edit` association. Linux
uses `xdg-open`; macOS uses `open`. Those default application openers do not
receive a line number. Opening a script never saves pending event edits; save
first if the external editor should see them. Reload explicitly after editing
externally. Launcher failures are reported without discarding the current draft.
In Events Mode, hovering a block shows its 16px event cell and coordinates.
View → Game Boy Screen (Ctrl/Cmd+Shift+M) toggles a translucent 160×144 guide around
that cell, doubling with zoom. It works without a loaded event script and stays
clipped to the map viewport. Hovering markers uses the original underlying-block
rules; the guide disappears over margins without an underlying block and while
dialogs are active. This option is persisted with the other view preferences.
View → Rulers (Ctrl/Cmd+Shift+R) adds horizontal and vertical event-coordinate
rulers. Their cells track canvas scrolling and zoom, highlighting both halves
of a block in Blocks Mode and the hovered half in Events Mode. Labels include
the map margin and stop at the source coordinate limit; ticks span the viewport.
View → Hexadecimal (Ctrl/Cmd+Shift+4, or the Hex toolbar button) switches ruler
and hover coordinates between decimal and hexadecimal. View → Block IDs
(Ctrl/Cmd+I, or IDs) shows IDs on the map and sidebar in the selected base.
These are display settings: event labels/options keep the number format read
from their script, and toggling the display does not edit or save any data.
View preferences are persisted; Print has independent ID and overlay options.
Native font/theme matching remains an acceptance gate.
Grid (Ctrl/Cmd+G) draws the original bottom/right edges in the map and sidebar.
Selections use black/white inset borders, including map hover and sidebar drag
source/target feedback. The light/dark grid colors follow upstream Greybird/Dark;
full native theme matching remains pending.

## Saving and recovery

Main Save processes edited tileset, roof, blockset and event domains before saving
the map if needed. Saves are sequential, not one atomic transaction. Event-only
changes do not rewrite a clean map.

Saving uses upstream's PNG destinations, including separate compound outputs,
and writes two-bit grayscale PNGs. Imported 2bpp/CHR/LZ files remain intact;
subsequent loads prefer the new PNG. Roof overlays remain separate. Palette-map
ASM retains unrelated content and comments; ambiguous structural rewrites fail
before any file is changed. Monochrome mode saves pixels only, so palette changes
in that mode do not persist. Unsafe upstream compound export layouts are rejected.

Block/collision and tileset saves stage before/after bytes and a manifest in a temporary
`.polished-map-recovery-*` directory beside the first output. Normal completion or
successful rollback removes it. Interrupted saves or failed rollback can leave
it for inspection and manual recovery; preserve it until recovery is resolved.
Automatic journal discovery/replay is still pending. Unrelated collision ASM
lines and comments are preserved.

For settings and defaults, see [preferences](preferences.md); for command
placement, see the [menu inventory](menus.md). Headless workflows are documented
in the [CLI guide](cli.md).
