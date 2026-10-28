# Architecture decisions

## ADR 1: Separate domain logic from platform adapters immediately

`polished-map-core` owns validated models, commands, history and deterministic
RGB rendering. It has no filesystem, egui or GPU dependency. `project` owns
discovery, codecs and persistence. `gui` and `cli` depend on these libraries.
Master/plusplus remain explicit format profiles; shared code must not erase
their differences. Public interfaces are experimental until objective four.

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
the original editor's PNG convention. The GUI pixel editor has not yet adopted
this document's editing lifecycle. The CLI requires a preview-state hash to apply
tileset writes; library callers also supply the block definitions used for trimming
and must verify their backing files if they require on-disk conflict protection.
