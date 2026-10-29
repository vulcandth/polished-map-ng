# Asset provenance

Game graphics and maps are not distributed. Reference checkouts and extracted
oracle inputs stay ignored. Only the 30 reviewed icon images marked with a
`distribution` entry in the inventory are imported. Other icons remain excluded;
this record does not authorize importing other assets.

## Menu and toolbar icon audit (in progress)

The master-parity work following zoom/scroll compatibility is the main menu and
toolbar port. `src/icons.h` at pinned master includes **42 XPM images, all 16×16**.
[`icon-inventory.json`](icon-inventory.json) records their Git-blob SHA-256,
decoded RGBA8 SHA-256, palette size and last-changing upstream commit. It contains
metadata only; each reviewed import has an explicit `distribution` entry.
Entries without that field are not cleared for import. Transparent pixels use zero
RGB in the decoded hash. Decode exactly the declared width: `tileset.xpm` and
`palettes.xpm` each have a 17-character final row for a 16-pixel image. All 42
decoded hashes were independently checked against Pillow's XPM decoder.

The complete toolbar now reserves all 27 source button positions. Its ten
unreviewed designs use short text placeholders inside 24×24 controls with full
accessible names/tooltips. They import no additional artwork and do not change
the inventory's clearance decisions. Replacing these placeholders remains part
of the visual port; they are not final icon designs.

### Authoritative licensing evidence

- The pinned Polished Map repository is LGPL-3.0-only. Its About dialog separately
  credits Yusuke Kamiyamane; that credit was added in
  [`4cd0350`](https://github.com/Rangi42/polished-map/commit/4cd03509ed8b38bd4295d99dcf487b3c9fde6571).
  Repository licensing alone does not establish rights to third-party imagery.
- [The creator's site](https://p.yusukekamiyamane.com/) and the downloaded Fugue
  3.5.6 archive's `README.txt` specify **CC BY 3.0**, with copyright © 2013 Yusuke
  Kamiyamane. [Attribution examples](https://p.yusukekamiyamane.com/icon/attribution/)
  include an About dialog or README. Retain creator, collection, source link,
  license link/text and adaptation notices when importing confirmed derivatives.
  Keep these asset terms distinct from the Rust code's LGPL terms.
- Reviewed archive:
  [Fugue Icons 3.5.6](https://p.yusukekamiyamane.com/icon/downloads/fugue-icons-3.5.6.zip),
  SHA-256 `59d54804343b9add8c96ea03bfb2fa3ff48f692459cf426c920a5612e771c37a`.
  The archive includes exceptions for some unrelated logo/project icons; do not
  treat its general license as blanket clearance of every file.
- [Diagona 1.0](https://p.yusukekamiyamane.com/icon/downloads/diagona-icons-1.0.zip)
  was also compared, SHA-256
  `bf6936ce62d30d24b545642c3cdd13d9cf3f920f65527a758ecb7a66569d584c`.
  Its README specifies CC BY 3.0 and copyright © 2007 Yusuke Kamiyamane. No exact
  icon matches were found in either archive. Archives and comparison images stay
  in ignored scratch storage, outside product assets.

### Review findings and remaining work

Visual comparisons identify the following Fugue families behind twelve upstream
images. These are **adaptations, not byte-identical originals**: reduced palettes,
hard transparency, removed shadows, changed shapes or composite details. A nearest
image score was used to find candidates, not as proof of provenance.

| Upstream XPM stem | Fugue PNG stem (under `icons/`) |
|---|---|
| new | document |
| copy | document-copy |
| paste | clipboard-paste |
| save | disk-black |
| grid | grid |
| palettes | light-bulb |
| print | image-export |
| reload | arrow-circle |
| resize | arrow-in-out |
| rulers | ruler |
| tileset | puzzle |
| zoom | magnifier-zoom-in |

The dark `grid` variant needs the same attribution chain. The zoom replacement
commit explicitly refers to Tilemap Studio
([`5adf7d2`](https://github.com/Rangi42/polished-map/commit/5adf7d2bf85375c9b6a0207572804455e331427e));
retain that intermediary history in its import record.

The initial import is now covered by the per-file records and
[`icons/NOTICE.md`](../crates/gui/assets/icons/NOTICE.md), the full CC BY 3.0 text,
root NOTICE/README and in-app Help attribution. `tools/icon_assets.py` verifies
every imported XPM and decoded RGBA hash; `--write` regenerates pixels from the
bundled XPMs. The dependency CI job runs verification without network access.

| Group | Import/review status |
|---|---|
| Twelve Fugue-family images above, plus grid-dark | Imported with original/adapted attribution, transformation history, CC BY 3.0 notice/text and LGPL adaptation notices. |
| ids/ids-dark, hex/hex-dark | Imported as fixed Polished Map bitmap lettering under LGPL-3.0-only. History reviewed at `bf08f47`, `3e057b0` and the `530c963` recolor; no external font file or separate asset source is identified there. |
| Eight collision-quadrant variants | Imported under LGPL-3.0-only with introducing commit `2086e18`, dark recolor `530c963`, individual source/pixel hashes and attribution. These geometric drawings now label the block editor's collision fields. |
| blank | Its transparent 16×16 extent is represented by measured menu geometry. No bitmap is imported or required. |
| priority | Imported under LGPL-3.0-only; Rangi's geometric zigzag matches all 252 opaque pixels in the 16×16 priority overlay, with four transparent corners. See the provenance below. |
| add-sub | Imported under LGPL-3.0-only after reviewing its geometric drawing and introducing commit `e894ac8`; see the component audit below. |
| open, load, load-palettes | Imported with Fugue CC BY 3.0 and LGPL adaptation notices after the component review below. |
| undo, redo, show, warp, gameboy, change | Still excluded. Trace composite artwork and other source families; nearest-image candidates remain unverified. |
| blocks/blocks-dark, events, roof, change-roof | Still excluded. These visually resemble game sprites/tiles. Trace the underlying artwork; repository attribution alone is insufficient to clear it. Appearance is not proof of ownership. Resolve rights or use separately licensed/original artwork and record the visual deviation. |

Application logos and warning/error/success imagery are outside the 42-image
`icons.h` inventory and require their own audit before import. The imported icons
and their disabled/dark variants appear in the Rust GUI and rendered snapshots;
those snapshots carry the same asset notices and terms.

### Resize Blockset and remaining component audit

`add-sub.xpm` is the outlined geometric plus-minus symbol introduced by Rangi in
[`e894ac8`](https://github.com/Rangi42/polished-map/commit/e894ac84fa4dd3fe021ea63d0fd9483fbac37c02),
"Start on add/remove block interface". The full introducing change adds this XPM,
its include, toolbar use and the resize dialog. It identifies no external image
or font source. Its drawing is a centered four-pixel-wide vertical bar crossed
by two fourteen-pixel-wide horizontal bars, with a one-pixel outline and lighter
corner pixels. There are three blue colors and transparency, with no lettering,
game tile or sprite component. The pinned Git blob is byte-identical to the
introducing blob (SHA-256
`a4d926150b91f3883ad27fcc020a263535da522e3842212cb8f7a912cc5c3066`).
Retain it as Polished Map's LGPL-3.0-only drawing, with author attribution and
an explicit distribution record. It uses the existing runtime disabled-color
conversion. That import brought the inventory to 27 files and 20 designs;
the later composite import below brings it to 30 files and 23 designs, with
368 atlas size/theme/disabled/DPI combinations.

The initial additional comparisons did **not** clear Open, Undo or Redo. An eight-orientation
(quarter turns and reflection), three-alpha-threshold, ±1-pixel-translation
search against the Fugue archive found no reliable Undo/Redo match. Redo's pixels
are Undo rotated 180 degrees; that relates the two upstream images but does not
establish their underlying artwork source. Their introducing commit `3e057b0`
and later change `e0a2d6c` do not identify another collection.

The initial whole-image comparisons left Open's arrow and Load Palettes' bulb
unresolved. The following component review supersedes that conclusion for those
composites; it does not clear Undo/Redo or any other excluded artwork.

### Open, Load and Load Palettes composite review

Review identifies Open's folder/document as a simplified/recolored adaptation
of Fugue `folder-open-document`, with `arrow-000-small` overlaid at offset
(4, 3). After thresholding the source alpha at >127, **all 28 arrow silhouette
pixels match exactly**. This resolves the previously missing arrow component;
the document-export family considered earlier was not the matching arrow.
The folder/document assessment is visual provenance evidence, not a claim of
pixel-identical source artwork. Polished Map removes shading/shadows, simplifies
the outline and retains the characteristic open-folder diagonal and folded page.

Load retains all **222 base pixels** outside the two arrows' combined footprint,
with exactly three folder-color substitutions. Its blue folder matches the Fugue
`blue-folder-open-document` family. The arrow shaft becomes a second chevron in
the same three-color scheme; it is an upstream adaptation of the composition,
not an unchanged `control-double` image. Open was present in the initial commit
`5b6e80603c3478b794fb6db89ed9967f48422093`; Rangi added Load in
`4fc39b25b79ec755af45f7b9f7a0942b996ebc4c`, alongside other reviewed icons.

Load Palettes retains the same **112-pixel folder strip** (columns 0–6), removes
the page fold and overlays Fugue `light-bulb-small` at (3, 1). Thresholding source
alpha at >74 reproduces **all 32 bulb silhouette pixels**, including its two
white highlights. Colors are simplified/recolored; shadows are removed. It was
introduced as `load-lighting.xpm` in
`04e802b33c149d560fa0dcfa88743dc61e2cb0a6` and renamed in
`5cc35fe734203a25123bdd3c8e55e4cc00807195`.

The review combines those component correspondences, inspection of the complete
compositions and introducing changes, upstream's explicit creator credit, and
the verified creator archive's license. The numerical checks alone do not prove
authorship. All three imports retain CC BY 3.0 creator/collection attribution and
LGPL-3.0-only upstream adaptation notices. No game imagery is involved. The
unchanged XPMs and decoded RGBA hashes remain tied to pinned master in the inventory.

The four original PNG SHA-256 values are recorded in each import's
`fugue_components` entries in `icon-inventory.json`. Reproduce the component
checks with Pillow installed and the previously verified creator archive:

```text
python tools/icon_components.py --archive .scratch/fugue-icons-3.5.6.zip
python tools/icon_assets.py
```

The first command checks the archive and component hashes, the two exact masks,
the three-color base recolor and the shared folder strip. It neither downloads
nor imports assets. The normal offline CI verifier checks all 30 bundled XPMs
and 7,680 decoded pixels. Comparison sheets/archive stay in ignored scratch storage.

### Port and validation sequence

1. Finish the licensing records and import only cleared source assets. Preserve
   the exact 16×16 pixels of those assets, including transparency and dark variants.
2. Port `Main_Window::initialize` and `Toolbar::draw` geometry: 24×24 buttons in a
   26-point row, separators composed of 2+2+2 points, centered 16×16 images, and
   the original order, palette controls and clipping. Derive text widths from
   the chosen font metrics; do not claim native-font equivalence yet.
3. Preserve popup menu structure and check/radio marks. The pinned source uses
   `BLANK_ICON` for non-check/radio menu entries to reserve the image column;
   it does not assign all the toolbar pictures to popup menu commands.
4. Disabled pixels are implemented for the imported assets from
   `Image::make_deimage` and the inspected FLTK 1.4.5 `Fl_Pixmap.cxx`: integer
   grayscale `(31*r + 61*g + 8*b)/100`, then `(gray + background)/2`. The portable
   light/dark backgrounds are Greybird 206 and Dark 83. This bypasses egui's extra
   disabled opacity. Extend the verified transformation to later imports and
   native theme choices. Keep accessible names, keyboard actions and tooltips.
5. Verify source hashes, every enabled/disabled/dark image pixel, 1×/2× placement,
   button bounds and input behavior. Review toolbar/menu snapshots before
   accepting baselines; reserve native tests for platform-specific interactions.

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
GUI tests check texture pixels at 1×/2× DPI over light/dark backgrounds. The
atlas also includes the later Resize Blockset import described above.
Native FLTK/egui compositing equivalence still needs platform verification.

## Application icon audit (unresolved)

The About `APP_ICON` path uses `res/app.xpm`, a 48×48 tree/grass image. It was
introduced with the About dialog in
[`4cd0350`](https://github.com/Rangi42/polished-map/commit/4cd03509ed8b38bd4295d99dcf487b3c9fde6571)
and replaced in
[`5862da0`](https://github.com/Rangi42/polished-map/commit/5862da0bccf0a8421ec25f57cefcbf53e7d898ef).
The 16×16 `res/app-icon.xpm` arrived with Linux support in
[`bc704f9`](https://github.com/Rangi42/polished-map/commit/bc704f9098c07076d840eac798fb7301dce74709).
These history entries establish the repository changes but do not identify the
underlying artwork's source or an independent permission chain. Visual inspection
alone does not establish ownership. Neither file has been cleared or imported.

Pinned-master SHA-256 metadata (original Git blobs, then decoded RGBA8 pixels):

| Image | Source SHA-256 | RGBA SHA-256 |
|---|---|---|
| app.xpm | `c12a076f845397709b2d61c9db4ea26d853baf6a2e1ad71b1f9d6582dcc51375` | `6dc8a03ed5cffd7c148fe0af647793852da71369ca18e79503c81706f8747f50` |
| app-icon.xpm | `072e9cba4bd023562067337ad366b62e2a24d9e0000816cb24711c59908793f8` | `0ce263222ecd94986e5220d34415b1d590dfce83e284b0bccafa0fbef6e3223b` |

About preserves the original 50×50 icon slot without drawing unverified artwork.
The `.ico` bundle and warning/error/success artwork still require separate audit.
The shared one-button message dialogs now also reserve the source's 50×50 shield
slot and 60-point icon-to-body offset without importing these unreviewed images.

## Fonts

Mac menu modifiers use an unchanged Noto Sans Symbols 2 face in a dedicated
font family. The Noto Project Authors' copyright and complete OFL-1.1 license
are preserved in `LICENSES/egui-fonts/NotoSansSymbols2-OFL.txt`. The immutable
Google Fonts source revision, Git blobs, original SHA-256 values and scope are
recorded in `crates/gui/assets/fonts/NOTICE.md`. `tools/font_assets.py` verifies
the font and license offline in CI. Headless tests verify all three required
modifier glyphs; actual macOS font metrics remain a separate parity gate.

Native startup selects installed faces using the reference's platform preference
lists; deterministic tests and missing-role fallbacks use egui's bundled fonts,
whose notices are retained under `LICENSES/egui-fonts/`. Print shares the selected
bold faces, collection indices and variable-font coordinates. Windows currently
uses an explicitly reported Arial Bold substitute when Microsoft Sans Serif Bold
is unavailable. Synthetic bold and native rasterization remain unfinished; see
`compatibility.md`. The digit masks above are independent of font discovery.
System font files have not been copied or redistributed.
