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
palette sets and project/roof colors. The pinned pokecrystal integration now
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
