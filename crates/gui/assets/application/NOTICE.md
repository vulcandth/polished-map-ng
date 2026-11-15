# Application cherry-tree artwork

The maintainer supplied `cherry-tree-32.png` and `cherry-tree-48.png` on
November 15, 2026 and explicitly selected them to replace the application icon.
They are preserved byte-for-byte from the two attached PNGs. Their SHA-256
values are pinned in `tools/application_assets.py`.

These are supplied artwork, not newly drawn NG code. This selection does not
establish an independent copyright license or change underlying artwork rights.
Retain the upstream attribution and origin limitations in
[the artwork notice](../artwork/NOTICE.md) and `docs/assets.md` in the matching source archive.
No other project/game graphics are included by this selection.

`tools/application_assets.py --write` (Pillow 11.2.1) generates a Windows ICO
with 16/24/32/48/64/128/256-pixel frames, macOS ICNS, Linux PNG, a 64-pixel
window fallback and the 48-pixel About image. The supplied 32/48-pixel frames
are exact; other sizes resample a single complete image with nearest-neighbor
scaling. The 64-pixel image is a 2× enlargement of the 32-pixel tree, not the
old upstream frame containing four trees. The original upstream artwork stays
unchanged in `../artwork` as provenance evidence and for other dialog images.

`hashes.json` records generated files. Verification needs only Python's standard
library; regeneration also decodes and checks every generated ICO frame.
