# Asset provenance

Game-project fixtures are excluded from source and binary distributions. The
attributed [README screenshot](#readme-screenshot) is the sole documentation
exception. Original UI artwork remains the user's explicit selection; repository
attribution does not independently establish rights to underlying third-party
imagery. Known licenses and unresolved origins must remain distinguishable.

## Original artwork selection and provenance

All 41 nonblank toolbar/editor XPMs, original message symbols, About image and
application icon are retained from pinned upstream. The transparent menu icon is
represented by geometry. Original XPM/ICO bytes are unchanged; decoded RGBA and
disabled variants are adaptations, not replacement artwork.

| Record | Owns |
|---|---|
| [Icon inventory](icon-inventory.json) | Source Git blobs, source/pixel hashes, per-file origin and component records |
| [Icon notice](../crates/gui/assets/icons/NOTICE.md) | Fugue mappings, copyright/licenses, adaptation history and unresolved artwork |
| [Artwork inventory](artwork-inventory.json) | Message/application XPM and original multi-size ICO hashes |
| [Artwork notice](../crates/gui/assets/artwork/NOTICE.md) | Authors, original source records, transformations and provenance limits |
| [Root notice](../NOTICE.md) | Code adaptations and project-wide attribution |

The original-artwork selection on October 29 includes `undo`, `redo`, `show`,
`warp`, `gameboy`, `change`, `blocks`/`blocks-dark`, `events`, `roof`, `change-roof`
and application imagery with unverified underlying origins. These are not
asserted to be independently cleared Fugue or NG originals. Preserve the
upstream citation and this limitation rather than inferring rights from similarity.

### Authoritative licensing evidence

Pinned Polished Map is LGPL-3.0-only; its About credit identifies Yusuke Kamiyamane
separately. Confirmed Fugue adaptations retain **CC BY 3.0** and copyright
© 2013 Yusuke Kamiyamane, creator/collection links, full terms and change notices.
The creator's [Fugue 3.5.6 archive](https://p.yusukekamiyamane.com/icon/downloads/fugue-icons-3.5.6.zip)
has SHA-256 `59d54804343b9add8c96ea03bfb2fa3ff48f692459cf426c920a5612e771c37a`;
its README states the license and exceptions for unrelated logo/project icons.
Do not treat the general license as blanket clearance. The per-image family table
belongs in the icon notice, not a second inventory here.

### Component review

Open/Load/Load Palettes are simplified/recolored Fugue composites, not identical
source PNGs. Open's arrow matches all 28 silhouette pixels of `arrow-000-small`
at (4,3) after alpha >127. Load retains 222 base pixels outside its modified
arrows with three folder-color substitutions. Load Palettes retains the 112-pixel
folder strip and matches 32 silhouette pixels of `light-bulb-small` at (3,1),
alpha >74. Introducing commits, component hashes and adaptations are retained in
the inventory/notice. Numerical similarity alone is not proof of authorship;
this review also used compositions, history, upstream credit and the creator archive.

`add-sub`, bitmap ID/hex letters, collision quadrants and priority patterns have
reviewed upstream drawing/history records under LGPL-3.0-only, with no identified
separate font or external image source. The icon notice owns their introducing
commits. No standalone font was imported for bitmap lettering.

### Reproduction and display limits

```text
python tools/icon_assets.py
python tools/artwork_assets.py
python tools/icon_components.py --archive .scratch/fugue-icons-3.5.6.zip
```

The first two checks run offline in CI and verify bundled sources and decoded
pixels; `--write` regenerates RGBA. The optional component check requires Pillow
and the verified creator archive; it does not download/import assets. For the two
XPM rows with an extra character, decoding respects the declared 16-pixel width,
as FLTK does. Transparent RGB is zero. Disabled colors use source integer
luminance `(31*r + 61*g + 8*b)/100`, then `(gray + background)/2`.

Windows embeds the original 16/24/32/48/64 ICO frames in all three launchers;
`tools/windows_icon_resources.py` verifies executable payloads. Linux receives
the 16×16 XPM and macOS the 48×48 application picture. Native icon display/DPI
and macOS bundle icon metadata remain separate acceptance work. Pixel/geometry
tests do not establish those native outcomes.

## Warp ID digit patterns

The normal-size event labels adapt the 80×7 `warp_digits_png_buffer` embedded in
[`src/event.cpp` at the pinned master revision](https://github.com/Rangi42/polished-map/blob/8b9bdc2b8aac8c865809516c2123af69c498da0b/src/event.cpp).
Rangi introduced it in
[`8f457b6`, “Show warp IDs on warp events”](https://github.com/Rangi42/polished-map/commit/8f457b682f35388bb143cfd7c36fc6c109b30898),
dated 2022-03-28. Both that commit and the pinned revision contain the repository's
LGPL-3.0 license. The source and introducing commit identify no separate license
or external font/image source for this digit array. Review also checked the
application's About attribution; its separate “Some icons by Yusuke Kamiyamane”
credit is handled by the toolbar audit above, separately from these digit masks.

The adaptation is distributed under LGPL-3.0-only with Rangi/Polished Map
attribution in `NOTICE.md` and `crates/core/src/warp_label.rs`. The image is decoded
to sixteen seven-row bit masks; this storage change does not remove the original
attribution or license. The original PNG payload is not included in the binary.

- Original PNG SHA-256:
  `fc499957baba4177147b982f080dca61d199886b06d85a7fff1748a69678ac6e`.
- Decoded row-major RGB SHA-256:
  `9d7ffc2e0b04273bb9dcaaf74d99b2f19a3d7aa1d554b127f68450a3278b9484`.
- Pixels: 80×7, opaque RGB, 402 white and 158 black.
- Reproduction: `tools/warp_id_oracle.py` extracts the pinned buffer and two
  unchanged label functions. The reference test checks every glyph pixel and
  8,192 placement/text cases. Generated files include source-function hashes.

## Flat event textures and screen guide

The twelve event texture buffers and Game Boy guide buffer are represented by
rectangles and source-derived color/alpha constants. Their pixels were decoded
locally to verify flat interiors and borders. These adaptations remain attributed
to the upstream source in `NOTICE.md`; source PNG payloads are not distributed.

## Priority zigzag overlays and toolbar icon

Master `tile.cpp::draw_with_priority` selects three distinct translucent zigzag
patterns, rather than enlarging the 8×8 image. Introduction on master:
`0a627967a3ff6e68ca062aef69184065e16a4efe` (Rangi, 2019-03-24), explicitly
ported from Polished Map++. The original introduction is Rangi's commit
`d090450b5649b5cfcc7645884d44f6414c3f0a96` (2019-01-15), "Use a translucent
zigzag overlay to show tile priority". These geometric patterns and the derived
toolbar image are retained with the original attribution under LGPL-3.0-only.
The pinned master buffers decode to the same four RGBA colors:
`(32,96,96,96)`, `(50,126,126,96)`, `(86,186,186,96)`, `(104,216,216,96)`.

| Buffer | Size | Original PNG SHA-256 |
|---|---|---|
| small | 8×8 | `19c7a689eed9e04b090fb16ff9cfc00c9c58b8123b1a11c8cd8c7bbb8a3085e6` |
| large | 16×16 | `eaee27e74a3b6c0edb1695a4023ab9171a99043a4b238e892f01539e6cd25f92` |
| chip | 24×24 | `6bf3dfe075b20ed46a9c714a101d4885f4617b0bce1ed1f0c0679e9c4e3534c3` |

The overlay tests the tile attribute and Show Priority value independently of
the Tile Priority editing/output option. Canvas/sidebar, block-editor sheet and
chips, and tileset thumbnails use the appropriate pattern size. Source drawing
order places selection borders above the overlay; clipboard tile images omit it.
The core stores periodic color-index masks instead of PNG payloads. The reference
CI job independently decodes all 896 original pixels and compares every RGBA
channel using `tools/priority_oracle.py` and the `priority_patterns` test.
GUI tests check texture pixels at 1×/2× DPI over light/dark backgrounds. Native FLTK/egui compositing equivalence still needs platform verification.

## Fonts

Mac menu modifiers use an unchanged Noto Sans Symbols 2 face in a dedicated
font family. The Noto Project Authors' copyright and complete OFL-1.1 license
are preserved in `LICENSES/egui-fonts/NotoSansSymbols2-OFL.txt`. The immutable
Google Fonts source revision, Git blobs, original SHA-256 values and scope are
recorded in `crates/gui/assets/fonts/NOTICE.md`. `tools/font_assets.py` verifies
the font and license offline in CI. Headless tests verify all three required
modifier glyphs; actual macOS font metrics remain a separate parity gate.

Native startup selects installed faces using the reference's Windows/Linux
preference lists and the reviewed macOS system-font fallbacks; deterministic tests and missing-role fallbacks use egui's bundled fonts,
whose notices are retained under `LICENSES/egui-fonts/`. Print shares the selected
bold faces, collection indices and variable-font coordinates. Windows currently
uses an explicitly reported Arial Bold substitute when Microsoft Sans Serif Bold
is unavailable. Synthetic bold and native rasterization remain unfinished; see
`compatibility.md`. The digit masks above are independent of font discovery.
System font files have not been copied or redistributed.

## Branch-specific Help documentation

The shared Help viewer bundles the complete pinned master and plusplus
`res/help.html` C++ literals, original byte hashes and generated Windows/Unix
display blocks. These are LGPL-3.0-only documentation adaptations; attribution
and immutable source links are in the [Help notice](../crates/gui/assets/help/NOTICE.md).
`tools/help_assets.py` verifies both sources and all four generated documents;
the reference CI job compares expansions with compiled original C++ for each
branch. No external linked page content or game graphics are bundled here.
See [viewer evidence](compatibility.md#plusplus-backend-and-gui-evidence).

## Polished Crystal fixture policy

The plusplus fixture is Rangi42/polishedcrystal at
[`1226ea17bb0bdf5de1bad706a688fe89af573981`](https://github.com/Rangi42/polishedcrystal/tree/1226ea17bb0bdf5de1bad706a688fe89af573981),
pinned in [upstream.lock.toml](../upstream.lock.toml). It predates the pinned
plusplus application revision and has paired binary offsets/attributes suitable
for its format. The audit found no repository-wide LICENSE/COPYING grant;
`tools/bsp/LICENSE` is tool-specific and is not a game-art license.

Keep the checkout in ignored `.reference/polishedcrystal`. Tests make temporary
copies; do not edit the checkout, commit fixtures, upload game screenshots/pixel
outputs, or include them in releases/source packages, except for the explicitly
authorized README illustration below. CI fetches the immutable
commit separately and retains only test results. Synthetic differential inputs
and original-editor algorithm harnesses are distinct from game assets. The user's
original UI artwork selection does not establish redistribution rights for this
fixture. Structural/roundtrip evidence is in the [plusplus audit](compatibility.md#plusplus-source-audit).


## README screenshot

[The application screenshot](images/editor.jpg) shows Cherrygrove City opened
from the maintainer's `polishedcrystal-up/maps/CherrygroveCity.ablk` checkout in
the Windows unified editor, with Day palettes. The maintainer explicitly requested
this illustration on November 11, 2026. It is a native application capture, not
an upstream screenshot or an independent compatibility test. No map edits were
saved and the underlying project assets are not bundled.

Map and tileset artwork comes from [Polished Crystal](https://github.com/Rangi42/polishedcrystal)
by Rangi42 and contributors, based on Pokémon Crystal. Game artwork is not
relicensed under NG's LGPL code license; this attribution does not establish
independent third-party redistribution rights. This one documentation image is
an explicit exception to the fixture-output policy above. Existing UI icon,
font and artwork notices still apply to the application surrounding the map.
