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
| 3. Unified editor | **In progress — active milestone** | U1 source/detection reconciliation is complete. Automatic Open/New/Change Tileset and representative five-project asset saves are verified; U2 package delivery and U3 integrated/native acceptance remain. The parity launchers remain available. |
| 4. Autonomous interface | Experimental foundation implemented; stabilization not started | Rust libraries and JSON CLI exist; stable contracts, batch recovery and acceptance remain. |
| 5. Measured improvements | Not started | Deliberate behavior changes follow compatibility acceptance. Structural refactoring is ongoing during unified work. |
| 6. Platform delivery | In progress — development builds available | Separate master and plusplus Windows/Linux/macOS tester packages and matching source are published. Clean-machine and final release acceptance remain. |

`[x]` means the stated work is implemented with linked evidence, not that its
entire milestone has passed. `[ ]` means work or acceptance remains. Unchecked
tasks are not started unless explicitly marked **in progress**; a next task is
scheduled, not completed or necessarily started. A milestone closes only when
its acceptance gate passes, including approved exceptions.
Native feasibility was originally listed under Foundation: it is **not complete**
and is carried explicitly by M3–M6 and P1. Bootstrap completion does not waive it.

**Next:** Finish **U2** verification and fresh package delivery after the native
Save As context fix, then continue
**U3** integrated/native acceptance using the five-project tester guide.
Unified Open/New, automatic tileset changes and alternating-project saves have automated
workflow evidence; full unified/native acceptance remains open. The user authorized unified work
on November 7 while retaining the separate parity launchers. Keep structural
refactoring alongside implementation; preserve the outstanding native gates.
PP3 desktop capture/drop, placement/focus/DPI and macOS acceptance and PP4
original-application interoperability remain open against the
[published plusplus packages](releases.md#plusplus-development-packages).
Master M5–M7 acceptance remains open on the
[latest master development build](releases.md#development-packages).
The remaining M4 aesthetic matching is
**deferred by user direction (2026-10-30)** after the palette-slider work;
resume M4 only to fix a functional issue. Exact original fonts are also deferred
to later manual review. See the [approved scope exception](deviations.md#remaining-m4-aesthetics-deferred).
M3's implementation and isolated native protocol checks are in place; integrated
file-manager, compositor, clipboard and mixed-DPI behavior remains open in M6.
M1 inventory/discovery/save feedback and M2 original artwork are complete.
The [approved sequencing exception](deviations.md#plusplus-development-before-master-native-acceptance)
permits plusplus work without claiming M7 has passed.

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

### Review follow-up

- [x] **M9 — November 7 tester review:** Priority alignment, swatch geometry,
  empty-map guidance, WRAM typography, first-launch theme selection and Windows
  multi-size icons are fixed for both launchers. User-approved roof/tileset
  previews retain bounded, scrollable dropdowns. Focused input/geometry tests,
  reviewed snapshots, native Windows inspection and all eight cross-platform CI
  jobs pass. Future suggestions remain separately planned; broader native
  acceptance stays open. Fresh master/plusplus development packages, matching
  source and licenses are [published for all four targets](releases.md).
  Evidence: [review disposition](compatibility.md#november-7-upstream-review).

- [x] **M8 — Upstream tester review:** reviewed input/dialog/usability fixes,
  portable assets and measured release size are verified. Refreshed four-target
  development packages and matching source/licenses are published. Shared fixes
  are backported to `codex/plusplus-port`; Windows workspace and Linux visual
  regressions pass. Native master milestone acceptance remains M5–M7.
  Evidence: [review disposition](compatibility.md#november-1-upstream-review).

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
- [ ] **M4 — Source-derived GUI and native-window fidelity (aesthetics deferred;
  input implemented; native acceptance remains).** Resize/asset/Event dialog client metrics,
  original-artwork confirmations, native editor/modal hosting, ownership and
  draft/close guards are implemented. Isolated Windows, macOS, X11 and Wayland
  lifetime checks pass. Evidence: [dialog metrics](compatibility.md#remaining-option-dialog-client-metrics),
  [messages](compatibility.md#error-warning-and-success-message-dialogs),
  [native hosting](compatibility.md#native-dialog-hosting-and-ownership).
  Palette sliders retain source input behavior with the earlier rounded egui
  appearance selected by the user; see [slider evidence](compatibility.md#source-palette-sliders).
  The user deferred remaining font, widget painting and purely visual menu/window
  matching on 2026-10-30. Preserve these as later manual work under the
  [approved exception](deviations.md#remaining-m4-aesthetics-deferred), not as
  completed fidelity. Source Alt+letter menu opening and item mnemonics are
  implemented with disabled/modal guards; see [functional fixes](compatibility.md#master-functional-gap-fixes).
  Visual underline matching stays deferred. Window focus, usable placement,
  keyboard/dismissal, clipboard, scrolling and resize acceptance remain M6.
  Do not resume aesthetic matching before M5–M7.
- [ ] **M5 — Finish preference and window lifecycle compatibility (in progress).**
  Read-only legacy FLTK user/executable-adjacent import is implemented for all
  master settings, with NG precedence and malformed-input protection. Root normal
  geometry now tracks moves/resizes independently of autosave. Mac HOME discovery
  now follows the original existing-HOME → Foundation → passwd fallback; see
  [functional fixes](compatibility.md#master-functional-gap-fixes). Verify native
  storage/restart/shutdown, placement, fullscreen/maximize restoration,
  transparency startup and canceled exits without losing edits across platforms.
  The [shared native process probe](compatibility.md#native-profile-preference-lifecycle)
  verifies isolated settings shutdown/restart and normal-size persistence;
  complete desktop lifecycle acceptance remains open.
  Done when [preferences](preferences.md) has evidence for each applicable source
  setting and supported platform. Headless import/tracker tests do not close this gate.
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

## 2. Plusplus parity — in progress

Master acceptance fixtures remain mandatory. Keep separate entry points and
format-specific documents; reuse common algorithms, codecs and platform adapters.
Do not expose automatic profile switching until the unified-editor milestone.

- [x] **PP1:** Inventoried the pinned source differences and pinned a contemporary
  Polished Crystal fixture. Scope, reuse boundaries and remaining owners are in
  the [audit](compatibility.md#plusplus-source-audit); the independent
  [fixture policy](assets.md#polished-crystal-fixture-policy) excludes game assets.
- [x] **PP2:** Implement expanded tile counts/banks, per-block
  attributes, flips, priority, palette/discovery/event rules and serialization.
  Attribute block commands, bank mapping and guarded offset/attribute/collision
  saves are implemented with source differentials and real-project tests.
  Raw 512-tile edits, CPU map/Print rendering, expanded codecs, compound graphics
  and roof loading/editing/saving are implemented. Map/root/asset metadata discovery,
  binary-map persistence and palette sessions now have source-reference and
  real-project evidence. The 26-macro event catalog, script discovery, warp rules,
  editing/persistence and Print symbols are implemented with source comparisons
  and real-project save/reopen coverage. Map IDs remain byte-sized. GUI integration
  and original-application acceptance remain PP3/PP4.
  Evidence: [blocks](compatibility.md#plusplus-block-backend),
  [graphics](compatibility.md#plusplus-graphics-backend),
  [project graphics](compatibility.md#plusplus-project-graphics-and-roofs),
  [maps and palettes](compatibility.md#plusplus-map-discovery-and-palette-sessions),
  [events](compatibility.md#plusplus-events-and-script-persistence).
- [ ] **PP3 (in progress):** Port branch-specific controls, dialogs, defaults and rendering
  through a separate plusplus entry point with isolated preferences. Reuse common
  GUI/platform code and preserve source layout/sizing; FLTK visual painting is
  excluded by user direction. Shared map/event editing and canvas/Print dispatch
  now accept explicit plusplus documents. Asset loading, palette controls/imports/
  exports, block editing/transfers/resize, tile/roof editing and ordered asset saves
  are integrated, including asset switching and 512-tile/bank-order options.
  Expanded lists reuse [middle-button panning](compatibility.md#plusplus-sheet-panning).
  The separate launcher, isolated settings/import and branch-specific Help/About
  are implemented; see [startup evidence](compatibility.md#plusplus-launch-settings-and-help).
  A shared [native process probe](compatibility.md#native-profile-preference-lifecycle)
  checks settings and normal-window restoration for both profiles.
  The [menu/default review](compatibility.md#plusplus-gui-source-reconciliation)
  and automated workflow/geometry checks are implemented. Initial
  [Windows native map/tile save/reopen and clipboard checks](compatibility.md#plusplus-windows-native-workflow-checks)
  pass; the shared quick-Alt menu failure they exposed is fixed. The
  [expanded native pass](compatibility.md#plusplus-expanded-native-acceptance)
  covers block/roof/palette/event/conflict and original-app reopen workflows.
  The Linux BMP clipboard fix passes independent X11 and bidirectional
  original-app clipboard checks and cross-platform CI. The real-project pass
  exposed a batched native event-drag defect; the fix passes automated and
  Windows native checks. Complete native
  workflows, integrated window/dialog/clipboard behavior and desktop acceptance
  remain. The [Wayland-only probe](compatibility.md#plusplus-wayland-image-clipboard-gap)
  exposed missing image transfer without data-control/X11 and false Print copy
  success. Native acknowledgment, focused asynchronous publication and retained
  clipboard offers now pass protocol and native exchange checks. Standard
  Wayland PNG/BMP transport and bounded failures are implemented; broader
  desktop/placement/focus/DPI and macOS acceptance remain open. Native capture
  checks also exposed batched pixel-input loss and unregistered dialog hit layers;
  shared fixes and their verification are tracked in
  [native input evidence](compatibility.md#native-pixel-gestures-and-dialog-hit-layers).
  Batched tile/palette/sidebar transfers are also fixed and natively verified;
  see [internal transfer evidence](compatibility.md#ordered-internal-asset-transfers).
  [Native 2× geometry and palette checks](compatibility.md#native-scale-two-geometry-and-remaining-placement-checks)
  add measured evidence and a verified fix for temporary-size dialog mapping;
  desktop pointer alignment, mixed-DPI and macOS checks remain open.
  [Native fullscreen/maximized restart](compatibility.md#native-wayland-window-restart)
  exposed a startup request-order crash, now fixed with a failing-before/passing-after
  protocol regression and native restart evidence; broader placement remains open.
  [Frame focus and placement checks](compatibility.md#native-wayland-frame-focus-and-placement)
  exposed lost keyboard input after titlebar interaction. Parent-surface routing
  passes the independent regression and native F11/Alt+F checks; compositor
  placement/focus limitations remain explicit.
  Separate [plusplus tester packages](releases.md#plusplus-development-packages)
  and matching source/licenses are published after CI, archive and packaged CLI
  verification for all four targets; use them for the remaining native checks.
  Evidence: [GUI integration](compatibility.md#plusplus-shared-gui-integration).
- [ ] **PP4 (in progress) — Acceptance:** Differential and real-project edit/save/reopen tests
  pass against original plusplus, with master regressions still green and every
  remaining difference resolved or explicitly approved. Backend roundtrips alone
  do not establish application parity. [Pinned-project application checks](compatibility.md#pinned-polished-crystal-application-interoperability)
  now cover bidirectional map/event/block/collision/roof/palette operations,
  compound graphics NG → original, valid common-sheet and standalone graphics
  roundtrips, and native Print export/clipboard comparisons. Plain and grid
  images match exactly; priority blending is mathematically accounted for and
  native text appearance remains under the approved aesthetic deferral. The
  original compound-export overread is reproduced and NG's existing rejection
  verified. Retain PP3's integrated desktop/gesture acceptance and master
  regressions before closing the full application acceptance gate.

## 3. Unified editor — in progress

Authorized on November 7 before native parity acceptance; retain both parity
launchers. No mode selector and no silent format conversion. See the
[sequencing decision](deviations.md#unified-work-before-native-parity-acceptance).

- [x] **U1 — Complete:** Detect storage and capabilities from map/project and
  selected-tileset evidence, with actionable missing/conflicting-context errors.
  Storage detection, union events, legacy graphics/collision adapters and metadata
  integration and final source drift reconciliation are implemented, including master
  improvements absent in plusplus. Deterministic tests cover both pinned projects, legacy
  layouts, compound assets and mixed/conflicting evidence; each drift family has a
  disposition in [compatibility evidence](compatibility.md#unified-format-detection-and-upstream-drift).
- [ ] **U2 — In progress:** Deliver one automatically adapting binary with safe loading,
  editing, saving and project changes. Keep application identity/preferences
  separate from document storage; retain both parity launchers. Consolidate
  duplicated code as encountered without erasing format semantics. The third
  launcher, Open/New, union events, separate palette sessions and stable settings
  identity, legacy metadata and cross-storage tileset changes are implemented.
  Representative five-project asset editing/saves are verified. Packaging supports
  the unified launcher. Native verification exposed and corrected stale Save As
  filename hints; fresh distributable package verification/delivery remains. Evidence:
  [unified workflows](compatibility.md#unified-opennew-and-session-transitions) and
  [native save context](compatibility.md#unified-native-save-context).
- [ ] **U3 — In progress, acceptance open:** Pinned pokered, pokeyellow, pokegold,
  pokecrystal and Polished Crystal suites pass through the unified
  workflow, including alternating projects and cancelled/failed loads. Missing
  or conflicting evidence requires corrected context, never a mode selector;
  wrong-format writes fail without modifying files. Native workflow checks
  remain distinct from automated evidence. Discovery, event parsing, classic-map
  rendering/round trips and alternating representative GUI workflows are covered;
  representative block/tile/palette/event edits and available collision/roof saves
  also pass. Full native and cross-storage asset acceptance remain open. Evidence:
  [five-project matrix](compatibility.md#five-project-compatibility-matrix).

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

Deliberate behavior/performance improvements begin after A4. Structural
refactoring is continuous, including U1–U3: simplify functions and remove
duplicate paths as encountered while keeping both compatibility suites running.

- [ ] **I1:** Propose and implement evidence-based internal consolidation and
  measured performance improvements, preserving the accepted compatibility baseline.
- [ ] **I2:** Add a shared egui RGB/HSV graphical picker on Linux, macOS and
  Windows, retaining exact RGB8 edits and RGB5 controls/export. Parity currently
  targets upstream's Windows native chooser and RGB/hex controls on all OSes.
- [ ] **I3:** Evaluate remaining themes, all-domain undo, adding/removing events,
  field-specific event controls, additional map zoom levels, a movable blocks
  sidebar, map-connection navigation, update checks and newer project conventions
  as separately scoped proposals. The November 6 direction defers this wishlist
  until after parity; it is not part of M8 tester builds.
  The [November 7 suggestions](architecture.md#november-7-future-suggestions)
  add NPC sprite previews with a custom-movement fallback and optional extensions,
  plus replacement application artwork. These remain unstarted.
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
- [ ] **P2 (in progress):** Produce Windows x86-64 MSVC ZIP, Linux x86-64 tarball and macOS
  Intel/Apple Silicon app bundles, including the CLI and clean-machine launch checks.
  Development packages are built, verified and published for all four targets.
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
