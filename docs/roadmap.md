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
tests cover the edit/accept/save flow. Native clipboard/child windows, priority
overlays and remaining specialist editors still prevent a tileset parity gate.
The roof draft editor now shares pixel/clipboard commands, preserves the original
226×228 client geometry and reversed tile rows, and has independent PNG saves.
The headless roof interface previews and requires an observed hash to apply.
Compiled upstream comparisons cover safe 8×8 roof image paste; temporary real
New Bark Town roof edits save/reopen with other pixels and source fixtures intact.
Loaded-roof switching/removal now preserves other editing domains, warns before
discarding roof edits and retains the previous overlay on failed reads. Its
280×80 client layout and platform button order have headless checks and snapshots;
all five pinned pokecrystal roof variants match fresh loads after switching.
Native child windows/clipboard, cross-map palette session behavior and other
matrix gaps still remain.
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
