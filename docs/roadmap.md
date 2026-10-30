# Delivery roadmap

This is the execution plan for all six accepted objectives. Follow the next
unchecked task in the active milestone; add discovered work here with a clear
completion condition. Detailed observations belong in [compatibility evidence](compatibility.md)
and design decisions in [architecture](architecture.md).

## Current position

| Milestone | Status | What that means |
|---|---|---|
| Foundation | Bootstrap complete | Repository, shared backend, working editor slice and CI exist. Native feasibility checks are still open below. |
| 1. Master parity | **In progress — active milestone** | Source inventory, discovery and original artwork are implemented; native integration and full application acceptance remain unfinished. |
| 2. Plusplus parity | Not started | Expanded-profile port and compatible Polished Crystal acceptance remain. |
| 3. Unified editor | Not started | Profile detection and a combined user workflow remain. |
| 4. Autonomous interface | Experimental foundation implemented; stabilization not started | Rust libraries and JSON CLI exist; stable contracts, batch recovery and acceptance remain. |
| 5. Improvement and refactoring | Not started | Deliberate behavior changes follow compatibility acceptance. |
| 6. Platform delivery | In progress — CI only | Three-OS build/test infrastructure exists; complete native verification and distributable packages remain. |

`[x]` means the stated work is implemented with linked evidence, not that its
entire milestone has passed. `[ ]` means work or acceptance remains. Unchecked
tasks are not started unless explicitly marked **in progress**; a next task is
scheduled, not completed or necessarily started. A milestone closes only when
its acceptance gate passes, including approved exceptions.
Native feasibility was originally listed under Foundation: it is **not complete**
and is carried explicitly by M3–M6 and P1. Bootstrap completion does not waive it.

**Next:** M4, finish source-derived GUI and native-window fidelity, then M5
lifecycle compatibility. M3's implementation and isolated native protocol checks
are in place; its integrated file-manager, compositor, clipboard and mixed-DPI
acceptance remains open and will run with M6 against the finished windows.
M1 inventory/discovery/save feedback and M2 original artwork are complete.
Follow with M6 and M7 acceptance. If a task is externally blocked, record the blocker and
advance another unchecked master task. Do not advance to plusplus merely because
one feature or test suite is complete.

## Foundation — bootstrap complete

- [x] Repository, pinned upstream references, license texts, attribution,
  contributor instructions and Cargo workspace/lockfile established.
  Evidence: [upstream lock](../upstream.lock.toml), [NOTICE](../NOTICE.md),
  [AGENTS](../AGENTS.md), [dependencies](dependencies.md).
- [x] GUI-independent validated models, editing/history, project codecs and
  deterministic CPU rendering support the Open → edit → save slice.
  Evidence: [architecture](architecture.md), [compatibility matrix](compatibility.md#current-coverage-matrix).
- [x] Windows/Linux/macOS CI, MSRV, format/Clippy/test/rustdoc, dependency/license,
  source-reference, visual and real-project jobs established.
  Evidence: [workflow](../.github/workflows/ci.yml).

## 1. Master parity — in progress

Target: the immutable master revision in [upstream.lock.toml](../upstream.lock.toml),
with compatible pinned pokecrystal. Preserve safe valid-input behavior before
adding improvements. Each completed group below still participates in M7.

### Implemented work

- [x] Map lifecycle: New/Open/Recent/Close/Save/Save As, binary/ASM imports,
  dimension selection, oversized-map acknowledgment, dirty guards and conflicts.
- [x] **M1a — Open map-size choices:** source-ordered factor pairs, preferred
  dimensions and decoded ASM byte counts now drive the selector; unknown sizes
  retain free spinners. Evidence: [map-size selection](compatibility.md#open-map-size-selection).
- [x] **M1b — Open/New asset selectors:** tileset/roof dropdowns, numbered
  labels, full-filename numeric ordering, compound/duplicate selection and
  project-folder refresh are implemented. Evidence: [asset selectors](compatibility.md#opennew-asset-selectors).
- [x] **M1c — Open/New client geometry and dismissal:** measured source rows,
  control bounds, font sizes, platform button order and guarded Enter/Escape
  handling are implemented. Evidence: [dialog layout](compatibility.md#opennew-client-layout-and-keyboard-actions).
- [x] **M1d — Binary-map metadata rules:** supported dimension/header macros,
  path precedence, filename overrides, group-byte behavior and shared-file
  non-inference are compared with original source. Evidence: [metadata discovery](compatibility.md#binary-map-metadata-discovery).
- [x] **M1e — Legacy asset-path discovery:** graphics/compound sources and PNG
  destinations, block/collision/palette fallbacks, roofs, events, constants and
  saved/unsaved-map palette selection match source fixtures. Evidence:
  [asset paths](compatibility.md#legacy-asset-path-selection).
- [x] **M1f — Master feature inventory reconciliation:** source feature families,
  implementation/tests and remaining owners are mapped in the
  [source inventory](compatibility.md#master-source-inventory). Newly identified
  gaps were control behavior (M1g) and save feedback (M1h), both now complete;
  classification is not parity acceptance.
- [x] **M1g1 — Middle-click defaults:** Resize Map/Blockset, Overworld Map Size,
  event coordinates, binary collisions and palette spinners/sliders restore
  source defaults on middle release, including palette selection/transfer
  capture boundaries. Evidence: [control defaults](compatibility.md#middle-click-control-defaults).
- [x] **M1g — Shared-control input behavior:** numeric stepping/repeat/wrap,
  typed-value clamping, signed hex, focus/cancellation and Event Options field
  modes/Enter handling are implemented and tested across affected forms.
  Evidence: [shared controls](compatibility.md#shared-numeric-and-event-options-controls).
  Source widget painting and native verification remain M4/M6.
- [x] **M1h — Save feedback and the 128-block warning:** source success messages,
  ordered Save continuations, cancellation/conflict guards and persisted warning
  acknowledgment are implemented across all save/export domains. Evidence:
  [save feedback](compatibility.md#save-feedback-and-block-limit-acknowledgment).
- [x] **M1 — Master coverage inventory and project-discovery gaps:** M1a–M1h
  are complete and the [source inventory](compatibility.md#master-source-inventory)
  is reconciled. Native window/widget fidelity, lifecycle and full project
  acceptance remain explicitly owned by M4–M7.
- [x] **M2 — Original icon and application artwork:** all ten toolbar placeholders,
  message symbols, About picture and desktop icon payload use pinned upstream
  artwork with attribution, known third-party notices and recorded provenance
  limitations, as explicitly selected by the user. Source hashes, decoded pixels,
  disabled/theme variants and 1×/2× bounds pass; reviewed snapshots preserve
  existing layout. Evidence: [artwork port](compatibility.md#original-toolbar-message-and-application-artwork),
  [asset records](assets.md). Native icon delivery remains M4/P1; executable/bundle
  resources remain P2.
- [x] Map editing: paint/fill/replace/swap, drag history, undo/redo, all resize
  anchors, zero fill and source-compatible history boundaries.
- [x] Graphics and project backend: 2bpp/LZ/CHR/PNG, metatiles, palette maps,
  six palette sets, compound graphics, roofs and ASM/binary collisions.
  Compiled-source codec differentials are present.
- [x] Block, tileset, roof and palette draft editing; internal copy/swap/drag,
  separate dirty domains, asset switching and guarded persistence.
- [x] Event parsing/editing for the master's 22 macros, preserved assembly,
  event movement/options, warp navigation, script lifecycle and editor dispatch.
- [x] Source-derived client layout, eight menu groups, all toolbar positions,
  initial light/dark themes, view overlays, rulers, status bar, zoom/panning,
  numeric block shortcuts, Help and About.
- [x] Preference record and recent history; Print composition with all five
  flags, Copy and Export; clipboard/browser/dialog platform adapters.
- [x] Experimental headless inspect/render/edit workflows across the above
  document types, with previews and conflict protection.

Evidence and limitations: [coverage matrix](compatibility.md#current-coverage-matrix),
[menus](menus.md), [preferences](preferences.md), [asset audit](assets.md),
[CLI guide](cli.md). Implemented adapters are not proof of native behavior.

### Remaining work, in execution order

- [ ] **M3 — Complete external file drops and whole-window transparency (in progress).**
  Positioned, ordered native drops and guarded workspace Open are implemented
  through pinned adapters, including Wayland receiving. Wayland opacity now
  covers the root and client frame, with explicit capability errors and guarded
  request/reply handling. Wayland receives retain independent offer lifetimes
  and cancel on seat/window teardown; native protocol probes cover opacity and
  transfer behavior, including X11 selection/INCR exchanges. Complete integrated
  native file-manager/clipboard/DPI acceptance alongside M6;
  verify opacity on Windows, macOS, X11 and Wayland. Evidence:
  [external drops](compatibility.md#native-external-file-drops),
  [transparency](compatibility.md#whole-window-transparency).
  Done when the gates in [architecture](architecture.md) pass;
  stale pointer coordinates or widget alpha are not substitutes.
- [ ] **M4 — Complete source-derived GUI and native-window fidelity (in progress).**
  Resize/asset/Event dialog client metrics and shared original-artwork
  confirmations are implemented; evidence: [option dialog metrics](compatibility.md#remaining-option-dialog-client-metrics)
  and [message clients](compatibility.md#error-warning-and-success-message-dialogs).
  Finish editor/modal child-window behavior, font fallback/Windows bold synthesis,
  popup placement/arrows/mnemonics/screen-edge behavior, focus and dismissal.
  Include the remaining source spinner/slider/dropdown/scrollbar rendering and
  remaining dialog default-focus and native placement details;
  stock egui controls do not by themselves establish that fidelity.
  Verify source geometry at 1×/2× before targeted native checks of dialogs,
  clipboard, external editor/browser dispatch, shortcuts, scrolling and resizing.
  Record platform-specific results and any accepted chrome/rasterization differences.
  Client snapshots alone do not close this task.
- [ ] **M5 — Finish preference and window lifecycle compatibility (in progress).**
  Resolve legacy FLTK user/executable-adjacent preference import, remaining
  inventory differences and native storage/restart/shutdown behavior. Verify
  placement, fullscreen restoration, transparency startup and canceled exits
  without losing edits. Done when [preferences](preferences.md) has evidence for
  each applicable source setting and supported platform.
- [ ] **M6 — Verify complete user workflows on native platforms.**
  Exercise map/block/tileset/roof/palette/event edits, Print overlays/Copy/Export,
  dialogs and clipboard with the integrated application. Cover Windows, macOS,
  Linux X11 and Wayland; record exact environment and outstanding differences.
  Reuse mathematical/headless evidence and reserve manual GUI work for behavior
  it cannot establish. This supplies the master portion of P1.
- [ ] **M7 — Pass master acceptance.**
  Run the complete inventory against the pinned reference, including compatible
  real projects. Save/reopen every editing domain in the original application,
  compare bytes or decoded pixels as appropriate, and exercise failure/conflict
  paths. M1–M6 must be closed or covered by explicitly approved exceptions in
  [deviations](deviations.md); record final evidence in [compatibility](compatibility.md).
  Passing codec or synthetic GUI tests alone is insufficient.

## 2. Plusplus parity — not started

Begins after M7. Preserve all master acceptance fixtures.

- [ ] **PP1:** Inventory differences at the pinned plusplus revision and pin a
  compatible Polished Crystal project with an independently audited fixture policy.
- [ ] **PP2:** Implement expanded tile counts/banks, per-block attributes,
  flips, priority, palette/discovery rules and serialization. Keep map IDs byte-sized.
- [ ] **PP3:** Port branch-specific controls, dialogs, defaults and rendering
  with the same geometry, asset and native verification standards as master.
- [ ] **PP4 — Acceptance:** Differential and real-project edit/save/reopen tests
  pass against original plusplus, with master regressions still green and every
  remaining difference resolved or explicitly approved.

## 3. Unified editor — not started

Begins after PP4; no silent format conversion.

- [ ] **U1:** Define explicit profiles/capabilities and deterministic project
  detection, with a user override and clear ambiguity errors.
- [ ] **U2:** Deliver one binary with capability-aware controls and profile-safe
  loading, editing, saving and switching.
- [ ] **U3 — Acceptance:** Both pinned project suites pass through the unified
  workflow; ambiguous projects require a choice and wrong-profile writes fail
  without modifying files.

## 4. Autonomous interface — stabilization not started

The shared backend and experimental CLI already exist. Stabilize them after U3
so both profiles are represented in the public contract; GUI work must continue
using the same underlying operations in the meantime.

- [x] GUI-independent Rust models/project adapters and experimental JSON
  inspect/render/edit previews/apply exist; see [CLI guide](cli.md).
- [ ] **A1:** Define versioned Rust/JSON contracts for inspect, validate, render,
  edit and batch, with stable error codes, schemas and deterministic responses.
- [ ] **A2:** Make dry-run, explicit apply, input/output hash conflicts and
  recoverable multi-file commits consistent across document types. Finish
  interrupted-journal discovery/replay; current backup/rollback support is partial.
- [ ] **A3:** Publish examples for autonomous callers and schema/compatibility
  tests, including malformed requests, conflicting writers and crash recovery.
- [ ] **A4 — Acceptance:** Both profiles support complete scripted workflows
  without a display/GPU or GUI process, preserve unrelated source and recover
  interrupted batches. MCP remains deferred.

## 5. Improvement and refactoring — not started

Begins after A4. Keep both compatibility suites running; distinguish structural
refactoring from deliberately changed user behavior.

- [ ] **I1:** Propose and implement evidence-based internal consolidation and
  measured performance improvements, preserving the accepted compatibility baseline.
- [ ] **I2:** Add a shared egui RGB/HSV graphical picker on Linux, macOS and
  Windows, retaining exact RGB8 edits and RGB5 controls/export. Parity currently
  targets upstream's Windows native chooser and RGB/hex controls on all OSes.
- [ ] **I3:** Evaluate remaining themes, expanded undo, newer project conventions
  and usability/format-preservation improvements as separately scoped proposals.
- [ ] **I4 — Acceptance:** Each chosen change has a stated benefit, appropriate
  regression/benchmark evidence and documented intentional differences. Unselected
  proposals are explicitly deferred rather than silently expanding scope.

## 6. Platform delivery — in progress (CI infrastructure)

Platform feasibility runs during parity work, not at the end. Final packaging
and release acceptance follow the preceding objectives.

- [x] Windows/Linux/macOS build/test CI and selected native adapter/font checks exist.
- [ ] **P1 (in progress):** Finish native runtime acceptance on Windows, Linux X11/Wayland and
  macOS, including both Mac architectures. Carry forward M6 and plusplus/unified
  results; distinguish compilation, automated native checks and interactive checks.
- [ ] **P2:** Produce Windows x86-64 MSVC ZIP, Linux x86-64 tarball and macOS
  Intel/Apple Silicon app bundles, including the CLI and clean-machine launch checks.
- [ ] **P3:** Automate checksums, licenses/notices, matching source, lockfile,
  toolchain/patch records and rebuild/relink instructions; verify archive contents.
- [ ] **P4 — Acceptance:** Install/run packages and complete release workflows
  on all targets, with [release obligations](releases.md) satisfied. Signing and
  notarization remain explicitly deferred until credentials are available.

## Standing constraints

- Pinned upstream/project revisions define parity; do not quietly retarget newer formats.
- Initial light/dark themes first; remaining themes are I3 work.
- Preserve safe valid-input behavior and source serialization/history limits.
  Fix corruption/crashes immediately and record observable [deviations](deviations.md).
- Derive GUI dimensions from source calculations; use geometry/input/pixel tests
  first, then targeted native verification. Original upstream UI artwork is
  selected with attribution; preserve known notices and provenance limitations.
- Every release respects LGPL and separate asset/font licenses. No project/game fixture assets,
  implicit conversion, web/mobile port or MCP server is part of the accepted scope.
