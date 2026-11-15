# Contributing to Polished Map NG

Work on `main`, which contains the unified editor and the separate parity
launchers. Start with [build instructions](INSTALL.md), then choose work from
the [roadmap](docs/roadmap.md). Read [AGENTS.md](AGENTS.md) for the project's
compatibility, documentation and licensing requirements; they apply to human
and automated contributions alike. The [documentation index](docs/README.md)
separates user help, design decisions and verification evidence.

## Finding the code

| Directory | Responsibility | Start here |
|---|---|---|
| `crates/core` | Validated models, edit commands, history, CPU rendering | [Library](crates/core/src/lib.rs) |
| `crates/project` | Project discovery, codecs, bounded file I/O, safe persistence | [Library](crates/project/src/lib.rs) |
| `crates/gui` | egui presentation and editor workflow orchestration | [Public API](crates/gui/src/lib.rs), [session state](crates/gui/src/editor.rs) |
| `crates/cli` | Experimental headless commands and JSON output | [Arguments](crates/cli/src/args.rs), [execution](crates/cli/src/commands.rs), [JSON records](crates/cli/src/output.rs) |
| `crates/platform` | OS integration and narrowly isolated native interop | [Library](crates/platform/src/lib.rs) |
| `tools` | Asset/provenance checks, packaging and independent reference probes | [Tool guide](tools/README.md) |
| `vendor` | Five patched native dependencies, with reproducible patch records | [Vendor notice](vendor/NOTICE.md), [dependency audit](docs/dependencies.md) |

Do not move filesystem or egui dependencies into core. Both frontends should
use the same project/core operations. The [architecture](docs/architecture.md)
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
| Map painting, resize, undo | [Core map commands](crates/core/src/map.rs), [GUI canvas](crates/gui/src/editor/canvas.rs) |
| Attribute bytes, banks or flips | [Attribute models](crates/core/src/attributed.rs), [paired persistence](crates/project/src/attributed/blockset.rs) |
| Detect/Open/Save a project | [Detection](crates/project/src/format.rs), [Open](crates/gui/src/editor/map_open.rs), [loading](crates/gui/src/editor/map_loading.rs), [Save](crates/gui/src/editor/map_save.rs) |
| File limits, conflicts or save errors | [Bounded reads and single-file writes](crates/project/src/file_io.rs), [paired transactions](crates/project/src/transaction.rs), [errors](crates/project/src/error.rs) |
| Map file lifecycle or metadata | [Map document](crates/project/src/map_document.rs), [discovery](crates/project/src/discovery.rs) |
| Event fields or markers | [Event dialog](crates/gui/src/editor/event_workflows/dialog.rs), [canvas](crates/gui/src/editor/event_workflows/canvas.rs); [source-preserving edits](crates/project/src/events.rs) |
| Tile/roof pixel editing | [Draft and save lifecycle](crates/gui/src/editor/tileset_editor.rs), [dialog](crates/gui/src/editor/tileset_editor/dialog.rs), [clipboard](crates/gui/src/editor/tileset_editor/clipboard.rs) |
| Palette controls or numeric inputs | [Palette form](crates/gui/src/editor/palette_editor.rs), [reusable widgets](crates/gui/src/editor/widgets.rs) |
| Menu commands, layout or startup | [Menu specs](crates/gui/src/editor/menu_popup/specs.rs), [geometry](crates/gui/src/geometry.rs), [frame](crates/gui/src/editor/frame.rs), [startup](crates/gui/src/editor/startup.rs) |
| Preferences, native windows | [Preferences](crates/gui/src/editor/preferences.rs), [native startup](crates/gui/src/editor/preferences/native_startup.rs), [dialog host](crates/gui/src/editor/dialog_host.rs) |

Follow an operation from GUI/CLI orchestration to a project document, then to its
core command. Keep validation in the backend; the UI owns drafts and user consent.
Tests for a feature live beside it or in the corresponding crate's `tests/`.
Files named `plusplus_reference` deliberately identify an upstream test oracle.

### Rust API documentation

Run `cargo doc --workspace --no-deps --locked --open` for public APIs. Add
`--document-private-items` when navigating editor internals. Crate landing pages
link the main concepts; examples run as doctests in the workspace test suite.
CI treats rustdoc warnings as errors for both public and private documentation.

Document new public items with purpose, units/invariants, meaningful examples and
`# Errors`/`# Panics`/`# Safety` where applicable. Prefer checked intra-doc links.
All four library crates deny `missing_docs` for public APIs. This includes fields
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
