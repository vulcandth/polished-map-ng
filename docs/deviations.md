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
- Binary map length mismatch is currently rejected rather than allowing an
  oversized file to be truncated on save; a faithful warning/consent flow remains
  pending. ASM `.map` imports cannot yet save in-place: Save As is pending.
- Single-file saves compare content rather than just modification timestamps,
  and stage output before replacement. Concurrent writers still have a narrow
  check/replace race; no filesystem-wide compare-and-swap guarantee is claimed.
