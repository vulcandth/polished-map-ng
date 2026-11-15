# Architecture decisions

The [roadmap](roadmap.md) owns delivery status. This document records boundaries
and constraints that a change must preserve; [compatibility](compatibility.md)
owns verification evidence and [deviations](deviations.md) owns intentional differences.

## Unified application boundary

`polished-map-unified` is the default application on `main`. Application identity
and settings are independent of document storage. `polished-map-gui` and
`polished-map-plusplus` remain fixed parity launchers; unified has no mode selector.

Detection separates **map encoding** (binary/ASM) from **attribute storage**
(per raw tile/per occurrence). `project::detect_map_format` uses metadata and the
selected tileset, returning evidence or a typed missing-context/conflict error.
Project markers locate roots; they do not decide storage. Detection and loaders
share asset resolution. A malformed selected asset fails instead of retrying
another interpretation.

Open prepares a complete candidate before adoption. Unified retains one palette
session per storage model, including import consent for edited colors. Change
Tileset stages graphics, blocks, collisions, roof and previews while retaining
map history, events and palette sessions. It never converts companion files or
rewrites headers. Crossing storage models requires explicit Save As, even after
switching back; cancellation retains both edits and that requirement.

Unified accepts the combined source layouts and event catalog, with storage-
appropriate bounds and persistence. Fixed parity entry points retain their
original discovery precedence and accepted formats. These distinctions must
survive shared implementation refactoring.

## ADR 1: Separate domain logic from platform adapters immediately

### Crate and module layout review

Keep the five crates. Core/project separation protects pure editing from I/O;
platform separation makes unsafe/native interop auditable; CLI stays usable
without a GUI or display server. A smaller file is not by itself a reason for
another crate. Font discovery/rasterization serves both GUI and headless Print
and belongs in project, outside core.

The [contributor source map](../CONTRIBUTING.md#where-to-make-a-change) routes common
changes to their owning modules. GUI `lib.rs` is a small public facade;
`editor.rs` owns private session state. Its child modules separate startup,
frame routing, views, drafts, workflows and reusable widgets. Event canvas/form
code and tile clipboard/dialog code have dedicated files. Native startup lives
with preferences, including the fixed launchers' optional legacy migration.
The CLI separates argument grammar, command execution and JSON records.

Implementation names describe storage, not upstream branches. Core/project
`attributed` owns per-occurrence attributes; shared events, map commands, codecs
and persistence remain shared. Event macro catalogs describe source syntax and
are independent of attribute storage. `plusplus` is only a compatibility import
alias in the public libraries. Existing public type/variant names, serialized
values, preference keys, launcher behavior and provenance stay intact. Their
historical spelling does not imply a second implementation of the unified app.

Avoid a trait layer solely to hide two storage models: enum dispatch in GUI
`documents` makes the supported cases explicit. `Loaded` is the publication
boundary for a fully prepared candidate; failed open/preview/save operations
must not partially replace the current document. Feature `impl Editor` blocks
can access private parent state without exposing it to library callers.

### Domain and adapter responsibilities

| Layer | Contract |
|---|---|
| Core | Validated models, commands, history and deterministic CPU pixels. No filesystem, egui, GPU or unsafe code. Validate whole command batches before mutation. |
| Project | Discovery, bounded codecs, source-span preservation, hashes and staged persistence. Public APIs are experimental until objective 4. |
| GUI / CLI | Presentation and orchestration over shared backend commands. The current CLI exposes master adapters, not the complete unified/plusplus workflow. |
| Platform / native patches | Narrow OS operations, borrowed live handles, typed failures, explicit lifetimes and target-specific features. |

Attributed occurrences own palette, bank, flips and priority; raw tiles own pixels
and independent definition flags. Do not widen classic IDs globally or reinterpret
classic palette-map bytes as occurrence attributes. Paired offsets/attributes stay
synchronized behind validated APIs. Shared raster, brush and collision operations
sit below concrete document adapters.

Events share a semantic model and source-span serializer, with explicit master,
plusplus and combined parsing catalogs. Editing does not change a parsed macro's
kind. Palette sessions share RGB commands but preserve master's monochrome palette
and plusplus's eight-palette restriction. GUI draft adapters commit model and
preview together; a failed preview cannot leave live state partially replaced.

Core/project/GUI/CLI forbid unsafe. Platform interop isolates and documents it.
External script opening validates a regular file, preserves native path encoding
and passes arguments as data, never shell code. Windows owns its worker COM
lifetime; Linux uses `xdg-open`, macOS `open`. The Windows color chooser borrows
its palette window only for the modal call; custom colors are instance state.

## ADR 2: Geometry derived from FLTK

Use pinned source measurements, not screenshot estimates: baseline main client
800×600, menu 21, toolbar 26, toolbar buttons 24×24, master block editor 466×304.
The original status strip was 23 points; the reviewed NG strip is 26. Map blocks
are 32 points (64 zoomed); sidebar width is four blocks plus scrollbar width.
Take event margins from `map-buttons.h`. Measure text, apply DPI once and snap
physical edges; keep independent geometry/hit-test fixtures.

Desktop forms use immediate native viewports with client origin (0,0). Native
chrome supplies titles. Initial centering accounts for child decoration; Wayland
placement belongs to its compositor. Reopening recenters, ordinary repaint does
not undo a user's move. Embedded egui fallbacks support deterministic tests.

Eframe registers weak references to renderer-owned winit windows. Egui-winit
validates modal ownership and takes scoped live-window leases; it restores input
before releasing children. No native call holds egui's data lock or uses cached
raw handles after replacement. Windows ownership, AppKit child ordering, X11
transient/modal hints and Wayland parent/xdg-dialog hints preserve this model.

The complete form callback runs in its viewport, including post-render keys and
clipboard shortcuts. Close means Cancel; active modals guard parent commands,
Help and root close. Nested errors retain the inactive draft. Exit targets ROOT.
Help alone is nonmodal, resizable and transient. Its viewer consumes generated
static blocks from the pinned help literals, not arbitrary HTML; external links
open only on activation. See [asset provenance](assets.md#branch-specific-help-documentation).

## ADR 3: Deterministic CPU pixel source

Core rasterizes pixels; eframe/wgpu displays nearest-neighbor textures. Visible
regions and dirty caches avoid full-map work each frame. Headless exports share
the same rendering. The Print compositor receives validated text coverage masks
and separate layout/ink offsets from the font adapter; font choice and outlines
remain outside core. Each distinct font/text pair is rasterized once per image.

`PrintFonts` uses Fontique, Skrifa and Vello CPU, preserving collection indices
and variable-axis coordinates. Explicit font bytes bypass system discovery.
Bounds include 32 MiB per font, 4,096 outline elements and 128×64 ink masks.
GUI Print captures document data and startup font faces, then renders/encodes on
a worker. UI-thread clipboard publication follows completion. Native metrics and
rasterization are not established by CPU reference comparisons alone.

`PngExportTarget` records the chosen path and streamed content hash. The GUI
requires overwrite consent, including a separately confirmed path when adding
`.png` changes the destination. Encoding stages beside the target and rechecks
the hash before installation. CLI exports use atomic no-clobber installation.
Existing-file replacement retains a narrow check/rename race; it is not an OS
compare-and-swap. Clipboard handoff is not proof of delivery to another app.

## ADR 4: Fail safely without pretending to have upstream parity

Validate before mutation, retain observed hashes, stage writes before replacement
and mark each domain clean only after success. Block/collision pairs and compound
tileset outputs stage a manifest plus before/after bytes, preflight all hashes and
attempt rollback on failure. Keep recovery data when rollback cannot safely finish.
These are individual replacements, not atomic publication across files. Automatic
journal discovery/replay remains objective 4.

Main Save processes tileset, roof, blockset, events and map in order. Failure or a
canceled destination stops the sequence; earlier successful domains remain saved.
ASM imports require binary Save As. Changed collision/event/palette-map rows retain
unrelated source, comments and line endings; ambiguous rewrites fail preflight.

Tileset and roof drafts have independent dirty state and guarded output plans.
Palette colors have a separate edited flag and explicit export: map Save/close
does not implicitly save them. Imports resolve ordered per-file consent before
adopting a new map. Required failures retain the old map; optional roof-color or
event failures warn without partially publishing that optional data.

GUI resize validates map and loaded events together in memory. Separate CLI map
resize and event translation do not imply a multi-document transaction. Dimension
constants remain caller-owned. Map WRAM capacity is a post-save warning, not a
mutation of engine limits. See [deviations](deviations.md) for specific safety changes.

## Native external-file-drop integration gate

The pinned registry window stack lacks the complete position/lifetime events
required by the source's workspace-only drop policy. Local patches retain the
transfer identity, viewport, ordered native paths and final position. The GUI
consumes only complete positioned transfers, never ordinary cached pointer state.
It opens the first path through guarded Open and ignores non-workspace/modal
locations. Internal asset dragging is independent.

Windows uses the by-value OLE drop point; macOS converts the dragging location
through its content view; X11 retains the matching position until selection/INCR
completion; Wayland uses the offer's surface-local position. Convert to egui
points once using native scale × egui zoom. Unix URI decoding preserves path bytes
and rejects remote/non-file URLs, malformed escapes and NULs.

Limits: 4 MiB, 4,096 paths, 16 queued transfers per viewport; Wayland allows 16
receives per seat, bounded nonblocking dispatch and a ten-second timeout. Pending
completed offers survive later hover leave/enter and cancel on seat/window
teardown. Clipboard offers are separately owned. See [transfer deviations](deviations.md#external-transfer-bounds-and-ole-correctness),
[vendor maintenance](../vendor/README.md) and [native evidence](compatibility.md#native-adapter-evidence).
Real file-manager, cancellation, clipboard coexistence and mixed-DPI acceptance
remain human checks.

## Native window-opacity boundary

`set_window_opacity` accepts borrowed window handles and a two-value enum. GUI
requests are queued once, with accepted state separate from saved intent; failure
reports an error without frame-by-frame retries. Windows uses layered-window
alpha, X11 a locked live display and native-long format-32 property buffer, and
macOS main-thread NSWindow alpha. Core/project pixels are unchanged.

Wayland uses alpha-modifier on the existing connection, including all five
subsurfaces of the adapted SCTK fallback frame. Resources are destroyed before
their surfaces and rebuilt with frames. Missing support, unavailable client
frames, visible compositor-owned decoration or optional Adwaita frames return
capability errors. Acquiring an unsupported frame restores opaque state and
notifies the GUI. Protocol acceptance cannot prove compositor pixel output.

## Standard Wayland image clipboard

The patched Smithay adapter transfers PNG/BMP through standard
`wl_data_device_manager` on the GUI connection. Prefer PNG; malformed preferred
data errors instead of falling back. Each viewport has a queued request and
matching acknowledgment; closing a draft cancels pending work, while completed
copies belong to the root. Print keeps its focused native dialog alive until
bounded publication completes. Text/images share the retained selection offer;
file-drop offers remain independent.

Workers own seat serials, immutable source payloads and nonblocking pipes. Limits:
64 MiB encoded, 8,192 pixels per axis, 16 Mi decoded pixels, 16 outstanding
requests/sources/transfers, four codec workers and ten-second pipe deadlines.
Dispatch advances at most 256 KiB per transfer. Teardown and failures release
resources; stale replies cannot alter later drafts. The
[protocol probe](../tools/wayland_clipboard_probe.py) uses real sockets/descriptors;
[desktop evidence](compatibility.md#wayland-checks-and-unresolved-limits) records
its distinct limits.

## Unified main branch

New work and normal CI target `main`. `codex/master-review` and
`codex/plusplus-port` retain historical review tips; their launchers also remain
in shared source. Promotion commit `4f595496f1ce7001e9dff528e413ff79c7782f4e`
preserved both ancestries without a force push. Manual packaging defaults to
unified and accepts either parity profile explicitly.

## Paletteswap preview after the initial release

Deferred under I3. Before implementation, define supported declarations,
rectangle coordinates/overlap order, palette/time-of-day interactions and behavior
for unsupported script expressions. Map and Print must agree without rewriting
project scripts; verify selected semantics against project/reference outputs.

## Deferred proposals

The complete feature wishlist belongs to [I1–I3](roadmap.md#5-improvement-and-refactoring--not-started).
Two proposals need more than a UI toggle:

- NPC previews need project sprite/constants resolution and a whitelist of known
  movement conventions, retaining generic markers for unsupported/custom data.
- External popup windows need a compatible winit/egui integration plus ownership,
  focus, grabs and dismissal on all backends. The popup API from
  [winit PR 4543](https://github.com/rust-windowing/winit/pull/4543) is not compatible
  with the pinned 0.30 adapter; bounded in-window dropdowns remain the current design.

Neither proposal resumes general deferred appearance matching.
