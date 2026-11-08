# Architecture decisions

## November 7 future suggestions

These suggestions from Sylvie/Rangi42's November 7 review are **not started**.
They belong to I1/I3 after parity and the unified-editor work, not M9:

- Display NPC graphics for a whitelist of known pokecrystal movement constants,
  with common Polished Crystal extensions. Retain “O” markers for custom or
  unsupported movement data; consider a user extension mechanism. Resolve sprite
  IDs through the project's constants, sprite table and graphics labels. General
  ASM control-flow interpretation and niche background-object movements are not
  prerequisites for a first proposal.
- Consider revised application artwork or a new design. M9 retains the original
  upstream multi-size ICO and its provenance limitations; it does not adopt the
  alternative images shared in chat.
- Measure executable size and memory before further optimization. The reviewed
  published packages are optimized release builds, despite the chat's tentative
  suggestion that they might be debug builds. Raw RAM figures are observations,
  not evidence of a leak or a requirement to redesign the renderer.

Roof/tileset selection previews were initially classified as future work. The
user explicitly requested them in this review follow-up to make the enlarged
native dropdown clients useful; they are now part of M9. The fixed-size preview
and dropdown each scroll independently as graphics or choice counts grow.
See [review evidence](compatibility.md#november-7-upstream-review).

## Unified application boundary

The unified application is being developed separately from both parity launchers.
Application identity, settings storage and legacy migration must remain stable
when the active document changes. Document storage and map input encoding are
separate backend facts: per-tile palettes versus per-occurrence attributes, and
binary versus ASM map input. Neither is a user-selectable application mode.

`project::detect_map_format` exposes read-only evidence and typed missing-context
or conflict results. It uses the map's metadata or an explicitly selected tileset;
project markers only locate the nearest root. Both loaders and detection share
compound-name/directory resolution. Detection does not validate asset contents:
the selected loader must fail on malformed input without retrying another model.
A shared map without a unique header requires the ordinary tileset choice.

The third `polished-map-unified` launcher now binds controls to the successfully
loaded document, retaining the old session on failed/cancelled loads. Its stable
application identity owns preferences and native windows; the current map owns
attribute capabilities. It retains one palette session per storage model, including
dirty-import consent when returning to an edited session. Both parity launchers
keep their fixed catalogs and settings. Shared load settings, size/tileset request
validation and worker orchestration prepare a full candidate before adoption.
Unified automatic/manual event loading uses `EventDocument::open_unified`,
sharing guarded parser/persistence. ASM map import also works with occurrence
attributes, with explicit binary Save As and no source overwrite. A shared bounded
graphics decoder separates syntax from physical tile capacity. Unified asset
entry points allow CHR and both source layouts, preserving that policy on reload;
compound components, contextual palettes and output paths follow the selected
asset directory. Attributed collision discovery includes master's binary path.
Both parity launchers retain their accepted input and precedence. Unified metadata
uses one shared discovery pass: union binary filename hints, master's legacy
dimensions/headers and the full asset inventory. Conflicting directory aliases
leave the choice unset and produce an actionable detection error. Both the
chooser and actual load worker consume these rules. In-document cross-storage
tileset integration remains in progress.

## ADR 1: Separate domain logic from platform adapters immediately

`polished-map-core` owns validated models, commands, history and deterministic
RGB rendering. It has no filesystem, egui or GPU dependency. `project` owns
discovery, codecs and persistence. `gui` and `cli` depend on these libraries.
Master remains the default application entry point; `polished-map-plusplus`
launches the separate plusplus editor through the same startup code. Plusplus uses explicit
`core::plusplus` and `project::plusplus` modules, with separate attribute-bearing
block documents. Shared code must not erase their differences. Public interfaces
are experimental until objective four.

Plusplus occurrences own bank, palette, flips and priority; raw graphics tiles do
not. Do not widen master tile IDs globally or reinterpret master palette-map bytes
as plusplus attributes. Attribute blocks compose the existing offset/collision
storage, reuse brush footprint math, and use the existing guarded multi-file
persistence and collision codec. No mutable access to offset storage may bypass
attribute synchronization. Raw-pixel operations and map/Print composition now
sit below the two format adapters; the internal block-raster interface exposes
rendered blocks and occurrence priorities without requiring master palette maps.
Plusplus raw tiles carry independent definition flags. Keep source bank ordering
explicit and keep project graphics/roof persistence behind the project adapter.

Plusplus map documents wrap the shared byte-map lifecycle with private, explicit
discovery rules: root markers, filename extensions, metadata macros, asset
directories and Save As defaults. The public master document keeps its original
defaults. Neither document exposes an automatic profile switch. Palette sessions
share RGB editing, export, roof overlays and ordered import continuations, with
plusplus restricting commands and reads to eight game palettes and excluding
master monochrome layouts. No mutable shared palette bank escapes that adapter.
This boundary allows later GUI reuse without accepting the wrong format today.

Events share one semantic model and editing/serialization implementation, with
explicit 22-macro master and 26-macro plusplus parsing catalogs. Adding a variant
to the common enum does not add it to master's parser. Plusplus project entry
points select their catalog and discovery paths; immutable event views and the
existing move/edit/translate commands cannot switch kinds after parsing. The
same CPU Print compositor consumes either model, including plusplus punctuation.

The GUI's internal document enums retain concrete master/plusplus map and event
documents. The owning map selects event discovery, reload parsing, warp paths,
Save As validation and default extensions. Shared map/history and event controls
consume their common views; they do not cast plusplus into a master document.
The canvas and captured Print jobs dispatch to the corresponding CPU renderer,
preserving per-occurrence attributes and independent worker snapshots. Master
block commands cannot mutate plusplus graphics. These adapters are internal GUI
implementation, not the stable automation interface or automatic profile detection.
See [integration evidence](compatibility.md#plusplus-shared-gui-integration).

The editor now has an internal fixed profile. Open options, map/renderer adoption
and palette commits reject profile mismatches. Plusplus asset sessions own their
attribute block document, raw tiles, optional roof, eight-palette document and
bank order; master retains its existing asset documents. Plusplus sessions never
populate those master asset slots. Shared palette controls use borrowed read views
and owned format-specific drafts, retaining each backend's validation and exports.
The existing load worker and ordered import prompts stage the entire candidate
before adoption. Closing a map retains its profile-specific palette session.
Asset saves use the shared feedback queue but their own source/output documents.

The block dialog shares collision controls, modal hosting and acceptance while
its plusplus draft retains occurrence attributes and a 16-bit tile selection.
An eight-palette atlas calls the core occurrence renderer used by the map;
palette/flip brush changes reuse it. Sidebar copy/swap/resize use the existing
input routes, translating only profile-independent commands. Plusplus Set/Paint
requires explicit attributes, with model and preview committed together.

The shared tile/roof dialog retains a concrete master, plusplus raw-tile or
plusplus roof draft. Pixel gestures, clipboard input and modal handling use the
same controls; adapters preserve master's palette-bearing snapshots and
plusplus's independent definition flags. Raw tiles have no palette controls.
Accept builds the composed preview before publishing the target asset domain;
roof edits cannot overwrite base tiles, and tile edits cannot discard roof edits.
The plusplus sheet uses 16-bit selection and the session's explicit bank order.

Asset selectors use profile-specific discovery behind shared forms and dirty
guards. Plusplus replacements stage raw tiles, occurrence blocks, roof and
preview together while retaining palettes and the owning map. Bank-order
changes rebuild the preview from the current assets; they never reload files
or rewrite occurrences. The 512-tile option controls visibility and requests
the source's guarded reload. It is not a conversion between asset formats.

The fixed startup profile selects the app ID, preference key, legacy filenames,
initial palette session and Help/About identity before settings are restored.
Both launchers share native window setup, font selection and platform adapters.
Their app IDs isolate eframe's window records as well as application settings;
neither launcher imports the other profile's preferences. See
[startup evidence](compatibility.md#plusplus-launch-settings-and-help).
A shared internal capability boundary is preparation for U1–U3, not completion of
profile detection, switching or unified write safety. Both format regression suites
must remain active during that later integration.

`polished-map-platform` isolates desktop color dialogs and external script opening.
Core, project, GUI and CLI retain the workspace `unsafe_code = forbid` rule.
The adapter denies unsafe generally and allows it only in documented Win32
modules. The color chooser borrows the registered palette window during its modal call;
the embedded fallback uses eframe's root owner.
Custom color storage belongs to the adapter instance, not a mutable global.
Its two unsafe calls follow Microsoft's
[CHOOSECOLORW contract](https://learn.microsoft.com/en-us/windows/win32/api/commdlg/ns-commdlg-choosecolorw).

`ScriptOpenRequest` retains a saved file path and optional one-based source line.
The GUI queues explicit menu/shortcut/double-click requests; its native adapter
dispatches workers and reports completion/errors through channels. Headless GUI
tests capture requests and inject launch callbacks, never opening applications.
The platform adapter validates an absolute regular-file path. Windows isolates
COM lifetime, association lookup and shell execution; known-editor arguments
are quoted as data using the
[Windows argument rules](https://learn.microsoft.com/en-us/cpp/c-language/parsing-c-command-line-arguments).
The APIs follow the documented
[FindExecutableW buffer contract](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-findexecutablew)
and [ShellExecuteW return/COM contract](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew).
Linux dispatch uses the [xdg-open desktop interface](https://portland.freedesktop.org/doc/xdg-open.html);
macOS uses Apple's `open` tool. Unix arguments use `Command::arg`, never a shell
command string. These are desktop adapters only; core/project and CLI operations
remain independent of display servers and external editors.

## ADR 2: Geometry derived from FLTK

Desktop option/editor/message clients run in immediate native viewports. Their
source-derived client rectangle starts at (0,0); native chrome supplies the title.
The embedded fallback retains egui modals for deterministic client tests.
Measured client size drives the fixed native size; initial position is
`parent_inner.center - client_size/2 - child_decoration_offset`. Wayland ignores
absolute positioning, so its compositor decides placement. Reopening recenters;
ordinary repaints preserve a user's moved window.

A small eframe patch registers weak references to renderer-owned winit windows.
Egui-winit validates a modal ownership tree, retains live windows only through
scoped leases, and filters input for inactive parents. The lease restores native
input before releasing its child; renderer-state replacement and destruction
cannot operate on a recycled raw handle. No native call holds egui's data lock.
Windows supplies ownership and input disabling, AppKit floating child ordering,
X11 transient/modal hints, and Wayland parent/optional xdg-dialog hints. This
matches the separate-window model rather than introducing macOS sheets.

The whole form callback executes in its viewport, including clipboard shortcuts
and post-render keys. Title-bar Close follows Cancel; root Close is refused while
a draft/modal is active. An error keeps its editor visible and inactive under a
nested message window. Accepted Exit explicitly targets ROOT. Help close/input
is also blocked while an application modal is active. Native ownership/protocol
checks and remaining integrated acceptance are recorded in
[compatibility](compatibility.md#native-dialog-hosting-and-ownership).


Help is a nonmodal immediate child viewport, retaining the source's initial
700×500 client size, parent offset, body margins and anchored OK button. Its
state is transient; it retains resized client dimensions within a session but
is not part of persisted application preferences. Backends with embedded
viewports use an egui Window fallback with an explicit close button. Other
application modals disable Help contents, while opening Help alone does not
disable document controls.

`tools/help_assets.py` expands the pinned C++ help literal and converts its
known HTML subset into static typed blocks. Both platform variants are bundled
with the exact original source/hash/license notice. Unknown syntax fails the
generator; the runtime parses only those compiled assets, never arbitrary HTML.
CI verifies regeneration without network access and independently compares
constant expansion with a compiled copy of the original in the reference job.
The viewer preserves source text/styles/URLs and adds an NG reference notice.
External URLs are emitted only through activated egui hyperlinks.

Reference main.cpp/main-window.cpp and block-window.cpp at pinned master.
Baseline client size 800×600; menu 21; toolbar 26; status 23; buttons 24×24;
block editor 466×304. Map block size is 32 (or 64 zoomed). Sidebar width is
four block columns plus theme scrollbar width. Rulers occupy scrollbar width.
Map/event margins must be taken from map-buttons.h, not inferred from screenshots.
Treat baseline units as logical points; apply DPI once and snap physical edges.
Text widths use font metrics. Keep independent rectangle/hit-test fixtures.

## ADR 3: Deterministic CPU pixel source

Rasterize domain content on CPU; nearest-neighbor GPU textures display it.
Visible-region rendering and dirty texture caches prevent full-map work per frame.
Headless PNG export uses the same pixels. Use wgpu through eframe 0.37 initially.

`Graphics::render_print(map, events, options, style, rasterize)` is the shared
Print compositor. Options are independent of current canvas visibility. The
callback receives `PrintFont` (Courier Bold 14 or Helvetica Bold 14) and text,
returning a validated `PrintTextMask` of straight coverage with a separate line
layout size (including ascent/descent) and ink offset. It owns typeface selection
and baseline placement; core
owns block/grid/priority/event/warp composition and clipping. Each distinct
font/text pair is requested once per render. The returned `RgbImage` has native
map dimensions with no viewport margin. The font adapter must remain usable
without a display server so the eventual CLI and GUI Print paths share it.
`project::PrintFonts` supplies CPU coverage using Skrifa and Vello. Fontique
enumerates installed faces through native services. `NativeFonts` owns the
source-derived role preferences for both editor and Print, including explicit
substitution metadata. Callers can instead provide face data and a collection
index with `PrintFontFace::from_bytes`, or validated variable-axis settings with
`from_bytes_with_variations`. `from_native` shares the chosen face bytes and
preserves its collection index and instance coordinates. Line metrics, advances
and outlines all use that instance; the CLI reports instance/substitution details.
Line layout dimensions are separate from the ink mask and its signed bearing,
so centering does not discard overhang. Fonts are bounded to 32 MiB, labels to
three ASCII digits/hex characters or one uppercase symbol, outline paths to
4096 elements and ink masks to 128x64. Native text metrics/hinting and driver
blending must be verified before claiming platform-exact Print output.

GUI Print captures the current map, composed graphics, events, options, style
and an `Arc` of its startup font selection in a queued job. A native adapter selects the PNG destination, then a worker
renders/encodes or constructs the clipboard image. Completion returns through
a channel; only the UI thread emits egui's image-copy command. Interaction
tests inject picker/worker adapters without opening OS windows. The source's
five flags persist independently of view flags, including on Cancel.

`PngExportTarget` records a normalized path and streamed content hash. Preparation
does not grant overwrite consent. The GUI relies on the native Save chooser's
confirmation and additionally confirms an existing destination when appending
`.png` changed the path. Encoding stages in the destination directory, rechecks
the observed hash and installs the complete PNG. CLI new-file output retains
atomic no-clobber semantics. Existing-file replacement has the same narrow
check/rename race as other document writes. An egui CopyImage request confirms
handoff, not successful OS clipboard ownership or delivery to another app.

## ADR 4: Fail safely without pretending to have upstream parity

`Map::overworld_buffer_bytes` reports the WRAM requirement including master's
three-block border. `MapDocument::summary` exposes the value; CLI `inspect`
returns it at `result.map.overworld_buffer_bytes`. It is independent of the GUI
capacity preference and does not require a display. The GUI warns after a
successful map write when the requirement exceeds its saved capacity; the
warning neither rejects the save nor edits WRAM declarations.

Validate before mutation. Snapshot on-disk content for conflict detection; stage
single-file replacements before publication. Block/collision pairs and compound
tileset PNG/palette outputs stage a manifest and before/after bytes, preflight
all observed hashes and attempt rollback on
failure. These are individual replacements, not atomic publication across files.
General journal discovery/replay remains pending. Do not mark data clean
until its corresponding writes succeed. UI and CLI use identical commands.

`MasterTilesetDocument` owns the editable base tiles, contextual colors, separate
roof overlay, compound regions and source/destination observations. Rendering
and headless editing use this same composition. Save prepares all PNG/ASM bytes
on a trimmed draft before staging; source imports remain intact when output uses
the original editor's PNG convention. The GUI pixel editor clones this document
for its modal draft and validates all block previews before replacing live state.
Accepting a base draft dirties only the tileset and preserves map history. The CLI requires a preview-state hash to apply
tileset writes; library callers also supply the block definitions used for trimming
and must verify their backing files if they require on-disk conflict protection.

`Roof` wraps a nine-tile command model; `RoofDocument` owns its independent dirty
state, source/destination observations and 24×24 PNG persistence. The same GUI
pixel controls operate on either draft. Roof acceptance validates the composed
previews before commit, while base acceptance synchronizes contextual roof
attributes without marking roof pixels dirty. Roof attributes are transient;
only pixels persist. Base and roof hashes/save operations are separate. Main Save
now follows upstream's edited-domain ordering: base tileset, roof, blockset,
events, then map if needed. Failure or a canceled blockset destination stops the
sequence. This is not an atomic transaction across documents; earlier successful
saves remain published if a later save fails.

`PaletteBank` owns all six sets and atomic color commands. `PaletteDocument`
tracks the selected set, explicit import source and edited flag. Tileset
documents retain the complete contextual bank rather than discarding unselected
sets after discovery. Selection and color edits affect composed rendering without
dirtying tile pixels or map history. Color export is a separate, explicit
destination operation, not part of tileset PNG/palette-map saving. Imports overlay
the upstream layouts onto the current bank; exports quantize RGB8 to RGB5 without
changing the in-memory colors. The CLI binds an apply to its observed input and
requires an additional destination hash for overwrite. Contextual discovery does
not treat source palette files as implicit write destinations.

The GUI clones palette state for its color/monochrome draft. It validates all
composed block images before committing colors and textures. Text fields retain
their own clipboard behavior; selected swatches use the source RGB5 clipboard
snapshot and exact live swap. As upstream, edited palettes have their own
load-replacement warning and explicit export, not map history or map Save/close
dirty state. `MapPalettePlan` and the GUI's pending load preserve session colors
while resolving per-file import decisions; required-load failures retain the
previous live map. The selected-set preference is persisted; colors and transient
warnings are not.

`Events` owns typed macro metadata, validated coordinates, field-edit commands,
warp resolution and sticky edited state. `EventScript` binds immutable source
spans to those entries; only edited event bodies are regenerated. `EventDocument`
owns the script's observed file hash and independent save. The CLI exposes the
same commands with required apply hashes. The GUI's `Loaded` state owns an optional
event document. Automatic load is part of the map worker, with optional-script
failures reported after committing the map. Manual replacement prepares the new
document before replacing the old one. GUI resize clones the map/event drafts,
validates both, then commits them together in memory. A standalone CLI `translate`
command affects events only and does not imply a multi-document transaction.

## Native external-file-drop integration gate

Research on 2026-10-28 found a concrete adapter gap. Pinned master attaches
`DnD_Receiver` only to `_map_scroll`. `Workspace::handle` forwards the drop;
`Main_Window::drag_and_drop_cb` rejects another modal, asks the usual unsaved
warning and opens the first path. The internal Drag and Drop option is unrelated.

The original registry winit 0.30.13 Windows `drop_handler.rs` ignores the OLE position
argument; its macOS `window_delegate.rs` forwards filenames without positions;
X11 `event_processor.rs` notes that position updates are not exposed. Its Wayland
backend does not implement the file-drop events. Egui-winit 0.37 forwards these
path-only events into `RawInput.dropped_files`; ordinary pointer state is not
evidence of the drop location. The observed problem also appears in
[egui issue 7913](https://github.com/emilk/egui/issues/7913).

Winit's [0.31 prerelease series](https://github.com/rust-windowing/winit/releases)
redesigns native data transfer and drag coordinates. As of this research,
0.31.0-beta.4 is a prerelease, not a compatible update to our locked 0.30 adapter.
Its evolving API must be inspected at a pinned revision, not inferred from old
beta migration notes or the moving master documentation.

Implementation and remaining acceptance:

1. The selected route is a pinned winit/egui-winit source patch, retaining the
   existing eframe and renderer versions. Its original Cargo archive hashes,
   modifications, license texts and maintenance checks are in
   [vendor](../vendor/README.md). Wayland receiving uses the existing native
   window connection/event queue and never sets the clipboard selection.
2. Preserve a transfer identity, ordered native paths, viewport identity and
   the final position for that transfer. Convert physical pixels to logical
   coordinates once, and clear state on leave/cancel. Never infer the location
   from an unrelated mouse event or accept anywhere in the window as a fallback.
3. Route a valid drop inside the source map workspace (including its blank
   margins) through the shared guarded Open flow. Ignore sidebar, rulers,
   toolbar and modal targets. Open only the first path and retain the current
   document on cancellation or failed loading. Do not decode paths lossily or
   treat pasted text/remote URLs as native file drops.
4. Verify target boundaries, multiple files, modal/dirty guards, cancellation,
   transfer ordering and DPI mathematically and with input tests. Then verify
   real file-manager drops on Windows, macOS, X11 and Wayland, including
   off-window cancellation and clipboard coexistence. Synthetic drops alone
   cannot satisfy this gate.

The adapters and guarded workspace route are implemented. The editor consumes
only complete positioned transfers, never `RawInput.dropped_files` or cached
ordinary pointer state. Windows uses the by-value OLE drop point; macOS converts
the dragging location through its flipped content view; X11 uses the last
position belonging to the source/target transfer and waits for its URI response,
preserving that captured point when later position messages arrive;
Wayland uses the offer's surface-local position. The bridge converts physical
pixels to egui points once using native scale × egui zoom.

Payloads are limited to 4 MiB and 4,096 paths; the bridge queues at most 16
transfers per viewport. Unix URI decoding preserves path bytes and rejects
non-file/remote URLs, malformed escapes and NULs. Wayland reads are nonblocking,
limited per dispatch, and time out after ten seconds. The native adapter owns
hover and completed offers separately: leave/new-enter cannot destroy a pending
completed transfer. Up to 16 receives per seat retain independent paths,
coordinates, timers and pipe sources. Removing the seat or target window cancels
the associated receives immediately. Clipboard offers are retained until replacement or seat teardown and shared with
the clipboard adapter; the file-drop path never reads or replaces their contents. These defensive limits are recorded in
[deviations](deviations.md#external-transfer-bounds-and-ole-correctness).
Native file-manager delivery, cancellation, mixed-DPI placement and clipboard
coexistence still require the desktop checks in item 4. See
[current evidence](compatibility.md#native-external-file-drops).

## Native window-opacity boundary

`polished-map-platform::set_window_opacity` accepts a borrowed owner implementing
both raw-window-handle traits and the two-value `WindowOpacity` enum. The GUI
queues F10/menu/startup intent and dispatches after its root UI pass. It stores
the last accepted state separately from saved intent and never retries failed
requests on every frame. The core and project renderer know nothing about it.

Windows follows the source's layered-window style and
[SetLayeredWindowAttributes](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setlayeredwindowattributes).
X11 uses the owner's live Xlib display, its display lock, a native-long property
buffer, `_NET_WM_WINDOW_OPACITY` and XFlush; see
[XChangeProperty's format-32 storage contract](https://xorg.freedesktop.org/archive/X11R7.5/doc/man/man3/XChangeProperty.3.html).
Sending the property is not confirmation that a compositor honors it. macOS
checks the main thread, borrows the NSView, obtains its retained NSWindow and
sets [alphaValue](https://developer.apple.com/documentation/appkit/nswindow/alphavalue).
No native pointer is cached by these production adapters.

Remaining acceptance: real root-window pixels/frame at both settings with wgpu,
F10/menu delivery, fullscreen transitions, normal child-window behavior, restart
and error recovery on Windows, macOS and composited X11. Hidden-window API and
Xvfb property tests cover dispatch/encoding, not those visual requirements.
Wayland's raw-handle adapter returns a typed request for window-layer dispatch.
The GUI queues one request in the pinned egui-winit bridge and waits for its
matching reply before changing the checkbox. Winit binds alpha-modifier v1 on
the window's existing connection/event queue. The root and the fallback frame's
five subsurfaces receive the same multiplier; synchronized child commits take
effect with the renderer's root commit. Frame reconstruction restores resources,
and every alpha object is destroyed before its owning surface. The narrowly
adapted SCTK fallback frame retains its MIT source/license records under `vendor/`.

Missing protocol support, an unavailable client frame or a compositor-owned
visible title bar returns an explicit capability error before changing opacity.
The optional Adwaita frame is also explicitly unsupported; production uses the
adapted fallback frame. Undecorated/fullscreen windows can use root opacity;
configure transitions that acquire an unsupported visible frame restore opaque
state and notify the GUI. Saved intent survives failure/capability loss without
an automatic retry loop. The protocol does not offer compositor-pixel feedback.
The isolated Linux protocol probe checks actual requests and resource lifetime;
compositor/GPU appearance, fullscreen and native preference acceptance remain
open. Winit's boolean buffer transparency is not used as a substitute.

## Standard Wayland image clipboard

The vendored Smithay adapter now transfers PNG and BMP through the standard
`wl_data_device_manager` on the GUI's existing connection, without X11 or the
optional data-control extension. PNG is preferred; malformed preferred data is
reported rather than silently switching formats. Windows and X11 retain arboard;
macOS retains its native arboard path. The core renderer remains display-free.

Each viewport has one queued image request and a matching acknowledgment.
Print and tile/roof Copy and Paste use that bridge; closing an editor cancels its
pending request and stale replies cannot modify a later draft. Completed tile/roof
copies belong to the root window, so closing the draft preserves the selection.
Print keeps its
native dialog alive while rendering/encoding/publishing, since destroying the
focused surface before publication can clear the Wayland seat's clipboard focus.
Its controls and dismissal shortcuts wait for that bounded operation to finish.
Success means the native adapter accepted publication, not that every future
clipboard consumer or compositor bridge will deliver the image successfully.

The window adapter retains the focused seat's selection offer and shares its
proxy/MIME snapshot with Smithay for both text and image reads. This avoids
competing clipboard receive offers: Weston 9 treats destruction of its last
delivered offer as cancellation and only services that offer. File-drop offers
remain separately owned and timed.

The Smithay worker owns focus/seat serials, independent selection sources and
nonblocking pipes. Replacing a selection cannot change bytes already requested
from its predecessor. Encoded selections are bounded to 64 MiB, with at most 16
outstanding requests/sources/transfers and a ten-second transfer deadline. Each
transfer advances by at most 256 KiB per dispatch. Seat removal, requester drop,
read/write failure and timeout release resources. PNG/BMP codecs run outside the
UI thread, with at most four codec workers even across cancellation/retry;
decoding limits dimensions to 8192 per axis and 16 Mi pixels before
RGBA expansion. Source, archive hashes, licenses and exact patches are retained
in [vendor provenance](../vendor/NOTICE.md).

The [independent protocol probe](../tools/wayland_clipboard_probe.py) exercises
real Unix sockets and descriptor passing, without a display server. Actual
application and compositor limits are tracked separately in
[compatibility evidence](compatibility.md#plusplus-wayland-image-clipboard-gap).
