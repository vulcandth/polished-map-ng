# Asset provenance

Game graphics and maps are not distributed. Reference checkouts and extracted
oracle inputs stay ignored. Third-party toolbar icons remain excluded pending
their separate review; this record does not authorize importing other assets.

## Menu and toolbar icon audit (in progress)

The master-parity work following zoom/scroll compatibility is the main menu and
toolbar port. `src/icons.h` at pinned master includes **42 XPM images, all 16×16**.
[`icon-inventory.json`](icon-inventory.json) records their Git-blob SHA-256,
decoded RGBA8 SHA-256, palette size and last-changing upstream commit. It contains
metadata only and is not redistribution clearance. Transparent pixels use zero
RGB in the decoded hash. Decode exactly the declared width: `tileset.xpm` and
`palettes.xpm` each have a 17-character final row for a 16-pixel image. All 42
decoded hashes were independently checked against Pillow's XPM decoder.

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

Before asset import, finish the per-file records and notices for these groups:

| Group | Remaining review |
|---|---|
| Twelve Fugue-family images above, plus grid-dark | Record original/adapted attribution and transformation history; bundle CC BY 3.0 notice/text alongside LGPL adaptation notices. |
| blank, ids/ids-dark, hex/hex-dark, eight collision-quadrant variants | Trace repository-authored geometry/text and recolors; do not infer a separate font license from their appearance. |
| open, load, undo, redo, show, warp, priority, gameboy, add-sub, change, load-palettes | Trace composite artwork and other source families; nearest-image candidates remain unverified. |
| blocks/blocks-dark, events, roof, change-roof | These visually resemble game sprites/tiles. Trace the underlying artwork; repository attribution alone is insufficient to clear it. Appearance is not proof of ownership. Keep excluded until resolved or replace with separately licensed/original artwork and record the visual deviation. |

Application logos and warning/error/success imagery are outside the 42-image
`icons.h` inventory and require their own audit before import. No toolbar artwork
has yet been added to the Rust application or its committed snapshots.

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
4. Port disabled pixels from `Image::make_deimage`: desaturate, then average with
   the theme's `FL_GRAY` at 0.5. Inspect FLTK's actual integer arithmetic before
   implementing this; ordinary egui opacity is not assumed equivalent. Keep
   accessible names, keyboard actions and tooltips on icon-only controls.
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
credit is retained upstream, and those toolbar icons remain excluded here.

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

## Fonts

GUI text currently uses egui's bundled fonts, whose notices are retained under
`LICENSES/egui-fonts/`. The digit masks above are independent of native font
discovery. Helvetica Bold/native monospace matching remains unfinished; system
font files have not been copied or redistributed.
