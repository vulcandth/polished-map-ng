# Compatibility evidence

The [roadmap](roadmap.md) owns current task status and execution order. Use the
[current coverage matrix](#current-coverage-matrix) for the implementation summary.
The feature records below retain source calculations and verification results;
test counts and local-run observations describe those individual changes, not
the current size or acceptance status of the whole suite.
References below to remaining M4 visual/font matching are deferred under the
[user-approved aesthetic exception](deviations.md#remaining-m4-aesthetics-deferred).
Functional acceptance remains required in M5–M7.

## Master source inventory

M1f reconciles the user-facing feature families in all 31 master `src/*.cpp`
translation units and their declarations at the pinned revision. It classifies
coverage; it does not certify every branch or close master acceptance. The
[menu inventory](menus.md) owns the eight menu groups and their commands, and
[preferences](preferences.md) owns stored settings. The table below connects
those commands, editors, gestures and formats to implementation/test evidence
and explicit remaining roadmap work. Every implemented row still needs M7
application acceptance; native interactions additionally need M6.

| Original source / feature family | Current implementation and evidence | Remaining owner |
|---|---|---|
| `main.cpp`: first path argument, startup theme/position/fullscreen/maximize | [GUI entry point](../crates/gui/src/main.rs), [preferences](preferences.md), [window title](../crates/gui/src/window_title.rs) | M5 startup/native restoration; additional themes remain I3 |
| `main-window.cpp`: New/Open/Recent/Clear/Close, Save/Save As and each domain save, Exit | [File workflows](../crates/gui/src/file_workflows.rs), [map-open tests](../crates/gui/src/map_open_tests.rs), [recent paths](../crates/gui/src/recent_maps.rs), [persistence tests](../crates/project/tests/persistence.rs) | M4/M5 close/focus/lifecycle |
| `main-window.cpp`: all eight menu groups, 27 toolbar positions, availability, shortcuts and mode changes | [Menus and tests](../crates/gui/src/menus.rs), [toolbar tests](../crates/gui/src/toolbar_tests.rs), [menu inventory](menus.md) | M4 popup traversal/mnemonics/placement; I3 deferred theme commands |
| `main-window.cpp`, `map.cpp`: map paint, fill, replace, swap, drag history, Undo/Redo, resize and event translation | [Map model](../crates/core/src/map.rs), [editing](../crates/core/tests/editing.rs), [resize](../crates/core/tests/resize.rs), [event workflows](../crates/gui/src/event_workflow_tests.rs) | M4 resize client/widget fidelity |
| `map-buttons.cpp`, `widgets.cpp`: sidebar selection/right-release editing, map/tile/pixel picking and painting, internal swap/Control-copy drags, enabled Drag and Drop | [Block gestures](../crates/gui/src/block_workflow_tests.rs), [tile gestures](../crates/gui/src/tileset_editor_tests.rs), [drag tests](../crates/gui/src/drag_tests.rs), [palette tests](../crates/gui/src/palette_editor_tests.rs) | M4 native input delivery and widget visuals |
| `main-window.cpp`, `widgets.cpp`: external URI/file drops and workspace eligibility | [Positioned drops](../crates/gui/src/external_drop.rs), [native adapter evidence](#native-external-file-drops) | M3 native file-manager/DPI/cancellation and clipboard acceptance |
| `main-window.cpp`, `ruler.cpp`, `gameboy-screen.cpp`: rulers, grids, IDs/hex, priority, event/warp overlays, status, zoom, pan, hover guide and numeric hotkeys | [Rulers](../crates/gui/src/rulers.rs), [status](../crates/gui/src/status_bar.rs), [hotkeys](../crates/gui/src/block_hotkeys.rs), [event hover tests](../crates/gui/src/event_workflow_tests.rs), [zoom tests](../crates/gui/src/zoom_tests.rs) | M4 fonts and native scrolling/resize/DPI; M3 window opacity |
| `block-window.cpp`, `metatile.cpp`, `metatileset.cpp`: tile selection, all eight modifier brushes, four collision fields, block copy/paste/swap, count and paired saves | [Block editor](../crates/gui/src/block_editor.rs), [block model tests](../crates/core/tests/blockset.rs), [project blockset tests](../crates/project/tests/blockset.rs), [collision tests](../crates/project/tests/collisions.rs) | M4/M6 integrated window acceptance |
| `tileset-window.cpp`, `tile.cpp`, `tileset.cpp`: pixel paint/fill/replace/swap, tile copy/paste/live swap/delete, swatches 1–4, Alt+1–8 palettes, priority and clipboard images | [Tile editor](../crates/gui/src/tileset_editor.rs), [editor tests](../crates/gui/src/tileset_editor_tests.rs), [tile model tests](../crates/core/tests/tileset.rs), [project tileset tests](../crates/project/tests/tileset.rs) | M4/M6 integrated window/clipboard acceptance |
| `roof-window.cpp`: nine-tile selection, pixel tools, internal/image clipboard, reversed roof display and independent save | [Roof editor tests](../crates/gui/src/roof_editor_tests.rs), [roof workflows](../crates/gui/src/roof_workflow_tests.rs), [project roof tests](../crates/project/tests/roof.rs) | M4/M6 integrated window/clipboard acceptance |
| `palette-window.cpp`, `colors.cpp`: color/monochrome grids, RGB5 sliders/spinners, RGB8 hex, Windows chooser, color copy/paste/live swap/drag, six palette sets and export | [Palette editor](../crates/gui/src/palette_editor.rs), [palette editor tests](../crates/gui/src/palette_editor_tests.rs), [palette model tests](../crates/core/tests/palette_edit.rs) | M4 native chooser; I2 shared graphical picker |
| `option-dialogs.cpp`: Open/New size, tileset, roof and metadata choices | [M1a–M1e evidence](#open-map-size-selection), [Map Options tests](../crates/gui/src/map_options/tests.rs) | M4 native dialog/widget fidelity |
| `option-dialogs.cpp`: Resize dimensions/nine anchors, Add/Sub block count, Overworld Map Size | [Resize tests](../crates/core/tests/resize.rs), [block workflows](../crates/gui/src/block_workflow_tests.rs), [overworld dialog/tests](../crates/gui/src/overworld.rs) | M4 client measurements and native windows |
| `option-dialogs.cpp`: Change Tileset and Change Roof, selection/defaults, cancel and dirty guards | [Tileset workflows](../crates/gui/src/tileset_workflow_tests.rs), [roof workflows](../crates/gui/src/roof_workflow_tests.rs), [dialog metrics](#remaining-option-dialog-client-metrics) | M4 source widget painting and native fidelity |
| `event.cpp`, `map-events.cpp`, Event Options: 22 macros, coordinates/prefix/suffix, hover, drag, options, double-click script dispatch, Shift-click warp, load/save/view/reload/unload | [Parser/model tests](../crates/project/tests/events.rs), [GUI workflows](../crates/gui/src/event_workflow_tests.rs), [source differential](../crates/project/tests/reference.rs) | M4 native editor/window |
| Print Options, `image.cpp`, `main-window.cpp`: five independent flags, Cancel/Copy/Export, image composition and PNG output | [Print dialog/tests](../crates/gui/src/printing.rs), [source comparisons](../crates/project/tests/reference/printing.rs), [real-project Print](../crates/project/tests/real_print.rs), [PNG export](../crates/project/tests/png_export.rs) | M4 fonts/native clipboard/chooser; M6 integrated workflows |
| `tiled-image.cpp`, `parse-asm.cpp`: 2bpp, padded LZ, CHR/ASM, PNG depths/interlacing/platform alpha, bounded errors | [Codec tests](../crates/project/tests/codecs.rs), [reference suites](../crates/project/tests/reference.rs), [PNG imports](../crates/project/tests/png_import.rs) | M7 application interoperability; safety differences in [deviations](deviations.md) |
| `palette-map.cpp`, `colors.cpp`, `tileset.cpp`: tilepal/priority, defaults, special/roof colors, compound graphics, 192/256 slots, trimmed PNG/ASM exports | [Graphics tests](../crates/project/tests/graphics.rs), [palettes](../crates/project/tests/palettes.rs), [tileset export tests](../crates/project/tests/tileset.rs) | M7 complete project save/reopen; widget painting remains M4 |
| `config.cpp`, `utils.cpp`: project root, filename conventions, path precedence, constants/metadata, dimensions, compound suffixes and numeric ordering | [Discovery](#binary-map-metadata-discovery), [asset paths](#legacy-asset-path-selection), [asset ordering](#opennew-asset-selectors), [discovery tests](../crates/project/tests/discovery.rs) | M6 native Unicode/case/permission behavior; safety differences in [deviations](deviations.md) |
| `widgets.cpp`, `hex-spinner.cpp`: defaults, numeric input, step/repeat/wrap/clamp, middle reset, labels, buttons, dropdowns, scrollbars and help HTML | Specialized GUI modules above and [popup tests](../crates/gui/src/menu_popup/tests.rs) provide partial coverage | M4 source widget rendering/focus; standard egui widgets are not evidence of FLTK parity |
| `modal-dialog.cpp`: errors, warnings, success, unsaved confirmations and About | [Message dialog tests](../crates/gui/src/message_dialog/tests.rs), [About](../crates/gui/src/about.rs), individual workflow tests | M4 source widget painting and native child-window fidelity |
| `help-window.cpp`: full help text, scrolling, external links, resizable nonmodal window and dismissal | [Help implementation](../crates/gui/src/help.rs), [Help tests](../crates/gui/src/help/tests.rs) | M4 native resize/focus/link dispatch |
| `themes.cpp`, icon/resource/version headers: Greybird/Dark, fonts, inactive colors, application identity and imagery | [Fonts](../crates/gui/src/fonts.rs), [menu marks](../crates/gui/src/menu_popup/marks.rs), [asset audit](assets.md), [notices](../NOTICE.md) | M4 native fonts/widget drawing; I3 other themes |
| `preferences.cpp`, startup/exit callbacks: settings, recent files, geometry, legacy stores and `meta128` acknowledgment | [Preferences](preferences.md), [preference tests](../crates/gui/src/preferences.rs), [recent paths](../crates/gui/src/recent_maps.rs) | Legacy import implemented; M5 native storage/restart/placement acceptance |
| `directory-chooser.cpp`, native file/clipboard/browser/editor calls | [Platform adapters](../crates/platform/src/lib.rs), [native adapter evidence](#native-clipboardbrowser-adapter-wiring), workflow tests | M4/M6 actual OS delivery; P1/P2 packaging/clean-machine checks |

The reconciliation exposed three gaps previously hidden by broad feature labels:
middle-click defaults and numeric-widget behavior; Event Options' input-mode
and default-button behavior; and save-success/128-block-warning routes.
These gaps are implemented in M1g/M1h and covered by the evidence below.
The inventory is reconciled; the remaining owners in the table still apply.
Source `Default_Spinner`/`Default_Hex_Spinner` reset
on middle release, `Hex_Spinner` arrows wrap at bounds while typed values clamp,
and palette defaults are refreshed when a swatch is selected rather than after
every channel edit. Event prefix/suffix fields switch between normal and wrapped
multiline input based on their measured height. Source `save_metatileset` shows
success, then the warning when collisions exist, count exceeds 128 and `meta128`
is false; acknowledgment stores that preference. NG now implements these modal
routes in addition to its status tooltip.

## Save feedback and block-limit acknowledgment

[Save feedback](../crates/gui/src/save_feedback.rs) adapts pinned master
`Main_Window::{save_cb,save_map,save_metatileset,save_tileset,save_roof,
save_event_script,export_palettes}`. The existing
[message dialog](../crates/gui/src/message_dialog.rs) supplies source-derived
client dimensions and Enter/Escape/OK dismissal. Original message artwork is imported; M4 native child-window/font fidelity
remains a separate requirement.

- Save processes dirty tileset, roof, blockset/collisions, events and map in that
  order. A clean map is skipped when another domain is dirty; an entirely clean
  Save acknowledges the map. Each modal must close before the next domain runs.
- Map and Save As report the committed basename. A written map's overworld
  capacity warning precedes success; a clean ordinary save does not repeat it.
- Blocksets report both filenames when collisions exist. Success precedes the
  source 128-block warning, which requires collisions, count greater than 128
  and an unset `meta128`. Display alone does not acknowledge it; OK, Enter or
  Escape does. The acknowledgment survives map Close and the preference
  save/restore cycle. Older v1 records missing it default to false.
- Dirty tilesets acknowledge the full main PNG name (including compound suffix)
  and then the palette map in color mode. Clean tileset/roof messages use the
  extensionless source name. Dirty roofs and event scripts use the saved basename;
  palette exports report `Exported <basename>!`.
- Parent editing, Save/Close shortcuts and native root-window close requests
  are blocked while feedback or a continuation is pending. A dismissed error
  cannot resume queued writes. Cancellation and failure preserve dirty state
  and emit no success/warning for the failed domain.

NG retains staged multi-file writes and stops subsequent domains after failure
or chooser cancellation. Source `save_cb` instead calls the subsequent domains
after a callback returns, including after failure. Within a tileset save, NG
finishes the staged domain write before either acknowledgment, whereas source
acknowledges PNG success before attempting the palette-map write. These safety
differences are recorded in [deviations](deviations.md); Save is not a transaction
across every domain, and earlier successful domains remain saved.

[Workflow tests](../crates/gui/src/save_feedback/tests.rs) exercise all five dirty
domains through successive keyboard acknowledgments, clean saves, event no-op
behavior with an externally changed file, compound/color/monochrome names,
map warning order, committed Save As names, cancellation result handlers, invalid
paired destinations and conflict interruption. Counts 128/129/256 with and
without collisions cover the warning threshold; dirty saves/conflicts and
in-memory eframe persistence cover acknowledgment lifetime. Existing editor tests
now dismiss actual success dialogs before further editing. Reviewed
[success](../crates/gui/tests/snapshots/save-map-success.png) and
[block-limit warning](../crates/gui/tests/snapshots/save-block-limit-warning.png)
snapshots preserve the measured message layout with original message artwork. Native chooser UI,
on-disk restart persistence and complete original-application save/reopen remain
M5–M7 acceptance work; these are synthetic/headless tests.

## Shared numeric and Event Options controls

The shared [integer input](../crates/gui/src/numeric_input.rs) replaces DragValue
in Open/New, Resize Map/Blockset, Overworld Map Size, Event Options, binary
collisions and palette RGB5 channels. It follows master `hex-spinner.cpp` and
pinned FLTK `Fl_Spinner`, `Fl_Input` and `Fl_Repeat_Button`:

- Text remains a draft until Enter or focus loss. Enter commits and selects the
  number without activating the dialog's default button. In-range spelling is
  retained; out-of-range typed values clamp and refresh the display.
- Up/down arrows wrap at each control's limits. Decimal arrows first commit the
  pending draft; hex arrows use the stored value. Pointer buttons step on press,
  wait 0.5 seconds, then repeat every 0.1 seconds. Leaving the button suspends
  repetition; reentry restarts its delay. Disabled and interrupted captures stop.
- Hex input accepts a leading sign and hexadecimal digits, with uppercase
  two-digit minimum formatting on programmatic updates. Paste trims ASCII
  whitespace, validates the entire input and replaces the entire field. Invalid
  keys preserve selected text. Middle-click resets discard stale numeric drafts.
- Palette horizontal sliders use integer steps, clamp at 0/31 and respond to
  left/right keys. Up/down does not change their values. Full slider pointer
  geometry and widget painting still belong to M4.

Event Options measures its prefix/suffix once when opening. The measurement
adapts `fl_measure`/`expand_text_`: ASCII-space word boundaries, integer width
comparison, tab/control expansion, no splitting of an oversized first word and
no extra line for a trailing newline. Heights above 22 points choose wrapped
multiline input; other fields stay single-line. Editing does not resize or switch
the field. Wrapped fields scroll internally, Enter inserts a newline there,
and Enter in a short field or on the default OK button accepts. Escape discards
the draft. Actual native font metrics and child-window behavior remain M4/M6.

[Control tests](../crates/gui/src/numeric_input/tests.rs),
[event tests](../crates/gui/src/event_workflow_tests.rs) and
[palette tests](../crates/gui/src/palette_editor_tests.rs) exercise stepping,
repeat timing, clamp/overflow bounds, typed/pasted input, focus, cancellation,
all affected forms and preservation of unrelated source/map state. The earlier
middle-release tests remain regression gates. Updated rendered snapshots are
reviewed against the previous images; they establish the portable client, not
native platform acceptance. Numeric text is bounded to 64 characters and
overflow is handled deterministically; see [safety differences](deviations.md).

## Middle-click control defaults

The shared `default_value::DefaultValue` adapter follows pinned master
`Default_Spinner::handle`, `Default_Hex_Spinner::handle` and
`Default_Slider::handle` in `widgets.cpp`. Middle press captures a control;
release restores its stored value once, even after dragging outside it. Slider
presses also request focus. Disabled/obscured controls cannot capture; pointer
loss, lost focus and abandoned forms cannot leave a later release armed.

| Controls | Value captured for reset |
|---|---|
| Resize Map width/height | Map dimensions when the form opens |
| Resize Blockset count | Current block count when the form opens |
| Overworld Map Size | Current configured capacity when the form opens, not an unconditional 1300 |
| Event X/Y, decimal or hex | Coordinates when Event Options opens |
| Binary collision quadrants | Each loaded collision byte when Edit Block opens |
| Palette RGB5 spinners/sliders | Selected color's RGB8 channels divided by eight when a swatch is selected or a clipboard/drag transfer reselects it |

Palette channel, hex and native chooser updates refresh the current value/hex
display without changing that selection baseline. Paste, swap and drag transfers
capture the resulting selected color, rather than the old destination color.
Only the reset channel is quantized back through source `RGB5C`; the other
channels retain their exact RGB8 values. Cancel discards draft resets normally.
There are no resets on Open/New's ordinary `OS_Spinner` controls: source uses
default-capable spinners only in the rows above.

Eight [input tests](../crates/gui/src/default_value/tests.rs) exercise actual
egui pointer delivery, drag-away release, one callback per release, disabled/
focus/pointer-loss cancellation, interrupted/reopened dialogs, text-input cache
behavior, every control family, palette selection/clipboard/drag baselines and
unchanged source files/unrelated map state. This completes M1g1. Arrow
stepping/repeat/wrap, numeric parsing/clamping, Event Options input modes and
default-button delivery are covered in the shared-control section above. Source control painting and
native event delivery remain M4/M6 gates.

## Binary-map metadata discovery

`tools/discovery_oracle.py` compiles unchanged master `guess_map_constant`,
`Map_Options_Dialog::{guess_map_size,guess_map_tileset,add_map_size,add_valid_sizes}`,
the three `Config` metadata-path selectors and their string helpers. It retains
the original `Map_Attributes` definition and records snippet hashes. Widget
storage is inert; ASCII fixture paths use standard streams. ASM decoding and
native filesystem case handling are outside this harness. `--windows` selects
the original case-insensitive global-header matching branch; the default is Unix.

The differential covers 391 cases: all three dimension macros and constant-file
paths, both global header macros and paths, symbolic/decimal/hex/unknown tilesets,
filename dimension/tileset overrides, both individual-header paths with legacy
`map_header`/`db` forms, existing-file precedence (including non-matching files),
map-name/acronym boundaries, global header case matching, repeated headers,
shared block files and group-byte boundaries. Filename overrides short-circuit
the corresponding metadata reads. Existing higher-priority files suppress
lower-priority profiles even when they contain no matching map. The first matching
global header wins, including an unresolved tileset token.

Two observed differences were corrected: individual `db` headers read one
whitespace-delimited token, so `db OVERWORLD, 0` retains `overworld,` rather than
being silently repaired; and `newgroup` increments wrap through the original
`uint8_t` range. The public metadata field can represent the result without
changing its Rust type. Regular regression fixtures retain punctuation, group
values and source bytes. Unknown shared block filenames remain unattributed:
master does not follow `INCBIN` references to infer another map's header or size.
The GUI offers explicit choices; strict headless opening requires dimensions.

This establishes the listed binary-map metadata rules, not complete support for
every legacy project's asset layout or arbitrary assembly expressions. Source
allocation/error safety rules remain in force. Full native filesystem behavior and
application save/reopen acceptance remain open. Asset-path fixtures are covered below.

## Legacy asset-path selection

`tools/asset_paths_oracle.py` compiles unchanged pinned master `Config` path
functions and `remove_suffix`/`before_suffix`/`after_suffix`, recording source
hashes. The four reference tests in `tests/reference/asset_paths.rs` observe
public Rust loaders, their source files and proposed output paths. Fixture
ladders begin with competing candidates and remove them in precedence order;
every stage is compared with the original functions, including absent inputs.

| Source rules | Verified observations |
|---|---|
| `gfx_tileset_dir`, `tileset_path`, before/after and PNG paths | `MAPDATA` wins even when empty; PNG → 2bpp → CHR → LZ precedence; plain, before-only, after-only, combined and dotted companion names; missing companions; no main-graphics fallback to the base name; original PNG destinations |
| Metatile, collision and palette-map paths | All full-name candidates precede all trimmed-base candidates; modern, SECTION-split, pokered, Red++ and old-layout alternatives; binary/ASM collision selection; absent collisions; missing palette-map monochrome fallback |
| Roof paths and PNG destination | Roofs remain under `gfx/tilesets/roofs` with either project graphics directory; extension precedence and PNG output; selected LZ roofs retain the original padded-length rejection |
| Event-script and tileset-constants paths | All four event locations and the final missing-script fallback; all three constants files via numbered dropdown labels; absent constants leave undecorated labels |
| Background, roof-color and special palettes | Modern/old default paths; map → landmark → full tileset → base tileset special precedence; absent imports; unsaved maps and missing landmarks do not invent filename/landmark palettes |

All four differential tests pass locally. This audit required no production
path-selection changes. The harness uses standard filesystem existence checks
for readable ASCII files/directories and an adapter for the tested filename
extension replacement; it does not prove native permissions, Unicode/case or
symlink behavior. Its `Config::monochrome` is false; explicit monochrome import
behavior is covered separately by palette/graphics tests. Windows canonical
prefixes are normalized only at the test boundary. The CI reference job repeats
the comparisons on Linux. Full application interoperability and native platform
acceptance remain M6/M7 requirements; synthetic layout names do not certify
complete compatibility with every historical game project.

## Open map-size selection

`MapSizeOptions` and the GUI Open form adapt pinned master
`Map_Options_Dialog::{add_valid_sizes,add_map_size,guess_map_size}` in
`src/option-dialogs.cpp`. Candidate dimensions are factor pairs of the decoded
block count, descending by width, restricted to 1–255 on each axis. The initial
selection starts at `(count - 1) / 2` and searches toward wider candidates for
width/height multiples of 10/9. Filename or metadata dimensions select an
existing pair or append a new one, even when the area differs from the file.
That mismatch remains the loader's warning/error decision. Empty or unfactorable
inputs use 10×9 and free spinners; New retains the same default and spinners.

The dialog shows decoded bytes for `.map` input, sharing the loader's bounded
binary/ASM decoding. A read/parse error preserves the current map. Dimensions
outside 1–255 are rejected under the existing malformed-input safety policy.
Only explicit GUI choices resolve ambiguous dimensions; the headless map-open
contract still requires metadata or supplied dimensions.

`tools/map_sizes_oracle.py` extracts unchanged `add_valid_sizes`, `add_map_size`
and `valid_size` from the pinned source, retaining LGPL attribution and snippet
hashes. A minimal dropdown stub observes the original list and selected index.
The reference test compares every byte count 0–65,026 (65,027 cases), covering
all representable map areas, primes, empty input and the first oversized area.
Project tests cover metadata/filename precedence, appended and duplicate choices,
decoded ASM data, invalid dimensions and no-write inspection. GUI interaction
tests choose an alternate factor pair and open both binary/ASM maps, verify
fallback/New controls, and retain edits on cancellation. Existing oversized,
short-input and palette-warning flows remain covered.

This establishes size-choice behavior, not complete Open/New parity:
the complete feature inventory and native dialog delivery remain M1/M4 work.
Client geometry is covered below; no native GUI validation is claimed here.

## Open/New asset selectors

The Open/New tileset and roof controls follow master
`Map_Options_Dialog::{limit_blk_options,add_tileset,add_roof,guess_tileset_names}`.
Both selectors consume full filenames in FLTK numeric order before removing
suffixes and deduplicating labels. Tilesets ignore `.colored.png` previews and
recognize `.2bpp.unique.lz`; roofs retain their source-specific extension list.
`MAPDATA` takes precedence over `gfx/tilesets`. Numbered constants decorate
two-digit names (for example `01: JOHTO`) while the loader retains the filename.
Hint matching uses the base before the first dot and numbered aliases, including
the original last-added-choice assignment when a matching duplicate is scanned.
The GUI defaults to the first tileset when no hint matches; headless metadata
discovery continues to report no match rather than silently defaulting.

Roof choices start at `(none)` on every Open/New and the control is disabled
when the roof inventory is empty. The existing NG project-folder field requires
explicit Reload project options after a change; stale or empty tileset choices
cannot be submitted. Failed refreshes preserve the previous choices and loaded
document. The normal load worker still validates selected graphics and files.

`tools/asset_sort_oracle.py` compiles unchanged FLTK 1.4.5 `numericsort` and
`fl_utf8decode`, including its default decoding constants/table, to compare
38,809 ordered pairs of valid UTF-8 filenames. The corpus covers case, leading
zeros, huge numeric runs, compound suffixes, accented/CJK text and non-ASCII
digits. Only the case-sensitive path used by `fl_filename_list` is exercised.
Source-equivalent comparisons get an explicit filename tie-break in the port;
see [deviations](deviations.md). This does not assert native directory-enumeration
order for ties. Project fixtures verify selector membership/order, labels,
duplicate/default/compound selection and root precedence. GUI input tests select
tilesets/roofs, clear roofs, refresh roots, guard stale/empty choices and check
New defaults. Source client layout is covered below; native chooser/window
behavior remains open.

## Open/New client layout and keyboard actions

`map_options::Layout` follows pinned master `Option_Dialog::refresh` and
`Map_Options_Dialog::refresh_content`. The source client is 300×158 logical
points. Header, dimensions, tileset and roof occupy 22-point rows at y=10, 36,
62 and 88. Controls use the maximum measured Size/Width and Tileset label width,
plus four points from `text_width(..., 2)` and the ten-point window margin.
Spinner width measures `999`; the size selector measures `9999 x 9999`, each
with four padding points and a 22-point control allowance. Asset widths use
the widest label plus twelve padding points and the same control allowance.
Height placement, decoded-byte label bounds and the three-point outside-label
gap follow the original and FLTK `Fl_Group::draw_outside_label` calculations.

OK and Cancel are 80×22 at y=126; Windows places OK at x=116 and Cancel at
x=210, while Unix reverses them. Measurements use the platform's 12/13-point
UI font. The caption is Map Options; the first row is the filename or New Map
with a colon. The visible OK button keeps descriptive Open/Create accessibility
labels. NG's project folder and tile-limit overrides live in a separate,
initially collapsed Project options section outside this measured client.
The embedded caption and that section are not part of the upstream 300×158
window-size claim; native frame/widget styling and focus remain M4/M6 work.

Independent metric vectors assert every source rectangle for both size-control
branches and platform button orders. Rendered tests check actual spinner,
selector and button bounds against those rectangles at 1×/2× with both platform
font sizes. Four inspected snapshots cover Open/New, light/dark and both scales.
Enter accepts valid choices once; Escape cancels. An open dropdown consumes its
own Escape before the dialog can close. Root changes are revalidated after the
advanced controls so Enter cannot submit a stale inventory. Existing Open/New,
dirty-state, cancellation, warning and initial-scroll tests cover integration.

## Remaining option dialog client metrics

The [Resize Map client](../crates/gui/src/resize_dialog.rs) follows
`Resize_Dialog::refresh_content` and `Option_Dialog::refresh`: 220×132 points,
width/height controls at y=10/36, 22-point control height, and a 3×3 anchor grid
of 24-point buttons with two-point gaps. The source reports 74 points of content
height even though the anchors occupy 76; the port retains that arithmetic.
OK/Cancel are 80×22 at y=100, x=36/130 with platform order. The first-show default
focus goes to OK. Resizing consequences are in its tooltip rather than extra
rows that enlarge the original client. All nine anchors preserve draft-only
selection until acceptance, and Escape discards the form.

Resize Blockset, Change Tileset, Change Roof and Event Options now measure and
draw with the same source font size: 12 on Windows, 13 on Linux/macOS. Integer
text measurements include both padding edges (`text_width(s, 2)` adds four).
Resize Blockset's previously two-point allowance is corrected. Shared integer
inputs honor the dialog font override. Event headings align left inside their
source rectangles; coordinate and prefix/suffix labels sit three points outside
the controls, with top alignment for multiline fields. Existing long-selector
clipping remains the separately recorded [difference](deviations.md).
Multiline field heights include the original theme's input border: six points
for Greybird, four for Dark, measured once at opening. A two-theme test verifies
both field heights and the resulting total client height without editing data.

[Resize tests](../crates/gui/src/resize_dialog/tests.rs) check independent formula
vectors, both platform button orders at 1×/2×, actual text font sizes, all anchor
hit regions and cancellation. Existing event/asset/blockset workflow tests retain
edit, keyboard, default-value and persistence coverage. [Native hosting](#native-dialog-hosting-and-ownership)
is implemented; source widget painting, native rasterization and integrated
window-manager acceptance remain M4/M6 work.

## Error, warning and success message dialogs

The six one-button message routes now share the pinned master's
`Modal_Dialog::refresh` empty-subject/icon layout: generic errors, automatic
roof-color warnings, oversized-map acknowledgment, overworld-buffer warnings,
Print success and manual roof-color success. `Main_Window::initialize` sets the
280/700 width range. Text is measured at 620 points; client width is
`max(ceil(text_width) + 80 + OS_FONT_SIZE, 280)`, with a 50×50 icon slot at
(10,10) and body at (70,10). Button y is `max(ceil(text_height) + 20, 70)`;
the 80×22 OK button has ten-point right/bottom margins. Windows uses a 12-point
body font and Linux/macOS use 13, as in `themes.h`.

The generic error and automatic roof-warning routes previously dismissed only
by mouse. They now also consume Enter and Escape: master's `Default_Button`
binds Enter, and FLTK `Fl::handle` sends unhandled Escape to the modal window's
cancel callback. The same shared handler preserves that behavior in the other
four routes. Dismissing an error does not also dismiss a queued warning.

Five independent geometry vectors, rendered platform-font/DPI checks and 30
editor cases verify button placement, wrapped text, mouse/Enter/Escape dismissal,
disabled parent actions, retained dirty map history/colors and unchanged files.
Text-shape assertions check the painted top-left origin; a centered widget
allocation alone is insufficient, especially for short success messages.
Existing oversized-map tests verify acknowledgment still publishes the staged
map exactly once. Light/dark and 1×/2× snapshots cover the rendered adapter.
Local formatting, workspace/visual warnings-as-errors Clippy, workspace tests
and rustdoc pass. The visual suite passes 185 tests with four existing ignored
native/reference checks. Eight new/updated baselines were inspected; pixel
differences in the four existing images are confined to their dialog regions.

This is client-layout/input evidence, not native child-window/frame or font
rasterization parity. Captions are now native titles on desktop; embedded test
clients retain their egui headings. Warning/error/success images fill their original space; see the
[completed artwork port](#original-toolbar-message-and-application-artwork). Error details can retain the port's structured diagnostics.

Two-button confirmations now use that same measured body and original warning
artwork. This covers unsaved map/close/exit, tileset/roof replacement, event-script
load/reload/unload, manual palette/roof-color import and each staged Open/New
palette import. The buttons have a 14-point gap: on Windows OK is left and
Cancel right; Linux/macOS reverse them. Both draw the original OK/Cancel labels;
descriptive accessible names preserve the action or palette-skip meaning.
Enter accepts and Escape cancels, with cancellation winning simultaneous input.
The palette Cancel continues the map open while retaining colors, matching the
original. Event and tileset warnings identify their source filenames/actions.
[Confirmation tests](../crates/gui/src/message_dialog/tests.rs) cover actual
rectangles, labels and keyboard outcomes at both platform metrics and 1×/2×.
The workflow tests additionally cover dirty-state preservation and sequential
import decisions. These client tests are separate from the native hosting and
protocol evidence below; neither replaces integrated native acceptance.

Verification for the option/confirmation client updates: Windows workspace
format, warnings-as-errors Clippy, tests and rustdoc pass. The Ubuntu 22.04.5
LTS/WSL2 offscreen Vulkan suite passes 236 tests with four existing ignored
native/reference checks; its visual-feature Clippy check also passes. Fifteen
changed baselines and four new confirmation baselines were reviewed. Material
pixel changes in existing images are confined to the affected dialog regions.
No desktop interaction was used for these client checks.

## Native dialog hosting and ownership

Pinned `Modal_Dialog::show`, `Option_Dialog::show` and the block/tileset/roof/palette
window `show` methods use separate, fixed modal windows centered on their parent.
[The common host](../crates/gui/src/dialog_host.rs) now applies that model to all
option, editor, About, warning, success and error routes. The same measured client
starts at (0,0), with no extra egui caption/frame. Source artwork supplies each
native icon. Client size is measured from the existing layout, then used for
native sizing; initial outer position subtracts the measured decoration offset
from the source client-centering formula. Repaints do not recenter a moved window.
Wayland supplies parent relationships but delegates absolute placement to the
compositor, as required by its protocol.

Whole form operations run in the child input context, including shortcuts before
rendering and Enter/Escape handling afterward. Native Close routes through Cancel,
including open popups. Main-window Close is refused while an uncommitted draft or
modal is active, even when the saved document itself is clean. Accepted Exit
explicitly closes ROOT. Errors retain an inactive editor and its draft under a
nested message. Help remains nonmodal, but cannot close through a blocked parent
workflow. The Windows native color chooser is owned by the palette editor.

The [audited adapter patches](../vendor/NOTICE.md) add three eframe registrations,
a weak window registry with scoped leases, and native ownership hooks. They use
Windows top-level ownership and restore the previous enabled state before child
release ([SetWindowLongPtrW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw),
[EnableWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enablewindow));
AppKit floating child ordering; [X11 transient/modal hints](https://specifications.freedesktop.org/wm/latest-single/);
and Wayland `xdg_toplevel.set_parent` plus optional `xdg-dialog-v1` from the locked
protocol XML. X11 dialogs also request the dialog window type. Input filtering is independent
of compositor modal-hint support; it resets retained mouse presses without
synthesizing a click when a release occurred over the child.
No native call runs under egui's data lock, and replacement registrations cannot
be detached by their old renderer state's destructor.

[Headless viewport tests](../crates/gui/src/dialog_host/tests.rs) exercise 1×/2×
client sizing, negative parent coordinates and decoration offsets, cancellation
across editor/option families, root-close guards, nested errors with retained
drafts, child keyboard routing, Help guards, and explicit root exit. Ownership
graph tests reject cycles, sibling modals and accidental reparenting.

The [hidden native probe](../crates/platform/examples/native_modals.rs) passes on
Windows and Ubuntu 22.04.5/WSL2 under private Xvfb. It checks actual owner/transient
properties, nested input restoration, renderer-window replacement, parent-first
state teardown and destruction of all four hidden windows. Windows additionally
checks preservation of an owner that was already disabled. The
[Wayland peer](../tools/wayland_modal_probe.py) passes with and without the optional
dialog protocol, verifying exact parent identities/order, three modal hints when
supported, and resource destruction before toplevel teardown. The same hidden
probe also passes on the macOS CI runner. The
[implementation CI run](https://github.com/vulcandth/polished-map-ng/actions/runs/41010677779)
records successful Windows/macOS ownership probes and both Linux native probes.

Windows workspace format, warnings-as-errors Clippy, tests and rustdoc pass.
Linux visual-feature Clippy, all seven viewport-host tests and the full offscreen
visual suite pass; CI also verifies the final visual suite with no baseline
changes. macOS target Clippy passes; the optional Glow backend also compiles on Windows.
Vendor archive/patch hashes, all three native production graphs and documentation
links/examples are verified.

These are implementation, input and protocol checks. They do **not** establish
native compositor placement, visible stacking/focus behavior, frame appearance,
clipboard transport, mixed-DPI transitions or complete desktop workflow parity.
Those remain M4/M6/P1 acceptance. Existing embedded visual baselines continue to
check the shared clients; no desktop computer use was needed for this work.

## PNG decoding, interlacing and platform transparency

`tools/png_oracle.py` extracts unchanged master `read_png_graphics`, `mono_hue`
and constants, FLTK 1.4.5 `Fl_PNG_Image` construction/loading,
`Fl_RGB_Image::desaturate`, and the Windows RGBA postprocessor. It builds the
pinned FLTK checkout's bundled libpng 1.6.44 and zlib 1.3.1 with portable C paths.
Each copied file/extracted snippet has a SHA-256 provenance record. Generated
sources, library notices and binaries stay in ignored scratch storage.

The adapter provides image storage/accessors, synthetic ASCII file paths,
suppressed warning output, a 64-MiB allocation ceiling, and no drawing/cache.
It selects the Unix/macOS no-op or the unchanged Windows postprocessor; both
paths run in reference CI without an OS window. This verifies decoder behavior,
not native Unicode paths, filesystem dialogs or displayed color management.

The 90-case corpus compares 1,720,320 hues through public Rust project loading:
all 65,536 sample values in 16-bit grayscale, gray-alpha, RGB and RGBA;
8-bit counterparts; packed 1/2/4-bit grayscale and indexed PNGs; palette alpha,
8/16-bit gray/RGB tRNS, six 16-bit alpha boundary values, gamma metadata, 8×2048/2048×8
extremes, invalid dimensions and excessive tile counts. Each case is also
repacked by libpng as Adam7, with the interlace flag and unchanged decoded hues
asserted independently of the Rust comparison. The original loader's successful
tile counts, tile order and hues must match exactly; invalid sizes must fail.

The differential exposed Rust's generic image conversion rounding 16-bit samples
where FLTK discards the low byte. The importer now uses PNG expansion and
`STRIP_16`, then the original integer `(31R + 61G + 8B) / 100` desaturation.
Windows clears RGB only for fully transparent expanded RGBA pixels, after
16-bit stripping; Unix/macOS preserve the color. Gray-alpha pixels bypass that
Windows adjustment. Alpha otherwise does not participate in hue selection.

`PngPlatform` defaults to the host and can be selected explicitly via
`MasterGraphicsOptions`, standalone roof opening, or CLI `--png-platform`.
Tests verify compound before/main/after parts, roof initialization/switching,
tileset replacement, different preflight hashes and CLI render/inspect/edit
behavior. Separate nonignored regressions cover all grayscale hue thresholds,
alpha truncation and truncated files. File/decode allocation bounds remain.
PNG re-encoding can still differ byte-for-byte; decoded pixels are the contract.

## Assembly/CHR graphics and empty images

`tools/asm_graphics_oracle.py` extracts unchanged pinned master `Parsed_Asm`,
`parse_value`, macro/comment/whitespace helpers and `Tiled_Image::read_asm_graphics`
with its original bitplane conversion. Generated C++ and per-snippet SHA-256
provenance stay in ignored scratch storage. The only filesystem adapter uses
`std::ifstream` for synthetic ASCII paths; it does not verify native Unicode paths.

The reference test covers 151 cases: every byte and every 16-bit word in decimal,
`$` hexadecimal, `%` binary and `h` hexadecimal suffix notation, little-endian
words, macro case, indentation, C whitespace, comments, CRLF/LF, ignored labels
and directives, signs, terminal commas, empty inputs, 16-byte alignment and the
256-tile limit. It checks original parser bytes against independent generated
values, compares Rust parsing, then compares tile counts and all 2,117,696 hues.
Successful CHR cases also pass through the public project tileset loader. Safe
overflow cases compare original statuses; malformed-number/empty-operand cases
explicitly assert the port's documented strict errors instead of source zero or
numeric-prefix substitution. This is not general RGBDS expression evaluation.

The differential exposed a real mismatch: upstream accepts zero-tile CHR/2bpp
images. The Rust decoder now accepts them too, allowing the existing project
compositor to create the original 256 blank slots. Empty binary, empty CHR and
comment-only CHR imports have edit/save/reopen tests: the original input is
unchanged, the emitted two-bit PNG has the expected edited/blank pixels, and its
reopened tiles match. Core editing models still require populated tile slots;
roof loading still requires exactly nine tiles. Existing unsafe compound-export
bounds protections remain intact.

Empty initial/interior numeric operands previously disappeared in Rust, shifting
later bytes. They now return `CodecError::Assembly`; a terminal comma remains
accepted. The unchanged-source differential runs in reference CI. Native file
adapters and full-application interoperability need separate coverage; the PNG
decoder has its own differential above.

Local formatting, warnings-as-errors workspace Clippy, workspace tests, rustdoc
and the compiled assembly differential pass. All seven pinned real-project tests
pass, including 236 automatically resolved maps across 35 tilesets and 388 event
scripts; the fixture's 18 shared-map files still require their explicit options.

## Initial map position on Open and New

Pinned master `Main_Window::open_map` computes the initial map position after
creating its block group. For each axis, `k = content - workspace + scrollbar`;
the result is the three-block margin when `k` exceeds that margin, zero when
`k < 0`, and integer `k / 2` otherwise. The scrollbar allowance is unconditional.
The strict comparison produces an intentional jump from half-margin to full
margin; it is not ordinary centering followed by a clamp.

The GUI now requests this positioning only after a new document, textures and
palette decisions are successfully published. It uses the current client layout,
zoom and rulers, clears the prior map's scroll request, and applies the position
without old-frame inertia. Canceled/failed opens and pending oversized-map
warnings preserve the previous view. Resize still resets to zero, while normal
window resizing and later frames do not reapply the opening position. Close
clears the request so it cannot suppress a subsequent Open or New.

`open_scroll_tests` checks 16 independent arithmetic boundary vectors and 44
Open/New combinations through actual worker/UI routes, at both DPI scales, both
zoom sizes, rulers, mixed small/large axes and the minimum 335×262 client size.
It also checks failed/canceled opens, acknowledgment timing, old/new document
identity, one-time positioning, scroll after opening, file bytes and clean/new
history state. These are source-derived and headless checks; native FLTK/eframe
positioning and window-resize comparisons remain separate verification work.

Three new rendered snapshots were inspected: the 15×12 partial-centering case
places the map origin at `(224, 112)` in an 800×600 client; the large zoomed/ruler
case starts at `(288, 63)`; the 335×262 zoomed/ruler case uses that same origin
with a 31-point content viewport before the vertical scrollbar. These positions
match `workspace origin + three-block margin - opening offset`. Existing visual
baselines remain unchanged.

Local formatting, workspace and visual-feature Clippy with warnings denied,
workspace tests and rustdoc pass. The full visual suite passes 181 tests with
four existing ignored tests, including the three reviewed new snapshots.

## Middle-button workspace panning

Pinned master `widgets.cpp::Workspace::handle` intercepts middle presses before
its children. This corrects the earlier child-only reading of
`Metatile_Button::handle`: a middle release in the sidebar does **not** select a
block. The port now captures map/sidebar middle drags, displays the move cursor,
and computes each offset from the original integer press coordinate and scroll
position. For each axis, the limit is `max(content - outer + perpendicular
visible scrollbar, 0)`. The sidebar remains vertical-only. Map extents include
the original three-block border on every side, and ruler offsets follow panning.
There is no movement threshold, anchor rebasing at an edge, or release fling.
Wheel events still scroll during capture; subsequent movement uses the original
press anchor. Zoom-retained blank space is clamped to natural content on drag.

`workspace_pan::tests` exercises real egui input at 1×/2× DPI, both zoom sizes,
rulers on/off, both drag-and-drop settings, integer/subpixel movement, edge
clamping and return, event markers, blank margins, same-frame press/move/release
outside the workspace, wheel interleaving, popup guards and cancellation on
pointer/focus loss, modals, zoom/reset and Close. Map bytes, event files,
selection, edit history and editor dialogs are checked for unwanted changes.
Nine tests include independently calculated limits for 800×600 and the original
335×262 minimum client size. The latter exposed egui's 64-point minimum viewport
placing a scrollbar outside the client; removing that floor keeps both bars
within the source workspace. Bounds are checked on actual scrollbar widgets,
not inferred from the final scroll offset alone.

This is source inspection plus headless input/geometry evidence. Native pointer
capture, cursor delivery and platform mouse mapping remain unverified. No
computer-use session or native parity claim is included.

The prior icon commit's Windows CI also exposed a test settling race in the
oversized-map warning helper. It now dispatches Open with one frame and uses its
existing bounded worker-completion wait before settling the result UI. This
changes test synchronization, not production file-loading behavior.

Local formatting, warnings-as-errors workspace Clippy, workspace tests and
rustdoc pass. The full visual suite passes 177 tests with four existing ignored
tests, using unchanged snapshot baselines.

## Original toolbar, message and application artwork

M2 now imports all 41 nonblank toolbar/editor XPMs from the pinned master,
including the ten formerly missing designs and the dark Blocks variant.
[Icon records](icon-inventory.json) and [application/message records](artwork-inventory.json)
pin original blobs and decoded pixels. The user selected original upstream
artwork with upstream attribution; known Fugue terms remain separately retained,
and unknown underlying origins are recorded without claiming new clearance.
See [assets](assets.md) for the current distribution decision and source history.

The toolbar retains the same 27 controls, 24-point bounds, source order, names,
keyboard actions and disabled guards. Bitmap faces replace only the interim text.
The [pixel atlas](../crates/gui/src/icons.rs) checks all 33 designs at 22/24 points,
light/dark, enabled/disabled and 1×/2× DPI (528 combinations). Source hash checks
cover all 10,496 toolbar/editor pixels. Popup menus retain their source blank
image column and check/radio marks; upstream does not give them toolbar images.

[Message/About artwork](../crates/gui/src/artwork.rs) paints the original 48×48
pixels with a one-point inset in each source 50×50 slot. Typed message kinds select
Error/Warning/Success independently of their captions. Every source pixel and
transparent margin is checked over both backgrounds/scales. Pillow independently
agrees with the XPM decoder for all 9,472 message/application pixels. CI runs both
offline asset verifiers, including the preserved Windows ICO hash.

The desktop viewport receives the original 16×16 Linux icon or the 48×48
application picture on Windows/macOS. The latter exactly matches the original
ICO’s 48×48 frame. This verifies the configured payload, not native OS icon
selection, scaling, taskbar/Dock or executable metadata. Native rendering remains
M4/P1; embedding the original multi-resolution ICO and macOS bundle metadata
remains P2. No GUI computer use was needed for this pixel/geometry work.

The workspace suite passes (362 tests; 33 explicit native/reference ignores),
and the serial visual suite passes (226 tests; four explicit ignores). All
99 changed snapshots were compared against their committed predecessors: changes
are confined to toolbar/image slots, with the previous 23 atlas columns unchanged.
The original-artwork atlas adds the four 48×48 pictures. Representative light/dark
full toolbars, message images and About crops were reviewed; both pixel atlases
also pass their per-pixel checks at 1×/2×. Format, workspace/visual Clippy, rustdoc,
native-adapter graph, asset hash checks and documentation links/examples pass.

## Open, Load Events and Load Palettes icons

The three toolbar controls now use the unchanged pinned `open.xpm`, `load.xpm`
and `load-palettes.xpm` pixels assigned by `Main_Window::initialize`. The
underlying controls, callbacks, accessible names, tooltips, source ordering and
24×24 extents are unchanged. Icons remain centered at 16×16 and use the existing
source-derived two-stage integer disabled-color conversion in both themes.

The asset review resolves the missing small-arrow and small-bulb components in
the previously documented composites. `tools/icon_components.py` checks the
verified Fugue archive/component hashes, 28 exact arrow-mask pixels, 32 exact
bulb-mask pixels, 222 Open/Load base pixels with three palette substitutions, and
the shared 112-pixel folder strip. The complete provenance assessment and its
limits are in `docs/assets.md`; masks are supporting evidence, not an automatic
copyright decision. CC BY 3.0 creator/collection attribution and LGPL adaptation
notices cover all three imports. The offline asset verifier now checks 30 XPMs
and 7,680 decoded pixels, while the rendered atlas covers 23 designs across 368
size/theme/enabled/DPI combinations.

All 168 active GUI visual tests pass, with four explicit host/reference tests
ignored in that run. The 85 updated snapshots were inspected; a comparison to
the previous Git blobs confines every changed pixel to the three existing icon
slots or the three appended atlas columns. The former atlas columns are
unchanged. Toolbar geometry/action/disabled-guard coverage remains active.
Formatting, workspace and visual-feature Clippy, workspace tests and rustdoc
also pass locally. No computer use or native chooser interaction was needed.

At that import, ten toolbar controls still used placeholders. The remaining
artwork is now imported below; native-window parity remains unverified.

## Compressed graphics loader differential

`tools/lz_oracle.py` extracts the pinned master LZ command constants, bit-reversal
table, `read_lz_graphics`, `parse_2bpp_data`, bitplane conversion and file-size
helper without editing their bodies. Generated C++ and SHA-256 provenance stay
in ignored scratch storage. The harness supplies a minimal image declaration and
an ASCII-path `fl_fopen` adapter; it does not reimplement decompression.

The reference test compares 12,390 cases and 202,964,992 decoded tile hues.
It covers all seven commands, every short length (1–32) and long length
(1–1,024), absolute/relative copies, every relative distance (1–128), overlapping
repeat/flip expansion, reverse copies, absolute address boundaries, 256 mixed
sequences, exact 4,096-byte output, empty output and ignored bytes after the
terminator. Each successful original load yields 256 tiles because its complete
4,096-byte buffer is zero-padded. Rust's raw decompressor returns just the decoded
prefix; `decode_lz_tiles` reproduces the original padding.

Only two malformed streams are passed to upstream: a reserved command and a
command after a full buffer, both of which return before unsafe access. Rust-only
tests cover every incomplete prefix of a mixed sequence, invalid references,
reverse underflow, a command crossing the output bound and a zero output limit.
The original's unchecked reads/writes are not exercised with those inputs.

The differential and safety tests pass locally. The Linux reference CI job now
builds and runs the same comparison. This validates extracted codec functions,
not native file dialogs or the complete original application. No codec behavior
changed in this work. A separately observed Linux CI race in the warp-navigation
test was corrected by dispatching its confirmation with one frame before its
existing bounded worker wait, instead of demanding immediate UI settling.

## Oversized map loading and acknowledgment

The GUI follows pinned master `Map::read_blocks`, `Map::read_asm_blocks` and
`Main_Window::open_map`: excess decoded map IDs are ignored after the chosen
width × height, and the original basename-specific warning is acknowledged
before opening completes. This is a one-button warning, not a cancelable consent
dialog. Short inputs still fail. The new prefix is initially clean with empty
history, and opening never writes the source.

`MapDocument::open_in_project_with_report` returns the document and an explicit
`trailing_blocks` count. Existing `open`/`open_in_project` and CLI callers remain
strict. Both paths retain dimension checks, the four-MiB read bound and malformed
ASM rejection. The SHA-256 covers the complete original file, including unused
trailing bytes; edits outside the loaded prefix therefore still cause a save
conflict. ASM imports continue to require binary Save As and preserve their source.

The GUI resolves palette choices first, then displays the map warning. The prior
document and recent history remain intact until acknowledgment and successful
publication, preserving the port's existing staged-open safety boundary. Pointer,
Enter and Escape acknowledgment proceed with opening; map commands remain blocked
while the warning is shown. Ordinary Save Map now follows `save_map(false)` by
skipping a clean binary map. An actual edit followed by Save Map writes the prefix;
Save As remains an explicit write. Merely dismissing the warning cannot truncate
the source file.

Three project tests cover binary/uppercase ASM input, decoded counts, whole-file
hashes, unchanged input during open, prefix Save As/reopen, tail-only conflicts,
short/exact input, invalid dimensions, maximum 255×255 dimensions, bounded reads
and malformed ASM. Two GUI tests use the real loading worker and Open options,
checking palette Skip ordering, retained old map/undo/history, modal guards,
all three acknowledgment routes, exact/short inputs and clean/dirty Save Map.
The warning snapshot was reviewed. Formatting, workspace and visual-feature
Clippy, workspace tests and rustdoc pass locally; all 168 active GUI visual tests
pass, with four host/reference integrations explicitly ignored in that run.
These are source-derived and integration checks, not a compiled full-application
differential. The warning's native window and exact geometry remain open; original artwork is imported.

## Popup panel frames and full-width separators

Popup panels adapt `themes.cpp::{greybird,dark}_panel_thin_up_box/frame` and
FLTK 1.4.5 `menuwindow::drawentry`. One-point filled rectangles preserve the
original inclusive line endpoints and corner ownership. Greybird uses gray
218/193 for its upper/lower edges; Dark uses 106/40. The egui frame still reserves
the original one-point inset, with its generic stroke made transparent. The
source panel is drawn before its rows, so dividers overwrite the side edges.

The previous fixed separator colors were incorrect. `Fl::background` derives
`FL_DARK3` and `FL_LIGHT3` from a power-law gray ramp: background 206 produces
110/247, and background 83 produces 3/216. Separators now use these values and
extend across the full popup width, including both borders. Their inclusive
endpoint extends one point beyond the window and is clipped, matching FLTK.
They retain the original two rows at the bottom of the preceding item.

`tools/menu_marks_oracle.py` now pins `Fl_get_system_colors.cxx` and
`FL/Enumerations.H` alongside its existing sources. It extracts the unchanged
gray-ramp function, constants, panel functions and divider block, adding 18
panel and 32 divider comparisons to the existing 72 indicator and eight text
color cases. Rendered tests check every physical pixel of selected panel corners
and every divider pixel in actual File popups at 12/13-point font sizes and
1×/2× DPI, plus the nested Theme panel's corners. The tests include the half-point
submenu placement possible at 2× DPI. Sixteen changed menu snapshots were reviewed.
Local formatting, workspace and visual-feature Clippy, workspace tests and
rustdoc pass. The complete GUI visual suite passes with 166 tests and four
explicitly ignored host/reference integrations; the extended compiled-source
comparison also passes when explicitly invoked.

This does not establish native popup placement/shadows, arrow shape, font
rasterization or mnemonic-navigation parity. Those remain separate work.

## Source Theme submenu for the initial supported themes

Historical implementation evidence: the initial port reproduced the source
Greybird/Dark submenu using `Main_Window::initialize`, `greybird_theme_cb` and
`dark_theme_cb`. M8 supersedes that two-entry submenu with Dark Theme, as requested
in the November 1 review. The other ten upstream themes remain deferred.

Current tests cover pointer/keyboard toggling, checked state, close-on-selection,
empty-editor use, modal guards, preservation of unsaved map/palette edits and
stable map textures. Preference tests save/reconstruct the editor and check the
startup theme, retaining the existing v1 `dark` boolean. See the
[review disposition](#november-1-upstream-review) for current verification status.

## Map window title and Print filename

`window_title.rs` follows master `Main_Window::open_map`, `save_as_cb`, `close_cb`
and `option-dialogs.h::NEW_MAP_NAME`. The fixed NG development identity gains
` - <basename>` after a completed open or Save As, uses `New Map` before the
first save, and resets after Close. Dirty state never adds an asterisk. The
originally chosen path spelling is retained independently of canonical I/O paths
and recent-history eviction. Only changed titles send a command, explicitly to
the root viewport, after the frame's document/dialog transitions.

The source also sets Print's filename at these transitions. The queued Print
request now captures that suggestion: `.blk` (case-insensitive) becomes `.png`,
other basenames retain their suffix and append `.png`, and new maps use
`New Map.png`. The injected native chooser receives the captured name, even if
the editor subsequently releases the loaded document. It remains a suggestion;
the chosen output path still goes through the existing confirmation/conflict
checks. No file is created while computing or displaying either name.

Tests exercise real asynchronous New/Open loads, failed loads, canceled Map
Options/Close, dirty edits, successful/failed Save As, Close, and unchanged-frame
command suppression. They also check chosen-name precedence, mixed-case/multiple
suffixes, Unicode, preservation of non-Unicode native path bytes/code units and
queued Print picker handoff. Native title-bar rendering and OS chooser handling
are not proven by these headless checks. The development identity intentionally
continues to distinguish this port from an upstream parity release.
Local formatting, Clippy with warnings denied, workspace tests and rustdoc pass.
The visual suite passes 163 tests (four explicit ignores), with no new snapshot
baseline changes for titles/filenames. The installed-font and native-window
checks remain separate from these document-state tests.

## Popup indicator themes and inactive colors

`menu_popup/marks.rs` adapts pinned master `themes.cpp` and FLTK 1.4.5
`Fl_Menu.cxx`, `fl_draw.cxx`, `fl_color.cxx` and `fl_contrast.cxx`. Greybird check
boxes use the original inset fill and distinct top/side/bottom/corner colors;
Dark uses the original button gradient and inner top highlight. Checkmarks are
filled six-vertex polygons. Radio marks retain the GTK inner fill/highlight and
the theme's own `box_dw` (Greybird 6, Dark 2), replacing the previous generic
quarter-radius dot. Disabled rows dim text/shortcuts/arrows with
`foreground * .33f + background * (1-.33f)`, truncating channels as in FLTK.
Their indicator boxes retain source colors, bypassing egui's additional disabled
tint. Selected text uses CIELAB contrast at level 50, as set by master `main.cpp`.

`tools/menu_marks_oracle.py` extracts unchanged original functions, verifies
downloaded FLTK source hashes, and compiles a drawing-command recorder. It checks
72 combinations of two themes, check/radio, on/off and row heights 18–26, plus
eight enabled/hover text colors. Rectangle endpoints, gradient channels, polygon
vertices, arc/pie bounds and angles are compared in order against Rust output.
The generated source, provenance and outputs stay ignored. Visual CI explicitly
runs this differential test; normal tests leave it ignored when the fixture is
absent. Headless integration checks disabled accessibility, untinted text/mark
colors and focus contrast at 1×/2×; snapshots cover both themes and scales.
Local formatting, workspace tests, Clippy with warnings denied and rustdoc pass.
The full visual suite passes 159 tests (four explicitly ignored), and the compiled
menu comparison passes separately. Thirteen changed menu snapshots and four new
indicator snapshots were visually reviewed before accepting their baselines.
Linux CI 40862109587 passed the original-command comparison and 158 visual tests;
the remaining dark 1× gallery had two radio-highlight pixels at RGB 225 versus
223 locally, with all other channel differences at most 1 and unchanged alpha.
The inspected artifact justifies a Linux-only threshold of 3 for that single
snapshot; all other snapshot tolerances and exact geometry/color checks remain.
The raw FLTK menu hash now pins LF bytes (the initial local cache had CRLF).

This is command/geometry equivalence, not native pixel equivalence. Egui still
tessellates arcs and polygons; FLTK's native drivers may cover edge pixels
differently. Native font coverage, popup shadows/placement, mnemonic/Alt navigation
and screen-edge behavior remain separate acceptance requirements. No artwork
or font assets were added for these procedural marks.

## Source palette sliders

The palette sliders retain the source-compatible interaction in `nice_slider.rs`,
with the earlier rounded egui rail and theme-specific rounded handle restored at the user's
request on 2026-10-31. This intentionally supersedes the FLTK box/gradient styling
introduced on 2026-10-30. The previous full-dialog snapshot baselines are the
appearance reference; source box-drawing comparisons no longer define acceptance.
See the [approved preference](deviations.md#palette-slider-appearance).

The control retains FLTK's 16-point drag extent. Clicking outside moves
the nearest edge; grabbing inside retains the pointer offset. Dragging clamps at
either end, secondary-button dragging works, horizontal arrow keys step/clamp,
and middle press/drag does not alter the value before the existing middle-release
reset. Focus loss, disabled controls, open popups and hidden/reopened sliders
cancel stale captures. Accessibility value/increment/decrement actions remain.
Drawing uses egui's rail height, rounded corners, handle radius, theme colors and
hover/focus/disabled appearance, independently of those input rules.

The [drag oracle](../tools/slider_oracle.py) compiles unchanged FLTK handler and
valuator functions, generating 84,864 observations: 42,432 for Windows rounding
and 42,432 for Unix ties-to-even. Each platform compares its corresponding branch,
including endpoint crossings and retained offsets. CI retains this comparison.
Headless widget tests cover input, accessibility and capture cancellation at
both DPI scales; palette workflow tests retain quantization, draft cancellation
and middle-reset coverage. Offscreen snapshots cover both themes/scales and
existing palette dialogs. Native integrated palette acceptance remains M6.

Verification on 2026-10-31: all five pre-redesign palette-dialog/drag baselines
pass unchanged under offscreen Linux rendering. The four theme/DPI slider
snapshots were reviewed and pass. Both Windows and Linux drag-oracle comparisons
pass; Windows workspace formatting, Clippy, tests and rustdoc pass. No desktop
automation or font changes were needed.

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
pixel-level GDI/Xft/Quartz equivalence remain unverified and are deferred to
later manual review under the [approved M4 exception](deviations.md#remaining-m4-aesthetics-deferred).

Print Options now measures with the selected body font (12 logical points on
native Windows, 13 elsewhere), following `Print_Options_Dialog::refresh`'s
`OS_FONT_SIZE` and original padding/spacing formula. Its installed-font test checks
that width and verifies that a queued worker retains and renders the captured
faces. Existing headless fixtures keep their explicit bundled 13-point style.

The shared font-instance commit `53879c9` passed all eight jobs in
[CI run 40860235999](https://github.com/vulcandth/polished-map-ng/actions/runs/40860235999),
including installed-font/worker/CLI checks on Windows, Linux and macOS.
Its local workspace tests, formatting, Clippy with warnings denied, rustdoc
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
Undo/Redo were excluded at that time; subsequent imports supersede that state.

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
button. Left release selects; right release also opens the block editor. The
parent Workspace intercepts middle presses for panning, as documented above.
There is no sidebar context menu or double-click editor action in this source.
The port now tracks the pressed cell/button despite using one egui interaction
region for the grid. Releases cannot cross into another cell, blank space or
clipped rows; right-dragging away and back still works, because upstream does
not require `event_is_click` for this callback. Pointer loss, disabled/modal UI
and document replacement clear the transient press. Existing drag copy/swap
operations still use their own validated source/target route.

The temporary Edit Block menu entry and its extra divider are removed. Existing
block editing/cancel/accept/save/conflict and collision-layout tests now enter
through actual right-button events. Four interaction tests cover left/right
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
X11/Wayland `0xBFFFFFFF`, and AppKit alpha 0.75; opaque values restore 255/`UINT32_MAX`/1.0.
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
Wayland binds alpha-modifier on the actual winit connection/queue. It applies
the multiplier to the root and all five client-frame subsurfaces, synchronized
with the renderer's root commit. The copied SCTK fallback frame retains its
original drawing/input/geometry and MIT provenance; see [vendor notices](../vendor/NOTICE.md).
Missing protocol support or a visible compositor-owned frame returns an explicit
capability error. A configure transition to an unsupported frame restores opaque
state and notifies the GUI. Saved intent survives a failed startup application
or capability loss; the menu changes only after the matching native reply.
GUI tests cover deferred success/failure, duplicate/stale request prevention,
viewport isolation and one-shot capability-loss notification.

The [Linux protocol probe](../tools/wayland_opacity_probe.py) drives the actual
patched winit adapter against an isolated socket peer using the locked protocol
XML. It checks matching root/frame multipliers at commit, all six resources,
unsupported-frame/missing-protocol rejection, server-frame transition rollback,
and alpha-before-surface destruction. It needs no GPU or running desktop.
Local Windows workspace tests (371 passed, 33 ignored), format, Clippy and
rustdoc pass; the full GUI also cross-compiles for Linux and macOS. Linux CI
[`40997726203`](https://github.com/vulcandth/polished-map-ng/actions/runs/40997726203)
executed the protocol probe successfully for client-frame, server-frame and
missing-protocol scenarios, alongside the Xvfb property test. The native macOS,
visual, dependency/license, MSRV, reference and real-project jobs also passed.
Protocol evidence cannot prove
compositor blending, wgpu output, fullscreen behavior or native preference
restoration. Those remain M3/M5/M6 acceptance requirements.

A subsequent Linux run exposed a harness shutdown race: the client had completed
teardown but closed with unread cleanup replies, producing a socket reset. The
peer now treats reset/broken-pipe as connection closure while still requiring
successful process exit, complete request parsing and every resource/state
assertion. All three opacity scenarios pass locally on Ubuntu 22.04.5 LTS under
WSL2, kernel `6.18.54.1-microsoft-standard-WSL2`, Rust 1.98.1, with a private
socket and no desktop/GPU connection. Protocol XML is resolved from the test's
actual Cargo build artifacts, avoiding unrelated whole-workspace downloads.

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

The application image was initially excluded. The original-artwork selection
and import below now fill that slot while retaining the provenance limitations.
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
canvas zoom, scroll and view toggles. Export uses `Print Screenshot`, suggests
the map-derived PNG filename, and retains existing filename suffixes. Appending `.png`
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
were captured using embedded test modals. Native hosting is now implemented;
integrated child-window frames, font metrics/hinting, actual
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

## Native external file drops

Pinned master `DnD_Receiver` receives only in `_map_scroll`; its callback rejects
modals, applies the unsaved guard and opens the first supplied filename. NG now
uses [the same guarded Open entry](../crates/gui/src/recent_maps.rs) for native
drops and recent files. [Workspace routing](../crates/gui/src/external_drop.rs)
includes blank canvas margins and excludes sidebar, rulers, menu/toolbar and
status areas. Rejected and accepted transfers are consumed once; a second drop
cannot replace a pending modal path. The internal Drag and Drop setting remains
independent.

The [pinned native patches](../vendor/NOTICE.md) emit one complete transfer before
legacy path-only events. Windows uses OLE's actual screen point and
ScreenToClient; macOS converts draggingLocation into its flipped content view;
X11 tracks source/target position and waits for the drop's matching selection
response, including INCR chunks; Wayland receives URI lists on winit's own
connection and calloop queue.
The egui bridge preserves ordered paths and transfer identity per viewport and
divides by native scale × egui zoom once. Path-only RawInput and ordinary cursor
state cannot initiate an Open.

Automated evidence:

- Egui input tests cover workspace edges, blank margins, 2.5× conversion,
  first-file selection, dirty/modal rejection, cancellation, failure retention,
  unchanged recent history, queue bounds and viewport isolation.
- The Windows [OLE callback test](../crates/platform/tests/ole_file_drop.rs)
  includes the exact patched vtable/handler. An unseen native window receives
  synthetic CF_HDROP data with Unicode and an unpaired UTF-16 filename. The test
  checks ordered paths, actual screen-to-client translation, QueryInterface
  identity and both STGMEDIUM owner releases. It does not operate the desktop.
- The [shared native helper tests](../crates/gui/tests/native_file_drop.rs)
  cover unique transfer IDs and invalid records; Unix additionally exercises
  non-UTF-8 filenames, escaped newlines, malformed/remote URLs and size limits.
- The [native X11 probe](../crates/platform/tests/x11_file_drop.rs) runs actual
  winit windows and an independent x11rb source under a private Xvfb server.
  Its 17 scenarios cover inline/type-list offers, INCR chunks, wrong source/target,
  positions arriving after Drop, stale/duplicate notifications, malformed URI or
  property format, payload/INCR limits, refused selection, leave, missing position,
  unsupported MIME, a new enter during a pending selection and target destruction.
  It checks ordered UTF-8/non-UTF-8 paths, negative root coordinates and physical
  client coordinates at 2× scale. All pass on Ubuntu 22.04.5 LTS/WSL2, Rust 1.98.1,
  Xvfb `2:21.1.4-2ubuntu1.7~22.04.16`. The probe exposed and now prevents two
  regressions: rejected actions must use numeric None (zero), and a later position
  must not erase an already dropped transfer's captured point. Both conform to
  the [XDND protocol](https://wiki.freedesktop.org/www/Specifications/XDND/).
- The [Wayland protocol probe](../tools/wayland_file_drop_probe.py) runs the actual
  adapter and Rust event consumer against private wl_data_device events and real
  SCM_RIGHTS pipes. Its 13 scenarios cover ordered paths and 2× coordinates,
  overlapping receives completing in reverse order, legacy protocol support,
  leave, unsupported MIME/action/decoration targets, invalid/oversized payloads,
  timeout, seat removal, window closure and the 16-receive bound. Every scenario
  includes a clipboard offer that must not be read or replace selection ownership.
  Partial pipes wait for a native resize acknowledgment before sending the rest,
  so a blocking read fails instead of being hidden by immediate EOF. All 13
  scenarios pass natively on Ubuntu 22.04.5 LTS/WSL2, kernel
  `6.18.54.1-microsoft-standard-WSL2`, Rust 1.98.1. Linux cross-target
  compilation/Clippy also pass. This exercises the real adapter, independently
  of WSLg or any running desktop compositor.
- Windows workspace checks run locally. Cross-target checks pass for the
  macOS GUI and Linux winit with X11/Wayland enabled. Cross-compilation is not
  native execution. Original crate archives and every bundled source/patch hash
  are verified by [vendor_sources.py](../tools/vendor_sources.py).

Still required: actual file-manager delivery, off-window cancellation,
multiple-monitor/mixed-DPI placement, clipboard coexistence, seat/window teardown
and compositor behavior on Windows, macOS, X11 and Wayland. Wayland's isolated
protocol cases and X11's asynchronous selection flow are verified locally;
real-desktop workflows remain.
Whole-window opacity implementation and its remaining
native gates are tracked in [transparency evidence](#whole-window-transparency).
These are M3 gates; no complete desktop parity claim follows from these tests.

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
is now implemented through the separate [native transfer route](#native-external-file-drops).

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

## Current coverage matrix

Implementation coverage is not completed application parity. Remaining work is
tracked as M1–M7 and A1–A4 in the [roadmap](roadmap.md).

| Area | Reference | State / required evidence |
|---|---|---|
| Binary maps / dimensions | master map.cpp, map.h | Exact byte I/O, New Map, Save As/ASM conversion, filename/explicit/metadata dimensions and oversized prefix/warning flow tested; strict headless opens remain available |
| Paint, fill, replace, swap, history | master main-window.cpp, map.cpp | Core and GUI drag→undo→redo→save tests; all nine resize anchors, zero fill, truncation and history clearing tested; GUI resize stages map/event shifts; full gesture matrix pending |
| 2bpp / LZ / CHR / PNG | master tiled-image.cpp, parse-asm.cpp | Compiled-source comparisons cover all 65,536 bitplane pairs, 12,390 LZ cases, 151 assembly/CHR cases and 90 PNG cases including Adam7 and both platform policies; empty-image composition and edit/save/reopen pass. Full-application interoperability remains pending |
| Metatiles, palettes, collisions | master metatileset.cpp, tileset.cpp, palette-map.cpp, colors.cpp | Binary metatiles, tilepal, six sets, project/special palette precedence, metadata selection, roofs, compound composition, palette commands/export, GUI Open/New session carry-over, ASM/binary collisions and paired saves tested; full differential and native workflow coverage pending |
| Project discovery / metadata | master config.cpp, option-dialogs.cpp | Binary-map metadata and legacy asset-path selection match compiled-source fixtures, including precedence and missing inputs; Open size choices match every representable area; Open/New asset dropdowns, numeric ordering and measured client layout are implemented. Source inventory is reconciled above; native filesystem/dialog fidelity and full-project acceptance remain incomplete |
| Main window geometry | master main-window.cpp, ruler.cpp | Derived rectangle/hit-test/DPI, ruler geometry/scroll/highlight, global hex, block-ID, middle panning and initial Open/New positioning tests pass; light/dark development snapshots inspected; native fonts/themes, placement and native scroll/resize comparisons pending |
| Block/tileset/roof/palette editors | master *-window.cpp | Drafts, pixel gestures, palette/priority attributes, clipboard operations, drag/copy, block count, asset switching and persistence implemented. Middle-release defaults and shared numeric input are implemented; native hosting is implemented; integrated window/clipboard verification remains pending |
| Events / warp navigation | master event.cpp, map-events.cpp | Shared model, 22-macro parsing, preserved saves/CLI, overlays/options/dragging/warps, auto-load, resize, editor dispatch, shortcuts and Print overlays tested. Field modes/default-button behavior are implemented; native fonts, full gesture coverage and native workflows are pending |
| Preferences, overlays, print/clipboard | master main-window.cpp | Twenty-eight implemented settings plus recent history restore/save with malformed-record protection; all five Print flags/Copy/Export, internal Drag and Drop and positioned external-drop routing implemented. The meta128 acknowledgment and read-only legacy import are implemented; native file-drop acceptance, storage/restart/placement and Print/clipboard parity remain pending |
| Expanded tile/attribute profile | plusplus tile.cpp, metatileset.cpp | PP2 block/graphics/roof/map/palette/event backends are implemented with source comparisons and real-project roundtrips. Separate GUI integration and full application acceptance remain PP3–PP4; see [audit](#plusplus-source-audit) and [backend completion](#plusplus-events-and-script-persistence). |
| Unified profile detection | both config.cpp variants | Pending |
| Stable automation / batch recovery | new API | Block/collision and compound tileset preflight, backups and rollback tested; tileset CLI preview and hash-required apply; general batch journal discovery/replay and stable schemas pending |
| Native packaging and runtime | all three OSes | Build/test CI and selected Windows/X11 opacity and installed-font checks exist; complete native runtime acceptance and distribution packages pending |

New tests must distinguish synthetic vectors, source-derived expectations, and
actual C++ differential observations. A source-derived vector is not a run of
the original application. Unverified assets never become redistributable fixtures.

## Master testing readiness

**Current candidate:** both audited functional gaps are fixed and
[development builds](releases.md#development-packages) are published for Windows,
Linux and both Mac architectures. Native/application acceptance remains open.

**Historical audit:** The 2026-10-31 audit of commit `8fd2f4e` reviewed the source inventory, command
routing, native adapters, preference migration, documented deviations and CI.
[All eight CI jobs passed](https://github.com/vulcandth/polished-map-ng/actions/runs/41230741505),
including the three OS jobs, source comparisons, visual tests and pinned-project
checks. This audit did not operate a desktop or establish native acceptance.

| Classification | Finding / next action |
|---|---|
| Implemented editor functionality | Map, block, tileset, roof, palette, event, Print/export and save workflows have implementations and automated coverage. No unimplemented editing/save command family was found in this audit. This is not proof of absence of bugs. |
| M4 functional gap | Upstream ampersand menu mnemonics are absent from NG's menu specification/input routing. A temporary headless probe confirmed Alt+F leaves File closed while clicking File opens it. Ordinary command shortcuts and arrow/Enter navigation after opening a menu are separate implemented paths. Implement source menu/item mnemonic input; underline styling remains deferred. |
| M5 functional edge case | Mac legacy import uses Rust home discovery and does not reproduce FLTK's fallback when HOME names a nonexistent directory. This can miss an existing original preference store. Restore that fallback or obtain an explicit compatibility exception. |
| M3/M5/M6 verification | Exercise actual settings restart, maximize/fullscreen/removed-monitor placement, canceled closes with dirty drafts, external file drops, clipboard/Print, native choosers, editor launching and opacity on Windows, macOS, X11 and Wayland. Adapters exist; isolated/headless checks do not close these gates. |
| M7 verification | Complete edit/save/reopen in both NG and original master for every domain on compatible project copies, including conflicts/failures. Pinned-project/codec tests alone do not complete application acceptance. |
| Tester delivery preparation | No GitHub releases or runnable CI build archives existed at audit time; CI uploads visual differences only. Testers can build from INSTALL.md. A nondeveloper handoff needs explicitly labeled development packages with source/notices and build identity; this is delivery work, not missing editing behavior. |

The two implementation gaps identified above are now fixed; see the
[current implementation evidence](#master-functional-gap-fixes). Tester delivery
is tracked in [development packages](releases.md#development-packages). Test on copies and collect the build commit, OS/backend,
steps, expected/actual behavior and affected file hashes. Remaining fonts, theme
styling and the approved rounded palette sliders are not functional blockers.
Plusplus is now being developed separately under the approved sequencing exception.
Unified profiles, stable automation and a new shared graphical color picker remain
later objectives; their absence does not block master testing.

## Legacy preferences and normal-window geometry

The [read-only project reader](../crates/project/src/legacy_preferences.rs) adapts
master `preferences.cpp`, with keys/defaults from `Main_Window` construction,
`exit_cb` and `main.cpp`. FLTK's pinned `Fl_Preferences.cxx` supplies root-group,
continuation, duplicate-entry, integer-prefix and escaped-string behavior.
Platform root selection follows `Fl_Unix_System_Driver::preference_user_rootnode`,
`Fl_Darwin_System_Driver::preference_rootnode` and the WinAPI shell-folder lookup.
The full policy, mapping scope and remaining gates belong to
[preferences](preferences.md#legacy-import); safety deviations are recorded
[separately](deviations.md#legacy-preference-import-safety).

[GUI migration tests](../crates/gui/src/preferences/legacy/tests.rs) verify all
master keys, 270 palette/roof/anchor combinations, theme fallback, import/save,
NG precedence, error retention and one-time fullscreen/maximize startup requests.
[Parser/discovery tests](../crates/project/src/legacy_preferences/tests.rs) cover
both executable names, OS user roots, Linux vendor-directory precedence, CRLF,
continuations, duplicate keys, all 256 octal bytes and invalid bounded input.
Original test stores are compared byte-for-byte after import. Unix additionally
checks non-UTF-8 paths. No original project graphics or real user settings are used.

Source inspection found eframe 0.37 updated its normal-geometry cache only at
construction/save. With NG's exit-only preference saving, resize → maximize →
exit could therefore restore the startup size. The adapter now records normal
root geometry on native move/resize/scale events, ignores child/maximized/
fullscreen/minimized states and rejects zero sizes. Saving while minimized also
retains the cached normal size. [Regression tests](../crates/gui/tests/normal_geometry.rs)
compile the exact tracking helper and exercise transition/capture isolation.
They do not simulate a compositor's ordering or establish real restart behavior;
those remain M5/M6. Patches, source hashes and dependency licenses are retained in
[the vendor records](../vendor/NOTICE.md).

Verification on 2026-10-30: Windows workspace format, Clippy with warnings denied,
tests and rustdoc passed, as did the production native dependency graph, vendor
source manifests and documentation checks. The five parser/discovery, four GUI
migration and two geometry-tracker tests passed on WSL Ubuntu 22.04.5. The full
workspace cross-compiled for `aarch64-apple-darwin`; that is compilation evidence,
not Mac runtime verification. No desktop automation was used.

## Historical implementation evidence (2026-10-28)

These entries describe the implementation at that point, including partial
preference counts. They are retained as test/source records; consult the current
matrix above and [preferences](preferences.md) for present coverage.

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
  headless egui snapshots; native modal geometry remains open. Message artwork
  has since been imported.

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
  native frame/theme styles remain pending. Open, Undo and Redo used text
  placeholders at this initial import; all now use original source artwork.
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
  initial positioning and resize still need native checks. Initial-position and middle-button source/headless
  coverage is now documented above; native panning remains unverified.
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
  unchanged. It is not a native platform test. Resize consequences are available in the source-sized dialog's OK tooltip;
  see the current [client metrics](#remaining-option-dialog-client-metrics).

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
  Native hosting is now implemented; integrated desktop verification remains pending.
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
  tests; native hosting is now implemented, while integrated desktop clipboard
  transport and DPI remain unverified.
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

## Master functional gap fixes

Menu input in [mnemonics.rs](../crates/gui/src/menu_popup/mnemonics.rs) and
[specs.rs](../crates/gui/src/menu_popup/specs.rs) adapts pinned master's
`Main_Window::initialize` ampersand keys and FLTK `Fl_Menu.cxx::find_shortcut`.
Alt opens the eight top-level menus; visible menu items accept their source
letter with or without Alt. The deepest submenu wins, followed by its parent
and the menu bar, preserving source order. Disabled rows and closed popups cannot
activate, and editor modals block this input route. The normal action callbacks
still own history, dirty guards and modal opening. Existing menu geometry,
artwork and slider painting are unchanged; underlines/fonts remain deferred.
[Headless tests](../crates/gui/src/menu_popup/tests.rs) cover top-level switching,
held Alt, nested/parent selection, disabled items, Escape, stale popup bindings
modal blocking and escaped/Unicode recent-filename mnemonics. A regression test
reproduced macOS Option+F inserting ƒ into an already focused editor field; menu
activation now consumes its text event and releases editor focus before opening
the popup. Unmatched typing in the open menu cannot alter that field.
The [Windows plusplus workflow check](#plusplus-windows-native-workflow-checks)
subsequently found that a complete Alt chord delivered between frames was
ignored after Alt release. The shared handler now matches the pressed event's
modifiers instead of the frame's final modifier state. A failing-then-passing
regression covers both profiles, shifted letters and Ctrl/Command exclusion;
the original held-Alt, modal, submenu and macOS text-input tests remain in place.
The same native Alt+F input opens File after the fix. Remaining desktop keyboard
acceptance stays in M6/PP3; this does not update the already published candidate.

[Mac home discovery](../crates/platform/src/legacy_home.rs) adapts pinned FLTK's
`Fl_cocoa.mm::getUserPreferencesDirectory`: use nonempty existing HOME, then an
existing `NSHomeDirectory`, then the real user's passwd home. The last lookup
uses bounded `getpwuid_r` storage instead of the original shared C buffer.
No environment variable, home directory or original settings file is modified.
Pure tests cover lazy precedence, empty/missing/nonexistent HOME and failure;
macOS-only tests call the actual Foundation/passwd providers without changing
the process environment. Reading the actual legacy preference file and NG's
existing-store precedence continue through the previously tested import path.

Windows format, Clippy with warnings denied, workspace tests, rustdoc and
production-adapter checks pass. The standalone macOS platform crate cross-check
also passes; native Foundation/passwd tests passed on both Mac package runners.
[Final candidate CI](https://github.com/vulcandth/polished-map-ng/actions/runs/41236338190)
passed all eight jobs, including all three OS suites. Native legacy settings restart and
window lifecycle acceptance remain M5/M6; implementing discovery does not close
them. Development package and corresponding-source evidence belongs in
[releases](releases.md#development-packages).

## Plusplus source audit

PP1 compares master `8b9bdc2b8aac8c865809516c2123af69c498da0b` with plusplus
`27c1f8479107c7495ac0c1818900190a137da6e1` (2.7.1). The diff changes 43 `src/`
files, adds `deep-tile`, and removes `palette-map`. Immutable sources are in
[the upstream lock](../upstream.lock.toml). The following inventory owns the
functional differences; it is not a claim that the port is complete.

| Family / pinned source | Difference and reuse plan | Owner |
|---|---|---|
| `tile`, `deep-tile`, `metatile`, `metatileset` | Raw pixels are independent of each block occurrence's offset, bank, palette, flips and priority. Map/block IDs remain bytes. Use separate attribute models; reuse map/history/collision storage. | PP2 |
| `block-window`, `map-buttons` | 512 raw tile choices; occurrence palette/flip/priority controls, right-click pick and mirrored brush footprints. Raw graphics copy does not copy occurrence attributes. | PP2 commands, PP3 controls |
| `tileset`, `tiled-image`, `image` | Up to 512 tiles in linear sheet order; compound before/main/after graphics with 16-tile row padding; no master's $60 hole. Reuse bounded 2bpp/LZ/PNG codecs, pixel edits, PNG output and clipboard conversion; do not expose master CHR import. | PP2 |
| `colors`, removed `palette-map`, `palette-window` | Eight game palettes across the same six sets; default first eight colors match master. No per-raw-tile palette-map ASM or editable monochrome ninth palette. Reuse RGB conversions, safe parsing/export and swatch controls with profile limits. | PP2–PP3 |
| `config`, `option-dialogs` | Root discovery uses `main.asm`/`layout.link` rather than master's Makefile; different asset candidates, compound/base-name fallback, no master ASM map/CEL/CHR workflows. Reuse parsers/path primitives only where precedence and macros agree; preserve explicit profile choice. | PP2–PP3 |
| `map-events`, `event` | 26 supported event macros: removes master's four pokered macros, adds `pokemon_event` and seven Red++ macros. Reuse coordinate/field/warp/edit/save logic with a profile-specific catalog; include inverted punctuation symbols. | PP2–PP3 |
| `map`, `main-window`, `ruler` | Common byte-map editing/history, resize, scrolling, guides and Print composition. Source menus replace 256-tile/monochrome/tile-priority settings with 512-tile and bank ordering. Preserve shortcuts, guards and per-occurrence overlays. | PP3 |
| `tileset-window`, `roof-window` | Raw hue editing uses `Deep_Tile`; 512 selection/clipboard bounds, monochrome tile preview, roof overlay at ordinal $00a. Share pixel algorithms, drafts, native hosting and save protection. | PP2–PP3 |
| `main`, `version`, preferences | Separate upstream identity `polishedmap-plusplus`; `all512=0`, `swap01=0` defaults. Separate NG launch/storage namespace; share platform adapters. No automatic switching before U1–U3. | PP3 |
| Unchanged platform/widgets/help mechanisms | Reuse current native adapters, dialogs, persistence, licensing and test harnesses. Audit branch-specific menu/help contents before exposing them. Do not recreate FLTK painting. | PP3–PP4 |

### Addressing and serialization contracts

Attributes use bits 0–2 for palette, bit 3 for bank, bits 5/6 for X/Y flip and
bit 7 for priority. Bit 4 is discarded by the original reader/writer. Offsets and
attributes are separate 16-byte rows. Attribute files may end after fewer whole
rows than offsets; remaining rows are zero. Excess/partial rows fail. Resaving
an edited blockset emits all rows with bit 4 cleared.

| Graphics ordinal | Default bank:offset | Bank 0 first option |
|---|---|---|
| $000–07f | $0:00–7f | $0:00–7f |
| $080–0ff | $1:00–7f | $1:00–7f |
| $100–17f | $1:80–ff | $0:80–ff |
| $180–1ff | $0:80–ff | $1:80–ff |

Changing bank order changes interpretation, not stored bytes. The original
attribute loader enables the 512 option only for a resolved ordinal **greater
than $100**, while graphics count enables it above 256 tiles. Preserve this
threshold in compatibility code; it is not the ordinary >=256 test.

Brush footprints share master's eight modifier patterns, but flip the destination
coordinates across the entire footprint and apply occurrence attributes. At
sheet edges, signed intermediate indices pass through `Tile::bank_offset`;
modulo-512 arithmetic would produce different results. The port bounds public
selected indices while reproducing these internal conversions.

`config.cpp` searches full compound names before the base before the first dot.
Offsets/attributes search `data/tilesets` then `tilesets`. Collisions search
`data/tilesets/*_collision.asm`, `.inc`, then `tilesets/*_collision.asm`, `.bin`;
master's `data/tilesets/*_collision.bin` candidate is absent. Graphics use
`gfx/tilesets` with PNG → 2bpp → 2bpp.lz, roofs use `gfx/tilesets/roofs`.
No palette-map file belongs to a plusplus blockset.

### Geometry baseline

The block editor client is 477×432, with the tileset scroll viewport at
(10,36,273,258), 16 columns of 16-point tiles and up to 32 rows. The block grid
is (337,36,130,130); palette, flips and priority occupy rows y=176,202,228.
Collision fields start at (313,279), width 154, height 22, with 26-point spacing.
The raw tileset editor is 441×304; roof editor 226×228; palette editor height 191
and width `295 + max(text_width("YELLOW:", 2), text_width("BROWN:", 2))`. These are baseline source measurements, not completed
Rust GUI evidence. PP3 must test final text-dependent formulas and hit bounds;
fonts, widget painting and exact FLTK appearance are excluded from this pass.

### Polished Crystal fixture

The [pinned project](https://github.com/Rangi42/polishedcrystal/tree/1226ea17bb0bdf5de1bad706a688fe89af573981)
is the last revision before the pinned plusplus source date. Its 51 blocksets all
have matching complete offset/attribute rows and at most 256 blocks. NewBarkTown
has 90 map bytes, and its johto_traditional blockset has 248 blocks. The compound
main PNG is 128×80 (160 tiles), with a 128×64 (128-tile) johto_common component.
These structures fit the source's expanded format. The backend evidence below
now covers project discovery, rendering and map/block/graphics/roof/palette/event
roundtrips. Original-application reopening and native workflow acceptance remain
PP4. Follow the independent [fixture policy](assets.md#polished-crystal-fixture-policy).

## Plusplus block backend

[Core](../crates/core/src/plusplus.rs) implements explicit bank ordering,
canonical attributes, short-row padding, occurrence painting, copy/swap/resize,
collision edits and atomic command batches. Shared `Blockset` storage keeps its
byte-sized offset behavior; only common brush footprint/stride math was factored.
No plusplus settings, raw tile IDs or palette semantics leak into the master GUI.

[Project persistence](../crates/project/src/plusplus.rs) discovers the plusplus
block paths and loads offsets/attributes with a 4096-byte cap. Saving uses the
existing content-hash conflict checks, staging/backups and rollback machinery
for two or three files. It preserves unrelated collision assembly and requires
explicit paired Save As destinations. This is guarded multi-file saving, not
crash-atomic filesystem transactionality; interrupted recovery remains A2.

Verification on Windows, with the plusplus source comparisons and persistence/real-project
suite also passing on WSL Ubuntu 22.04.5:

- [Independent cases](../crates/core/tests/plusplus.rs) cover bank boundaries,
  row errors/padding, canonicalization, flipped footprints, 256-block bounds,
  collision synchronization and failed-batch atomicity.
- [Persistence tests](../crates/project/tests/plusplus.rs) cover full-name
  precedence, excluded master paths, exact save/reopen, unrelated ASM retention,
  conflicts in all three files and duplicate destinations.
- [Original-source comparisons](../crates/project/tests/plusplus_reference.rs)
  compile unchanged method bodies using the [oracle](../tools/plusplus_block_oracle.py).
  All 131,072 offset/attribute/order inputs and 524,288 modifier/tile/click/flip
  combinations match. The harness tests arithmetic, not FLTK event delivery.
- All 51 pinned real blocksets pass swap → save → reopen → swap back → save,
  restoring exact offsets, attributes and collision bytes in temporary copies.
  Fixture files are neither modified nor distributed. CI repeats these checks
  separately from the existing master suite.
- Workspace format, Clippy (`--all-targets`, warnings denied), tests and rustdoc
  pass on Windows. The original master brush differential also passes after
  factoring shared footprint math. Production native-feature validation and
  documentation link/CLI-example checks pass. No GUI interaction was needed;
  these are backend checks, not native application acceptance.

The [graphics backend](#plusplus-graphics-backend) supplies raw editing,
rendering and expanded codecs. Palette sessions, map/root/asset discovery and
branch event catalogs are also implemented in the sections below. PP3 has no
plusplus GUI yet; PP4 original-application interoperability/native acceptance remains open.

## Plusplus graphics backend

[Raw graphics and rendering](../crates/core/src/plusplus/graphics.rs) adapt pinned
plusplus `deep-tile`, `tileset-window`, `metatileset`, `tileset` and `image`.
`RawTileset` keeps 1–512 validated hue tiles and independent defined flags;
occurrence attributes remain in `AttributeBlockset`. Painting/filling/replacing
hues defines the selected tile; copy/swap transfers hues and definition state;
delete clears hues and undefines the tile. Batch failures leave the complete
original state intact. Clipboard RGB paste clips to 16 columns and either 16
or 32 rows according to the 512 option, retaining uncovered pixels.

The [shared pixel operations](../crates/core/src/pixel_edit.rs) now serve master,
plusplus and master roofs. Plusplus trim clears every unused blank tile, including
interior holes and tile zero, unlike master's trailing-only trim. Linear grayscale
export pads partial rows white and bounds source ranges before allocation. The
nonzero-offset source overread is covered by the [safety exception](deviations.md#plusplus-graphics-export-bounds).

The plusplus renderer selects a raw tile by bank order, applies the occurrence
palette and X/Y flips, substitutes the nine roof tiles at ordinal $00a when
provided, and reports occurrence priorities. Undefined tiles use the original
diagnostic palette; absent block IDs use EMPTY_RGB. The existing map and Print
compositors now consume an internal block-raster interface, preserving master's
public APIs and Print ordering, clipping, overlay, font-mask and event behavior.
This is shared infrastructure for later combination, not a unified application.

[Expanded loaders](../crates/project/src/plusplus.rs) reuse the existing bitplane,
LZ and PNG implementations with explicit plusplus bounds. Master retains 256-tile
limits and CHR support. Plusplus accepts 512 tiles, including 4096×8 and 8×4096
PNGs, rejects CHR, and preserves the original LZ loader's 8192-byte zero-padded
output buffer. PNG byte allocations remain bounded and preserve the existing
Windows/Unix alpha/desaturation distinction.

Verification:

- [Core graphics cases](../crates/core/tests/plusplus_graphics.rs) cover high-index
  edits, undefined state, trim holes, image paste at both bank limits, malformed
  commands, nonzero-offset export, flipped corner colors, roofs and shared Print.
- [Codec cases](../crates/project/tests/plusplus.rs) cover 512-tile roundtrip,
  partial/excess input, LZ output caps, wide/tall PNGs and excluded CHR files.
- The [original-source oracle](../tools/plusplus_block_oracle.py) now also compiles
  unchanged `Metatileset::draw_metatile`. Synthetic 2× RGB tile buffers and a
  recording `fl_draw_image` adapter verify its signed pixel/row strides at native
  map resolution. All 16,384 bank-order/roof/base-offset/attribute combinations
  match Rust pixels. This does not assert driver/font or native window fidelity.
- Master's compiled-source pixel edits/RGB paste and all-options Print comparisons
  still pass after the shared-code extraction. The existing master suite remains
  mandatory; plusplus tests do not replace it.

Windows workspace format, Clippy (all targets, warnings denied), tests and rustdoc
pass. The new graphics, expanded-codec and compiled-source comparisons also pass
in WSL Ubuntu 22.04.5. Production native-feature validation passes; no GUI
interaction was used for these backend checks.

[Compound graphics and roof documents](#plusplus-project-graphics-and-roofs) now
connect the model to project files. [Contextual palettes and map discovery](#plusplus-map-discovery-and-palette-sessions)
and [branch events](#plusplus-events-and-script-persistence) are also implemented.
No plusplus GUI or CLI editing workflow is claimed from these library tests;
integration and acceptance remain PP3–PP4.

## Plusplus project graphics and roofs

[The tileset document](../crates/project/src/plusplus/tileset.rs) resolves PNG →
2bpp → 2bpp.lz under `gfx/tilesets`, splits before/after components using the
pinned first-dot/first-comma rules, and composes a 512-slot raw model. Before
components round up to 16 tiles; main components round up only when a nonempty
after component was loaded. Padding retains undefined flags. Loaded counts above
256 enable the 512 option; aggregate counts above 512 fail before indexing.

Save planning preserves source PNG destinations, including named optional
components. Explicit zero component counts mean infer the defined tail, as in
source `Image::write_tileset_image`; missing-before behavior is tested rather than
silently reinterpreted. All images encode before replacement. Source and output
hashes are checked, duplicates and impossible reopened sizes fail in preview,
and the shared staging/rollback layer installs the outputs. Binary imports remain
intact; after PNG promotion, observations follow the active PNG source. No palette
map is created or rewritten. Dirty drafts survive failed saves.

[The roof adapter](../crates/project/src/plusplus/roof.rs) selects only plusplus
formats/paths, requires nine tiles, and exposes raw snapshots/commands without
master palette attributes. It reuses the existing nine-tile editor, clipped RGB
paste, grayscale export and guarded `RoofDocument` persistence. Roof dirty state
and files remain independent of base graphics. Renderers consume its nine tiles
as an overlay; saving a base sheet does not bake that overlay into the sheet.

[Project tests](../crates/project/tests/plusplus_graphics.rs) cover row padding,
component boundaries, PNG promotion, unchanged binary inputs, all source/output
conflicts, missing optional components, duplicate destinations, aggregate bounds,
roof batch rejection, clipped paste and separate persistence. The pinned Polished
Crystal fixture passes edit → save → reopen for all 51 tilesets, comparing edited
hues and complete block-map rendering, and all six roofs, comparing all decoded
tiles. Tests edit temporary copies, never the fixture checkout. The original-app
PP4 acceptance gate is still separate from these NG roundtrips.

Windows workspace format, Clippy (all targets, warnings denied), tests and rustdoc
pass, along with native-feature and documentation checks. The complete project
graphics test file, including all real-project cases, also passes under WSL
Ubuntu 22.04.5. CI runs both the block and graphics fixture suites.

Palette sessions, metadata/root/asset discovery and branch event catalogs are
implemented below. Existing master defaults are not silently applied as plusplus policy. The
GUI/CLI integration, complete workflow checks and native acceptance remain PP3–PP4.

## Plusplus map discovery and palette sessions

[Explicit map documents](../crates/project/src/plusplus/map.rs) share the byte-map
model, history, prefix-load reports and conflict-protected persistence. Open and
Save As accept `.ablk`/`.blk`; extensionless destinations default to `.ablk`.
ASM import remains master-only. The private discovery rules preserve the nearest
`main.asm`/`layout.link` root, filename hints and header short-circuiting,
`map_const`/`mapgroup` dimensions, wrapping group byte and existing-file precedence.
Plusplus excludes master's `mapconst` and individual pokered header fallbacks.
Selectors use fixed `gfx/tilesets`, original numeric filename ordering and labels,
no CHR choices, and the pinned roof selector's case-sensitive suffix matching.
The shared master entry points retain their existing rules.

[Palette sessions](../crates/project/src/plusplus/palettes.rs) expose eight game
palettes across six sets, reuse RGB8/RGB5/hex/copy/swap editing, and validate both
ends of a transfer before changing the draft. Master-only monochrome slots cannot
be edited; 1–6-palette imports leave colors/selection untouched while still clearing
the accepted import's dirty flag, as the source does. The recognized 8, 16, 24, 32,
40, 42, 43, 44 and 48 layouts preserve source set/water assignment. All other
complete counts are no-ops. Malformed RGB input fails without changing the session.
Exports preserve the live RGB8 bank, quantize output to RGB5 and use shared source
and destination conflict guards. The existing [export-label correction](deviations.md#palette-export-labels-and-validation)
applies to both branches.

The shared import plan retains distinct default/special/selection/roof steps, so
GUI and headless callers can skip an import or retry a failure without losing the
draft or advancing past it. Plusplus special paths always use `gfx/tilesets`, even
if `MAPDATA` exists. Saved-map → landmark → full tileset → base tileset precedence,
sticky Custom selection, contextual night/indoor/morning/theme order, explicit
selection and independent roof overlays are tested. Accepting an absent default
clears palette edits; skipping it preserves them. Unsaved map suggestions never
become implicit special-palette sources.

[Map tests](../crates/project/tests/plusplus_maps.rs) cover profile isolation,
byte IDs above 127, history, oversized/short inputs, Save As root refresh, extension
rejection and concurrent-writer failures. The [shared discovery oracle](../tools/discovery_oracle.py)
compiles unchanged pinned plusplus functions; 582 cases cover filename extensions,
dimension/header macros and paths, numeric/symbolic tileset hints, precedence and
group overflow. [Palette tests](../crates/project/tests/plusplus_palettes.rs) and
the [palette oracle](../tools/plusplus_palette_oracle.py) compare all 0–64 palette
counts with all six starting selections (390 cases), 120 roof layout/group/table
cases and 16 complete export byte streams. The palette oracle uses original
defaults/functions and a synthetic parsed-color input; it does not prove native
dialogs or arbitrary ASM evaluation.

The pinned Polished Crystal New Bark Town fixture opens with its discovered
10×9 dimensions and compound tileset, runs contextual palette/roof imports,
exports/reloads selected colors, edits/saves/reopens the map and restores original
bytes and rendered pixels. All writes are temporary; no fixture/game pixels are
committed. A source limitation is explicit: the pinned header parser reads the
fourth/seventh arguments as landmark/palette, so this fixture's newer SIGN column
yields `sign_wood`/`0`. The port preserves those observations rather than silently
claiming support for the newer header schema. PP4 still requires original-app
workflow comparison and resolution or approval of remaining differences.

Windows workspace format, Clippy (all targets, warnings denied), tests and rustdoc,
plus the production native-feature check, pass. The map/palette suites,
compiled-source comparisons and real fixture also pass under WSL Ubuntu 22.04.5.
Master's compiled-source metadata comparisons still pass after the shared extraction.
CI retains master regressions and runs the new source/fixture cases.
These backend checks do not close PP3's separate GUI or PP4's native acceptance.

## Plusplus events and script persistence

[The explicit catalog](../crates/core/src/plusplus/events.rs) contains the pinned
branch's 26 macros. It excludes master's `warp`, `sign`, `object`, `warp_to`, and
adds `pokemon_event`, `pc_chansey_event`, `pc_blissey_event`, `cavebase_event` and
the four `treebase_*` macros. Macro names, coordinate order, prefix operands,
warp classification, destination operand order, marker colors and symbols follow
`map-events.cpp`/`event.cpp`. Master's 22-macro parser remains unchanged.

[Project adapters](../crates/project/src/plusplus/events.rs) reuse the shared
parser, model commands, bounded source spans, previews and conflict-protected
saves. They preserve unrelated assembly, indentation, mixed line endings and
final-newline state. Edited coordinates retain source formatting; moves, options
edits and resize translation validate before publishing the draft. Only the
three recognized warp kinds increment IDs, and local targets use those IDs.
External warp paths preserve the current extension and directory; invalid map
operands cannot escape it. Shared GUI existence, already-open and dirty-consent
handling now use that path boundary and the plusplus asset/session loader.
End-to-end navigation and native acceptance remain PP3 verification.

Script discovery uses `maps/<name>.asm`, then an adjacent `.asm` filename when
opening an existing map. Master-only SXY/pokered paths are ignored. Unsaved maps
do not auto-load scripts, and a malformed preferred script is an error rather
than a reason to load a different fallback. The shared Print font adapter now
accepts the four punctuation symbols, with single-character validation and
bounded rasterization; this is functional symbol support, not resumed font matching.
The [recorded event safety differences](deviations.md#event-assembly-preservation-and-validation)
cover strict operands, negative-hex portability and valid Unicode punctuation.

[Event tests](../crates/project/tests/plusplus_events.rs) verify catalog isolation,
marker metadata, warp numbering/navigation, field normalization, atomic failure,
file conflicts, repeated save/reopen and complete Print composition
with all symbols. The [shared source oracle](../tools/event_oracle.py), selecting
the pinned plusplus revision explicitly, compares 40,530 coordinate/base/macro and
warp-destination cases against original functions and metadata. Its driver reports
symbol bytes numerically; the source functions and table are unchanged.

All 607 `.asm` map scripts in the pinned Polished Crystal fixture parse and
roundtrip unchanged. A temporary-copy test moves every one of their 6,518 events,
saves/reopens, and compares coordinates, kinds, source line numbers, prefix/suffix
data, warp IDs/destinations and every unowned source line. It includes 65
`pokemon_event` records. The fixture has no Red++ secret-base records; those use
synthetic, compiled-source and Print coverage, not a claim of real Red++ project
acceptance. No fixture scripts or generated game images are distributed.

Windows format, workspace Clippy with warnings denied, workspace tests, rustdoc,
native-feature and documentation checks pass. The complete event suite, including
source comparisons and all fixture edits, passes on Windows and WSL Ubuntu
22.04.5. The compiled master event comparison also passes under Linux after the
shared parser extension. CI runs the plusplus reference and fixture cases in
addition to all existing master checks. No GUI automation was used here.

PP2's backend implementation is complete with the evidence in this and the four
preceding plusplus sections. Master regressions remain mandatory. PP3 still owns
the separate application, controls/layout, preferences and workflow integration;
PP4 owns comparison in the original application and native acceptance. Backend
roundtrips do not close either gate.

## Plusplus shared GUI integration

**PP3 is in progress.** [GUI document adapters](../crates/gui/src/documents.rs)
retain the explicit master or plusplus backend while exposing common map/history
and event views. The owning map selects script discovery and Load/Reload parsing;
loading a plusplus script never expands master's catalog. Shared Event Options,
move/resize, dirty guards and save feedback operate on either document. Save As
retains plusplus `.ablk`/`.blk` restrictions, defaults extensionless destinations
to `.ablk`, and records that destination in recent history. Print filenames remove
the plusplus `.ablk` suffix. Origins are pinned `Main_Window` lifecycle/warp/Print
callbacks and `map-events.cpp`; backend source comparisons remain applicable.

[Renderer dispatch](../crates/gui/src/documents/graphics.rs) carries the concrete
CPU renderer through the shared canvas texture cache, priority overlays and Print
worker snapshots. It never flattens occurrence attributes into master's tile
palette map. Master asset-editor paths retain their existing renderer and reject
a mismatched plusplus renderer before publishing block changes.

[Headless integration tests](../crates/gui/src/documents/tests.rs) load synthetic
plusplus maps/graphics through the editor's normal pending-load path. They edit
all 26 macros through Event Options, check its shared 300-point width, save/reload
with macro identity intact, and preserve unrelated mixed-line-ending assembly.
They exercise map paint/undo/redo, extension handling and recent paths, invalid
Save As, concurrent-writer failures, atomic invalid resize, event translation,
dirty reload cancellation, automatic script precedence and confined warp paths.
An independent pixel vector checks ordinal 511, palette 7, both flips and occurrence
priority, then checks the Print Copy snapshot after a subsequent map edit. The
fixture uses generated tiles/colors and does not include game assets.

[Asset-session integration](../crates/gui/src/plusplus_session.rs) uses the shared
Open/New forms and load worker with explicit plusplus discovery and formats. It
loads offset/attribute/collision documents, compound raw graphics and selected
roofs, then drives default/special/roof palette imports through the existing
Load/Skip prompts. The previous map remains live until the candidate succeeds.
Expanded graphics or block references enable the 512 setting using the pinned
thresholds. Internal profile guards reject mismatched Open options, map/renderers
and palette drafts; no automatic profile switch is exposed.

[Palette views/drafts](../crates/gui/src/documents/palettes.rs) let the existing
palette dialog edit plusplus's eight palettes and six sets, with its source-derived
191-point client height and measured width. RGB/hex, rounded sliders, transfers,
native Windows chooser and import/export paths keep their shared controls.
Plusplus never exposes the master monochrome palette, and exports preserve RGB5
quantization. Palette selection/edits refresh map textures independently of map
history and asset dirty flags; closing a map retains the palette session.

The shared Save All sequence now dispatches plusplus raw graphics, roof,
offset/attribute/collision, event and map saves in order, waiting for each success
message and stopping on failure. A clean Save Tileset trims unused blank definitions
without dirtying pixels or writing files, matching `Metatileset::trim_tileset`
before the source callback's dirty check. Attribute saves include all paired
outputs and use existing guarded persistence; no master palette-map file is created.

[Asset-session tests](../crates/gui/src/plusplus_session/tests.rs) use the actual
Open dialog/worker, palette editor, load prompts and save dispatch. They cover
cancel/accept previews, eight-palette limits, export/reload quantization, close
retention, wrong-profile rejection, failed imports preserving the prior map,
ordered saves and a conflicting attribute file preventing later map writes.
The pinned Polished Crystal New Bark Town also opens through the GUI worker with
10×9 blocks, its 248-block compound tileset and 288 loaded raw tiles. The test edits
and exports palettes, edits the map and saves/reopens a temporary copy, while
checking the fixture map and observed assets remain unchanged. This fixture test
is in CI; it does not claim original-application interoperability.

Windows workspace format, Clippy with warnings denied, tests, rustdoc and the
production native-feature graph pass. The five map/event integration tests and
six asset-session tests also pass on WSL Ubuntu 22.04.5 with no display server.
The pinned-project GUI worker test passes on both Windows and WSL; dirty tile-save
conflicts preserve the live definition flags as well as file bytes. Existing master tests remain in the
workspace run. These are input/model/rendering checks, not native-window,
clipboard-service or macOS runtime verification.

The default entry point remains master; the [separate plusplus launcher](#plusplus-launch-settings-and-help)
uses these shared workflows. Plusplus [block controls](#plusplus-block-editor-and-transfers),
[tile/roof controls](#plusplus-tile-and-roof-editors) and
[asset switching](#plusplus-asset-switching-and-tile-options) are implemented.
The [source reconciliation](#plusplus-gui-source-reconciliation) accounts for
menu/default differences; full native workflow verification remains in PP3. PP4 still requires original
application interoperability and native acceptance; these tests do not close it.
No desktop automation or aesthetic matching is part of this change.

## Plusplus block editor and transfers

PP3's [shared block dialog](../crates/gui/src/block_editor.rs) and
[occurrence controls](../crates/gui/src/block_editor/plusplus.rs) adapt the pinned
plusplus `Block_Window::{initialize,refresh,select,change_chip_cb,change_attributes_cb}`
and `map-buttons.cpp` tile/chip handlers. The client is 477×432 points; the tile
list is 273×258 or 273×386 with 16-point cells, and the block grid has 32-point
chips inside its 130-point frame. Existing collision controls, platform button
order, native modal hosting, middle-click defaults and accept/cancel are shared.
Eight palette choices, X/Y flips, priority, Alt+1–8/X/Y/P and bank:offset labels
retain occurrence semantics. Right-button press picks all attributes and reveals
the selected row. A held right click no longer loses the pick to egui's drag
threshold; the shared master picker also follows the original press timing.
Flip/priority choices survive reopening the dialog, including after Cancel;
the selected raw tile and palette reset as in `refresh`. Priority is always
shown in block drafts, independently of the main map's display option.

The visible sheet expands for the 512 option or any current-block reference
at or above ordinal 256, including the boundary case distinct from automatic
load-time option detection. Both source bank orders are preserved. A single
eight-palette texture atlas uses the core occurrence renderer for palettes,
roof substitution and undefined colors; UVs handle flips. The map compositor
and block atlas share the same pixel routine. Painting calls the existing
attribute backend, including mirrored multi-tile footprints and edge clipping.
Accept validates the entire draft and constructs its preview before replacing
live state; Cancel leaves blocks, collisions, map history and files unchanged.

[Shared sidebar workflows](../crates/gui/src/block_workflows.rs) preserve offset,
attribute and collision rows through clipboard snapshot paste, live-ID swap,
drag copy/swap and resize. Shrinking prunes stale clipboard/hotkey/selection
state without rewriting the map. Master-only Set/Paint commands cannot flatten
plusplus attributes; invalid batches retain the old model and preview. Block
changes use the integrated guarded offset/attribute/collision save path.

[Input/geometry tests](../crates/gui/src/block_editor/plusplus/tests.rs) cover both
bank orders and ordinal 511, scroll reveal, mirrored brushes, attribute shortcuts
and repeat suppression, Option-generated text isolation, collision layouts,
cancel/accept, save/reopen, transfer/resize paths and atomic failures. The rendered
pixel test checks independent RGB values for both flips, palette 7 and roof
substitution; it does not add or replace aesthetic snapshot baselines.
[Core tests](../crates/core/tests/plusplus_graphics.rs) also compare individual
occurrences with the existing independent map/Print pixel vector.
The pinned-project GUI workflow additionally edits a New Bark Town block and
saves/reopens its accepted model through the backend to temporary paths; original
fixture files remain untouched. This is not original-application acceptance.

Windows and WSL Ubuntu 22.04.5 block input/geometry tests pass, as does the Windows
offscreen rendered-pixel check. Windows workspace format, Clippy with warnings
denied, tests, rustdoc and production adapter checks pass. The extended pinned-project
workflow passes on Windows and WSL. Native window, clipboard and original-application
checks remain PP3/PP4. FLTK painting and original fonts remain deferred.

The preceding asset-integration CI run passed its Ubuntu workspace/native checks
but the runner exhausted disk during the final font-test builds. The shared
[CI workflow](../.github/workflows/ci.yml) now disables incremental compilation
on fresh runners and uses level-1 debug information for dev/test profiles. This
retains source locations in backtraces and all existing verification jobs while
reducing build storage; release profiles and packaged binaries are unchanged.

## Plusplus tile and roof editors

The [shared tile dialog](../crates/gui/src/tileset_editor.rs) and
[explicit drafts](../crates/gui/src/tileset_editor/draft.rs) adapt plusplus
`Tileset_Window`, `Roof_Window` and `map-buttons.cpp` at the pinned revision.
The raw tile client is 441×304 points with a 273×258 scroll frame, 16-point
sheet cells, 17-point pixels and horizontal swatches. The roof client remains
226×228 with reversed thumbnail rows and the source's 146-point frame around
its 136-point pixel grid. Platform button order follows the original coordinates.
No original font or FLTK widget-painting work is included.

The sheet exposes 256 or 512 raw tiles, including master's otherwise reserved
offsets, with bank:offset labels in either configured order. Reopening resets
selection and scrolling to the first tile. Raw graphics use monochrome editing;
undefined sheet thumbnails retain their distinct colors. Palette, occurrence
flip/priority and master monochrome restrictions do not apply to raw tiles.
Roof selection starts at ordinal six; Delete remains unavailable there.

Shared paint/fill/replace/swap-hue gestures, snapshot paste, live-ID swap,
drag copy/swap and image clipboard routing call the existing backend commands.
Roof image paste affects only the selected tile, retaining the existing
[safe clipping policy](deviations.md#tileset-editing-and-export-safety). Acceptance marks the
edited domain dirty even without pixel changes, composes its preview before
publication and preserves other assets and map history. Cancel discards the
draft. Saves retain the source PNG destinations and conflict guards; no master
palette-map file is written.

[Input tests](../crates/gui/src/tileset_editor/plusplus_tests.rs) verify geometry,
both bank orders, ordinal 511, clipped image paste, definition-preserving
clipboard operations, drag behavior, reopen reset, dirty-domain isolation and
save conflicts. An offscreen rendered-pixel test checks monochrome and undefined
RGB values directly. Existing master tile snapshots pass without new baselines.
The [real-project GUI test](../crates/gui/src/plusplus_session/tests.rs) copies
Polished Crystal's compound Johto graphics, blocks and New Bark roof to temporary
storage, then edits, saves and reopens both graphics domains through the shared
GUI. It compares reloaded pixels and confirms the original inputs are unchanged.
CI runs this alongside the existing real-project map/palette/block workflow.
These checks do not establish native clipboard/window or original-application
acceptance; PP3/PP4 retain those gates.

Windows workspace format, Clippy with warnings denied, tests, rustdoc and the
production adapter graph pass. The five new tile/roof input tests and both
pinned-project GUI workflows pass on WSL Ubuntu 22.04.5 without a display server.
Windows offscreen tile/roof rendering checks pass with unchanged master snapshots.

## Plusplus asset switching and tile options

The existing [tileset](../crates/gui/src/tileset_workflows.rs) and
[roof selectors](../crates/gui/src/roof_workflows.rs) now dispatch through the
fixed editor profile. Plusplus uses its own discovery, attribute/collision
documents, compound graphics and roof codecs while retaining the shared 280×80
client geometry, shortcuts, platform button order and two-stage cancellation.
Source behavior comes from pinned plusplus `Main_Window::{change_tileset_cb,
change_roof_cb,read_metatile_data,force_add_sub_metatiles}` and
`option-dialogs.cpp`. Changing assets never changes the editor profile.

A tileset replacement stages all assets and previews before adoption. It retains
the current roof name, palette session, map bytes/history and events; removed
block IDs are pruned from selection/clipboard/hotkeys without rewriting the map.
Block, raw tile or roof edits trigger the unsaved warning; map/palette edits do
not. Roof replacement warns only about roof edits and retains the original
sticky roof dirty flag. Removal clears that domain. Read failures preserve the
loaded assets, following the existing documented
[roof](deviations.md#loaded-roof-replacement-safety) and
[tileset safety policies](deviations.md#loaded-tileset-replacement-safety).

The plusplus Options menu replaces master's Monochrome, Tile Priority and
256 Tiles rows with 512 Tiles and `$0:80-FF Before $1:80-FF`, including source
mnemonics. `allow_512_tiles_cb` changes visibility before its guarded same-name
reload; Cancel retains the option and existing edits. Successful reload can
re-enable 512 tiles when graphics/attributes require it. Bank-order input is
disabled while 512 Tiles is off. `arrange_0_before_1_cb` changes only interpretation
and rendering: it preserves raw pixels, occurrence bytes, dirty flags and
history, without a reload, confirmation or disk write. These options persist in
the [isolated plusplus settings](#plusplus-launch-settings-and-help).

[Input tests](../crates/gui/src/plusplus_session/switching_tests.rs) exercise
successful replacement and rendering, both cancellation stages, all replaced
dirty domains, a late roof failure, removal, mode reloads, automatic expansion,
disabled input and keyboard bank ordering. The copied
[Polished Crystal workflow](../crates/gui/src/plusplus_session/tests.rs) switches
between traditional/modern compound Johto tilesets and New Bark/Violet roofs
before the existing edit/save/reopen checks. Original project inputs remain
unchanged. Native selectors and original-application acceptance remain PP3/PP4.

Windows workspace format, Clippy with warnings denied, tests, rustdoc and the
production adapter graph pass. The five switching tests and both real-project
GUI workflows pass on WSL Ubuntu 22.04.5 without a display server. These checks
exercise source geometry and input/model behavior; they do not establish native
dialog or compositor acceptance.

## Plusplus launch, settings and Help

The [separate executable](../crates/gui/src/bin/polished-map-plusplus.rs) selects
the fixed plusplus profile before constructing the editor. Master remains
Cargo's default GUI binary. Both call the same
[native startup](../crates/gui/src/preferences/legacy.rs), native font/platform
adapters and command-line map opening. Pinned plusplus `main.cpp`, `version.h`,
`preferences.cpp` and `Main_Window::{Main_Window,initialize,exit_cb}` supply the
identity, defaults and legacy keys. The initial client remains 800×600 with a
335×262 minimum. Root titles identify NG++, including after closing a map.
No automatic profile detection or conversion is introduced.

The source review also found a retained master file filter in Open's accepted
discard continuation. Both initial Open and that continuation now call the same
profile-aware chooser: plusplus offers `.ablk`/`.blk`, master `.blk`/`.map`.
Native chooser display remains a desktop acceptance check.
[Lifecycle input tests](../crates/gui/src/plusplus_session/tests/lifecycle.rs)
cancel Open, Close and Exit for each dirty map/event/tile/roof/block domain,
confirm retained edits and unchanged source bytes, then accept Close and check
the empty plusplus session/title. A complete New → Save As → Close → Open
workflow exercises default dimensions/name, unsaved creation, plusplus assets,
binary save/reopen and preservation of the existing project. The native Save
chooser is substituted at its returned-path boundary, not operated by these tests.

The app ID `polished-map-ng-plusplus` isolates the native `app.ron` path and
window record; its JSON preference key is also distinct. The pinned eframe
WGPU integration passes the viewport app ID to its file-storage constructor.
Common settings and legacy parsing are shared, while `all512`/`swap01` replace
master's tile modes and `meta128`. Existing NG files win even if malformed;
legacy discovery uses only the plusplus filenames and never writes the source.
[Settings documentation](preferences.md#separate-plusplus-settings) owns paths
and defaults. Tests use the production plusplus constructor/save callback to
verify own-key restoration, master isolation, native viewport identity,
first-launch precedence, malformed-record retention and ignored master flags.
Project tests cover all three platform path rules and Unix vendor precedence.

The shared [Help viewer](../crates/gui/src/help.rs) selects the branch's complete
pinned document, retaining source layout, platform substitutions, links and
the explicit incomplete-parity notice. The plusplus variants each contain 104
blocks and 15 linked spans. On WSL, compiling the original C++ literals produced
15,119 Unix bytes and 15,143 Windows bytes exactly matching the converter.
Master's 95-block/16-link documents remain unchanged. About identifies
Polished Map++ 2.7.1 and retains upstream, port and asset attribution. Source
bytes, hashes and LGPL notice are in the [Help asset record](assets.md#branch-specific-help-documentation).
Windows offscreen Help/About tests pass with unchanged master snapshot baselines;
plusplus input tests verify branch identity, reference text and shared dismissal.

Windows workspace tests, format, Clippy with warnings denied, rustdoc, both GUI
binary builds and the production adapter graph pass. WSL Ubuntu 22.04.5 passes
the shared preference tests, legacy parser/discovery tests and both copied
Polished Crystal GUI edit/save/reopen workflows, and builds the plusplus binary.
That binary also remained running through a 15-second isolated Xvfb/Vulkan
startup check with temporary home/settings directories. Timeout termination
does not verify graceful shutdown or preference persistence.
These tests do not establish native storage/restart, compositor placement,
clipboard or original-application interoperability. Complete PP3 workflows and
PP4 acceptance remain open; fonts and FLTK painting remain deferred.

## Native profile preference lifecycle

The opt-in [native example](../crates/gui/examples/native_preferences.rs) runs
master and plusplus in independent processes with real eframe native windows,
file storage and shutdown flushing. Its
[probe](../crates/gui/src/preferences/legacy/native_probe.rs) wraps the production
editor, delegates its UI/save/autosave/persistence hooks, and calls the normal
exit action. Only fixture paths and the app wrapper differ from normal startup.
The example supplies temporary legacy files and explicit native persistence
paths, so the user's stores are neither read nor written. The profile still
selects its real app ID, legacy filename and JSON key.

For each profile, the first process imports 640×480 normal geometry and selected
legacy settings, changes one setting and exits. A second process verifies the NG
record wins over the unchanged legacy file, restores that size, resizes to
720×520 and exits without an intervening autosave. Subsequent processes verify
that resized geometry and settings survive native restart. A simulated future
application record then survives two further process exits unchanged while
defaults are used. Every child must complete a single native shutdown save;
the parent verifies original legacy bytes and the other profile's entire store.
No synthetic call to the save callback substitutes for native shutdown.

This covers temporary-path storage, normal sizing and empty-session exit. It
does not exercise default OS store-directory discovery, moved/maximized/fullscreen
windows, mixed DPI, Wayland compositor placement, power-loss recovery, or
interactive canceled exits with loaded documents. Those remain M5/M6/PP3 gates.
The [CI matrix](../.github/workflows/ci.yml) runs this opt-in feature on all three
OS families, with Xvfb for Linux; local instructions are in [INSTALL](../INSTALL.md).

All ten native child processes pass locally on WSL Ubuntu 22.04.5 using an
isolated Xvfb display and Vulkan. Windows workspace format, Clippy, tests,
rustdoc and opt-in-feature Clippy pass. Windows, macOS and Ubuntu native settings
probes in
[the probe CI run](https://github.com/vulcandth/polished-map-ng/actions/runs/41267258063)
also pass. This is new development-branch evidence and does
not retroactively certify the published master candidate or close M5/PP3.

## Plusplus sheet panning

Pinned plusplus `Block_Window` and `Tileset_Window` use `Workspace` for their
expanded tile lists. The shared [panning adapter](../crates/gui/src/workspace_pan.rs)
now supplies the same middle-button capture, integer anchor/delta calculations,
clamping, pointer/focus-loss cancellation and no-fling behavior to both lists.
Wheel and scrollbar movement remain available; panning suppresses tile selection
and copy/swap drags. Each reopened draft resets its scroll state.

Plusplus's `widgets.cpp::Workspace::handle` adds the frame's `box_dw`/`box_dh`
to the outer scroll bounds. The port passes the inset list client rectangle,
which produces the same calculation without another frame allowance. The raw
512-tile list limit is `512 - 258 + 2 = 256` points; the expanded block-editor
list is `512 - 386 + 2 = 128`. The 256-tile lists have a zero scroll limit.
Main map/sidebar Workspaces use `FL_NO_BOX`, so the branch's additional frame
term is zero there and the existing shared bounds remain valid.

[Block input tests](../crates/gui/src/block_editor/plusplus/tests.rs) and
[raw-tile input tests](../crates/gui/src/tileset_editor/plusplus_tests.rs) exercise
256/512 modes at 1×/2× DPI, dragging past both limits, returning from an edge
without rebasing the anchor, release and retained selection/attributes/pixels.
Cancel leaves the project unchanged. The surrounding suites retain wheel-scroll,
high-ID selection, right-pick reveal, primary-button copy/swap and reopen tests.
Native pointer capture outside a real editor window remains PP3 acceptance.

Windows format, Clippy with warnings denied, workspace tests and rustdoc pass.
Both new panning tests pass on Windows and WSL Ubuntu 22.04.5. Windows offscreen
tile-editor checks pass with unchanged master snapshot baselines.

## Plusplus GUI source reconciliation

The PP3 review compares the GUI-related changes among the 43 source files in
the pinned master-to-plusplus diff. The implemented families below retain
separate documents/launchers and shared controls. This completes source inventory
reconciliation, not native or original-application acceptance.

| Source difference | Implemented behavior and evidence |
|---|---|
| `main`, `version`, `Main_Window` constructor/exit | Fixed plusplus identity, own startup/settings and `all512`/`swap01`; [launcher](#plusplus-launch-settings-and-help), [native persistence](#native-profile-preference-lifecycle). The approved two-theme scope replaces the original Windows Rose Gold default. |
| `Main_Window` menus/control activation | Same eight menu groups; 512/bank-order options replace master tile modes; priority display is independent. [Options and asset-switching evidence](#plusplus-asset-switching-and-tile-options). |
| `Main_Window` choosers/lifecycle and `option-dialogs` | `.ablk`/`.blk` filtering, `.ablk` New/Save defaults, branch discovery, shared measured forms and dirty guards; [GUI workflows](#plusplus-shared-gui-integration), [New/save/close/reopen and discard paths](#plusplus-launch-settings-and-help). |
| `block-window`, `map-buttons` | Occurrence palette/bank/flip/priority controls, shared collision fields, mirrored brushes, pick/reveal and attribute-preserving transfers; [block evidence](#plusplus-block-editor-and-transfers). |
| `tileset-window`, `roof-window` | Independent raw pixels, 512 selection, bank labels, no master palette-map/monochrome controls, isolated asset drafts and saves; [tile/roof evidence](#plusplus-tile-and-roof-editors). |
| `palette-window`, `colors` | Eight game palettes, six sets, shared RGB editing/import/export, no editable monochrome ninth palette; [palette integration](#plusplus-shared-gui-integration). |
| `map-events`, `Main_Window` event/Print paths | Explicit 26-macro catalog, shared Event Options/history/persistence and occurrence-aware canvas/Print rendering; [event backend](#plusplus-events-and-script-persistence), [GUI input/render checks](#plusplus-shared-gui-integration). |
| `widgets`, `ruler` | Shared main geometry/ruler behavior; expanded list frame terms and middle panning handled by [sheet panning](#plusplus-sheet-panning). The ruler source change only swaps a tile header. |
| Help/About | Complete original branch Help text, platform expansion and attribution through shared viewers; [Help evidence](#plusplus-launch-settings-and-help). |

PP3 still needs complete native workflows with real windows, clipboard services,
file choosers, capture outside child windows, placement and DPI transitions.
The [Windows map/tile check](#plusplus-windows-native-workflow-checks) supplies
initial integrated evidence and exposed a shared menu-input timing bug, now fixed.
Automated native process probes cover only the specifically recorded lifecycle
cases. PP4 requires saved outputs reopened in the original application, alongside
both pinned project suites and retained master regressions. Deferred fonts and
FLTK painting are approved exceptions, not missing functional work.

## Plusplus Windows native workflow checks

On 2026-11-06 the separate `polished-map-plusplus` debug executable was exercised
on Windows 11 25H2, build 26220.9757, using actual window input, native file
choosers and the system clipboard. The starting revision was `a7690eb`; the
menu-timing correction described above was rebuilt and checked in a second
process. All project files were generated under ignored scratch storage;
no pinned project or game graphics were edited or redistributed.

- Open's native filter showed `.ablk`/`.blk`. Map Options inferred an 8×6 map
  and `demo` tileset from its filename, listed the synthetic roof, and disabled
  the parent editor until accepted. The loaded project exposed 16 blocks and
  its `pokemon_event` script.
- Selecting a block and painting one cell enabled Undo; Ctrl+Z and Ctrl+Y
  reversed/reapplied it. Ctrl+S displayed the native success dialog. Independent
  byte comparison found exactly map byte 0 changed from 0 to 1; all other
  project files retained their original hashes.
- The native tile editor accepted a color/pixel edit, copied it through the
  Windows clipboard, and pasted into the next tile. Accept and Save created
  the source-compatible `gfx/tilesets/demo.png` destination. An independent
  decoder compared every pixel of the 128×256 PNG against all 512 source tiles:
  only the intended first-pixel edit and complete destination-tile copy differed.
  Original 2bpp bytes and unrelated assets/scripts remained unchanged.
- Close and process exit completed without a dirty prompt after saving. Restart
  retained the test map in Open Recent; reopening loaded the saved map and PNG,
  and the tile editor displayed the edited pixels. Enter dismissed native success
  dialogs and Escape closed the inspection draft. A second clean exit completed.
- Quick Alt+F failed before the shared modifier-timing fix and opened File after
  rebuilding. The Recent submenu then opened the saved map through the normal
  options/load path. This verifies this native key route, not every shortcut.

This initial pass covered a synthetic Windows map/tile workflow and native
clipboard roundtrip within NG. The [expanded native pass](#plusplus-expanded-native-acceptance)
now supplies block/roof/palette/event/conflict and original-application evidence.
At the end of the initial pass, block attributes/collisions, roof/palette/event
editing, Print/export, dirty/conflict cancellation, inter-application clipboard
exchange, external capture/drop, placement and mixed-DPI checks were still open.
Linux/macOS integrated workflows and original-application save/reopen were also
unverified. The expanded pass below records the current evidence and limits. Existing
headless/source/real-project suites retain their separate evidence; this native
check does not close PP3 or PP4 or resume deferred aesthetics.

The shared modifier-timing change passes Windows format, workspace Clippy with
warnings denied, workspace tests, rustdoc and the production native-adapter
graph check. A workspace test build initially encountered the running Windows
executable's file lock; it passed after the native session exited cleanly.

## Plusplus expanded native acceptance

PP3/PP4 resumed on 2026-11-06 after the M8 backport at `9c4c05a`.
Windows checks used Windows 11 25H2 (26220.9757), the rebuilt separate launcher,
and a disposable copy of the earlier synthetic 8×6 project. Reference checks
used the pinned plusplus `27c1f84` application built with g++ 11.4.0, `-O2`,
and pinned FLTK 1.4.5 on Ubuntu 22.04.5 / WSL2 6.18.58.1, through WSLg X11.
Original source files were unmodified; objects/binaries and fixture outputs are
ignored scratch data. FLTK used Xft, with Cairo/Pango and Wayland disabled.
This is an actual original-application check, not only extracted functions.

| Workflow | Observed evidence |
|---|---|
| Block attributes and paired persistence | Native right-click editor accepted RED, X flip and priority on occurrence 0, plus collision quadrant 0 = 1. Save changed exactly attribute byte 0 to `0xA1` and collision byte 0 to `1`; unrelated files retained their bytes. Original plusplus reopened and picked the same flags and collision. |
| Original edits and conflict protection | Original plusplus added Y flip to occurrence 1 and saved `0xE1`. NG's existing session attempted a different collision edit: Save rejected the externally changed attributes and left **all** paired files intact, including collision = 1. Cancel Close retained the session; explicit discard and Recent reopened saved files. |
| Roof editor/save/reopen | Native pixel edit on initial tile `$0:10` wrote a 24×24 PNG with only pixel `(0,16)` black, preserving 2bpp input. This follows the original reversed display rows. Original plusplus reopened the PNG and displayed the same pixel in `$0:10`. |
| Palette session and export | Hex `123456` remained exact in the editor with RGB5 channels 2/6/10. Enter applied without closing. Export produced six eight-palette sets with Custom gray starting `RGB 02,06,10`; palette export is separate from map Save. |
| Event movement/options/save | Drag moved `pokemon_event` from `(0,0)` to `(2,2)`; native options accepted suffix `, EEVEE`. Independent bytes are `\tpokemon_event  2,  2, EEVEE\r\n`. Original plusplus reopened the moved marker and fields. Its Linux text reader also exposes the existing CR as `^M`; NG preserves the fixture's CRLF rather than rewriting unrelated line endings. |
| Print export | All five options were toggled, the native save chooser used the map-derived PNG name, and export produced an RGB 256×192 image with success feedback. Full overlay pixel comparison and Print clipboard exchange remain acceptance work. |
| Image clipboard | NG Windows roof image copy and Ctrl+Shift+V worked on a distinct tile; cancel discarded that draft. Original Linux copy/paste also worked locally. Original → NG on the same Linux X11 desktop exposed the BMP transport gap described below. Cross Windows/WSLg image transfer was unsuccessful in either direction and is not counted as native OS interoperability acceptance. |

The original's WSLg modal surfaces occasionally captured black until moved;
redrawing exposed their controls. That observation is environment/reference
behavior, not a reason to change NG styling. Exact original fonts remain deferred.

### Linux clipboard BMP interoperability

FLTK `src/Fl_x.cxx` at the pinned toolkit commit advertises only `image/bmp`
for image selections. Locked arboard 3.7.0 requested only `image/png`, causing
native Paste to do nothing even with both apps on the same X11 desktop.
The [audited patch](../vendor/patches/arboard-3.7.0.patch) adds BMP fallback to
X11 and Wayland readers and enables the existing BMP decoder. PNG remains first;
decoding failure is an error rather than permission to select another payload.
Output formats and Windows/macOS transport behavior are unchanged.

The [independent X11 owner test](../crates/gui/tests/clipboard_images.rs) passes
under isolated Xvfb: exact 24-bit BMP red/green pixels, PNG precedence, malformed
BMP/PNG, excessive dimensions and unavailable content. The production feature
graph requires the patched reader and PNG/BMP support without dev dependencies.
CI runs the probe on its own server. Post-fix native X11 verification passed in
both directions: original BMP → NG Ctrl+Shift+V into tile `$0:11`, and NG PNG →
original Paste into `$0:12`. Saving from NG produced exactly two black pixels
at `(0,16)` and `(8,16)` in the 24×24 roof PNG; all other 574 pixels stayed white.
The reverse-reference draft was canceled without overwriting NG's saved file.
Windows format/Clippy/workspace tests/rustdoc and production graph pass; Linux
Clippy/workspace tests/rustdoc, production graph and the independent X11 probe
pass. All eight [CI jobs at `10b3519`](https://github.com/vulcandth/polished-map-ng/actions/runs/42312990074)
pass, including Windows/Linux/macOS, MSRV, licensing, real-project integration
and visual checks. CI compilation/tests do not establish native desktop acceptance.

### Pinned Polished Crystal application interoperability

The native Windows NG and unmodified Linux X11 original application opened a
disposable complete archive of pinned Polished Crystal `1226ea17bb0bdf5de1bad706a688fe89af573981`.
Both inferred New Bark Town's 10×9 dimensions, 90-byte `.ablk`, compound tileset
`johto_traditional.johto_common` and 248 blocks. Selecting roof `new_bark` and Day
palettes rendered the town in both applications; this observation is not a pixel
comparison. The archive and game graphics remain ignored local scratch data.

| Workflow | Observed evidence |
|---|---|
| Map NG → original | NG painted block 0 at `(0,0)`, undid/redid it and saved. Only map byte 0 changed, from 5 to 0. The original reopened that edit. |
| Map original → NG | Original painted block 2 at `(1,0)` and saved. NG closed and reopened through Recent, displaying both edits. The complete file equals `[0,2]` followed by the pinned file's bytes from offset 2. |
| Event NG → original | NG Event Options moved the first warp X from 6 to 7 and saved. Original Reload Events displayed `(7,3)`. The script differed only by that coordinate. |
| Event original → NG | Original dragged that warp to `(8,3)` and saved. The rebuilt NG opened it at `(8,3)`, dragged to `(9,3)` and saved. The complete script equals the pinned bytes with only the first `warp_event  6,  3, ELMS_LAB, 1` changed to X = 9. |
| Plain Print export | Both full applications exported with roof `new_bark`, Day palettes and all five Print overlays off. Decoded RGB images are 320×288 and match at every one of 92,160 pixels; RGB SHA-256 is `80fce00f1fdb54e61d64f7302dc2280de739df1e6c97477964a5099d7d7c3f0c`. PNG container bytes need not match. Text/overlay and clipboard comparisons remain open. |

A complete SHA-256 comparison of all 7,803 archived project files after the
map/event operations found changes only in `maps/NewBarkTown.ablk` and
`maps/NewBarkTown.asm`, with no files added. Print exports are outside the project.

This pass exposed a shared native event-input defect: Windows delivered a primary
press and movement off the marker in one frame; egui's final-position hit test
missed the starting marker. Event capture now uses the actual press position,
matching upstream `event.cpp`'s `FL_PUSH`/`FL_DRAG` sequence, with clipping, layer,
modal and editing-mode guards. Regression tests inject events directly into
`RawInput` (the normal kittest event queue processes one event per frame), covering
separate frames, combined press/move and whole-gesture batches at both zoom levels.
They also cover overlapping markers, empty starts, Shift, modal and Blocks mode.
Native Windows dragging and exact-byte saving pass after the fix.
Windows format, Clippy, full workspace tests, rustdoc and the production native
feature graph also pass for the drag fix. All eight
[CI jobs at `a77ac98`](https://github.com/vulcandth/polished-map-ng/actions/runs/42315648003)
pass, including Windows/Linux/macOS and the shared master regressions.

PP3 remains open for integrated cross-application clipboard, capture/drop,
placement/DPI and complete Linux/macOS workflows. PP4 remains open for the rest
of the pinned Polished Crystal full-application matrix: block attributes/collisions,
graphics/roofs/palettes and remaining Print overlay/clipboard comparisons,
together with master regressions for subsequent changes.
The map/event evidence above does not establish complete real-project parity.

## November 1 upstream review

M8 follows the November 1 Windows tester review, read through its final message
and attachments. Status below distinguishes diagnosed issues from verified fixes.
The user selected reviewed fixes now and future-feature planning on November 6.

| Finding | Disposition and acceptance |
|---|---|
| Quick Alt mnemonics | Shared plusplus fix carried over; master regression and native quick Alt+F pass. Native testing also exposed the same frame-final modifier bug in About/Print/F10; these now match per-event modifiers exactly and have press/release regressions. Native quick Ctrl+/ opens About after the fix. |
| Ctrl+Shift+digit cannot clear bindings | Normalize shifted physical digits only after editor text/modal guards; all-digit regression and native Ctrl+1 then Ctrl+Shift+1 pass. |
| Ctrl+- shrinks the whole UI without a matching zoom-in | Disabled implicit egui keyboard zoom and restore normal UI zoom at startup; regression and native Ctrl+-/Ctrl+= checks pass. A startup regression preserves 2× native DPI. |
| Selectable placeholder labels | Ordinary labels no longer select; About credits, Help and errors explicitly retain selection. Automated regression suite passes. |
| Project-folder chooser and bottom padding | Native folder chooser and bottom margin implemented; Windows folder selection refreshed the roof inventory. Cancellation retains the draft. |
| Small native dropdown windows | 300-point native minimum for Open/New and asset selectors; Windows Open and Change Roof lists expose eight or more rows, scroll and select later entries. |
| Theme submenu | Dark Theme checkbox implemented; keyboard/pointer, dirty-data preservation and preference restart regressions pass. |
| About URLs | Four links emit their exact browser URLs in headless tests; credits remain selectable. |
| Scrollbar contrast | Widget background fills implemented; reviewed light/dark snapshots preserve scrollbar width and geometry. The suggested custom hover-outline painting remains deferred with M4. |
| Portable assets | An isolated Windows executable launched with icons and no adjacent assets. Binary packaging now retains asset notices only; full artwork remains in matching source. Packaging regression checks notice bytes and font license. |
| Original XPM files | Retained as source/provenance; runtime uses embedded pixels and does not need XPM or PNG neighbors. |
| Executable size | Previous packages already used release builds: Windows GUI 18,750,464 bytes; Linux GUI 26,835,744 bytes. Added symbol stripping while retaining thin LTO and codegen-units=1. New measured sizes and successful package evidence are in [releases](releases.md#development-binary-size). |
| Window-size persistence and recent shortcut | Positive tester observations; retain behavior in regression verification. |
| Future features | I3 owns expanded undo/events/zoom/sidebar, connection navigation and update checks. Excluded from M8 by user direction. |

Project options are session discovery overrides; application preferences use the
NG settings store described in [preferences](preferences.md), not an upstream
`.cfg` compatibility promise. New tester packages remain development builds;
M5–M7 and clean-machine platform acceptance remain open.

Windows review checks ran on Windows 11 25H2 (26220.9757), using an isolated
executable and an ignored synthetic 20×18 master map with generated graphics.
No game fixtures are distributed. The map opened with selected assets, the
reviewed shortcuts behaved correctly and exit completed without a save prompt.
These focused checks do not close native acceptance on other platforms.

The first WSL snapshot attempt crashed in its default graphics backend; rerunning
with Vulkan completed. Inspected differences cover scrollbar fills, About links,
Map Options bottom spacing and the Dark Theme menu row. All inspected snapshots
pass. The remaining menu-pixel assertion needed to distinguish checkbox rows and preserve a half-point anchor at 2× DPI; the targeted
pixel check now passes. Master [CI](https://github.com/vulcandth/polished-map-ng/actions/runs/42297779471)
and [packaging](https://github.com/vulcandth/polished-map-ng/actions/runs/42297818324)
passed for the published candidate. Compilation is not native acceptance.

### Plusplus backport of M8

The shared changes from master candidate `73c0e362ead36be8b8a44c0e7b999ed187b8ac17`
are applied to the separate plusplus branch. Conflict resolution retains the
plusplus 512-tile checkbox, profile-specific About/Help identity, isolated
preferences, and two-profile native-key regression. Windows workspace formatting,
Clippy, tests, rustdoc, native-feature graph and packaging-notice tests pass.
The Linux visual suite passes (299 tests, seven explicitly ignored integration
checks). [Cross-platform CI](https://github.com/vulcandth/polished-map-ng/actions/runs/42298610282)
passed all eight jobs, including Windows/Linux/macOS, source-reference,
real-project, dependency, MSRV and visual checks. PP3/PP4 native and
original-application acceptance remain open; this backport does not advance them.
