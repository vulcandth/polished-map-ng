# Polished Map NG contributor instructions

## Objective and order

Implement the complete roadmap in `docs/roadmap.md`. Deliver master parity,
plusplus parity, unified editing, a stable Rust/JSON interface, measured
improvements, and distributable Windows/Linux/macOS builds. The core and CI
exist from day one; a partial slice is not a completed parity milestone.

## Evidence and compatibility

- Reference the immutable commits in `upstream.lock.toml`. Ignored checkouts in
  `.reference/master` and `.reference/plusplus` are evidence, not product code.
- Read the original behavior before porting it. Record the originating source
  in module documentation and link acceptance evidence in `docs/compatibility.md`.
- Preserve safe behavior on valid inputs, field widths, discovery precedence,
  output paths, rendering, history boundaries, and serialization conventions.
- Fix crashes and corruption hazards immediately, recording observable changes
  in `docs/deviations.md`. Never silently convert formats or rewrite unrelated ASM.
- Initial themes are upstream default light and dark. Other themes are deferred.
- Do not claim pokecrystal/Polished Crystal parity from synthetic tests alone.

## Architecture and Rust

- `core` owns validated models, edit commands, history, and CPU rendering;
  `project` owns discovery/codecs/persistence; GUI and CLI consume both.
- No egui, filesystem operations, global mutable configuration, or GPU dependency
  in core. Headless operations must not require a window or display server.
- Use private invariant-bearing fields, checked conversions, typed errors,
  `Path`/`PathBuf`, idiomatic ownership, and Rust API Guidelines naming.
- Forbid unsafe in core; platform interop must be narrowly isolated and justified.
- Malformed files must return errors, never panic. Bound decompression and image
  allocation. Validate whole commands before modifying state.
- Keep UI work responsive. Share editing operations between GUI and CLI.
- Add dependencies deliberately, minimizing features; commit Cargo.lock.

## Verification

Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked
-- -D warnings`, `cargo test --workspace --locked`, and `cargo doc --workspace
--no-deps --locked`. CI also checks MSRV and `cargo deny check`.

Test behavior, malformed inputs, independent golden vectors, reference outputs,
and real project integration. Test geometry mathematically and input through
egui_kittest. Inspect snapshot differences before accepting new baselines.
Reserve computer use for native windows, clipboard, dialogs, drag/drop, and DPI.
Report which platform checks actually ran; compilation is not native verification.

## Licensing and delivery

Derivative Rust code is LGPL-3.0-only. Preserve upstream attribution and record
adaptations in `NOTICE.md`. Audit assets independently; do not distribute game
assets or unverified icons. Release matching source, licenses, lockfile, and
rebuild/relink instructions with every binary. Do not label development builds
as parity releases. Update the roadmap and compatibility evidence with changes.
