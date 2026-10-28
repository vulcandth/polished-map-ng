# Compatibility evidence

Status: initial implementation in progress. Neither upstream parity gate passed.

| Area | Reference | State / required evidence |
|---|---|---|
| Binary maps / dimensions | master map.cpp, map.h | Exact byte I/O, New Map, Save As/ASM conversion, filename/explicit/metadata dimensions tested; oversized-file consent pending |
| Paint, fill, replace, swap, history | master main-window.cpp, map.cpp | Core and GUI drag→undo→redo→save tests; all nine resize anchors, zero fill, truncation and history clearing tested; resize event shifts and full gesture matrix pending |
| 2bpp / LZ / CHR / PNG | master tiled-image.cpp, parse-asm.cpp | Initial codecs; all 65,536 bitplane pairs match compiled upstream function; full-codec differential tests pending |
| Metatiles, palettes, collisions | master metatileset.cpp, tileset.cpp, palette-map.cpp, colors.cpp | Binary metatiles, tilepal, six sets, project/special palette precedence, metadata selection, roofs, compound composition, palette color commands/export and GUI, ASM/binary collisions and paired saves tested; cross-map palette lifecycle and full differential coverage pending |
| Project discovery / metadata | master config.cpp, option-dialogs.cpp | Makefile root, dimensions/header precedence, legacy macro ordering, numbered aliases and compound companion paths tested; shared-map options, compound selection ordering and complete option selectors need further coverage |
| Main window geometry | master main-window.cpp | Derived rectangle/hit-test/DPI tests pass; light/dark development snapshots inspected; native fonts/themes/rulers and initial scroll positioning pending |
| Block/tileset/roof/palette editors | master *-window.cpp | Block, tileset, roof and palette drafts, pixel gestures, palette/priority attributes, clipboard operations, drag/copy, block count, loaded-tileset/roof switching and persistence implemented; native child windows, native clipboard verification and priority overlays pending |
| Events / warp navigation | master event.cpp, map-events.cpp | Pending |
| Preferences, overlays, print/clipboard | master main-window.cpp | Pending |
| Expanded tile/attribute profile | plusplus tile.cpp, metatileset.cpp | Pending; compatible Polished Crystal test required |
| Unified profile detection | both config.cpp variants | Pending |
| Stable automation / batch recovery | new API | Block/collision and compound tileset preflight, backups and rollback tested; tileset CLI preview and hash-required apply; general batch journal discovery/replay and stable schemas pending |
| Native packaging and runtime | all three OSes | Pending |

New tests must distinguish synthetic vectors, source-derived expectations, and
actual C++ differential observations. A source-derived vector is not a run of
the original application. Unverified assets never become redistributable fixtures.

## Current evidence (2026-10-27)

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
  and comprehensive DPI/theme behavior remain pending. Clipboard icons are drawn
  in code; upstream image assets have not been redistributed.
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
