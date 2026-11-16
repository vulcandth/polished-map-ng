# Contributing to Polished Map NG

Work on `main`, which contains the unified editor and the separate parity
launchers. Start with [build instructions](INSTALL.md), then choose work from
the [roadmap](docs/roadmap.md). Read [AGENTS.md](AGENTS.md) for the project's
compatibility, documentation and licensing requirements; they apply to human
and automated contributions alike. The [documentation index](docs/README.md)
separates user help, design decisions and verification evidence.

## Finding the code

**Know an original C++ filename? Start with the [complete upstream filename index](docs/upstream-source-index.md).**
It has one clickable row per original source/header file, including
[`gameboy-screen.cpp`](docs/upstream-source-index.md#example-gameboy-screencpp).

| Directory | Responsibility | Start here |
|---|---|---|
| `crates/core` | Shared headless models, edits, codecs, discovery, persistence, fonts and CPU rendering | [Library](crates/core/src/lib.rs) |
| `crates/gui` | egui presentation and editor workflow orchestration | [Public API](crates/gui/src/lib.rs), [session state](crates/gui/src/main_window.rs) |
| `crates/cli` | Experimental headless commands and JSON output | [Arguments](crates/cli/src/args.rs), [execution](crates/cli/src/commands.rs), [JSON records](crates/cli/src/output.rs) |
| `crates/platform` | OS integration and narrowly isolated native interop | [Library](crates/platform/src/lib.rs) |
| `tools` | Asset/provenance checks, packaging and independent reference probes | [Tool guide](tools/README.md) |
| `vendor` | Five patched native dependencies, with reproducible patch records | [Vendor notice](vendor/NOTICE.md), [dependency audit](docs/dependencies.md) |

Both frontends use the same core operations. Model methods stay independent of
I/O; domain document children own explicit file access and guarded writes. Keep
egui, native windows and GPU dependencies outside core. The [architecture](docs/architecture.md)
explains the boundaries and why the native dependencies are patched. Scratch
outputs belong in ignored `.scratch/`; immutable reference checkouts belong in
ignored `.reference/`. Neither is product source or a release input.

### Where to make a change

The implementation uses **classic** for per-tile palettes and **attributed** for
per-occurrence palette/bank/flip/priority. These are file-storage contracts used
by the same editor. `attributed` replaces the former `plusplus` implementation
directories; old Rust imports remain compatibility aliases. Historical names
remain in launcher identities, serialized values and upstream reference tests.

| Change | Start here |
|---|---|
| Map painting, resize, undo | [Core map commands](crates/core/src/map.rs), [GUI canvas](crates/gui/src/main_window/canvas.rs) |
| Attribute bytes, banks or flips | [Attribute models](crates/core/src/attributed/metatileset.rs), [paired persistence](crates/core/src/attributed/metatileset/document.rs) |
| Detect/Open/Save a project | [Detection](crates/core/src/format.rs), [Open](crates/gui/src/main_window/map_open.rs), [loading](crates/gui/src/main_window/map_loading.rs), [Save](crates/gui/src/main_window/map_save.rs) |
| File limits, conflicts or save errors | [Bounded reads and single-file writes](crates/core/src/file_io.rs), [paired transactions](crates/core/src/transaction.rs), [errors](crates/core/src/error.rs) |
| Map file lifecycle or metadata | [Map document](crates/core/src/map/document.rs), [discovery](crates/core/src/config.rs) |
| Event fields or markers | [Event dialog](crates/gui/src/main_window/event_workflows/dialog.rs), [canvas](crates/gui/src/main_window/event_workflows/canvas.rs); [source-preserving edits](crates/core/src/map_events.rs) |
| Tile/roof pixel editing | [Draft and save lifecycle](crates/gui/src/main_window/tileset_window.rs), [dialog](crates/gui/src/main_window/tileset_window/dialog.rs), [clipboard](crates/gui/src/main_window/tileset_window/clipboard.rs) |
| Palette controls or numeric inputs | [Palette form](crates/gui/src/main_window/palette_window.rs), [reusable widgets](crates/gui/src/main_window/widgets.rs) |
| Menu commands, layout or startup | [Menu specs](crates/gui/src/main_window/menu_popup/specs.rs), [geometry](crates/gui/src/geometry.rs), [frame](crates/gui/src/main_window/frame.rs), [startup](crates/gui/src/main_window/startup.rs) |
| Preferences, native windows | [Preferences](crates/gui/src/main_window/preferences.rs), [native startup](crates/gui/src/main_window/preferences/native_startup.rs), [dialog host](crates/gui/src/main_window/dialog_host.rs) |

Follow an operation from GUI/CLI orchestration to a project document, then to its
core command. Keep validation in the backend; the UI owns drafts and user consent.
Tests for a feature live beside it or in the corresponding crate's `tests/`.
Files named `plusplus_reference` deliberately identify an upstream test oracle.

### Coming from the C++ source tree

The [exact-filename source index](docs/upstream-source-index.md) covers all 64
master `src/` files and the two plusplus-only additions. It links each old file
to its actual Rust implementation, split owners or explicit lack of a standalone
counterpart. For example, `gameboy-screen.cpp` maps to the real
[gameboy_screen.rs](crates/gui/src/gameboy_screen.rs) component.

Keep Cargo's four crates and underscore spellings for matching upstream names.
Do not flatten model, I/O and UI ownership into the old C++ `src/` directory.
Public `Master*` names and `plusplus` compatibility aliases remain unchanged.
The former project crate is merged into core; use `polished_map_core` for both
models and document APIs. `map`, `metatileset`, `tileset` and `roof` own focused
`document` children, while `colors` groups models, sources and palette documents.
Fonts also belong to core. No parallel project module tree or compatibility crate
is retained. Platform remains separate from GUI.
The source tree now has real matching modules for 32 of 35 upstream stems.
The [coverage ledger and exceptions](docs/upstream-source-index.md#physical-filename-coverage)
explain each. `main_window/` owns the entire GUI feature tree; `roof_window`
owns roof lifecycle while `tileset_window` shares pixel controls. Existing
`*_editor_tests` names and snapshot IDs remain; fully qualified GUI test names
now begin with `main_window::`.

Upstream `res/` maps to [GUI assets](crates/gui/assets), whose paths are consumed
by packaging; `doc/` and Help content map to [documentation](docs/README.md) and
[Help](crates/gui/src/main_window/help_window.rs). Cargo, platform adapters and
[build tooling](tools/README.md) replace C++ `include/`, `ide/` and `Makefile`
responsibilities. Upstream `example/` and external game projects remain reference
inputs, not release assets.

### Rust API documentation

Run `cargo doc --workspace --no-deps --locked --open` for public APIs. Add
`--document-private-items` when navigating editor internals. Crate landing pages
link the main concepts; examples run as doctests in the workspace test suite.
CI treats rustdoc warnings as errors for both public and private documentation.

Document new public items with purpose, units/invariants, meaningful examples and
`# Errors`/`# Panics`/`# Safety` where applicable. Prefer checked intra-doc links.
All three library crates deny `missing_docs` for public APIs. This includes fields
and enum variants; integration-test shims are not library API. Examples cover
map edits/saves, source-preserving event edits, palettes and bank addressing.
Documentation coverage does not make the experimental API stable. Follow the
[Rust API documentation guidelines](https://rust-lang.github.io/api-guidelines/documentation.html).

## Making a change

1. Read the relevant pinned upstream behavior and existing regression tests.
   [upstream.lock.toml](upstream.lock.toml) identifies source and project fixtures.
2. Implement the change in the owning module. Keep public API invariants checked,
   return errors for malformed input, and preserve unrelated project bytes.
3. Run the [required checks](INSTALL.md#development-checks). Unit tests live
   beside their implementation; integration tests are in each crate's `tests/`.
   GUI interaction tests use egui_kittest. Optional reference, real-project and
   visual tests have their exact prerequisites and commands in
   [CI](.github/workflows/ci.yml). Inspect snapshot differences before accepting
   a new baseline. Desktop acceptance is separate from automated checks.
4. Update the owning document and roadmap evidence. Keep progress notes and
   implementation details out of the user-facing README. Describe the concrete
   behavior change, validation and remaining limits in your commit or PR.

Preserve upstream attribution, the LGPL code license and separate asset/dependency
terms. Check [asset provenance](docs/assets.md) before adding artwork and
[release requirements](docs/releases.md) before distributing binaries. Do not
commit game fixtures, secrets, build outputs or personal settings.
