# Architecture decisions

## ADR 1: Separate domain logic from platform adapters immediately

`polished-map-core` owns validated models, commands, history and deterministic
RGB rendering. It has no filesystem, egui or GPU dependency. `project` owns
discovery, codecs and persistence. `gui` and `cli` depend on these libraries.
Master/plusplus remain explicit format profiles; shared code must not erase
their differences. Public interfaces are experimental until objective four.

`polished-map-platform` is a small Windows-only FFI adapter for `ChooseColorW`.
Core, project, GUI and CLI retain the workspace `unsafe_code = forbid` rule.
The adapter denies unsafe generally and allows it only in its documented Win32
module, borrowing eframe's live owner handle during a synchronous modal call.
Custom color storage belongs to the adapter instance, not a mutable global.
The two unsafe calls follow Microsoft's
[CHOOSECOLORW contract](https://learn.microsoft.com/en-us/windows/win32/api/commdlg/ns-commdlg-choosecolorw).

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

## ADR 4: Fail safely without pretending to have upstream parity

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
previous live map. Persistent preference handling remains pending.

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
