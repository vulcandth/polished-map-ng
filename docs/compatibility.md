# Compatibility evidence

## Shared editor/Print font policy and variable instances

Print now consumes `NativeFonts`, removing its separate Courier/Helvetica fallback
lists. GUI jobs retain an `Arc` of the startup selection, so a queued export uses
the exact faces shown by that editor even after the editor releases its reference.
The CLI uses the same policy through `PrintFonts::system`; explicit caller font
data still bypasses discovery. Font bytes are shared without copying, and
collection indices and variable coordinates survive the handoff. All of this
remains in the project/GUI adapters; core rendering still has no OS/font/IO API.

`PrintFontFace::from_bytes_with_variations` accepts bounded explicit instances,
rejecting unknown/duplicate axes, nonfinite/out-of-range values and more than 32
settings. Normalized coordinates now affect line metrics, glyph advances and
outlines together. A test-only in-memory extension of the existing licensed Hack
fixture supplies one fvar/gvar axis: at weight 700 the A outline moves 100 font
units and its advance grows by 500. Independent expected size-14 advances are
8 and 12 pixels; the changed ink and untouched B glyph are also checked. A
separate test goes through Fontique selection and `NativeFontFace` before Print,
catching a dropped weight coordinate in either adapter. No changed font file is
bundled or written; the minimal variation tables follow Microsoft's OpenType
[fvar](https://learn.microsoft.com/en-us/typography/opentype/spec/fvar) and
[gvar](https://learn.microsoft.com/en-us/typography/opentype/spec/gvar) specifications.

The Windows Helvetica Bold role has an explicit temporary Arial Bold substitution
when the pinned FLTK default Microsoft Sans Serif has no real bold face. The GUI
and Print now share that substitution; the exact face takes precedence if it is
available. Startup logs and CLI JSON `fonts.instances.*.substituted_for` report
the reference family. JSON also retains existing resolved font names and adds
collection indices and variation settings. This is **not** synthetic-bold parity:
FLTK's GDI font enumeration advertises synthesized bold faces and CreateFontW
requests them, while the current portable renderer requires actual outlines.
Native synthesis, font enumeration/name aliases, glyph metrics/hinting and
pixel-level GDI/Xft/Quartz equivalence remain open requirements.

Print Options now measures with the selected body font (12 logical points on
native Windows, 13 elsewhere), following `Print_Options_Dialog::refresh`'s
`OS_FONT_SIZE` and original padding/spacing formula. Its installed-font test checks
that width and verifies that a queued worker retains and renders the captured
faces. Existing headless fixtures keep their explicit bundled 13-point style.

The preceding font-policy commit `58a4e74` passed all eight jobs in
[CI run 40856606148](https://github.com/vulcandth/polished-map-ng/actions/runs/40856606148).
Current local workspace tests, formatting, Clippy with warnings denied, rustdoc
and the production native-adapter graph pass. The visual suite passes 157 tests
with three explicit ignored tests and unchanged snapshot baselines. Source and
variable-instance tests, Windows installed-font rendering, actual worker-thread
capture and CLI metadata checks pass separately. The three-OS CI steps explicitly
run project, GUI and CLI installed-font tests.

## Installed GUI font roles

Desktop startup now calls `Editor::new_native`, using `NativeFonts` from the
GUI-independent project layer. The policy follows pinned master
`themes.cpp::use_native_fonts`: Windows' `NONCLIENTMETRICS.lfMessageFont` first,
then Segoe UI/Tahoma/MS Sans Serif; Consolas/Lucida Console/Courier New for
regular and bold monospace independently. Linux and macOS both try
Ubuntu/Noto Sans/Droid Sans/DejaVu Sans and their Mono families in source order.
Family names are compared exactly. A failed Windows settings query or invalid
UTF-16 name falls through safely instead of reading an uninitialized structure.

Unmatched lists fall through to the pinned FLTK 1.4.5 backend defaults: Microsoft
Sans Serif/Courier New on Windows, Arial/Courier on macOS, and Fontconfig generic
sans/mono on Linux. Helvetica Bold remains a separate role because the original
does not remap it with the UI family. These defaults were checked in the pinned
GDI, Quartz and Xlib/Xft `*_font*.cxx` sources. The Pango backend and platform
font-name aliases still require differential verification.

Fontique loads host faces once at startup. Collection indices and supported
variable-axis coordinates pass through to egui; font bytes remain local and are
not bundled or written into project files. Empty/oversized/unusable faces are
rejected. Native roles precede the existing bundled glyph fallbacks; the separately
licensed Mac menu-symbol face remains installed. Actual face choices and missing
roles are reported on stderr. The explicit Windows bold-sans substitution is
described above; other missing bold roles retain an explicit bundled stack rather
than accidentally selecting the newly installed regular UI face.

Map IDs and hotkey badges request the dedicated bold monospace family, and event
symbols request the separate bold sans family. Consolas alone on Windows changes
ruler text from 9 to 10 and zoomed warp IDs from 11 to 12, matching `ruler.cpp`
and `event.cpp`; other sizes do not change. Native default control text is 12
logical points on Windows and 13 elsewhere. `Editor::new` explicitly retains
bundled fonts for deterministic embedding/tests, with no system enumeration.

Five isolated-collection tests check precedence, independent bold selection,
exact names/weights/widths, backend defaults, unavailable roles and the
Windows-only Consolas rule. A GUI test checks role/size isolation and reverting
to bundled fonts.
Host-font tests run explicitly in the three-OS CI matrix; local Windows discovery
and egui layout passed with Segoe UI and regular/bold Consolas. They exercise
installed files and shaping, not native window appearance or FLTK rasterization.
On this host Microsoft Sans Serif has no real bold face: GDI synthetic bold is
not implemented by the egui adapter, and that role now reports the shared Arial
Bold substitution. Native hinting, baselines, synthesis and full font/name
fallback equivalence remain parity gates. No snapshot of an
installed proprietary font is used as a redistributed fixture.

Local validation: workspace tests, formatting, Clippy with warnings denied,
rustdoc and the production native-adapter graph pass. The full visual suite
passes (157 tests, two explicit ignored tests) without changing any snapshot
baseline. The installed-font tests are executed separately from that suite.

## Five-field status bar

`status_bar.rs` ports master `Main_Window::initialize`, `update_status` and
`update_event_cursor`, with `Toolbar::draw`/`Label` placement and the Greybird/Dark
frame functions in `themes.cpp`. The 23-point bar has a one-point frame, five
21-point labels and four two-point separators. Each field keeps the measured
width of its original sample (`Blocks: 999`, `Map: 999 x 999`, `ID: $99`,
`X/Y ($99, $99)`, `Event: X/Y ($999, $999)`) plus 16 points. FLTK 1.4.5's
three-point left/right label inset is preserved. Narrow windows clip the row;
they do not squeeze or wrap fields. Frame corner ownership follows the original
`fl_yxline`/`fl_xyline` order, with the outer frame painted after its children.

Map/blockset changes refresh the count and dimensions, which always use decimal.
The other fields follow global decimal/hex display, padding only the hex ID to
two digits. Event coordinates are shown only in Events mode, including without
a loaded script. The current cursor already supplies the source half-block and
marker-overlap rules. Leaving the map clears hover fields; Close clears all five.
Live painting reports the preview block ID before the drag's history command
commits, matching `change_block_cb`'s immediate ID/status update. Release and
Undo preserve the existing history behavior.

Six nonvisual tests cover independent layout vectors, source strings and maxima,
Windows/Linux/macOS font sizes at 1×/2×, hover halves/base/mode changes, empty and
closed maps, blockset/map resizing, live painting/Undo, and diagnostic tooltips.
One rendered test checks frame and separator pixels at both scales and verifies
that truncated fields cannot paint over the right border or outside the bar.
The focused Windows/Unix light/dark snapshots use synthetic maximum-value labels.
Bundled font metrics, egui modal dimming and native event timing remain separate
from these formula/pixel checks. No native-font equivalence or computer-use
verification is claimed. Existing NG operation feedback remains in a hover
tooltip rather than replacing the original fields; see `deviations.md`.

Formatting, warnings-as-errors Clippy, workspace tests and documentation builds
pass. The complete GUI visual suite passes 156 tests with one existing ignored
test. All 81 updated window snapshots were inspected and independently compared
with the preceding Git revision: every changed pixel lies within the bottom
23 logical points (46 physical pixels at 2×). The four new focused snapshots
were also inspected. Forty selected tests regenerated those reviewed baselines;
the final full suite passed in normal comparison mode. No image thresholds were
changed for this port.

## Resize Blockset icon and Linux toolbar rendering

The Resize Blockset button now draws the original `add-sub.xpm` instead of its
temporary `+/-` label. Review of upstream `e894ac8` identifies the geometric
plus-minus drawing with the original blockset resize interface; its pinned blob
is byte-identical to that introduction. Attribution, LGPL-3.0-only terms and
source/pixel hashes are recorded in the asset notice and inventory. Open and
Undo/Redo remain excluded after the component comparisons in `docs/assets.md`.

The asset verifier checks 27 source files and 6,912 decoded pixels. The rendered
atlas checks 20 designs across 320 size/theme/disabled/DPI combinations, including
every rendered pixel and transparent background. Existing toolbar tests retain
24-point bounds, enabled/modal guards and the resize-dialog action. Comparing
the 81 affected window snapshots with the preceding commit confines every
changed pixel to this control's existing icon slot. The separate atlas adds a
column; all previous icon columns are unchanged. All changes were visually
inspected before accepting the new baselines. No computer use was performed.

Toolbar CI run
[`40845621982`](https://github.com/vulcandth/polished-map-ng/actions/runs/40845621982)
passed Windows/Linux/macOS Rust checks, MSRV, dependencies, the original-source
oracles and real-project integration. The visual job had one failure in
`main-menu-tools`. Inspection of its downloaded Linux image found no layout
difference: maximum RGBA channel differences were `[1, 2, 1, 0]`, with exactly
one pixel exceeding 1/255. That pixel, at (417, 32), belongs to the underlying
Warp IDs toolbar label. Tools now uses the existing Linux-only 3.0 perceptual
threshold used for the other affected menus. Other platform thresholds and
geometry assertions are unchanged.

Formatting, warnings-as-errors Clippy, workspace tests, documentation builds and
the full visual suite pass locally (149 visual tests passed, one existing ignored
test). The 40 selected tests passed when regenerating only their reviewed
snapshots; the final full run used normal comparison mode.

## Complete toolbar membership and source grouping

The toolbar now has all 27 buttons from pinned `Main_Window::initialize` in
source order. Previously absent event loading, Game Boy/event/warp visibility,
block/event modes, palette loading, blockset resize, tileset change and roof
change/edit controls share their existing command/dialog paths. Undo and Redo
remain present but disabled without a map, matching `update_active_controls`.
Buttons are 24×24 at y=22..46, in a 26-point toolbar below the 21-point menu.
Eight groups use the original 2+2+2-point separators. The palette selector keeps
its source text-width formulas, 24-point label, centered 22-point dropdown and
four-point gap before Load Palettes. Narrow windows clip rather than reflow.

Four interaction/geometry tests cover all button positions at 1×/2× DPI,
empty-map and modal availability, radio selection and gesture clearing,
keyboard/mouse activation, map undo/redo, and entry/cancellation of the existing
blockset, tileset and roof dialogs. Source inspection also corrected Change
Roof in the menu and toolbar: availability comes from a roof-file inventory
cached at successful map open, as in `Map_Options_Dialog::num_roofs`. Listing a
roof does not enable Edit Roof, which still requires loaded roof pixels. A new
inventory test checks missing roofs, deferred refresh, reopening and closing.

At the toolbar-membership commit, no artwork was imported: thirteen buttons used
already reviewed designs and fourteen used short text placeholders. The later
Resize Blockset import above reduces the placeholder count to thirteen.
Source-size controls and complete actions are not a claim of native theme,
font, hover/focus, file-chooser or artwork equivalence. No computer use occurred.

Local formatting, warnings-as-errors Clippy, workspace tests and documentation
pass. The complete GUI visual suite passes 149 tests with one existing ignored
test. The 79 changed existing snapshots and two new full-width light/dark views
were inspected. Pixel comparison against the preceding commit confines changes
in 76 existing images to the toolbar band; the two palette dropdowns also move
with their source position, and the Tools menu reflects disabled Change Roof.
No snapshot thresholds were changed.

## Numeric block hotkeys, badges and selection reveal

`block_hotkeys.rs` adapts pinned master's `Main_Window::handle_hotkey` and
`select_metatile`. Physical Ctrl+0–9 assigns the selected block; Ctrl+Shift+0–9
clears that digit. A block has at most one digit and a digit at most one block.
Assignments and clears propagate to menu shortcuts, including the View toggles
on Ctrl+Shift+3/4. A bound digit without Ctrl selects and consumes its press,
even with Alt/Shift/Command; a bound Alt+digit therefore precedes Open Recent.
Unbound digits and key releases propagate. Existing modal/text-input guards apply.

Bindings are transient: successful map replacement and Close clear them; failed
opens keep them. Block resize and tileset replacement remove only out-of-range
IDs, and growing again does not resurrect bindings. Swapping block contents does
not move bindings. Preferences, serialized documents and undo history exclude
them. Selection reveal uses the original inclusive half-block thresholds and
four-column row math at 32/64-point block sizes. Canvas right-button picking now
uses press-time selection, as `Block::handle(FL_PUSH)` does, and shares this reveal.

Red digits with white diagonal outlines use 14-point text. The top-right position
comes from `Metatile_Button::draw` and the `ms+1` button width in `update_layout`.
Unzoomed decimal IDs of 100 or more move the digit down 14 points only when IDs
are shown. Two inspected atlas snapshots cover all ten digits, decimal/hex,
99/100 boundaries, hidden IDs, zoom and 1×/2× DPI. No artwork is imported.

Tests cover all digit events, conflicting assignments, per-event modifiers,
physical Ctrl versus macOS Command, event propagation, focus/modal blocking,
Open Recent precedence, failed/successful opens, Close, resizing, tileset changes,
preference exclusion, first/last-row reveal and map-history preservation.
`egui-winit` 0.37 maps digit and numpad keycodes to the same egui digits, preferring
logical keys and falling back to physical keys. Native keyboard layouts/NumLock
and OS shortcut delivery remain unverified. The bundled map-label font remains
an approximation of native Courier Bold; these are source-derived/headless
checks, not a claim of native font or input equivalence. No computer use occurred.

Local formatting, warnings-as-errors Clippy, workspace tests and documentation
pass. The full GUI visual suite passes 144 tests with one existing ignored test.
Only the two new hotkey atlases were added; all existing snapshots pass unchanged.

## Sidebar release gestures and original block-editor entry

Pinned master's `Metatile_Button::handle` calls back on release, and
`Main_Window::select_metatile_cb` requires the pointer to be over the pressed
button. Any mouse button selects; right release also opens the block editor.
There is no sidebar context menu or double-click editor action in this source.
The port now tracks the pressed cell/button despite using one egui interaction
region for the grid. Releases cannot cross into another cell, blank space or
clipped rows; right-dragging away and back still works, because upstream does
not require `event_is_click` for this callback. Pointer loss, disabled/modal UI
and document replacement clear the transient press. Existing drag copy/swap
operations still use their own validated source/target route.

The temporary Edit Block menu entry and its extra divider are removed. Existing
block editing/cancel/accept/save/conflict and collision-layout tests now enter
through actual right-button events. Four new interaction tests cover all three
buttons, both zoom sizes, both transfer settings, near-border release mismatch,
blank/off-sidebar releases, return-to-source drag, pointer loss, modal guards
and partially clipped rows after scrolling. Source-derived layout determines
the target positions. Numeric hotkeys are now covered above; native input/child-
window behavior remains an unverified gate.

Local formatting, warnings-as-errors Clippy, workspace tests and documentation
pass. The full visual suite passes 134 tests with one existing ignored test.
Only the Edit-menu snapshot changed for removal of the extra entry/divider;
it was inspected. All existing block-editor snapshots remain unchanged despite
entering through the real pointer workflow. No computer use was performed.

The preceding transparency CI run
[`40837746785`](https://github.com/vulcandth/polished-map-ng/actions/runs/40837746785)
passed all eight jobs. Linux explicitly passed the Xvfb opacity property test;
macOS compiled and passed its Rust checks. Those results do not prove compositor
appearance or macOS native opacity behavior.

## Whole-window transparency

View/Transparent and F10 now follow pinned master's `Main_Window::initialize`,
`transparent_cb`, `apply_transparency`, `exit_cb` and `main.cpp` startup rule.
The native boundary requests 75% opacity on the root window: Win32 alpha 191,
X11 `0xBFFFFFFF`, and AppKit alpha 0.75; opaque values restore 255/`UINT32_MAX`/1.0.
This does not change client paint colors, palette data, map rendering or exports.
Native handles are borrowed only for the call; unsafe operations are confined
to the platform crate. Core/project remain independent of window APIs.

Independent numeric tests check truncation; a real hidden, test-owned Windows
window verifies both alpha values and preservation of other extended-style bits.
Missing handles return errors before native calls. GUI tests cover F10/menu
ordering, accessible shortcut/check state, one-shot dispatch, error rollback,
modal/modifier guards, dirty history, Close retention and saved startup intent.
An unmapped X11 window test reads back type/format/count/value; CI runs that test
explicitly under Xvfb. It has not been run locally on this Windows host.

Four existing View-menu snapshots were updated for the new row; one checked
Transparent snapshot was added. All five images were inspected, including the
Mac caption and Unix dark 2× views. No icons or external artwork were imported.
No computer-use session was resumed. The hidden Win32 API test is not evidence
of wgpu/DWM visual correctness, frame appearance or actual F10 delivery. X11
still needs compositor verification; macOS needs native runtime verification.
Wayland currently returns an explicit unsupported-integration error and remains
an open parity gate. Saved intent survives a failed startup application.

Local formatting, warnings-as-errors Clippy, workspace tests, documentation,
dependency audit and the production adapter graph check pass. The full visual
suite passes 130 tests with one existing ignored test, without snapshot updates.
The native Linux test and macOS compilation/runtime still require their own
platform results; Windows-only local success is not evidence for them.

The preceding adapter CI run
[`40835449345`](https://github.com/vulcandth/polished-map-ng/actions/runs/40835449345)
passed all eight jobs, including platform builds/tests, production feature
checks, MSRV, dependency/license checks, reference/real-project tests and visuals.

## Native clipboard/browser adapter wiring

Inspection of the resolved production dependency graph and egui-winit 0.37's
`clipboard.rs`/`open_url_in_browser` found that eframe's disabled default features
also disabled native clipboard and URL output. Image copies from Print and the
tile/roof editors were queued by headless tests but could not reach arboard;
Help/About links likewise had no browser dispatcher. Text editing used only an
in-process fallback clipboard. The direct arboard dependency used by explicit
image-paste buttons did not enable the separate egui-winit feature.

Explicit eframe `clipboard` and `links` features now enable these adapters.
`tools/native_features.py` failed on the previous production graph, then passed
for Windows x64, Linux x64 and both macOS architectures after the change. It
excludes dev dependencies, checks actual downstream features/packages, and runs
on every native CI host. These are dependency-graph checks, not cross-compilation
or proof of successful OS delivery. Existing tests cover queued image/URL
commands and incoming clipboard events; native ownership/paste, Linux compositor
compatibility and browser launch remain unverified. No computer use was performed.

The preceding popup CI run
[`40833843444`](https://github.com/vulcandth/polished-map-ng/actions/runs/40833843444)
passed seven jobs, including all three platform Rust checks, MSRV, dependency,
reference and real-project checks. Visuals reported three snapshot differences.
Downloaded Linux File/Options/roof-palette images were inspected and compared
with the committed Windows images: all channel differences were at most 2/255;
only two/one/one pixels respectively exceeded 1/255, at underlying toolbar text
edges. Those three snapshots now use the existing narrowly scoped Linux 3.0
perceptual threshold. No baseline images, geometry assertions or other platform
thresholds changed.

Local formatting, warnings-as-errors Clippy, workspace tests, documentation and
the full visual suite pass (126 visual tests passed, one existing ignored test).
All four production target-graph checks and the dependency audit pass. The
visual suite ran without updating snapshots.

## Measured popup menus and platform shortcut captions

The eight current main groups plus recent files, palettes and roof palettes now
share a renderer derived from master `Main_Window::initialize`, `widgets.h` and
FLTK 1.4.5 `Fl_Menu.cxx`. The latter is pinned in `upstream.lock.toml`, with its
original license and attribution retained. Layout uses supplied font metrics,
one/nine label padding spaces, blank-image/check reserves, four-point leading,
separate shortcut columns and preceding-row divider lines. Source 12/13-point
sizes, Greybird/Dark background colors and square popup frames are implemented.

Independent numeric vectors check maximum-width/height/title/shortcut rules.
Interaction tests check every File row's bounds at both font sizes and 1×/2×
DPI, zero extra divider rows, image-column text offset, aligned shortcut keys,
accessible shortcuts, disabled entries, arrow-key submenu opening and Enter
activation. A regression test prevents activating Enter from also dismissing
the newly opened About modal. Duplicate recent names retain distinct rows and
literal Unicode punctuation. Existing workflow tests exercise the same actions.

Mac captions use a dedicated Noto Sans Symbols 2 family after glyph coverage
checks found the original bundled faces incomplete. Original font/license bytes
are hash-verified offline; no font modification or Rust dependency was added.
The font remains OFL-1.1 with its full notice. Ordinary UI fonts are unchanged.

Popup snapshots cover all eight groups, recent/roof submenus, fullscreen state,
Mac symbols and a Unix dark menu at 2× scale. These are headless renderings, not
native platform verification. Remaining gates include native fonts and frames,
exact check/radio/theme/disabled-color painting, mnemonics, native screen-edge
behavior, unported commands and unresolved toolbar artwork. No computer use
was performed.

Formatting, warnings-as-errors Clippy, workspace tests and documentation pass
locally. The full visual suite passes 126 tests with one existing ignored test.
Eleven existing popup snapshots changed and two platform/DPI snapshots were
added; every changed/new image was inspected. Other existing baselines are
unchanged. The new font/license verifier also passes.

The preceding About CI run
[`40828921493`](https://github.com/vulcandth/polished-map-ng/actions/runs/40828921493)
passed all eight jobs after the Unix default-lint correction.

## About command, measured layout and attribution

Help now has the original Help/separator/About order. About and command-/ open
one modal before or after a map is loaded; OK, Enter and Escape dismiss it.
Document actions and root-window closing are guarded while it is open. The
content distinguishes NG's version/source/license from the retained original
Rangi notice, and includes Fugue creator/source/license/adaptation attribution.
It replaces the interim attribution-only menu without importing new artwork.

The APP_ICON branch of `Modal_Dialog::refresh` supplies the width formula,
620-point measurement limit, 280-point minimum, heading/body/icon coordinates,
25-point heading slot and anchored 80×22 OK button. `themes.h` specifies body
sizes 12 on Windows and 13 elsewhere, with headings four points larger. Pure
geometry vectors cover both allowances, including the source's possible final
width of 712/713 despite its 700-point measurement setting. Interaction tests
check identity/attribution, control bounds, menu/shortcut/Enter/Escape, empty-map
access, modal guards, unchanged project bytes and retained dirty-map history.

The application image remains excluded after history/hash/visual review; the
icon slot stays blank. That is an unresolved visual gap, not artwork clearance.
Native window chrome/system fonts also remain unverified. Windows-light,
Unix-light and Windows-dark snapshots were inspected, with platform metrics
selected before the first layout pass for reproducible cross-platform baselines.
No computer use was performed.

Formatting, warnings-as-errors Clippy, workspace tests and documentation pass
locally. The full visual suite passes 120 tests with one existing ignored test;
the three About tests also pass after isolating platform metrics before initial
layout. The Help-menu baseline changed and three About snapshots were added;
other existing baselines are unchanged.

The first About CI run exposed a Unix-only `derivable_impls` warning for a
boolean platform default. About now stores its actual 12/13-point font size,
preserving behavior on every host without suppressing the lint. The targeted
visual checks require the same snapshots after this representation change.

The preceding Help CI run
[`40826594651`](https://github.com/vulcandth/polished-map-ng/actions/runs/40826594651)
passed all eight jobs, including the new original-C++ help comparison.

## Help window and complete upstream documentation

Help/F1 now uses a nonmodal, resizable child viewport based on `Help_Window`
and `HTML_View`. The initial native client size is 700×500, at parent position
plus (48,48). Document and button rectangles follow the source formulas; reopening
resets scroll/position and retains resized client dimensions. Native closure,
OK, Enter and Escape dismiss Help. The embedded fallback exposes the same
document and an explicit close button. It leaves main controls available and
does not save/edit documents or persist its transient state.

The complete source `res/help.html` is bundled with immutable commit/SHA-256 and
LGPL attribution. The offline generator retains 95 blocks and 16 linked spans
for both platform variants. Its independent compiled-original comparison passes
for all 14,210 Unix and 14,234 Windows expanded UTF-8 bytes, normalizing C++ source
and stdout line endings. Both generation verification and the original comparison
are now CI steps. No help images, remote content or additional fonts are imported.

Four tests cover complete content/URLs/platform paths, native builder geometry,
resize retention, body/button geometry, scroll-to-bottom/reset, emitted link
requests, menu/F1/close controls, modal guards and nonmodal map interaction.
The top/bottom document and updated Help-menu snapshots were inspected. Snapshot
text uses an explicit Windows variant on every host; both native variants are
tested separately. The original arrow glyph uses the already-bundled Hack
fallback after the normal proportional faces. Native viewport placement/focus/
resize/close, external browser dispatch and FLTK HTML/font appearance remain
unverified; no computer use was performed. About remains a separate parity item.

Formatting, warnings-as-errors Clippy, workspace tests and documentation pass
locally. The full visual suite passes 117 tests with one existing ignored test.
Only the Help menu baseline changed, with two new document snapshots; other
existing baselines are unchanged. Both offline asset verification and the
compiled-original Help comparison pass.

The preceding Print workflow CI run
[`40824755056`](https://github.com/vulcandth/polished-map-ng/actions/runs/40824755056)
passed all eight jobs, including Windows/Linux/macOS and the visual suite.

## GUI Print options, image copy and PNG replacement

The Print command now follows pinned master's `Print_Options_Dialog` and
`Main_Window::print_cb`. File/Print, toolbar Print and command-P open the same
form. Its five flags default off independently of view toggles; Cancel and Escape
retain changes as the source does. Settings serialize those flags on shutdown.
The content area follows the source's text-width formulas, 80-point height,
10-point margins, 22-point control slots and Windows/Unix button order. Initial
Export focus, Enter, command-C and Escape are implemented. An exact modifier
check prevents Print from stealing command-Shift-P from Show Priority.

Copy and Export capture current unsaved map, composed graphics, events, flags
and style. A worker runs the shared font adapter/compositor; output ignores
canvas zoom, scroll and view toggles. Export uses `Print Screenshot`, defaults
to `screenshot.png`, and retains existing filename suffixes. Appending `.png`
requires another confirmation if that changed destination already exists.
Observed file hashes are checked before and after staging a complete PNG.
Clipboard output is handed to egui on the UI thread, then the source success
wording is shown. Print options, queued work and completion dialogs guard
document interactions and native root-window closing.

Five headless workflow tests cover both button orders and calculated geometry,
independent flags, cancellation/reopening, all three invocation routes, keyboard
actions, picker cancellation/errors, worker errors/disconnection, close guards,
image-copy commands and actual PNG replacement. A captured output test edits
map blocks, RGB8 palette colors and event coordinates without saving, verifies
those captured values, then proves Copy and PNG use identical RGB data while
preserving project files and map history. Four project PNG tests cover image
validation, no-clobber creation, replacement, a 9-MiB existing destination,
suffix rules, conflict preservation and cleanup. Preference tests include the
five flags, old-record defaults and malformed nested-field preservation.

The Windows/Unix form and clipboard-success snapshots were inspected. These
remain egui modals; native child-window frames, font metrics/hinting, actual
clipboard ownership/paste, chooser/overwrite behavior and display-driver alpha
are still platform acceptance gates. No manual computer use was performed.

Local formatting, warnings-as-errors Clippy, workspace tests and rustdoc pass.
The full visual suite passes 113 tests with one existing ignored test. Only
three new Print snapshots were added; no existing baselines changed.

The preceding font/CLI commit's CI run
[`40822639418`](https://github.com/vulcandth/polished-map-ng/actions/runs/40822639418)
passed all eight jobs, including installed-font tests on all three OSes.

## Print fonts and autonomous PNG output

`PrintFonts` now supplies bounded CPU masks for source Print font requests.
Fontique selects installed bold faces through DirectWrite/Core Text/Fontconfig;
explicit TTF/OTF bytes and collection indices are also supported. Windows tries
Courier New and Arial first; other systems try Courier and Helvetica first.
Liberation and DejaVu bold faces are explicit fallback families. There is no
synthetic bolding and no OS font redistribution. Missing fonts/glyphs, malformed
data, excessive metrics and unsupported labels return errors.

Skrifa reads metrics/outlines at 14 pixels per em; Vello CPU produces straight
coverage without a window/GPU. Integer advances and a rounded ascent/descent
line box are separate from the ink rectangle and its signed offset. This keeps
negative bearings/overhang without shifting centered event letters. An added
core test fixes the expected centered pixel independently of its ink width.
Adapter tests cover all 512 decimal/hex block labels and all 22 event symbols,
using already-licensed dependency test fonts, plus malformed inputs. Windows
system-font checks select Courier New Bold and Arial Bold; the three-OS CI job
now explicitly exercises installed bold fonts. Linux CI installs Liberation.

CLI `render` accepts the five `--print-*` overlays, `--print-hex`, `--print-dark`,
an optional explicit event script and paired font files. Fonts initialize only
when labels are needed. Plain output retains the original JSON shape; Print
output records options, style, event source and resolved font names/rasterizer.
CLI tests verify pixel-exact dark grid edges, real glyph/outline presence,
event overlays, no-font operation, malformed inputs, no writes on failure,
and unchanged source documents. A 320x288 pinned New Bark Town export with roof,
all overlays and installed Windows fonts was inspected locally. That is a
layout/legibility check, not native FLTK pixel equivalence.

New-file PNG output now stages encoding and installs with `persist_noclobber`.
Both CLI and GUI new-file exports use it. Tests decode RGB pixels, preserve an
existing output, reject malformed/oversized images and check failure cleanup.
The follow-up Print Options UI, persistence, Copy and overwrite workflow are
recorded above. Native font hinting/metrics, font substitution and platform alpha
differences remain fidelity gates; this implementation does not close them.

Local checks pass: workspace formatting, warnings-as-errors Clippy, tests and
documentation; dependency advisory/license/source audit without new exceptions;
the compiled 256-case Print comparison; real-project composition; installed
Windows bold-font coverage; and the visual suite (108 passed, one existing
ignored test). No GUI snapshot baselines changed.

The preceding compositor CI run
[`40820129348`](https://github.com/vulcandth/polished-map-ng/actions/runs/40820129348)
passed all eight jobs, including the new 256-case comparison.

## CPU Print composition

`Graphics::render_print` adapts the pinned master's `Main_Window::print_map`,
`Block::print`, `Event::print` and `draw_outlined_text`. It prints at 32 pixels
per block and 16 per event, with all five options initially off. Grid color and
block-ID number base are explicit inputs. Warp labels retain each event's
coordinate base. It completes each block before the next, then draws events in
source order. Unknown block IDs keep the existing EMPTY_RGB fill. Clipping acts
on actual pixels, including text overhanging an off-map event. Font coverage is
cached per font/text and bounded to 128x64; adapter errors return no image.

`tools/print_oracle.py` extracts five unchanged source functions, the event
metadata table, six marker PNGs, warp digits and the small priority PNG, recording
hashes and provenance. Its C++ driver substitutes synthetic fonts and a software
image surface. The Rust reference test decodes the original PNGs independently
and compares all 3,145,728 output bytes across 256 cases: every flag combination,
both block-ID bases, both grid colors and two text masks including overhang and
partial coverage. Cases include all 22 event kinds, overlapping/off-map events,
warp IDs above 15 with mixed per-event bases, missing blocks and priority slots.
This comparison passes locally and is wired into the reference CI job.

Four core tests check mask bounds, straight alpha, clipping, font-cache requests,
error propagation/input preservation, default/plain export identity, independent
warp-ID flags, grid edges and known priority pixels. `real_print` passes 96
compositions of pinned New Bark Town with three roof choices, actual palettes
and parsed events; it also verifies plain-export identity and unchanged input
files. Its one-pixel test font does not verify appearance of real glyphs.

Local validation passes: workspace formatting, Clippy with warnings denied,
workspace tests and documentation, the new original-function comparison and
real-project Print test, and the full visual suite (108 passed, one existing
ignored test). No snapshot baselines changed. The GUI's event-color lookup now
uses the same core mapping checked against all six source marker PNGs.

Alpha follows the inspected [FLTK 1.4.5 Xlib software fallback](https://github.com/fltk/fltk/blob/release-1.4.5/src/drivers/Xlib/Fl_Xlib_Graphics_Driver_image.cxx):
normalize alpha to 0..256, then integer interpolation. Native XRender/GDI/Quartz
paths and real Courier/Helvetica layout/rasterization remain unverified. No font
adapter, Print Options UI, image clipboard or overwrite integration is claimed
by this core change. Those remain on the master-parity roadmap.

The preceding fullscreen/drag correction CI run
[`40817994986`](https://github.com/vulcandth/polished-map-ng/actions/runs/40817994986)
passed all eight jobs, including three OS builds/tests and the visual suite.

Status: initial implementation in progress. Neither upstream parity gate passed.

## Internal drag-and-drop option and palette transfers

The pinned master's `Main_Window` preference/menu callbacks and
`Metatile_Button`, `Deep_Tile_Button` and `Color_Button` in `src/map-buttons.cpp`
define a default-on internal-transfer option. The port applies it to the block
sidebar and both tile editors and adds palette drag-to-swap/Ctrl-drag-to-copy.
Color copying preserves live RGB8 channels, independently of the original
keyboard clipboard's RGB5 quantization. Swatches now use the source's three-ring
black/white/black selection border; tile and palette sources/destinations show
that border during a drag.

`drag_tests` exercises enabled/disabled transfers, exact colors, outside/same-slot
releases, draft cancellation, acceptance and ordinary selection/edit/clipboard
commands with transfers disabled. The event workflow also moves an event with
the option disabled. Preferences tests cover the default and restoration. Visual
tests assert source/target border pixels for palette, base tiles and roof tiles;
the existing selection-border oracle additionally covers 20-pixel swatches at
1x and 2x. These are headless egui checks, not native OS drag verification.
External file dropping (`Workspace`/`DnD_Receiver` and
`Main_Window::drag_and_drop_cb`) is independent of this option in upstream and
remains unimplemented.

The three new drag-feedback snapshots and seven changed baselines were reviewed.
Four palette-form changes are confined to the selected 20x20 swatch, two menu
changes to the new option and shifted bottom rows, and the border atlas gains
one palette case and the canvas height needed to contain it. Formatting,
workspace Clippy/tests/docs and visual-feature Clippy pass locally. The final
visual suite passes 105 tests with one independent-oracle test ignored by default.

CI 40816806957 passed the three OS test jobs, reference/real-project checks, MSRV
and dependency audit, but failed the three new drag snapshots on Linux. Inspected
artifacts show all differences over 2/255 confined to native-default OK/Cancel
labels (palette: x=412..543/y=381..390; tileset: x=470..601/y=438..447;
roof: x=350..481/y=400..409). The drag snapshot fixture now explicitly selects
Windows button order on every host, preserving its baseline and thresholds.
Existing editor geometry tests continue to cover both platform button orders.

## Other master parity evidence

Full Screen follows the source View entry, F11 binding and startup/exit window
state handling. The adapter sends `ViewportCommand::Fullscreen` and reads the
reported viewport flag for the checkmark, including restored or OS-driven
changes. Headless tests cover both directions, unknown/initial state, use without
a map, one-shot requests, Escape, modal guards and retention of map bytes,
dirty state and undo history. The checked-menu snapshot and existing View menu
baseline cover placement after Palettes. Eframe 0.37's `WindowSettings` and
`epi_integration` source show that normal geometry, fullscreen and maximization
are stored/restored together; the native executable explicitly enables that
existing persistence. Native mode transitions, geometry restoration and actual
exit/relaunch remain unverified on Windows, Linux and macOS.

Local formatting, workspace Clippy/tests/docs and visual-feature Clippy pass.
The visual suite passes 108 tests with one oracle test ignored by default. The
new checked-menu image and updated View baseline were inspected; the latter's
changes are confined to x=94..236/y=284..325, covering the added row and shadow.

Overworld Map Size follows `Overworld_Map_Size_Dialog` and
`Main_Window::save_map`: default 1300, range 1–4096, measured 380-point form,
platform button order and a post-write warning using the padded map area.
Tests cover under/equal/over capacity, 255×255 arithmetic, dialog accept/cancel,
modal shortcuts, save failures, asset-only saves, clean Save and forced Save As.
The CLI exposes the same required-byte count without a window. Four named pinned
pokecrystal maps provide independent metadata-derived size vectors. These checks
do not prove native fonts, frame/icons or actual OS settings persistence.
Local formatting, workspace Clippy/tests/docs, visual-feature Clippy and all
seven pinned real-project tests pass. The final visual suite passes 101 tests
with one existing ignored test. The Options and Roof Palettes submenu baselines
and three new dialog/warning snapshots were inspected. The description's left
alignment and 10-point inset also have a geometry assertion.

Monochrome and 256 Tiles follow `monochrome_cb`, `allow_256_tiles_cb` and the
chooser-free `change_tileset_cb(NULL)` reload path. GUI tests cover clean reloads,
three dirty domains, both cancellation/acceptance routes, failed reads, missing
palette maps, map history and session colors. Preferences restore both options;
New/Open and Close carry them forward. Cancellation retains the preference while
the document keeps its prior interpretation and preflight save outputs, a safety
correction recorded in `deviations.md`.
`prepare_replacement_in_mode` exposes this operation without a GUI. Synthetic
tests check every source tile/attribute slot for all four mode combinations;
these are source-derived expectations, not a compiled C++ differential test.
The pinned real-project switch test compares all four Johto transitions with
fresh reads, including map rendering, retained colors and the New Bark roof.
Formatting, workspace Clippy/tests/docs, visual-feature Clippy, all seven pinned
real-project tests and the final visual comparison (98 passed, one ignored)
pass locally. Only the Options menu and Roof Palettes submenu snapshots changed;
both were inspected. Native restart and menu dispatch remain unverified.

Show Priority now follows the original default-on state, Ctrl/Cmd+Shift+P shortcut,
menu/toolbar synchronization, and disabled-control behavior. Tile Priority being
off disables the controls without clearing the Show Priority value or overlays.
Canvas/sidebar, block sheet/chips and tileset thumbnails use separate source
patterns before selection/grid/ID/event drawing. The headless graphics metadata
includes roof-aware priority flags; plain RGB rendering and tile clipboard omit
the display overlay. Roof thumbnails avoid the source's invalid window cast as
recorded in `deviations.md`.

The pinned original PNGs independently verify all 896 RGBA pixels via
`tools/priority_oracle.py` and `priority_patterns_match_all_original_rgba_pixels`.
GUI tests verify all three textures at 1×/2× DPI over light/dark backgrounds,
control routing, disabled/modal shortcuts, attribute/dirty-state retention and
unadorned clipboard output. The new priority icon has reviewed LGPL provenance;
26 imported image files now cover 19 designs and 304 atlas combinations.
Native FLTK/egui alpha-compositing equivalence remains unverified.

Local formatting, workspace Clippy/tests/docs, visual-feature Clippy, all seven
pinned real-project tests and the full visual suite (94 passed, one ignored)
pass. The independent priority PNG comparison and all 26 icon hash checks pass.
All 60 changed existing snapshots were inspected: 53 changes are confined to
the toolbar; the others cover the View menu, editor thumbnails, shifted palette
popup and expanded icon atlas. Four new priority snapshots were also inspected.

Tile Priority follows `Main_Window::allow_priority_cb`, `Palette_Map::read` and
`write_palette_map`: changes preserve attributes/dirty state, control editor
availability and serialization, and survive Close/persistence. GUI tests cover
both option states, subsequent plain/priority palette reads and the disabled
editor control. Project tests check automatic enablement, mode-sensitive hashes,
no writes for a clean document and exact gap changes after pixel-only edits.
CLI preview/apply/reopen tests cover both ordinary and priority palette commands;
the CLI retains its existing automatic inference of priority mode for batches.
Show Priority is covered by the subsequent display work above.
Local fmt, workspace Clippy/tests/docs, visual-feature Clippy, all seven pinned
real-project tests and the full visual suite (92 passed, one ignored) pass.
Only the Options and Roof Palettes submenu baselines changed; both were inspected.

CI 40808925173 passed all checks except the recent submenu visual comparison.
Inspected artifacts show exactly three shadow-edge pixels differ by 2/255 RGB;
all other channel differences are at most 1/255. Only `open-recent-ten` now uses
a Linux threshold of 3.0, matching the measured rasterization variation. Its
baseline pixels are unchanged.

Open Recent follows master's ten-entry ordering, exact-string promotion, basename
labels, Alt+1..Alt+0 shortcuts and Clear Recent. `recent_maps::tests` covers
eviction, original path spelling, Unicode/native encodings, all ten shortcut
routes, menu selection, both cancellation stages, completed/failed opens, New
Map, Save As and ordinary Save. The file menu and populated recent submenu have
headless visual baselines. Preference tests restore long recent paths without
accessing files or reopening maps, and retain Clear Recent across reconstruction.
Native menu navigation and real shutdown/storage delivery are still unverified.
The full local visual suite passed (91 tests, one ignored); only the existing
File popup baseline changed, with one new ten-entry submenu baseline. Both were
inspected. Formatting, workspace Clippy/tests/docs and visual-feature Clippy pass.

| Area | Reference | State / required evidence |
|---|---|---|
| Binary maps / dimensions | master map.cpp, map.h | Exact byte I/O, New Map, Save As/ASM conversion, filename/explicit/metadata dimensions tested; oversized-file consent pending |
| Paint, fill, replace, swap, history | master main-window.cpp, map.cpp | Core and GUI drag→undo→redo→save tests; all nine resize anchors, zero fill, truncation and history clearing tested; GUI resize stages map/event shifts; full gesture matrix pending |
| 2bpp / LZ / CHR / PNG | master tiled-image.cpp, parse-asm.cpp | Initial codecs; all 65,536 bitplane pairs match compiled upstream function; full-codec differential tests pending |
| Metatiles, palettes, collisions | master metatileset.cpp, tileset.cpp, palette-map.cpp, colors.cpp | Binary metatiles, tilepal, six sets, project/special palette precedence, metadata selection, roofs, compound composition, palette commands/export, GUI Open/New session carry-over, ASM/binary collisions and paired saves tested; persistent preferences and full differential coverage pending |
| Project discovery / metadata | master config.cpp, option-dialogs.cpp | Makefile root, dimensions/header precedence, legacy macro ordering, numbered aliases and compound companion paths tested; shared-map options, compound selection ordering and complete option selectors need further coverage |
| Main window geometry | master main-window.cpp, ruler.cpp | Derived rectangle/hit-test/DPI, ruler geometry/scroll/highlight, global hex and block-ID tests pass; light/dark development snapshots inspected; native fonts/themes, persisted preferences and initial scroll positioning pending |
| Block/tileset/roof/palette editors | master *-window.cpp | Block, tileset, roof and palette drafts, pixel gestures, palette/priority attributes and display, clipboard operations, drag/copy, block count, loaded-tileset/roof switching and persistence implemented; native child windows and native clipboard verification pending |
| Events / warp navigation | master event.cpp, map-events.cpp | Shared model, 22-macro parsing, preserved saves/CLI, GUI overlays/options/dragging/warp navigation, automatic loading, save lifecycle, resize, editor dispatch, mode shortcuts and normal-size digit pixels tested; native fonts, export overlays, remaining gestures and native workflows pending |
| Preferences, overlays, print/clipboard | master main-window.cpp | Twenty-seven implemented settings plus recent history restore/save with malformed-record protection; Print flags/Copy/Export and internal Drag and Drop join the tested view, import and size options. Legacy import, remaining preferences, native storage/placement, complete overlays and native print/clipboard parity pending |
| Expanded tile/attribute profile | plusplus tile.cpp, metatileset.cpp | Pending; compatible Polished Crystal test required |
| Unified profile detection | both config.cpp variants | Pending |
| Stable automation / batch recovery | new API | Block/collision and compound tileset preflight, backups and rollback tested; tileset CLI preview and hash-required apply; general batch journal discovery/replay and stable schemas pending |
| Native packaging and runtime | all three OSes | Pending |

New tests must distinguish synthetic vectors, source-derived expectations, and
actual C++ differential observations. A source-derived vector is not a run of
the original application. Unverified assets never become redistributable fixtures.

## Current evidence (2026-10-28)

- Implemented preferences now restore through the real eframe creation context
  and save callback, using a stable app ID and versioned JSON record. Tests cover
  all sixteen settings, defaults, 270 combinations of saved enums, invalid/newer
  records retained without overwriting, restored dark theme and exclusion of
  documents, edited colors and transient GUI state. Periodic settings autosave
  and generic widget-memory persistence are disabled to match source lifecycle.
  Native window persistence is enabled through eframe, but actual disk/shutdown/
  placement behavior still requires native testing. See `preferences.md`.
  Auto-Load Special Palettes and Auto-Load Roof Colors now join Auto-Load Events
  in Options. Four combination tests verify no immediate recoloring, independent
  imports on the next open, skipped malformed optional files, clean palette
  state and manual roof loading while automatic loading is disabled.
  Only the Options and Roof Palettes submenu baselines changed; both were
  inspected. The remaining existing snapshots are unchanged with persistence
  enabled.

- Data → Load Roof Colors and Options → Roof Palettes now follow the pinned
  callbacks, including all five table layouts, known-group enablement, manual
  indoor/monochrome loading, edited-palette consent, retained selected set/dirty
  state, atomic preview refresh and success/error handling. Changing layout
  affects future manual and automatic imports, survives Close and does not
  recolor immediately. Three headless workflow tests cover these transitions,
  cancellation, invalid files, live graphics, modal shortcut guards, unchanged
  map/tileset history and untouched source files. Native dialogs and native
  preference persistence verification remain pending.
  `PaletteDocument::load_roof_colors` and `roof_palette_path` expose the same
  import and precedence without a GUI. Source-derived tests check every palette
  cell for all five layouts, both dirty states, selected-set retention, missing
  and malformed files, insufficient groups and index overflow.
  The extended C++ palette oracle compiles unmodified `Color::read_roof_colors`:
  all 120 outcomes and complete 648-byte banks match (five layouts, six table
  lengths, groups 0/1/2/255). Its parser stub supplies complete synthetic
  palettes; this is an overlay-function differential, not an ASM-parser or
  native GUI comparison. The pinned New Bark Town palette test also verifies
  manual import equals automatic loading on the real roof table, without
  modifying the project.
  The Data/Options popup changes, five-choice submenu and confirmation/success
  snapshots were inspected. No other existing baselines changed. These are
  headless egui snapshots; native shield artwork and modal geometry remain open.

- File Close, Save Map and Exit now follow `Main_Window::close_cb`,
  `save_map_cb`, `exit_cb`, `unsaved` and `modified_filename`. Five headless tests
  cover all five dirty domains, warning filenames, Cancel preservation, discard
  without source writes, retained palette/monochrome/view state, cleared
  document/graphics/clipboard/scroll state, dedicated map saving with other
  domains still dirty, undo retention, external-map conflicts, keyboard/modal
  guards and event-unload shortcut precedence. Exit tests inspect the actual
  one-shot viewport Close command and verify Escape/Enter plus both platform
  button orders. Palette-only edits remain outside the source unsaved guard.
  The shared warning now has the original OK/Cancel actions and wording rather
  than the earlier Save/Discard/Cancel prototype. Warning geometry/shield,
  preference persistence and native window dispatch remain unverified.
  The updated File popup and both fixed-platform warning snapshots were
  inspected; other existing snapshot baselines remain unchanged.

- Menu snapshot CI run 40801569788 found only rasterization differences in
  Data/View/Mode: six glyph/rounded-control edge pixels differ by 2/255 RGB;
  all remaining pixels differ by at most 1/255. Downloaded artifacts were
  inspected and compared per channel. Only those three Linux snapshots use
  a perceptual threshold of 3.0; all other snapshots retain their defaults.

- Main-menu grouping now follows the pinned source's File/Data/Edit/View/Mode/
  Tools/Options/Help order. Event/palette data operations moved to Data; resize
  and editor dialogs moved to Tools; modes and auto-load moved to Mode/Options.
  Undo/Redo menu entries share map history. Mode radio choices and Switch Mode
  use the same transition as shortcuts, clearing active gesture state. Headless
  tests verify the 21-point bar bounds, ordered groups/commands, unavailable-data
  guards, menu undo/redo and mode changes without map edits. Existing editor,
  save and script workflows now exercise their final menu routes. See
  `menus.md` for the explicit remaining command and geometry inventory.
  All eight open-menu snapshots were inspected. The 45 changed existing
  baselines differ only in the menu-bar band (y=0..20); toolbar, editor body,
  dialog and status pixels remain unchanged.

- The toolbar's six-set palette selector follows `Main_Window::initialize`,
  `palettes_cb` and `Toolbar::draw`: 24-point label, 22-point dropdown centered
  in the row, widths measured from "Palettes:"/"Custom" plus six-point text
  padding (and 24 points for the dropdown), followed by four-point spacing.
  It shares the View menu's palette-switching operation. Interaction tests check
  all six sets, live block recoloring, two-way menu synchronization, stable
  control bounds, accessibility names/values, Space/Escape, modal guards and
  selection before loading a map. Existing RGB edits, palette dirty state and
  map undo/redo history survive selection. Native font/frame and popup input
  equivalence remain unverified.
  Light/dark open-dropdown snapshots were inspected; all 43 changed existing
  baselines differ only in the toolbar band (y=21..46). The tileset-gesture test
  identifies its own palette dropdown by name now that the toolbar has one too.

- Linux CI exposed a host-dependent button order in the new disabled-collision
  snapshot. Its artifact differed meaningfully only in OK/Cancel text; all other
  per-channel differences were at most one level. That test now explicitly uses
  Windows button order on every host, as do the other fixed-order baselines.
  No snapshot tolerance was loosened; Windows and Unix order retain separate
  coverage in the existing block-editor workflow test.

- Eight collision-quadrant images now replace the procedural placeholders in
  the block editor, bringing the reviewed icon inventory to 25 files. Exact XPM
  and RGBA hashes and LGPL provenance are recorded per asset. Named and binary
  collision fields use the source's 16×16 labels, positioned three points before
  the field and vertically centered; missing collisions show disabled icons.
  This placement follows `Block_Window::initialize`, `OS_Input/OS_Hex_Spinner`,
  [FLTK 1.4.5 outside-label geometry](https://github.com/fltk/fltk/blob/release-1.4.5/src/Fl_Group.cxx)
  and [image alignment](https://github.com/fltk/fltk/blob/release-1.4.5/src/fl_draw.cxx).
  Binary fields now include `text_width(s, 2)`'s four points of padding before
  the 22-point spinner allowance. Tests check field positions, enabled state,
  unchanged history after cancellation and independently reconstructed quadrant
  pixels in the actual dialog. The expanded atlas checks all 18 icon designs,
  light/dark, enabled/disabled, 22/24-point bounds and 1×/2× scales (288 cases).
  The atlas, binary dialog and new disabled-collision dialog were inspected;
  changed existing dialog pixels stay within collision-control regions. These
  are source-derived/headless checks, not native FLTK font/frame equivalence.

- Seventeen reviewed icon images (including dark variants) now reproduce the
  pinned XPM pixels in the toolbar and tileset/roof clipboard controls. Imported
  XPM and decoded RGBA hashes are checked by `tools/icon_assets.py` in CI, with
  Fugue CC BY 3.0 and Polished Map LGPL adaptation notices retained. Help and
  README include attribution. The renderer preserves hard transparency and
  source colors; disabled colors use FLTK's two integer truncations:
  `(31*r + 61*g + 8*b)/100`, then `(gray + background)/2`, with Greybird 206 or
  Dark 83 background. Seven fixed color vectors and rendered 22/24-point controls
  cover enabled/disabled and light/dark states at 1×/2× scale. This is source-
  derived/headless rendering evidence, not compiled FLTK or native DPI comparison.
  Interaction checks cover 24×24 bounds in the y=22..46 toolbar, 22×22 clipboard
  controls, accessible labels, keyboard activation, toggle state, editor dispatch
  and modal/disabled guards. Complete toolbar membership is now covered above;
  native frame/theme styles and remaining artwork are still pending. Open,
  Undo and Redo now use source-sized text placeholders while provenance is reviewed.
  The icon atlas and representative light/dark/tool-dialog snapshots were
  inspected. All 42 changed existing baselines differ only in the toolbar band
  and the four tile/roof clipboard-control areas; the remaining pixels are
  unchanged. The atlas covers all 14 commands, dark variants, enabled/disabled
  states, two control sizes and two DPI scales (224 rendered cases).

- Zoom now follows `Main_Window::update_zoom/update_layout`: double offsets on
  zoom-in, integer-halved offsets on zoom-out, and reset the sidebar to row zero.
  Toolbar/menu/command-plus/command-equals tests cover both axes, fractional and
  odd offsets, modal suppression, explicit reset precedence, empty views and
  unchanged document bytes/history. The new layout is calculated after controls
  so a click cannot apply a new offset against old content geometry. Scrollbar
  spacing reserves exactly 16 points, removing egui's extra four-point gap.
  Far-edge tests retain the source's blank space after zoom-out and verify that
  the scroll range contracts when returning toward the map. This follows
  [FLTK 1.4.5 Fl_Scroll](https://github.com/fltk/fltk/blob/release-1.4.5/src/Fl_Scroll.cxx)
  (unclamped `scroll_to` and child-derived position) and
  [Fl_Slider::scrollvalue](https://github.com/fltk/fltk/blob/release-1.4.5/src/Fl_Slider.cxx)
  (range includes position plus viewport size). These are source-derived and
  headless egui tests, not a native FLTK interaction differential; native wheel,
  initial positioning, resize and middle-button panning coverage remain pending.
  Three zoom snapshots (scrolled normal, enlarged and far-edge space) were
  rendered and inspected. All 39 changed existing baselines differ only within
  sidebar/canvas scrollbar bands; menu, toolbar, dialog and status pixels remain
  unchanged. Representative light/dark, ruler, clipped-guide and modal views
  were also inspected.

- `map_buttons` ports bottom/right grid lines, the grid-reserved outer row/column,
  and opaque black/white/black or black/white/white/black selection rings from
  `map-buttons.cpp`. The shared renderer is used by the map, sidebar, event
  half-cell cursor and block/tileset/roof editor selections. Twenty independent
  cases verify every rendered pixel at 1× and 2× scale, covering both map zooms,
  grid on/off, selection on/off, light/dark colors and 16/24/32/64-point selections.
  GUI pixel tests verify map hover/leave, event-mode suppression of the full-block
  border, Cmd/Ctrl+G, modal suppression, sidebar source/target drag outlines and
  a no-op invalid drop. These are headless GPU and source-derived expectations,
  not native DPI or compiled FLTK output comparisons. Native theme completeness,
  native drag dispatch remain pending. Hotkey badges are now covered above.
  The border atlas, hover/drag views and representative light/dark, zoomed and
  modal snapshots were inspected. All 35 updated existing snapshots keep menu,
  toolbar and status pixels unchanged; differences stay in the editor body.

- Global Hexadecimal and Block IDs controls adapt `Main_Window` and
  `map-buttons.cpp`. Tests cover decimal/hex widths and prefixes, two zooms,
  hovered yellow versus ordinary white ID text, View/toolbar/keyboard controls,
  input/modal guards, and unchanged map bytes and event-script serialization.
  Event Options retain `$1`/`#2` titles from per-event formats regardless of the
  global view setting. A rendered ID atlas and three decimal/hex/map/event views
  were inspected. The 34 changed prior snapshots differ only in toolbar pixels,
  plus corrected live block-hover text in `rulers-blocks`; automated full-image
  comparison found no other changed regions. Native Courier Bold metrics,
  complete status sections, export overlays and persisted settings remain open.
  Negative hex ruler labels omit the formatter's terminal newline because
  [FLTK 1.4.5 fl_draw.cxx](https://github.com/fltk/fltk/blob/release-1.4.5/src/fl_draw.cxx)
  consumes it without creating another line. Passing it unchanged to egui caused
  an extra line and clipped text; the inspected snapshots verify the adaptation.

- Rulers adapt `Ruler::draw`, `Main_Window::update_layout` and `Map::max_event_x/y`:
  16/32-point cells, six-cell margin, alternating full/half ticks, source text/
  highlight rectangles, capped labels, and tick continuation across the viewport.
  Independent vectors verify positions at zero, partial-cell and fractional scroll,
  including a slightly negative offset from pixel rounding of the content origin,
  coordinate caps and block-pair versus event-half highlighting. GUI tests verify
  View/Shift+command+R, zoom, both scroll axes, modal guards, leave handling and
  an empty map. Five normal/event/zoomed/scrolled/empty-dark snapshots were rendered
  and inspected. This is source-derived and egui rendering evidence; no compiled
  FLTK ruler comparison or native font/theme equivalence is claimed. Decimal and
  hexadecimal formatting are covered; preference persistence remains pending.
  The Game Boy shortcut was corrected to Ctrl/Cmd+Shift+M: upstream encodes it with
  uppercase `M`, and its toolbar explicitly names Ctrl+Shift+M. Tests reject
  unshifted command+M and retain the modal guard. See the
  [FLTK shortcut contract](https://www.fltk.org/doc-1.1/Fl_Menu_Item.html).

- `WarpIdLabel`/`WarpDigit` in core retain the original digit patterns, 5×7 cell
  offsets, right-to-left draw order and overlapping opaque white borders.
  `tools/warp_id_oracle.py` extracts the pinned source PNG and compiles unchanged
  `draw_warp_id`/`draw_warp_id_zoomed` functions with drawing-call recorders. The
  reference test verifies every one of the 560 source pixels and 8,192 layouts/
  texts (IDs 1–4096 in decimal and hex). Normal labels retain trailing digits;
  zoomed labels retain three leading characters. This is compiled function and
  source-pixel evidence, not native font rendering. A new rendered atlas covers
  both bases/sizes and overflow cases; pixel assertions independently check the
  GUI's actual black/white ID-1 glyph. Mesh quads avoid egui dropping 1×1 filled
  rectangles. The five existing event snapshots change only at ID labels; the
  atlas and changed views were inspected. Native Helvetica/monospace selection
  remains pending. See `docs/assets.md` for the glyphs' provenance and licensing.

- Event cursor/guide geometry adapts `Map::block_under`, the block-half helpers,
  `Main_Window::update_event_cursor/update_gameboy_screen`, and
  `Game_Boy_Screen::draw`. Independent vectors cover 16/32-point event cells,
  160×144/320×288 guides, exact half boundaries, invalid floats, empty margins,
  and C++ truncation that maps event coordinate -1 to block 0. GUI tests cover
  overlapping markers, no-script operation, decimal hover text, Ctrl/Cmd+Shift+M,
  mode/modal guards, actual wheel scrolling and viewport clipping. Three new
  normal/zoomed/clipped snapshots were rendered and inspected. The source PNG
  was decoded locally: all 92,160 pixels are RGBA(224,255,0,64); the guide uses
  this constant fill and source black/white one-point borders, without a bitmap.
  Geometry is matched; native callback timing, full status layout and persisted
  preferences are not yet verified/implemented as a complete feature.

- External editor tests capture actual double-click, Shift precedence and File/
  Ctrl-Cmd-U requests, preserving source line numbers and unsaved event drafts.
  Injected worker callbacks exercise successful completion and visible failure
  without launching a desktop application. Tab toggles repeatedly without stealing
  focus; modal editing suppresses mode/script shortcuts. Platform unit tests cover
  all six supported Windows executable names, unknown-editor fallback planning,
  Unicode/verbatim/UNC paths, Windows parameter quoting, invalid lines/files, and
  Unix argument boundaries including non-UTF8 path bytes. These are planning and
  dispatch tests, not native association/editor checks. Windows is locally tested;
  Unix argument tests run in CI on their respective targets.
  Prior CI run 40780636085 passed all three platform builds/tests and other gates except
  one Linux snapshot: direct inspection found a 2/255 RGB delta at (249,149) in
  the dimmed warp glyph; all other differing channels were at most 1/255. Only
  `event-options-warp` uses a Linux perceptual threshold of 1.0 instead of 0.6;
  no baseline image, geometry assertion or other snapshot tolerance changed.

- `Events` retains master coordinate bounds, legacy X/Y order, prefix/suffix
  fields, one-based warp IDs, same-map target lookup, sticky modified state and
  resize translations without adding history. `EventScript` preserves unowned
  bytes; `EventDocument` bounds reads/edited output and verifies content before
  saving. `tests/events.rs` covers 22 macros, script/event `warp` disambiguation,
  decimal/hex/binary coordinates, negative hex round trips, field normalization,
  atomic failed batches, discovery precedence, repeated saves and external-change
  conflicts. CLI tests cover preview, required/stale hashes and unchanged files
  after invalid batches. `tools/event_oracle.py` extracts seven unmodified C++
  functions and the original metadata table; 34,188 cases compare parsed X/Y,
  hex mode, warp classification, marker letter, reformatted text and destination.
  This is compiled function evidence, not the original GUI or writer. The pinned
  real-project test reads all 388 map scripts (3,684 events), verifies byte-exact
  no-op round trips and saves/reopens one event change per nonempty script on
  temporary copies. GUI evidence follows below.

- `event_workflow_tests` covers actual right-click/drag/Shift-click dispatch,
  source-sized options, hex/prefix edits, cancel/accept, reload/unload protection,
  independent dirty state, external-change failures and failed-load preservation.
  Resize tests stage map and event changes, verify twice-block translation and
  preserve both on invalid resize. Save tests exercise the original ordering and
  event-only saves with an externally changed clean map. Automatic-load tests
  cover discovery precedence, sibling fallback, disabled/New behavior, and malformed
  script warnings while retaining the newly loaded map. Warp tests cancel or
  discard edits and load destination map/events without writing the source script.
  Five inspected snapshots cover decimal/hex prefix forms, Windows/Unix buttons,
  and 16/32-point overlays. Mathematical checks verify positions, form dimensions
  and original negative drag-boundary behavior. The twelve embedded source marker
  backgrounds were decoded locally: every pixel matches a flat alpha-128 fill
  and one-pixel border; GUI rectangles reproduce those colors without bitmap assets.
  These are source and headless egui checks, not native FLTK comparisons.
  The local ignored New Bark Town GUI integration auto-loads the pinned real
  project's events, compares them to a direct backend load, edits through Event
  Options and saves/reopens a temporary script copy, leaving the real project
  unchanged. It is not a native platform test. Existing snapshot changes are
  confined to the resize dialog's now-correct event help text and resulting width.

- `MapPalettePlan` orders default and special imports, metadata selection and
  roof colors without global mutable state. `begin_for_map` takes an explicit
  palette session; regular headless opens remain fresh. Project tests cover
  partial set overlays, edited-flag transitions, contextual selection, and atomic
  failed steps that can be retried/skipped. `palette_session_tests` covers actual
  Open/New workers, pre-map edits, successive opens, per-file Load/Skip decisions,
  Escape skipping, the missing-default edited-flag quirk, Custom retention,
  independent roof prompts, optional roof warnings and preservation of the old
  map/colors/preview on required-import failure. The staged import prompt has an
  inspected snapshot; the existing visual baselines remain unchanged. The pinned palette integration
  carries colors from New Bark Town to Violet City with imports skipped and
  compares accepted imports/full renders against a fresh load. These are project
  adapter and headless GUI tests; native workflow verification is still pending.

- `MasterTilesetDocument::prepare_replacement` implements the asset reload used
  by `Main_Window::change_tileset_cb`, retaining the palette document, roof name,
  256-tile mode, sticky monochrome mode and enabled priority mode. New assets are
  clean; `tileset_switch` tests numbered labels, collisions, edited colors, roof
  reload and failed/malformed loads. It writes no files and leaves the source
  document intact. `tileset_workflow_tests` covers the menu/shortcut, individual
  dirty-domain warnings, both cancel stages, same-name reload, smaller/larger
  blocksets, map undo preservation, valid/invalid copied-block behavior and
  failure recovery. Two inspected snapshots and rectangle assertions follow
  `Tileset_Options_Dialog` and `Option_Dialog` geometry. The ignored real-project
  test switches Johto → Kanto → house → Johto with edited session colors, compares
  complete map renders and collisions against fresh loads, and leaves pinned
  project files unchanged. Native dialog/keyboard behavior remains unverified.
- `roof_workflow_tests` verifies Tools → Change Roof, Ctrl/Cmd+Shift+H, both
  cancellation stages, same-name reload, sticky dirty state, `(none)`, failed-load
  preservation and rendered previews without changing map history, block edits,
  base pixels or palette edits. Button rectangles follow `Option_Dialog::refresh`
  and `Roof_Options_Dialog::refresh_content`: 280×80 client area, 22px controls,
  80×22 buttons at y=48 with platform ordering. Two snapshots are inspected;
  existing snapshot differences are confined to the added Tools menu. These are
  headless egui tests, not Linux/macOS/native child-window verification.
  Project tests cover suffix discovery/deduplication and atomic failed reads.
  The pinned pokecrystal roof integration switches through azalea, goldenrod,
  new_bark, olivine and violet in one document, comparing full rendered maps to
  fresh loads and restoring base graphics with `(none)`. It reads the reference
  assets without changing them. This is adapter evidence, not a run of upstream.
- `PaletteBank` supports atomic RGB8/hex, RGB5 channel, quantized copy and exact
  live swap commands. `PaletteDocument` tracks six sets, independent edited state,
  bounded imports and explicit conflict-checked exports. CLI tests cover preview,
  required/stale input hashes, explicit output overwrite hashes and RGB5 reopen.
  `tools/palette_editor_oracle.py` extracts five original functions: 256 clipboard
  cases and 32 full-color/monochrome exports match compiled C++. Its harness
  explicitly corrects the incomplete set-name table while retaining the export
  function. Core tests separately cover channels, hex, malformed atomic batches
  and hue order. The GUI evidence is listed below.
  A real pinned New Bark Town context retains all sets, switches rendering to an
  edited night roof color, exports/reloads RGB5, and leaves project data and map/
  tile dirty state unchanged. This validates the project adapter, not the original
  GUI's load/export interactions or cross-map session behavior.
- `palette_editor_tests` covers color and monochrome draft accept/cancel,
  unchanged acceptance, channel/hex input, RGB5 clipboard snapshots/live swap,
  six-set selection, pre-map editing, preview refresh, explicit export, load
  replacement confirmation and malformed-input preservation. Four inspected
  snapshots use source text-width equations, 20px swatches and 21px grid pitch,
  with Windows/Unix button order. Windows native smoke testing confirms
  Ctrl+Shift+L, owner-bound ChooseColorW initialization from DEFFDE, Escape
  retaining that color, and accepting red returning FF0000 to the draft; the
  outer draft was cancelled and no files written. Native palette file dialogs,
  custom-color persistence, Linux/macOS runtime and cross-map palette lifecycle
  remain unverified. The native chooser bridge is isolated from the unsafe-free
  core/project/GUI/CLI crates.
  All 31 current GUI tests pass in the serialized Windows snapshot suite.

- Windows local workspace tests, Clippy and rustdoc passed during foundation work.
- `tools/reference_oracle.py` extracts the original conversion function from Git,
  and all default palette constants without modifying them; compiler invocation and generated provenance are retained
  locally. `tests/reference.rs` exhaustively compares its output. CI has a dedicated
  reference job; the default test command explicitly reports this test as ignored.
- `tools/block_brush_oracle.py` compiles the unmodified master placement callback
  with minimal UI stubs. All 65,536 combinations of tile mode, modifier pattern,
  tile ID and click position match Rust, including byte wrap and clipped writes.
  This verifies the placement math, not FLTK event dispatch. Block editing has
  separate dirty state and no map undo entries. GUI tests cover draft cancellation,
  acceptance, preview refresh, unsaved guards, save and external-change conflicts.
  Both platform button orders have reviewed synthetic snapshots. The 466×304
  content layout and 16/24-pixel cells come directly from Block_Window::initialize.
  The current dialog is embedded; native child-window verification is still pending.
- Collision tests cover full/base name and ASM/INC/binary precedence, four-quadrant
  ordering, short-row defaults, trailing data preservation, copy/swap/resize,
  ASM comment/line-ending preservation, and paired save conflicts. GUI tests edit
  both named fields and all four hex spinners. The CLI previews paired copies and
  requires independent overwrite hashes. A real Johto blockset/collision copy
  changes the intended first row while leaving the pinned sources untouched.
  Failure injection verifies rollback and retained backups when an external writer
  prevents rollback. Process termination/power-loss recovery is not yet verified.
- `block_workflow_tests` exercises snapshot paste versus live-source-ID swap,
  sidebar drag-to-swap and Ctrl-drag-to-copy, invalid drops, count-dialog cancel
  and unchanged acceptance, shrink selection/clipboard reset, growth to 256,
  collision movement, preview refresh and paired persistence. Map IDs and map
  history stay unchanged, including references beyond a shrunken blockset.
  These are source-derived GUI tests, not a compiled C++ differential.
  The count dialog uses the original 194×80 client-area equations with reviewed
  Windows and Unix button-order snapshots. Clipboard event and empty-system-
  clipboard key-release paths are tested headlessly; native dispatch is pending.
- The shared `Tileset` draft model now implements pixel paint/fill/replace/swap,
  palette/priority changes, snapshot and live-source copy/swap, deletion, clipped
  RGB image paste and trailing-unused-tile trimming. Commands validate atomically.
  `tools/tile_editor_oracle.py` compiles seven unmodified upstream functions with
  UI stubs. Rust matches 30,720 fill/replace/swap cases and 88 RGB-paste cases
  across both tile modes and reserved-row/boundary origins. This does not verify
  FLTK dispatch, native clipboard transport, tile-editor controls, or saving.
  Source-derived tests cover palette-map serialization and grayscale export
  pixels, including bank gaps, odd entry padding, partial-line newline behavior,
  and rejected unsafe layouts. These APIs are experimental, not a completed editor.
- `Roof` and `RoofDocument` provide independent nine-tile editing and saving.
  Core tests cover atomic invalid batches, copy/swap, contextual palette sync,
  safe image clipping and file-order export. The compiled upstream oracle adds
  288 exact-8×8 roof RGB-paste comparisons; other dimensions use the documented
  safety correction. Project tests cover 24×24 two-bit PNGs, imported-source
  preservation, separate base/roof dirty state, save/reopen and external changes.
  CLI tests cover no-write preview, missing/stale apply hashes, rejected commands
  and reopened pixels. A temporary copy of pinned New Bark Town roof graphics
  preserves every other pixel and leaves the reference source untouched.
  `roof_editor_tests` covers cancel/accept, preview refresh, save/conflict/dirty
  guards, private copy/live-source swap, bounded image paste and reversed rows.
  Inspected Windows/Unix button-order snapshots use Roof_Window::initialize's
  226×228 client area, 16px tile cells and 18px pixel cells. These are headless
  tests; native child windows, clipboard transport and DPI remain unverified.
  All 25 current GUI tests pass with snapshot comparisons serialized on Windows.
- `MasterTilesetDocument` connects the draft model to discovery, contextual
  rendering and PNG/palette persistence. Source-derived tests cover two-bit
  grayscale PNG headers and decoded pixels, 2bpp import preservation, roof/base
  separation, compound outputs, save/reopen, ASM preservation, priority gaps,
  ambiguous-ASM refusal, duplicate destinations and external-file conflicts.
  The CLI tests preview without writes, required/stale hashes, block-definition
  changes, and apply/reopen. Failure injection on the third file verifies rollback
  of both a replaced file and a newly created one. A temporary copy of pinned
  Johto graphics/palette/block definitions saves and reopens with the intended
  pixel/palette change and all other tile pixels preserved; fixture files remain
  unchanged. PNG compressed bytes are not claimed to match libpng. This is not
  an original-application save differential, native GUI validation or power-loss
  recovery test. General journal replay remains pending.
- `tileset_editor_tests` covers draft cancel/accept, unchanged acceptance dirtying,
  paint/fill/replace/swap/pick gestures, palette hotkeys, priority/deletion,
  private snapshot paste versus live-source swap, swap auto-repeat suppression,
  empty-clipboard key-release fallback, drag swap/copy, save/reopen and conflicts,
  dirty guards, unchanged map/block history, grayscale copy output and clipped
  image-paste event delivery. Geometry assertions check the 16px sheet and 18px
  pixel grids, reserved-slot mode restrictions and total modal bounds. Windows
  and Unix button-order snapshots are reviewed. The 466×304 client area comes
  directly from Tileset_Window::initialize. These are headless GUI tests backed by
  the separately compared core operations, not FLTK/native event differentials.
  Native image clipboard transport, separate child windows, priority overlays
  and comprehensive DPI/theme behavior remain pending. The initial code-drawn
  clipboard symbols have since been replaced with reviewed upstream icon pixels;
  their provenance and tests are described above.
- Headless GUI tests exercise toolbar state, accessible open options, a complete
  paint/undo/redo/save flow, New Map, resize accept/cancel, and source-derived geometry. Optional visual tests:
  `cargo test -p polished-map-gui --features visual-tests -- --test-threads=1`.
  A local Windows parallel visual run exited with `STATUS_INVALID_HANDLE` before
  completing; all 22 tests passed serially. CI now serializes the visual suite to
  avoid concurrent renderer setups. The native-handle failure's root cause is
  unconfirmed; no assertion or snapshot comparison has been disabled.
- CLI rendered the unmodified upstream master Kanto example at 5760×4320. The PNG
  was visually inspected locally; this is a smoke test, not a pixel differential.
  Neither the game assets nor resulting PNG are included in the repository.
- Dependency license/source/advisory checks pass with the explicit maintenance
  exceptions in `docs/dependencies.md`; duplicate versions remain warnings.
- Initial foundation CI passed on Windows, Linux and macOS, including MSRV 1.95,
  Linux snapshots, dependency policy and the C++ bitplane oracle. This verifies
  builds/headless tests, not native platform interaction. No packaged release.
- Pinned pokecrystal fixture `bd983395006298e976ae1871ef092e87f3385506`:
  236 map files discover/open/render across 35 tilesets; 18 shared block files need
  explicit dimensions/tileset from an owning map. The real-project test checks all
  254 top-level .blk files without modifying them. New Bark Town's unannotated
  render was inspected, including project palette and roof colors. All 236 maps
  with named metadata also render with project palettes. Full pixel parity remains
  pending. All default palette channels match the compiled upstream color table.
- Compound graphics tests cover full/base path precedence, optional companions,
  malformed companion errors, 16-tile row padding, palette indices, both slot
  layouts and the original first-256-slot limit. Roof tests cover slots 0x0a–0x12,
  exact nine-tile validation and padded LZ rejection. These are source-derived
  vectors, not compiled C++ composition comparisons. CLI PNG output and GUI
  option entry exercise the same compositor. All five roofs in pinned pokecrystal
  load; New Bark matches the base johto graphics and the four alternatives change
  the rendered roof. Graphics sources remain unmodified.
- Native Windows smoke test: launched the built eframe app; opened NewBarkTown
  through the Windows picker; checked inferred dimensions/tileset; saved a copy
  through Ctrl+Shift+S; observed the native overwrite prompt; declined replacement
  and canceled Save As; closed the app. Copy and source SHA-256 both remained
  `41a1012a051d22ade1b6cdb14577c4b6ad3e195be7024d6f3a020a9f7f5fedd3`.
  The pinned reference checkout remained clean. This does not verify DPI changes,
  clipboard, child viewports, or Linux/macOS native dialogs.
