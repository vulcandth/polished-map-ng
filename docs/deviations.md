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
  Existing binary blockset writes preserve row-major tile bytes. Collision files
  remain untouched by the current tile-only editor; collision editing is pending.
  GUI Save processes map and blockset separately; recoverable multi-file commits
  are still required before the stable automation milestone.
- Headless discovery leaves an unmatched tileset unset instead of accepting the
  GUI's first dropdown item. Callers must supply an explicit choice; this avoids
  silently rendering or editing an unrelated tileset. Ambiguous compound name
  ordering and the original full options dropdown remain parity work.
- Palette banks reset per document. Upstream stores palette edits globally and
  can carry the previous map's custom colors into the next map when no file
  overrides them. Deliberate reuse/import and persistent preferences remain
  pending; accidental carry-over is not reproduced in the headless interface.

## Incomplete resize integration

The block-level resize matches upstream's nine anchors, zero fill, signed division
and history reset. Event movement is pending with event support. Neither upstream
nor this port rewrites map dimension constants on resize; users must update those
and any filename-encoded dimensions before reopening. The current GUI states this
limitation; the CLI reports `dimensions_changed`.
