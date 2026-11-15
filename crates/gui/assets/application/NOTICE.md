# Desktop application icon containers

These files repackage the original Polished Map cherry-tree artwork selected by
the maintainer, reaffirmed for review issue #26 on November 15, 2026. No new icon
design or game-project graphics have been introduced.

The source is `../artwork/app.ico`, from Rangi42/Polished Map at pinned commit
8b9bdc2b8aac8c865809516c2123af69c498da0b. Retain the full attribution, source
license and underlying-origin limitations in `../artwork/NOTICE.md`,
`docs/artwork-inventory.json` and `docs/assets.md`. Container conversion does not
establish or change rights to the underlying imagery.

`tools/application_assets.py --write` uses Pillow to copy the original
32/64-pixel frames into a macOS ICNS (including 16/32-point Retina representations),
repeating 64-pixel cells with nearest
neighbor scaling for larger representations. `app.png` is the 256-pixel Linux
launcher image; `window.rgba` is the original 64-pixel frame for window adapters.
Windows continues embedding the unchanged original multi-resolution ICO.
`hashes.json` records both the source and generated files. Ordinary verification
requires only Python's standard library; these files were generated with
Pillow 11.2.1, which can also be used for regeneration.
