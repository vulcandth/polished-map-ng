# Architecture decisions

## ADR 1: Separate domain logic from platform adapters immediately

`polished-map-core` owns validated models, commands, history and deterministic
RGB rendering. It has no filesystem, egui or GPU dependency. `project` owns
discovery, codecs and persistence. `gui` and `cli` depend on these libraries.
Master/plusplus remain explicit format profiles; shared code must not erase
their differences. Public interfaces are experimental until objective four.

`polished-map-platform` isolates desktop color dialogs and external script opening.
Core, project, GUI and CLI retain the workspace `unsafe_code = forbid` rule.
The adapter denies unsafe generally and allows it only in documented Win32
modules. The color chooser borrows eframe's live owner handle during its modal call.
Custom color storage belongs to the adapter instance, not a mutable global.
Its two unsafe calls follow Microsoft's
[CHOOSECOLORW contract](https://learn.microsoft.com/en-us/windows/win32/api/commdlg/ns-commdlg-choosecolorw).

`ScriptOpenRequest` retains a saved file path and optional one-based source line.
The GUI queues explicit menu/shortcut/double-click requests; its native adapter
dispatches workers and reports completion/errors through channels. Headless GUI
tests capture requests and inject launch callbacks, never opening applications.
The platform adapter validates an absolute regular-file path. Windows isolates
COM lifetime, association lookup and shell execution; known-editor arguments
are quoted as data using the
[Windows argument rules](https://learn.microsoft.com/en-us/cpp/c-language/parsing-c-command-line-arguments).
The APIs follow the documented
[FindExecutableW buffer contract](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-findexecutablew)
and [ShellExecuteW return/COM contract](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew).
Linux dispatch uses the [xdg-open desktop interface](https://portland.freedesktop.org/doc/xdg-open.html);
macOS uses Apple's `open` tool. Unix arguments use `Command::arg`, never a shell
command string. These are desktop adapters only; core/project and CLI operations
remain independent of display servers and external editors.

## ADR 2: Geometry derived from FLTK

Reference main.cpp/main-window.cpp and block-window.cpp at pinned master.
Baseline client size 800×600; menu 21; toolbar 26; status 23; buttons 24×24;
block editor 466×304. Map block size is 32 (or 64 zoomed). Sidebar width is
four block columns plus theme scrollbar width. Rulers occupy scrollbar width.
Map/event margins must be taken from map-buttons.h, not inferred from screenshots.
Treat baseline units as logical points; apply DPI once and snap physical edges.
Text widths use font metrics. Keep independent rectangle/hit-test fixtures.

## ADR 3: Deterministic CPU pixel source

Rasterize domain content on CPU; nearest-neighbor GPU textures display it.
Visible-region rendering and dirty texture caches prevent full-map work per frame.
Headless PNG export uses the same pixels. Use wgpu through eframe 0.37 initially.

`Graphics::render_print(map, events, options, style, rasterize)` is the shared
Print compositor. Options are independent of current canvas visibility. The
callback receives `PrintFont` (Courier Bold 14 or Helvetica Bold 14) and text,
returning a validated `PrintTextMask` of straight coverage, including the line's
ascent/descent space. It owns typeface selection and baseline placement; core
owns block/grid/priority/event/warp composition and clipping. Each distinct
font/text pair is requested once per render. The returned `RgbImage` has native
map dimensions with no viewport margin. The font adapter must remain usable
without a display server so the eventual CLI and GUI Print paths share it.
Current differential tests use synthetic masks; native text metrics and driver
blending must be verified before claiming platform-exact Print output.

## ADR 4: Fail safely without pretending to have upstream parity

`Map::overworld_buffer_bytes` reports the WRAM requirement including master's
three-block border. `MapDocument::summary` exposes the value; CLI `inspect`
returns it at `result.map.overworld_buffer_bytes`. It is independent of the GUI
capacity preference and does not require a display. The GUI warns after a
successful map write when the requirement exceeds its saved capacity; the
warning neither rejects the save nor edits WRAM declarations.

Validate before mutation. Snapshot on-disk content for conflict detection; stage
single-file replacements before publication. Block/collision pairs and compound
tileset PNG/palette outputs stage a manifest and before/after bytes, preflight
all observed hashes and attempt rollback on
failure. These are individual replacements, not atomic publication across files.
General journal discovery/replay remains pending. Do not mark data clean
until its corresponding writes succeed. UI and CLI use identical commands.

`MasterTilesetDocument` owns the editable base tiles, contextual colors, separate
roof overlay, compound regions and source/destination observations. Rendering
and headless editing use this same composition. Save prepares all PNG/ASM bytes
on a trimmed draft before staging; source imports remain intact when output uses
the original editor's PNG convention. The GUI pixel editor clones this document
for its modal draft and validates all block previews before replacing live state.
Accepting a base draft dirties only the tileset and preserves map history. The CLI requires a preview-state hash to apply
tileset writes; library callers also supply the block definitions used for trimming
and must verify their backing files if they require on-disk conflict protection.

`Roof` wraps a nine-tile command model; `RoofDocument` owns its independent dirty
state, source/destination observations and 24×24 PNG persistence. The same GUI
pixel controls operate on either draft. Roof acceptance validates the composed
previews before commit, while base acceptance synchronizes contextual roof
attributes without marking roof pixels dirty. Roof attributes are transient;
only pixels persist. Base and roof hashes/save operations are separate. Main Save
now follows upstream's edited-domain ordering: base tileset, roof, blockset,
events, then map if needed. Failure or a canceled blockset destination stops the
sequence. This is not an atomic transaction across documents; earlier successful
saves remain published if a later save fails.

`PaletteBank` owns all six sets and atomic color commands. `PaletteDocument`
tracks the selected set, explicit import source and edited flag. Tileset
documents retain the complete contextual bank rather than discarding unselected
sets after discovery. Selection and color edits affect composed rendering without
dirtying tile pixels or map history. Color export is a separate, explicit
destination operation, not part of tileset PNG/palette-map saving. Imports overlay
the upstream layouts onto the current bank; exports quantize RGB8 to RGB5 without
changing the in-memory colors. The CLI binds an apply to its observed input and
requires an additional destination hash for overwrite. Contextual discovery does
not treat source palette files as implicit write destinations.

The GUI clones palette state for its color/monochrome draft. It validates all
composed block images before committing colors and textures. Text fields retain
their own clipboard behavior; selected swatches use the source RGB5 clipboard
snapshot and exact live swap. As upstream, edited palettes have their own
load-replacement warning and explicit export, not map history or map Save/close
dirty state. `MapPalettePlan` and the GUI's pending load preserve session colors
while resolving per-file import decisions; required-load failures retain the
previous live map. The selected-set preference is persisted; colors and transient
warnings are not.

`Events` owns typed macro metadata, validated coordinates, field-edit commands,
warp resolution and sticky edited state. `EventScript` binds immutable source
spans to those entries; only edited event bodies are regenerated. `EventDocument`
owns the script's observed file hash and independent save. The CLI exposes the
same commands with required apply hashes. The GUI's `Loaded` state owns an optional
event document. Automatic load is part of the map worker, with optional-script
failures reported after committing the map. Manual replacement prepares the new
document before replacing the old one. GUI resize clones the map/event drafts,
validates both, then commits them together in memory. A standalone CLI `translate`
command affects events only and does not imply a multi-document transaction.

## Native external-file-drop integration gate

Research on 2026-10-28 found a concrete adapter gap. Pinned master attaches
`DnD_Receiver` only to `_map_scroll`. `Workspace::handle` forwards the drop;
`Main_Window::drag_and_drop_cb` rejects another modal, asks the usual unsaved
warning and opens the first path. The internal Drag and Drop option is unrelated.

The locked winit 0.30.13 Windows `drop_handler.rs` ignores the OLE position
argument; its macOS `window_delegate.rs` forwards filenames without positions;
X11 `event_processor.rs` notes that position updates are not exposed. Its Wayland
backend does not implement the file-drop events. Egui-winit 0.37 forwards these
path-only events into `RawInput.dropped_files`; ordinary pointer state is not
evidence of the drop location. The observed problem also appears in
[egui issue 7913](https://github.com/emilk/egui/issues/7913).

Winit's [0.31 prerelease series](https://github.com/rust-windowing/winit/releases)
redesigns native data transfer and drag coordinates. As of this research,
0.31.0-beta.4 is a prerelease, not a compatible update to our locked 0.30 adapter.
Its evolving API must be inspected at a pinned revision, not inferred from old
beta migration notes or the moving master documentation.

Before implementing the native route:

1. Evaluate an eframe/egui-winit integration with that data-transfer API, including
   Wayland clipboard ownership. If no compatible release is available, evaluate
   a reproducible adapter patch with its own license/source notices and CI.
2. Preserve a transfer identity, ordered native paths, viewport identity and
   the final position for that transfer. Convert physical pixels to logical
   coordinates once, and clear state on leave/cancel. Never infer the location
   from an unrelated mouse event or accept anywhere in the window as a fallback.
3. Route a valid drop inside the source map workspace (including its blank
   margins) through the shared guarded Open flow. Ignore sidebar, rulers,
   toolbar and modal targets. Open only the first path and retain the current
   document on cancellation or failed loading. Do not decode paths lossily or
   treat pasted text/remote URLs as native file drops.
4. Verify target boundaries, multiple files, modal/dirty guards, cancellation,
   transfer ordering and DPI mathematically and with input tests. Then verify
   real file-manager drops on Windows, macOS, X11 and Wayland, including
   off-window cancellation and clipboard coexistence. Synthetic drops alone
   cannot satisfy this gate.

External file dropping remains unimplemented until this integration is in place;
the existing File/Open and headless path APIs remain available.
