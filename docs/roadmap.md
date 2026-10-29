# Delivery roadmap

The accepted plan has six objectives. Every objective remains required;
foundation work is not a substitute for a complete port.

## Foundation (in progress)

- Repository, provenance, LGPL/GPL texts, AGENTS.md, workspace and lockfile.
- GUI-independent models and commands, project I/O, deterministic CPU rendering.
- Open/render/edit/save vertical slice with headless tests and derived geometry.
- Windows/Linux/macOS CI, MSRV, formatting, Clippy, rustdoc and license checks.
- Early eframe child-window, dialog, clipboard and DPI feasibility checks.

## 1. Master parity (not complete)

Inventory every menu, dialog, input gesture, preference and format. Port discovery,
map/block/graphics/palette/collision/assembly codecs, painting/fill/swap/resize,
history, block/tileset/roof/palette editors, event editing and warp navigation,
overlays, export/clipboard, external-change handling and unsaved dialogs.
Accept only when all matrix entries pass or have an approved exception and
outputs reopen correctly in upstream. Include pinned compatible pokecrystal.

File Close, dedicated Save Map and Exit now share source-derived dirty-domain
guards and preserve the source distinction between document changes and palette
session edits. Headless tests cover state clearing/retention, all dirty domains,
map-only writes/conflicts and Close/Exit/event-unload shortcut routing. Native
warning geometry and remaining preferences remain pending.

Open Recent now has ten entries, Alt+1 through Alt+0 and Clear Recent. Successful
opens and Save As promote the source path spelling; canceled/failed operations
leave history intact. Headless checks exercise the submenu, all shortcuts,
unsaved-warning and Map Options cancellation, eviction/duplicates, native path
serialization and persistence across app reconstruction. Native popup navigation
and actual shutdown/storage delivery remain separate platform checks.

Manual Load Roof Colors and all five Roof Palettes choices now use a shared
headless import API. Tests cover consent, dirty-state preservation, failure
atomicity, live preview updates and the layout used by subsequent map opens.
The compiled original overlay function matches 120 complete-bank cases; manual
and automatic imports also agree on the pinned real project. Native dialog
fidelity and native restart verification remain outstanding.

Twenty implemented view/mode/palette/auto-load/resize settings now restore and
save through eframe with a versioned record and stable app ID. The two palette
auto-load menu options affect subsequent imports independently. Headless restart
and malformed-record tests pass; legacy preference import, remaining options,
native storage/window placement and restart behavior still require work.

Monochrome and 256 Tiles now have persistent Options controls and direct current-
tileset reloads with unsaved-asset guards. The shared backend accepts an explicit
tile interpretation; map history, session colors and roof identity survive.
Tests cover all 256 source-to-slot mappings in four modes, cancel/failure
atomicity, missing-palette fallback and real Johto mode transitions. Canceled
switches retain the preference but cannot reinterpret the loaded save layout.

Tile Priority now controls priority editing and palette output independently of
attribute bits. The GUI retains it across maps and Close, and priority palette
imports automatically enable it. The backend exposes an explicit setter and
includes the option in preflight fingerprints. Pixel-only saves also update
owned palette-bank gap sizes when required by the option. Show Priority now has
8/16/24-pixel overlays, menu/toolbar/shortcut controls and persistence. All 896
original RGBA pixels are independently verified; native compositing comparisons
remain outstanding. Roof thumbnails avoid an invalid upstream window cast.

Implemented slices include New Map, Save As, block resize, map editing/saving, master metadata discovery, six
palette sets, project/roof colors, compound graphics and roof tile replacement.
Block editing now has shared placement/collision commands, a GUI draft editor,
ASM/binary collision fields and paired persistence with rollback/backups.
Clipboard copy/paste/swap, sidebar drag/copy and block-count controls are also
implemented. Native shortcut dispatch, remaining block controls/hotkeys and
automatic recovery remain.
The shared tileset draft/command model and palette-map/grayscale export routines
are implemented with C++ pixel-operation comparisons. Project persistence now
stages compound two-bit PNG/palette outputs with conflict checks, and the CLI
supports preview and hash-checked apply. Save/reopen is tested on a temporary
copy of the pinned Johto tileset. The GUI now has a source-sized tileset draft
dialog with pixel gestures, palette/priority controls, private/image clipboard
operations, drag copy/swap, deletion, preview refresh and saving. Headless GUI
tests cover the edit/accept/save flow. Native clipboard/child windows and remaining
specialist editors still prevent a tileset parity gate.
The roof draft editor now shares pixel/clipboard commands, preserves the original
226×228 client geometry and reversed tile rows, and has independent PNG saves.
The headless roof interface previews and requires an observed hash to apply.
Compiled upstream comparisons cover safe 8×8 roof image paste; temporary real
New Bark Town roof edits save/reopen with other pixels and source fixtures intact.
Loaded-roof switching/removal now preserves other editing domains, warns before
discarding roof edits and retains the previous overlay on failed reads. Its
280×80 client layout and platform button order have headless checks and snapshots;
all five pinned pokecrystal roof variants match fresh loads after switching.
Native child windows/clipboard and other matrix gaps still remain.
Changing the loaded tileset now reloads graphics, blocks, collisions and the
current roof while retaining map history and session palettes. Numbered file
labels, dirty guards, selection/clipboard bounds and same-name reload are tested.
The 280×80 chooser has inspected snapshots; a pinned Johto/Kanto/house transition
checks real assets. Complete discovery ordering, main menu/toolbars and native
child-window behavior still prevent a parity gate.
Palette-color commands, all-six-set contextual state, explicit RGB5 export and
the headless palette interface are implemented. Compiled original clipboard and
export functions verify copy quantization, live swap and export layout; export
set names use a documented correction to upstream's incomplete label table.
The source-sized color/monochrome forms, channel/hex controls, swatch clipboard,
six-set selector, load/export actions and Windows native chooser are implemented.
Headless interaction tests and four snapshots cover the forms; a focused Windows
native check verifies shortcut dispatch and chooser cancel/accept. Native file
dialog workflows and Linux/macOS runtime verification remain pending.
GUI Open/New now retains the palette session, applies staged per-file import
decisions and contextual set selection, and handles optional roof-color warnings.
Headless loads stay independent unless a caller explicitly supplies a session.
Synthetic interaction tests and a pinned New Bark Town → Violet City transition
cover carry-over; persistent preferences and remaining native workflows are pending.
Event models, legacy macro parsing, warp destination resolution, source-preserving
script saves and hash-checked CLI preview/apply are implemented. The compiled
original matches 34,188 coordinate/formatting/warp cases; 388 pinned pokecrystal
scripts with 3,684 events round-trip and edited temporary copies reopen. The GUI
now auto-loads optional scripts, displays source-derived colored markers, edits
options/coordinates, drags events, navigates warps and protects reload/unload.
Event saves join the original Save ordering; map resize stages both domains.
Source-sized forms and zoomed overlays have inspected snapshots. Exact marker
native typography, export overlays, persistent
preferences, remaining gestures and native workflows still prevent event parity.
External script opening, double-click line requests, Tab mode switching and the
warp-ID shortcut now have headless dispatch/focus coverage. The Windows adapter
retains the original known-editor line arguments; Linux/macOS use their desktop
file openers. Actual editor/association behavior still needs native checks.
Event-mode half-block hover, decimal coordinates and the Game Boy screen guide
now use source-derived geometry, colors and clipping, with tests for both zooms,
scrolling, overlapping markers and the original negative-coordinate boundary.
Rulers now use source-derived insets, tick/label geometry, coordinate limits and
mode-dependent highlights; headless tests cover scrolling, both zooms, menus,
shortcuts and modal guards. Global decimal/hexadecimal display and Block IDs now
have View, toolbar and keyboard controls, with source-format hover text and
outlined map/sidebar labels. Event labels and dialogs retain each script's own
number format. The complete status-bar layout, export overlays, native theme/font
matching and preference persistence remain pending;
current ruler/guide tests are headless rather than native FLTK comparisons.
Map/sidebar grid edges, hover/selection rings and sidebar drag feedback now
follow `map-buttons.cpp`, sharing the ring renderer with event and editor cursors.
Rendered pixel tests cover both map zooms, event/tile/chip sizes and 1×/2× scale;
native DPI/drag dispatch and complete native themes still need verification.
Zoom now scales map offsets with source integer rounding, resets the sidebar,
and preserves far-edge blank space until scrolling back. Headless controls and
edge tests pass; initial positioning and native pan/scroll checks remain pending.
The menu/toolbar port is in progress within master parity, before plusplus.
Twenty-six reviewed image files now provide toolbar icons, tile/roof clipboard
icons and collision-quadrant labels, including dark variants, source-sized controls and integer disabled-color
conversion. CC BY 3.0 Fugue attribution, LGPL adaptation notices, source/pixel
hashes and reproducible asset verification are included. Continue the per-asset
audit in `docs/assets.md`, full toolbar membership, popup-menu
structure, and native frame/theme matching. Open/Undo/Redo still use text;
unreviewed or game-derived imagery remains excluded pending resolution.
The toolbar palette selector now shares the View menu's six sets and palette
operation, with source-derived text widths, padding and control heights. Tests
cover live recoloring, menu synchronization, existing palette edits, map history,
keyboard activation/cancellation, modal guards and use before loading a map.
The eight source top-level menu groups are restored, including Data, Mode and
Options. Existing commands use their source groups; Undo/Redo menu actions now
share map history, and mode choices share shortcut transitions. The remaining
commands, preferences, popup geometry and native behavior are tracked in
`docs/menus.md`; restored grouping does not complete menu parity.
Normal-size warp ID glyphs now match every original bitmap pixel. A compiled
source harness verifies 8,192 decimal/hex placement and zoomed text cases, including
the original overflow/truncation differences. Source-derived outline colors,
diagonal offsets and zoomed label positioning are implemented; native font
selection and rasterization remain incomplete. Asset provenance is recorded in
`docs/assets.md` with LGPL attribution retained for the decoded row masks.
The pinned pokecrystal integration now
covers all 254 top-level block files (18 need explicit shared-map options).
See the compatibility matrix for the substantial remaining work.

## 2. Plusplus parity (not started)

Add the expanded tile profile, banks, per-block attributes, flips, priority,
palette rules, discovery and GUI differences. Preserve byte-sized map IDs.
Keep master tests green; validate pinned compatible Polished Crystal.

## 3. Unified editor (not started)

One binary with evidence-based profile detection, explicit override, ambiguity
handling, and capability-aware controls. Never silently convert projects.

## 4. Autonomous interface (not complete)

Stabilize Rust APIs plus JSON CLI: inspect, validate, render, edit and batch.
Versioned schemas, stable errors, deterministic responses, dry-run, explicit
apply, content-hash conflict checks, recoverable multi-file commits and examples.
No display/GPU required; MCP is deferred.

## 5. Improvement and refactoring (not started)

Retain the compatibility suite while consolidating duplicated logic and optimizing
measured bottlenecks. Separate proposals for format preservation, more undo,
remaining themes, newer project conventions and usability changes.

Plan a shared egui RGB/HSV graphical color picker for Linux, macOS and Windows,
retaining exact RGB8 edits and the existing RGB5 controls/export rules. The first
parity version follows upstream: native ChooseColor on Windows, RGB/hex editing
on every OS. The shared picker is an intentional usability addition after parity;
separate GTK/macOS native picker integrations are not planned unless a concrete
platform need justifies them.

## 6. Platform delivery (not complete)

CI starts in foundation; completion requires native verification and packages:
Windows x86-64 MSVC ZIP, Linux x86-64 tarball (X11 and Wayland), macOS Intel and
Apple Silicon app bundles. Bundle CLI, notices, checksums and matching source.
Signing/notarization is deferred until credentials are available.

## Accepted defaults

- Pinned branch tips in upstream.lock.toml define behavior; use compatible real
  project revisions instead of changing the target to current unsupported formats.
- Default light and dark themes first; remaining themes deferred.
- Match safe valid-input behavior, including serialization and undo limitations.
- Fix crashes/corruption immediately and document deviations.
- Geometry and workflow fidelity; native chrome/font rasterization may differ.
- Rust + JSON CLI; no MCP, web/mobile, or implicit format conversion.

## Evidence required

Differential C++/Rust fixtures; deterministic bytes and decoded image pixels;
malformed-input/property tests; conflict and failed-write tests; DPI geometry;
egui_kittest interaction/snapshots; native smoke tests; reproducible benchmarks.
No milestone is complete on compilation or a narrow synthetic test alone.
