# Polished Map NG contributor instructions

## Objective and order

Implement the complete roadmap in `docs/roadmap.md`. Deliver master parity,
plusplus parity, unified editing, a stable Rust/JSON interface, measured
improvements, and distributable Windows/Linux/macOS builds. The core and CI
exist from day one; a partial slice is not a completed parity milestone.

## Documentation ownership and hygiene

- Keep `README.md` a short, user-facing landing page. Follow the structure of
  [upstream's pinned README](https://github.com/Rangi42/polished-map/blob/8b9bdc2b8aac8c865809516c2123af69c498da0b/README.md):
  purpose, availability/install link, basic controls, further help and credits.
  Describe only support this port actually provides. Do not copy upstream's
  release claims, project support promises, screenshots or assets unverified.
- Never append implementation progress, test counts, internal type names,
  per-turn notes, detailed CLI examples or outstanding-task lists to README.
  Change it only when its user-facing introduction or navigation needs changing.
- `docs/roadmap.md` is the authoritative execution plan. Before feature work,
  choose the next unchecked item in the active milestone. Update that item and
  its evidence when work advances; do not append a chronological narrative.
  Add newly discovered work as an actionable item with an acceptance condition.
  Keep completed work visible and separate implementation from verification.
  Close a milestone only when its full acceptance gate passes or an explicitly
  approved exception is recorded. Never move unfinished work out of scope just
  to mark a milestone complete.
- Put build/setup instructions in `INSTALL.md`, GUI workflows in `docs/usage.md`,
  and command/JSON examples in `docs/cli.md`. Keep menu and setting details in
  `docs/menus.md` and `docs/preferences.md`.
- Put source comparisons, geometry calculations, test observations and native
  verification limits in `docs/compatibility.md`; design decisions in
  `docs/architecture.md`; intentional behavior changes in `docs/deviations.md`;
  provenance in `docs/assets.md`/`NOTICE.md`; distribution in `docs/releases.md`.
  Link to the owning document instead of duplicating its details.
- Update existing statements when behavior changes. Do not leave contradictory
  "pending" claims next to newer evidence. Clearly label historical observations
  as historical; they must not override the roadmap's current task status.
  Keep temporary work logs in ignored `.scratch/`, not product documentation.

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
- Specify UTF-8 explicitly in text tooling (including Python on Windows); use
  byte-preserving operations for fixtures and assets with exact-byte contracts.

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

For code or build changes, run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked
-- -D warnings`, `cargo test --workspace --locked`, and `cargo doc --workspace
--no-deps --locked`. CI also checks MSRV, `cargo deny check`, and the production
native-adapter graph with `python tools/native_features.py` (no dev dependencies).

For documentation-only changes, verify local links/anchors, examples against
the actual CLI/menu definitions, and status against implementation evidence;
run `git diff --check`. A full Rust rebuild is unnecessary unless the changes
also affect code, build configuration or executable documentation examples.

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
