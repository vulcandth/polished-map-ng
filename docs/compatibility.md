# Compatibility evidence

Status: initial implementation in progress. Neither upstream parity gate passed.

| Area | Reference | State / required evidence |
|---|---|---|
| Binary maps / dimensions | master map.cpp, map.h | Exact byte I/O and filename/explicit dimensions tested; metadata dimensions and oversized-file consent pending |
| Paint, fill, replace, swap, history | master main-window.cpp, map.cpp | Core tests and headless GUI drag→undo→redo→save pass; full gesture matrix and resize pending |
| 2bpp / LZ / CHR / PNG | master tiled-image.cpp, parse-asm.cpp | Initial codecs; all 65,536 bitplane pairs match compiled upstream function; full-codec differential tests pending |
| Metatiles, palettes, collisions | master metatileset.cpp, palette-map.cpp, colors.cpp | Binary metatiles, tilepal and DAY colors; collisions, other palettes, compound tilesets and roofs pending |
| Project discovery / metadata | master config.cpp, option-dialogs.cpp | Pending full precedence and compatible pokecrystal test |
| Main window geometry | master main-window.cpp | Derived rectangle/hit-test/DPI tests pass; light/dark development snapshots inspected; native fonts/themes/rulers and initial scroll positioning pending |
| Block/tileset/roof/palette editors | master *-window.cpp | Pending |
| Events / warp navigation | master event.cpp, map-events.cpp | Pending |
| Preferences, overlays, print/clipboard | master main-window.cpp | Pending |
| Expanded tile/attribute profile | plusplus tile.cpp, metatileset.cpp | Pending; compatible Polished Crystal test required |
| Unified profile detection | both config.cpp variants | Pending |
| Stable automation / batch recovery | new API | Pending |
| Native packaging and runtime | all three OSes | Pending |

New tests must distinguish synthetic vectors, source-derived expectations, and
actual C++ differential observations. A source-derived vector is not a run of
the original application. Unverified assets never become redistributable fixtures.

## Current evidence (2026-10-27)

- Windows local workspace tests, Clippy and rustdoc passed during foundation work.
- `tools/reference_oracle.py` extracts the original conversion function from Git,
  without modifying it; compiler invocation and generated provenance are retained
  locally. `tests/reference.rs` exhaustively compares its output. CI has a dedicated
  reference job; the default test command explicitly reports this test as ignored.
- Headless GUI tests exercise toolbar state, accessible open options, a complete
  paint/undo/redo/save flow, and source-derived geometry. Optional visual tests:
  `cargo test -p polished-map-gui --features visual-tests`.
- CLI rendered the unmodified upstream master Kanto example at 5760×4320. The PNG
  was visually inspected locally; this is a smoke test, not a pixel differential.
  Neither the game assets nor resulting PNG are included in the repository.
- Dependency license/source/advisory checks pass with the explicit maintenance
  exceptions in `docs/dependencies.md`; duplicate versions remain warnings.
- Linux/macOS CI and native smoke tests are not yet verified. No packaged release.
