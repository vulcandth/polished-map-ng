# Asset provenance

Game graphics and maps are not distributed. Reference checkouts and extracted
oracle inputs stay ignored. Third-party toolbar icons remain excluded pending
their separate review; this record does not authorize importing other assets.

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
