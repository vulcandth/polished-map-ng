# Intentional differences

Approved: fail safely for malformed input rather than reproduce upstream crashes
or out-of-bounds access; report corruption/conflicts rather than silently discard
data. Initial theme coverage is default light and dark. OS chrome/font rasterization
may differ. All other safe observable behavior remains a parity requirement.

Implementation-specific exceptions will be recorded here with tests and rationale.

## Safety corrections already implemented

- Undo after saving marks changed data dirty. Upstream's undo/redo callbacks do
  not update the modified flag, potentially allowing edits to be lost on close.
  The normal sticky dirty flag and no-op-click history entries are retained.
  Evidence: core `undo_after_saving_must_not_lose_unsaved_changes`.
- Malformed assembly numbers fail explicitly instead of strtol silently turning
  unknown text into zero. LZ reads/back-references/output lengths are checked.
  RGB palette channels outside 0..31 and incomplete colors/palettes are rejected.
  Incomplete roof tables return a format error instead of continuing after a GUI
  warning; a recoverable warning flow remains pending.
- Binary map length mismatch is currently rejected rather than allowing an
  oversized file to be truncated on save; a faithful warning/consent flow remains
  pending. ASM `.map` imports use Save As to produce binary `.blk` output;
  the original source is preserved. Save As currently requires a `.blk` extension
  (or adds it when omitted), rather than accepting arbitrary binary-file suffixes.
- Single-file saves compare content rather than just modification timestamps,
  and stage output before replacement. Concurrent writers still have a narrow
  check/replace race; no filesystem-wide compare-and-swap guarantee is claimed.
- ASM `.CEL` blockset imports require binary `.bin`/`.bst` Save As. Upstream
  writes binary bytes over the assembly source; the port preserves that source.
  Existing binary blockset writes preserve row-major tile bytes. Paired Save As
  requires a separate collision destination instead of modifying source collisions.
- Collision writes preserve unrelated ASM, untouched rows, inline comments,
  line endings and trailing binary data. Upstream rewrites the whole collision
  file into canonical rows and drops other content. Changed field lists retain
  their original prefix/suffix; newly added rows use upstream's canonical syntax.
  Partial binary rows, malformed ASM rows, field delimiters and line breaks are
  rejected. Individual fields are limited to 4096 bytes and files to 4 MiB.
- Block/collision saves preflight both files, retain before/after bytes, and roll
  back a completed first write if the second fails. External changes prevent
  rollback rather than being overwritten; recovery data is retained. GUI Save
  still processes the map separately. General journal discovery/replay and
  process-termination tests remain required before the stable automation milestone.
- Headless discovery leaves an unmatched tileset unset instead of accepting the
  GUI's first dropdown item. Callers must supply an explicit choice; this avoids
  silently rendering or editing an unrelated tileset. Ambiguous compound name
  ordering and the original full options dropdown remain parity work.
- Headless opens start from a fresh palette bank unless the caller explicitly
  uses `begin_for_map` with a session and resolves its `MapPalettePlan`. GUI
  Open/New carries colors forward and prompts separately for default, special
  and roof imports when edited, matching upstream. Skip replaces the ambiguous
  Cancel label for these prompts: it skips that import and still opens the map.
  Palette choices and malformed-file failures are staged before publishing the
  new map, so a failed required import retains the previous map and colors.
  Optional roof-color failures warn and continue without changing those colors.
  Persistent preferences and native dialog workflows remain pending.

## Tileset editing and export safety

The new tileset export routines reject entirely undefined palette maps instead of
reproducing `Palette_Map::write_palette_map`'s backwards array underflow. Grayscale
export checks every destination row. In upstream, `Tileset::print_rgb` subtracts
two tile rows for every slot at or above $80, even when a compound output starts
there; some offsets/counts therefore address memory before or beyond the image.
Those combinations return a layout error. Valid layouts retain upstream pixel
placement, grayscale levels, row padding and palette-map serialization quirks.
Project saves use these checks before writing any compound outputs. A corrected
policy for those unsafe layouts remains separate work. Named missing companions
with an empty explicit region are also rejected rather than allowing upstream's
zero-count fallback to copy unrelated regions into a new companion file.

Palette-map writes preserve unrelated ASM, comments and line endings instead of
replacing the whole file with canonical rows. Resizing a simple table retains its
comments; interspersed labels/macros with ambiguous ownership cause a preflight
error. Tests cover both paths. All compound PNG/palette outputs are staged and
conflict-checked together, using the existing rollback/retained-backup mechanism.
PNG output preserves the original two-bit grayscale format and decoded samples;
compression bytes can differ between the Rust encoder and libpng/zlib.
Project-local names reject drive prefixes and alternate-stream colons in addition
to directory separators, preventing headless writes outside the selected names.

The GUI disables the priority checkbox for an undefined palette. Upstream ORs
the priority bit into UNDEFINED and then indexes beyond its palette-color table
(`Color::color` only recognizes the exact UNDEFINED value). Choosing a defined
palette re-enables the checkbox when the project supports priority. The existing
core validation rejects that invalid combination; a GUI test verifies the guard.

Roof image paste clips to the selected 8×8 tile and leaves uncovered pixels
unchanged. Upstream `Roof_Window::paste_tile_graphics_cb` iterates to
`max(width, 8)` and `max(height, 8)`, which can read or write beyond image/tile
bounds for dimensions other than exactly 8×8. The safe 8×8 behavior is compared
against compiled original C++; smaller and larger imports have explicit Rust
tests. The roof command model rejects palette/delete commands absent from the
original roof editor, and malformed batches leave the entire draft unchanged.

## Loaded-roof replacement safety

`Main_Window::change_roof_cb` updates the roof name before decoding its graphics.
Failed decoding can therefore leave the selected name and previous pixels out of
sync. The port validates the replacement and all block previews before committing
either. Canceling either the unsaved warning or selector preserves all edits.
Successful reloads retain upstream's sticky dirty flag, including same-name
reloads. Removing a roof clears its dirty domain instead of retaining an invisible
edited flag with no roof to save. Other document domains stay untouched.

The selector refreshes discovered roofs when opened rather than copying the last
Open Map dialog's cached list. Its fixed 280px client width is retained; unusually
long names are truncated in the closed control and available in full in its
dropdown/tooltip, instead of overflowing the client area. The selector is an
embedded modal pending the native child-window parity work.

## Loaded-tileset replacement safety

Upstream clears the live metatileset before loading its replacement, and closes
the map when loading the new palette map/graphics/blocks fails. The port prepares
the complete replacement (including collisions and the selected roof) and every
block preview first; any failure retains the original document and edits. A
missing or malformed selected roof also fails this transaction instead of
continuing with a warning and a partially loaded asset set. Successful changes
retain the original map/history and session palette semantics; switching does
not update map header constants. The selector refreshes file discovery and
constrains long closed-control labels like the roof selector. Original compound
choice ordering and the complete main menu layout remain pending.

## Palette export labels and validation

Upstream declares six palette sets but initializes the export label table with
only `morn`, `day`, `nite`, `indoor`, `custom`. The sixth pointer is null and is
passed to `%s`; the fourth/fifth names also describe the wrong sets. Export now
labels all six sets correctly, including `darkness`. Numeric RGB5 values, hue
order, whitespace and line endings follow the original writer. The compiled
oracle records its corrected label table separately from unmodified functions.
Invalid hex characters and values longer than six ASCII digits are rejected,
rather than relying on partial C conversion or GUI input filtering. RGB8 edits
remain exact in memory; copying and exporting still use upstream RGB5 quantization.

## Incomplete resize integration

The block-level resize matches upstream's nine anchors, zero fill, signed division
and history reset. The GUI now stages loaded event movement with resize. Neither upstream
nor this port rewrites map dimension constants on resize; users must update those
and any filename-encoded dimensions before reopening. The current GUI states this
limitation; the CLI reports `dimensions_changed`.

## Event assembly preservation and validation

Upstream rewrites every recognized event line with a tab and its tooltip text,
normalizing indentation/line endings even when untouched. Event saves here retain
unrelated assembly and unchanged events byte for byte, and replace only the body
of edited event lines. Original coordinate formatting, macro-specific X/Y order,
prefix/suffix data and negative hexadecimal representation are retained for those
edited lines. This avoids unrelated source churn and follows the repository's
assembly-preservation rule.

Upstream's `strtol` accepts partial coordinate expressions and treats unknown
symbols as zero. This port rejects unsupported expressions, missing coordinates,
overflow and values that wrap below -6, rather than silently relocating events.
Valid integer literals retain the original signed remainder and 32-bit narrowing.
Warp destination ID interpretation remains permissive as upstream; unresolved
IDs are reported as zero and do not resolve a local target. Event input/output
is bounded to four MiB and individual prefix/suffix fields to 64 KiB; embedded
NUL/CR and field edits that shift coordinates or turn a `warp` event into an
unrecognized script command are rejected atomically. These are safety changes,
not an assembly expression evaluator. UTF-8 scripts are currently required.

Resize translation uses wide intermediates before clamping, avoiding the original
int16 narrowing overflow for extreme headless displacements. The model exposes
this operation, and the GUI now applies it during map resize. Headless map-resize
and standalone event-translation commands remain separate operations.

## Event GUI safety and remaining visual differences

Failed manual event loads/reloads retain the previous document instead of
unloading it before parsing. A malformed auto-loaded script warns after the map
opens, with no partially parsed events. Resize validates both in-memory drafts
before replacing either; an event serialization-limit error cannot leave only
the map resized. Event save conflicts stop rather than offering a blind overwrite.
Main Save now follows the original domain ordering, but stops after a failure or
canceled destination instead of continuing to save later domains.

Event mode controls temporarily live in Tools while the complete main-menu port
is unfinished. Source rectangles/colors/alpha, cell sizing, coordinate bounds and
options geometry are implemented; event letters and zoomed ID text still use
egui's bundled fonts rather than native bold Helvetica/monospace faces. Normal
warp ID pixels now match the original digit sheet. Event
Options is an embedded modal, and auto-load/show preferences are session-only.
Event export overlays and native event GUI checks remain explicit parity gaps.

External script opening now follows the original Windows association/known-editor
line rules and Linux `xdg-open` behavior. macOS uses `/usr/bin/open`, fixing the
original unconditional Unix `/usr/bin/xdg-open` path. Requests validate a regular
file, preserve OS path encoding, and report launcher errors instead of silently
exiting successfully. Known Windows editors are matched by exact case-insensitive
basename rather than arbitrary suffix (so `evilcode.exe` is not treated as Code).
Windows uses a worker-owned COM apartment and a null owner window; shell dialogs
are not parented to the GUI. File opening runs off the GUI thread; the Unix worker
reaps the opener, which may remain running until its editor exits. No automatic
save or reload is added. Native association, editor-line navigation, and shell
dialog behavior still require platform verification.

The event cursor and Game Boy guide recompute hover from current egui geometry
and are suppressed under dialogs and popup menus. Their coordinates, zoom sizes,
fill and border formulas follow source; exact native enter/leave/callback timing
has not been compared. Hover coordinates use the global decimal/hexadecimal
display setting. The complete status-bar sections and preference persistence
remain incomplete; transient operation messages still occupy the status area
when the event cursor is absent.

Ruler geometry and hover pairing follow the pinned source. During egui's smooth
scrolling, ruler cells use the actual fractional content displacement instead of
truncating it to FLTK's integer offset, keeping ticks aligned with the map. At
integer offsets the formulas agree. Rulers use the current egui foreground,
panel and inactive-widget colors and bundled 9-point monospace font; original
native FL_DARK2/theme colors and Courier/Consolas metrics remain unfinished.
Their visibility and number-base settings are session-only until persisted
preferences are ported. The negative hexadecimal formatter omits the source's
terminal newline to match FLTK's single-line result in egui; this preserves
visible behavior rather than the intermediate formatting bytes.

Block IDs use the original 14-point logical font size, zoom-dependent offsets,
four diagonal black outline passes and white/yellow foreground colors. The
current bundled monospace face does not reproduce native Courier Bold's weight
and metrics. Hotkey badges and export ID options still need their own parity
work. Display-base changes do not alter
event formats or saved data; the status area now shows live block coordinates
in both modes, with event coordinates appended only in Events Mode.

Map/sidebar grids now use bottom/right one-point edges and three/four inset
selection rings from `map-buttons.cpp`. Explicit mesh quads preserve opaque
pixels instead of relying on egui's antialiased stroke corners. Shared editor
and event cursors use the same ring geometry. Grid colors are the upstream
Greybird light (85,85,85) and Dark (38,38,38) inactive colors on every OS; this
does not reproduce Windows' default Blue theme or the remaining native themes.
Sidebar drag feedback follows source/target/selected state, while the existing
egui drag gesture remains in-process rather than the source's native FLTK DND.

The icon port currently includes 25 reviewed image files (14 commands and four
collision-quadrant labels, including seven dark variants). Source-colored pixels,
24-point toolbar/22-point clipboard bounds
and disabled-color arithmetic are retained. Button selection/hover/focus frames
still use egui styling; full native toolbar themes remain pending. Disabled icon
colors use upstream Greybird/Dark backgrounds on all platforms, consistently with
the existing portable grid palette. The remaining toolbar items and popup-menu
layout are not yet complete; Open/Undo/Redo retain text controls. Unreviewed
artwork is not replaced with silently redesigned copies or imported under an
assumed repository-wide grant. See `docs/assets.md` for the per-asset audit.
