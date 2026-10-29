# Icon copyright and adaptation notices

These assets reproduce selected icons from **Polished Map**, by Rangi42/Rangi
and contributors, at commit `8b9bdc2b8aac8c865809516c2123af69c498da0b`:
<https://github.com/Rangi42/polished-map/tree/8b9bdc2b8aac8c865809516c2123af69c498da0b/res>.
Polished Map's adaptations are covered by LGPL-3.0-only. Original notices and
the license texts are retained at the repository root and under `LICENSES/`.

The following icons adapt **Fugue Icons**, copyright © 2013 Yusuke Kamiyamane,
licensed under **Creative Commons Attribution 3.0 Unported**:

| Included stems | Original Fugue PNG stem |
|---|---|
| new | document |
| copy | document-copy |
| paste | clipboard-paste |
| save | disk-black |
| grid, grid-dark | grid |
| palettes | light-bulb |
| print | image-export |
| reload | arrow-circle |
| resize | arrow-in-out |
| rulers | ruler |
| tileset | puzzle |
| zoom | magnifier-zoom-in |

Creator and collection: <https://p.yusukekamiyamane.com/>.
License: <https://creativecommons.org/licenses/by/3.0/>; the full text is included
in `LICENSES/CC-BY-3.0.txt`. No endorsement by the original author is implied.
The original Fugue artwork remains under CC BY 3.0; this project's LGPL license
does not replace those terms. These assets use none of Fugue's separately
credited logo/project icons.

The original Fugue artwork was modified by Polished Map: simplified/recolored
palettes, changed outlines/details, removed shadows and hard transparency.
The dark grid recolor was added in upstream commit `530c963158fceb9d4836363d6d5e39d51019b616`.
The zoom icon passed through Rangi's Tilemap Studio, as recorded by upstream
commit `5adf7d2bf85375c9b6a0207572804455e331427e`.

The `ids`, `ids-dark`, `hex` and `hex-dark` bitmap lettering is retained from
Polished Map under LGPL-3.0-only. Its introducing/recolor history was reviewed;
no external font file or separate third-party asset credit is identified there.
These are fixed bitmap artwork, not a redistributed font.

The eight `coll-top-left`, `coll-top-right`, `coll-bottom-left` and
`coll-bottom-right` images (including their `-dark` variants) are Polished Map's
geometric quadrant drawings, retained under LGPL-3.0-only. Rangi introduced the
four drawings in `2086e18550ecfe88c764591eab2b56a015e7e87c`, "Use grid icons
instead of arrows for collision quadrants"; the dark recolors appeared in
`530c963158fceb9d4836363d6d5e39d51019b616`. Review of that history identifies no
separate external asset or font source for these drawings.

The `priority` icon is Rangi's geometric zigzag, introduced on master in
`0a627967a3ff6e68ca062aef69184065e16a4efe`. Its 252 opaque RGB pixels exactly
match the 16×16 overlay originating in Polished Map++ commit
`d090450b5649b5cfcc7645884d44f6414c3f0a96`; only the four icon corners are
transparent. It is retained under LGPL-3.0-only with the original attribution.
It does not use Fugue artwork.

The `add-sub` image is Polished Map's geometric plus-minus drawing, introduced
with the blockset resize interface in
`e894ac84fa4dd3fe021ea63d0fd9483fbac37c02`. The pinned source is byte-identical to
that introduction. Review of the introducing change identifies no external
asset or font source for this drawing. It is retained under LGPL-3.0-only with
the original attribution; the component review is recorded in `docs/assets.md`.

Polished Map NG preserves the original XPM sources unchanged, converts their
declared 16×16 pixels to row-major RGBA8, and generates disabled grayscale/color
blends at runtime. Conversion does not remove the above attribution or terms.
`docs/icon-inventory.json` records source and pixel hashes and the per-file
license/provenance entries. `tools/icon_assets.py` verifies or regenerates the
RGBA files. Copies of these icons in rendered GUI snapshots carry the same
attribution and asset terms. Include this notice and the license texts with
binary and source distributions.
