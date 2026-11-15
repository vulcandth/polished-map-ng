# Delivery roadmap

This is the execution plan for all six accepted objectives. Follow the next
unchecked task in the active milestone; add discovered work here with a clear
completion condition. Detailed observations belong in [compatibility evidence](compatibility.md)
and design decisions in [architecture](architecture.md).

## Current position

| Milestone | Status | What that means |
|---|---|---|
| Foundation | Bootstrap complete | Repository, shared backend, working editor slice and CI exist. Native feasibility checks are still open below. |
| 1. Master parity | **In progress — awaiting native acceptance** | Editor feature families and the two audited functional gaps are implemented; native/application acceptance remains. |
| 2. Plusplus parity | **In progress — awaiting native acceptance** | PP2 backend complete; PP3 functionality and menu/default review implemented with automated workflow evidence. Native workflow acceptance and PP4 original-application interoperability remain. |
| 3. Unified editor | **In progress — awaiting native acceptance** | Unified is the default on main. Functionality and automated five-project/cross-storage checks pass; U3 remains with human testers. |
| 4. Autonomous interface | Experimental foundation implemented; stabilization not started | Rust libraries and JSON CLI exist; stable contracts, batch recovery and acceptance remain. |
| 5. Measured improvements | Not started | Deliberate behavior changes follow compatibility acceptance. Structural refactoring is ongoing during unified work. |
| 6. Platform delivery | In progress — development builds available | Unified and separate master/plusplus Windows/Linux/macOS tester packages and matching source are published. Clean-machine and final release acceptance remain. |

`[x]` means the stated work is implemented with linked evidence, not that its
entire milestone has passed. `[ ]` means work or acceptance remains. Unchecked
tasks are not started unless explicitly marked **in progress**; a next task is
scheduled, not completed or necessarily started. A milestone closes only when
its acceptance gate passes, including approved exceptions.
**Next:** Collect **U3** reports from human testers using the
[published unified build](releases.md#unified-development-packaging) and
[five-project checklist](testing-unified.md); fix reported functional issues.
Native testing was handed to humans on November 8. Do not resume autonomous
GUI/platform campaigns without a new request. Existing evidence does not waive
master M3–M7, plusplus PP3/PP4 or platform P1/P4 acceptance.

Remaining M4 aesthetics and original-font matching are **deferred** under the
[approved exception](deviations.md#remaining-m4-aesthetics-deferred). Keep the
separate parity launchers and refactor shared code as needed without changing
format semantics.

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

## Initial-release review

Authorized November 12 on `main`; preserve functionality and deferred aesthetics.
This preparation does not waive native parity or final release acceptance.

- [x] **R1:** Contributor navigation and GUI module boundaries reviewed and
  improved without behavior/snapshot changes. [Layout decision](architecture.md#crate-and-module-layout-review).
- [x] **R2:** License coverage, notices and source/relink delivery verified;
  artwork provenance limits retained. [License checks](dependencies.md#continuous-license-checks).
- [x] **R3:** Four rebuilt binary archives and matching source verified against
  the reduced content policy. [Archive audit](releases.md#initial-release-archive-audit).
- [x] **R4:** Five-crate input/persistence review completed; two panics and a
  wrong-row palette write fixed with regressions. CI and packaging passed.
  [Review evidence](compatibility.md#initial-release-review).
- [x] **R5:** Storage-based naming, feature ownership, shared file/error services
  and event-document boundaries reviewed and clarified. Public item docs and
  examples cover all library crates; missing docs and invalid rustdoc fail checks.
  Contributor navigation and automated behavior checks pass. The direct PNG
  writer's malformed-buffer hazard is fixed; native acceptance remains open.
  [Review evidence](compatibility.md#code-organization-and-rustdoc).

## 1. Master parity — in progress

Target: the immutable master revision in [upstream.lock.toml](../upstream.lock.toml),
with compatible pinned pokecrystal. Preserve safe valid-input behavior before
adding improvements. Each completed group below still participates in M7.

### Review follow-up

- [x] **M8:** November 1 input/dialog/usability fixes implemented, verified,
  backported and delivered in master/plusplus tester builds.
  [Review disposition](compatibility.md#reviewed-corrections).
- [x] **M9:** November 7 controls, theme, Windows icons and bounded asset previews
  implemented, verified and delivered. [Review disposition](compatibility.md#reviewed-corrections).

### Implemented work

- [x] Map lifecycle: New/Open/Recent/Close/Save/Save As, binary/ASM imports,
  dimension selection, oversized-map acknowledgment, dirty guards and conflicts.
- [x] **M1a — Open map-size choices:** source-ordered factor pairs, preferred
  dimensions and decoded ASM byte counts now drive the selector; unknown sizes
  retain free spinners. Evidence: [map-size selection](compatibility.md#discovery-and-codecs).
- [x] **M1b — Open/New asset selectors:** tileset/roof dropdowns, numbered
  labels, full-filename numeric ordering, compound/duplicate selection and
  project-folder refresh are implemented. Evidence: [asset selectors](compatibility.md#discovery-and-codecs).
- [x] **M1c — Open/New client geometry and dismissal:** measured source rows,
  control bounds, font sizes, platform button order and guarded Enter/Escape
  handling are implemented. Evidence: [dialog layout](compatibility.md#controls-and-geometry).
- [x] **M1d — Binary-map metadata rules:** supported dimension/header macros,
  path precedence, filename overrides, group-byte behavior and shared-file
  non-inference are compared with original source. Evidence: [metadata discovery](compatibility.md#discovery-and-codecs).
- [x] **M1e — Legacy asset-path discovery:** graphics/compound sources and PNG
  destinations, block/collision/palette fallbacks, roofs, events, constants and
  saved/unsaved-map palette selection match source fixtures. Evidence:
  [asset paths](compatibility.md#discovery-and-codecs).
- [x] **M1f — Master feature inventory reconciliation:** source feature families,
  implementation/tests and remaining owners are mapped in the
  [source inventory](compatibility.md#master-source-inventory). Newly identified
  gaps were control behavior (M1g) and save feedback (M1h), both now complete;
  classification is not parity acceptance.
- [x] **M1g1 — Middle-click defaults:** Resize Map/Blockset, Overworld Map Size,
  event coordinates, binary collisions and palette spinners/sliders restore
  source defaults on middle release, including palette selection/transfer
  capture boundaries. Evidence: [control defaults](compatibility.md#controls-and-geometry).
- [x] **M1g — Shared-control input behavior:** numeric stepping/repeat/wrap,
  typed-value clamping, signed hex, focus/cancellation and Event Options field
  modes/Enter handling are implemented and tested across affected forms.
  Evidence: [shared controls](compatibility.md#controls-and-geometry).
  Source widget painting and native verification remain M4/M6.
- [x] **M1h — Save feedback and the 128-block warning:** source success messages,
  ordered Save continuations, cancellation/conflict guards and persisted warning
  acknowledgment are implemented across all save/export domains. Evidence:
  [save feedback](compatibility.md#editing-and-persistence).
- [x] **M1 — Master coverage inventory and project-discovery gaps:** M1a–M1h
  are complete and the [source inventory](compatibility.md#master-source-inventory)
  is reconciled. Native window/widget fidelity, lifecycle and full project
  acceptance remain explicitly owned by M4–M7.
- [x] **M2 — Original icon and application artwork:** all ten toolbar placeholders,
  message symbols, About picture and desktop icon payload use pinned upstream
  artwork with attribution, known third-party notices and recorded provenance
  limitations, as explicitly selected by the user. Source hashes, decoded pixels,
  disabled/theme variants and 1×/2× bounds pass; reviewed snapshots preserve
  existing layout. Evidence: [artwork port](compatibility.md#controls-and-geometry),
  [asset records](assets.md). Broader native icon delivery remains M4/P1; Windows
  executable resources are covered by M9 and macOS bundle metadata remains P2.
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

- [ ] **M3 — In progress:** Positioned external file drops and whole-window
  transparency are implemented. Complete real file-manager/cancellation,
  clipboard coexistence, opacity and DPI acceptance on Windows, macOS, X11 and
  Wayland. Stale pointer coordinates or widget alpha are not substitutes.
  [Drop evidence](compatibility.md#native-adapter-evidence),
  [opacity evidence](compatibility.md#native-adapter-evidence).
- [ ] **M4 — Aesthetics deferred; functional acceptance open:** Native dialog
  hosting, input, measured client geometry and original artwork are implemented.
  Retain the preferred rounded palette sliders and the approved visual exception.
  Complete focus, usable placement, keyboard/dismissal, draft preservation,
  clipboard and resize/scroll acceptance with M6.
  [Hosting evidence](compatibility.md#native-adapter-evidence),
  [scope exception](deviations.md#remaining-m4-aesthetics-deferred).
- [ ] **M5 — In progress:** Preference storage, read-only legacy import and normal
  geometry tracking are implemented. Complete native restart/shutdown, placement,
  fullscreen/maximization/transparency restoration and canceled-exit checks for
  applicable settings/platforms. Isolated probes do not close this gate.
  [Lifecycle evidence](compatibility.md#native-profile-preference-lifecycle).
- [ ] **M6 — Native acceptance:** Exercise integrated map/block/tileset/roof/
  palette/event workflows, Print/Copy/Export, dialogs and clipboard on Windows,
  macOS, X11 and Wayland. Record exact environments and unresolved differences;
  reuse automated evidence for behavior it can establish.
  [Tester guide](testing.md), [coverage](compatibility.md#current-coverage-matrix).
- [ ] **M7 — Master acceptance:** Reconcile the complete source inventory, reopen
  every editing domain in the pinned original with real projects, compare bytes
  or decoded pixels, and exercise failure/conflict paths. Close M1–M6 or retain
  explicitly approved exceptions before claiming parity.
  [Source inventory](compatibility.md#master-source-inventory).

## 2. Plusplus parity — in progress

Master regressions remain mandatory. Fixed parity launchers keep their own
formats; automatic adaptation belongs to the unified application.

- [x] **PP1:** Source differences and the contemporary Polished Crystal fixture
  are pinned and audited. [Source audit](compatibility.md#plusplus-source-audit),
  [fixture policy](assets.md#polished-crystal-fixture-policy).
- [x] **PP2:** Expanded banks, occurrence attributes, graphics/roof codecs,
  map/palette discovery, 26-macro events and guarded persistence are implemented
  with differential and real-project evidence. Map IDs remain byte-sized.
  [Backend evidence](compatibility.md#plusplus-backend-and-gui-evidence).
- [ ] **PP3 — In progress:** Shared GUI integration, separate startup/settings,
  all asset drafts/transfers, 512-tile/bank controls and source menu/default
  reconciliation are implemented. Protocol and focused Windows/Linux checks
  pass. Finish integrated native workflows, dialog/focus/input/clipboard,
  directly hosted pointer/DPI/multi-monitor behavior and macOS acceptance.
  [GUI evidence](compatibility.md#plusplus-backend-and-gui-evidence),
  [native scope](compatibility.md#plusplus-original-application-interoperability).
- [ ] **PP4 — In progress, acceptance:** Complete differential and real-project
  edit/save/reopen coverage against the original application, with master
  regressions green and every remaining difference resolved or approved.
  Existing checks cover bidirectional map/event/block/collision/roof/palette,
  supported graphics and Print exchange; original unsafe compound export is
  reproduced and NG rejects it. Backend roundtrips alone do not close PP3/PP4.
  [Original-application evidence](compatibility.md#plusplus-original-application-interoperability).

## 3. Unified editor — in progress

Authorized on November 7 before native parity acceptance; retain both parity
launchers. No mode selector and no silent format conversion. See the
[sequencing decision](deviations.md#unified-work-before-native-parity-acceptance).

- [x] **U1:** Storage/capability detection, combined events/metadata/asset paths
  and upstream drift reconciliation are implemented, with explicit missing/
  conflicting-context errors. [Detection evidence](compatibility.md#unified-format-detection-and-upstream-drift).
- [x] **U2:** One adapting application with stable identity/settings, separate
  palette sessions, guarded loading and cross-storage Change Tileset/Save As;
  both parity launchers retained. Four-target tester packages are published.
  [Workflow evidence](compatibility.md#unified-opennew-and-session-transitions),
  [save context](compatibility.md#unified-opennew-and-session-transitions).
- [x] **U4:** November 8–9 macOS startup/Print, format options, context and
  bounded dialog/input fixes implemented and delivered after CI/package checks.
  [Review disposition](compatibility.md#reviewed-corrections).
- [x] **U5:** November 10–11 Project options, event fields, menus and status-bar
  corrections implemented and delivered. `paletteswap` remains I3 work.
  [Review evidence](compatibility.md#reviewed-corrections).
- [x] **U6:** Project options padding fixed; unified promoted to main with
  historical branches retained; authorized screenshot added; fmt/Clippy/deny/about
  enforced; all five vendor patches audited. [Padding evidence](compatibility.md#reviewed-corrections),
  [vendor audit](dependencies.md#vendor-necessity-audit--november-11-2026),
  [branch policy](architecture.md#unified-main-branch).
- [ ] **U3 — In progress, awaiting human acceptance:** Automated pokered,
  pokeyellow, pokegold, pokecrystal and Polished Crystal workflows and cross-storage
  save/reopen checks pass. Focused Windows and isolated Linux X11 checks exist.
  Complete integrated chooser, focus/DPI, clipboard, persistence and original-app
  checks on Windows, Linux X11/Wayland and both Macs, including alternating
  projects, canceled/failed loads and wrong-format write rejection. Do not infer
  acceptance from CI or package availability.
  [Five-project matrix](compatibility.md#five-project-compatibility-matrix),
  [cross-storage evidence](compatibility.md#unified-cross-storage-asset-persistence),
  [Windows cycle](compatibility.md#unified-windows-five-project-native-checks),
  [Linux exchange/drops](compatibility.md#unified-linux-x11-original-app-exchange-and-file-drops),
  [human checklist](testing-unified.md).

## 4. Autonomous interface — stabilization not started

The shared backend and experimental CLI already exist. Stabilize them after U3
so both storage models are represented in the public contract; GUI work must continue
using the same underlying operations in the meantime.

- [x] GUI-independent Rust models/project adapters and experimental master JSON
  inspect/render/edit previews/apply exist; see [CLI guide](cli.md).
- [ ] **A1:** Define versioned Rust/JSON contracts for inspect, validate, render,
  edit and batch, with stable error codes, schemas and deterministic responses.
  Public item documentation and `missing_docs` enforcement are covered by R5;
  extend error contracts and runnable examples alongside the stabilized API.
- [ ] **A2:** Make dry-run, explicit apply, input/output hash conflicts and
  recoverable multi-file commits consistent across document types. Finish
  interrupted-journal discovery/replay; current backup/rollback support is partial.
- [ ] **A3:** Publish examples for autonomous callers and schema/compatibility
  tests, including malformed requests, conflicting writers and crash recovery.
- [ ] **A4 — Acceptance:** Both storage models support complete scripted workflows
  without a display/GPU or GUI process, preserve unrelated source and recover
  interrupted batches. MCP remains deferred.

## 5. Improvement and refactoring — not started

Deliberate behavior/performance improvements begin after A4. Structural
refactoring is continuous, including U1–U3: simplify functions and remove
duplicate paths as encountered while keeping both compatibility suites running.

- [ ] **I1:** Propose and implement evidence-based internal consolidation and
  measured performance improvements, preserving the accepted compatibility baseline.
- [ ] **I2:** Add a shared egui RGB/HSV graphical picker on Linux, macOS and
  Windows, retaining exact RGB8 edits and RGB5 controls/export. Parity currently
  targets upstream's Windows native chooser and RGB/hex controls on all OSes.
- [ ] **I3:** Evaluate separately scoped future proposals after parity:
  [remaining themes](https://github.com/vulcandth/polished-map-ng/issues/36),
  [all-domain undo](https://github.com/vulcandth/polished-map-ng/issues/30),
  [adding/removing events](https://github.com/vulcandth/polished-map-ng/issues/11),
  [field-specific event controls](https://github.com/vulcandth/polished-map-ng/issues/31),
  [more map zoom levels](https://github.com/vulcandth/polished-map-ng/issues/32),
  [movable blocks sidebar](https://github.com/vulcandth/polished-map-ng/issues/33),
  [map-connection navigation](https://github.com/vulcandth/polished-map-ng/issues/7),
  [update checks](https://github.com/vulcandth/polished-map-ng/issues/34),
  [newer project conventions](https://github.com/vulcandth/polished-map-ng/issues/10),
  [NPC sprite previews with custom-data fallback](https://github.com/vulcandth/polished-map-ng/issues/35),
  [replacement artwork](https://github.com/vulcandth/polished-map-ng/issues/26),
  [native macOS menus](https://github.com/vulcandth/polished-map-ng/issues/38),
  [searchable collision choices](https://github.com/vulcandth/polished-map-ng/issues/18),
  [theme-library assessment](https://github.com/vulcandth/polished-map-ng/issues/37)
  and [external popup integration](https://github.com/vulcandth/polished-map-ng/issues/39).
  [`paletteswap` rectangle previews](https://github.com/vulcandth/polished-map-ng/issues/40)
  are explicitly deferred until after the initial release.
  [Unresolved design questions](architecture.md#deferred-proposals),
  [paletteswap scope](architecture.md#paletteswap-preview-after-the-initial-release).
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
  Human review follow-ups: [high/mixed DPI](https://github.com/vulcandth/polished-map-ng/issues/27)
  and [older Intel/Apple Silicon macOS retesting](https://github.com/vulcandth/polished-map-ng/issues/28).
- [ ] **P2 (in progress):** Produce Windows x86-64 MSVC ZIP, Linux x86-64 tarball and macOS
  Intel/Apple Silicon app bundles, including the CLI and clean-machine launch checks.
  Development packages are built, verified and published for all four targets;
  unified downloads now include the reviewed `main` code and packaging cleanup.
  [Packaging and tester handoff](releases.md#development-packages) track evidence;
  clean-machine GUI launch and macOS bundle icon metadata remain open;
  Windows multi-resolution resources are implemented in M9.
- [x] **P3:** Automate checksums, licenses/notices, matching source, lockfile,
  toolchain/patch records and rebuild/relink instructions; verify archive contents.
  The development workflow builds all four binaries offline from the published
  complete source archive. Archive contents, notices, executable metadata and
  uploaded digests are verified; see [release evidence](releases.md#development-packages)
  and [rebuild instructions](rebuild.md).
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
