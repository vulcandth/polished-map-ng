# Intentional differences

Approved: fail safely for malformed input rather than reproduce upstream crashes
or out-of-bounds access; report corruption/conflicts rather than silently discard
data. Initial theme coverage is default light and dark. OS chrome/font rasterization
may differ. All other safe observable behavior remains a parity requirement.

Implementation-specific exceptions will be recorded here with tests and rationale.

## Safety corrections already implemented

- Undo after saving marks changed data dirty. Upstream's undo/redo callbacks do
  not update the modified flag, potentially allowing edits to be lost on close.
  The normal sticky dirty flag and no-op-click history entries are retained.
  Evidence: core `undo_after_saving_must_not_lose_unsaved_changes`.
- Malformed assembly numbers fail explicitly instead of strtol silently turning
  unknown text into zero. LZ reads/back-references/output lengths are checked.
  RGB palette channels outside 0..31 and incomplete colors/palettes are rejected.
  Incomplete roof tables return a format error instead of continuing after a GUI
  warning; a recoverable warning flow remains pending.
- Binary map length mismatch is currently rejected rather than allowing an
  oversized file to be truncated on save; a faithful warning/consent flow remains
  pending. ASM `.map` imports use Save As to produce binary `.blk` output;
  the original source is preserved. Save As currently requires a `.blk` extension
  (or adds it when omitted), rather than accepting arbitrary binary-file suffixes.
- Single-file saves compare content rather than just modification timestamps,
  and stage output before replacement. Concurrent writers still have a narrow
  check/replace race; no filesystem-wide compare-and-swap guarantee is claimed.
- ASM `.CEL` blockset imports require binary `.bin`/`.bst` Save As. Upstream
  writes binary bytes over the assembly source; the port preserves that source.
  Existing binary blockset writes preserve row-major tile bytes. Paired Save As
  requires a separate collision destination instead of modifying source collisions.
- Collision writes preserve unrelated ASM, untouched rows, inline comments,
  line endings and trailing binary data. Upstream rewrites the whole collision
  file into canonical rows and drops other content. Changed field lists retain
  their original prefix/suffix; newly added rows use upstream's canonical syntax.
  Partial binary rows, malformed ASM rows, field delimiters and line breaks are
  rejected. Individual fields are limited to 4096 bytes and files to 4 MiB.
- Block/collision saves preflight both files, retain before/after bytes, and roll
  back a completed first write if the second fails. External changes prevent
  rollback rather than being overwritten; recovery data is retained. GUI Save
  still processes the map separately. General journal discovery/replay and
  process-termination tests remain required before the stable automation milestone.
- Headless discovery leaves an unmatched tileset unset instead of accepting the
  GUI's first dropdown item. Callers must supply an explicit choice; this avoids
  silently rendering or editing an unrelated tileset. Ambiguous compound name
  ordering and the original full options dropdown remain parity work.
- Palette banks reset per document. Upstream stores palette edits globally and
  can carry the previous map's custom colors into the next map when no file
  overrides them. Deliberate reuse/import and persistent preferences remain
  pending; accidental carry-over is not reproduced in the headless interface.

## Tileset editing and export safety

The new tileset export routines reject entirely undefined palette maps instead of
reproducing `Palette_Map::write_palette_map`'s backwards array underflow. Grayscale
export checks every destination row. In upstream, `Tileset::print_rgb` subtracts
two tile rows for every slot at or above $80, even when a compound output starts
there; some offsets/counts therefore address memory before or beyond the image.
Those combinations return a layout error. Valid layouts retain upstream pixel
placement, grayscale levels, row padding and palette-map serialization quirks.
Project saves use these checks before writing any compound outputs. A corrected
policy for those unsafe layouts remains separate work. Named missing companions
with an empty explicit region are also rejected rather than allowing upstream's
zero-count fallback to copy unrelated regions into a new companion file.

Palette-map writes preserve unrelated ASM, comments and line endings instead of
replacing the whole file with canonical rows. Resizing a simple table retains its
comments; interspersed labels/macros with ambiguous ownership cause a preflight
error. Tests cover both paths. All compound PNG/palette outputs are staged and
conflict-checked together, using the existing rollback/retained-backup mechanism.
PNG output preserves the original two-bit grayscale format and decoded samples;
compression bytes can differ between the Rust encoder and libpng/zlib.
Project-local names reject drive prefixes and alternate-stream colons in addition
to directory separators, preventing headless writes outside the selected names.

The GUI disables the priority checkbox for an undefined palette. Upstream ORs
the priority bit into UNDEFINED and then indexes beyond its palette-color table
(`Color::color` only recognizes the exact UNDEFINED value). Choosing a defined
palette re-enables the checkbox when the project supports priority. The existing
core validation rejects that invalid combination; a GUI test verifies the guard.

## Incomplete resize integration

The block-level resize matches upstream's nine anchors, zero fill, signed division
and history reset. Event movement is pending with event support. Neither upstream
nor this port rewrites map dimension constants on resize; users must update those
and any filename-encoded dimensions before reopening. The current GUI states this
limitation; the CLI reports `dimensions_changed`.
