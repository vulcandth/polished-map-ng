# Original application and message artwork

These unchanged XPM images and the Windows ICO come from Polished Map by
Rangi42/Rangi and contributors, pinned at
[8b9bdc2](https://github.com/Rangi42/polished-map/tree/8b9bdc2b8aac8c865809516c2123af69c498da0b/res).
The repository distributes them alongside its
[LGPL-3.0 license](https://github.com/Rangi42/polished-map/blob/8b9bdc2b8aac8c865809516c2123af69c498da0b/LICENSE.md).
The original source license and attribution are retained in this project's root
license files and NOTICE.md.

- `error.xpm`, `warning.xpm`, `success.xpm`: original 48×48 message symbols,
  introduced by Remy Oukaour in `5b6e80603c3478b794fb6db89ed9967f48422093`.
  Their original blobs are unchanged at the pinned revision. The same author
  added the repository LGPL grant in `01a374288d3ba6261c978ac1d71f54eefae00a22`.
- `app.xpm`: original 48×48 About picture, also identical in pixels to the
  Windows ICO's 48×48 frame.
- `app-icon.xpm`: original 16×16 Linux window icon.
- `app.ico`: original Windows bundle with 16/24/32/48/64-pixel frames.

The application artwork's underlying source is not independently established.
It is reproduced with upstream attribution at the user's explicit direction,
not represented as newly created NG art or independently cleared third-party
artwork. Attribution to this repository does not relicense underlying imagery.
No endorsement by upstream or any underlying rights holder is implied.

NG keeps original source bytes, decodes XPM pixels to RGBA8, and paints message/
About images without filtering at their original logical size. Source and decoded
hashes are in `docs/artwork-inventory.json`; `tools/artwork_assets.py` verifies
or regenerates the pixels. Retain this notice, the inventory and upstream license
with source, binaries and snapshots that contain these images.
