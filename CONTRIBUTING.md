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
| `crates/gui` | egui presentation and editor workflow orchestration | [Editor](crates/gui/src/lib.rs), [application identity](crates/gui/src/application.rs) |
| `crates/cli` | Experimental headless commands and JSON output | [Command entry point](crates/cli/src/main.rs) |
| `crates/platform` | OS integration and narrowly isolated native interop | [Library](crates/platform/src/lib.rs) |
| `tools` | Asset/provenance checks, packaging and independent reference probes | [Tool guide](tools/README.md) |
| `vendor` | Five patched native dependencies, with reproducible patch records | [Vendor notice](vendor/NOTICE.md), [dependency audit](docs/dependencies.md) |

Do not move filesystem or egui dependencies into core. Both frontends should
use the same project/core operations. The [architecture](docs/architecture.md)
explains the boundaries and why the native dependencies are patched. Scratch
outputs belong in ignored `.scratch/`; immutable reference checkouts belong in
ignored `.reference/`. Neither is product source or a release input.

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
