# Copyright and provenance

Polished Map NG is a derivative Rust port of Polished Map by Rangi42/Rangi
and contributors: <https://github.com/Rangi42/polished-map>.
The original application identifies Copyright © 2021 Rangi. Original notices
are retained; see `upstream.lock.toml` for exact reference revisions.

Rust adaptations and new implementation: Copyright © 2026 Polished Map NG
contributors. Code is distributed under LGPL-3.0-only. See `LICENSE.md` and
`LICENSES/GPL-3.0.txt` for the complete applicable license texts.

This port changes implementation language, ownership, validation, architecture,
and UI toolkit. Individual modules identify upstream algorithm sources.

Event handling adapts master `event.cpp`, `map-events.cpp`, event path discovery
in `config.cpp`, and resize/drag coordinate rules in `main-window.cpp`/`map.h`.
The reference harness extracts original event functions and metadata unchanged,
with generated source/provenance kept outside distributed assets. Assembly byte
preservation and input validation changes are recorded in `docs/deviations.md`.
Event marker backgrounds reconstruct the flat fill and one-pixel border from
the LGPL source's embedded texture data using rectangles and RGB/alpha constants;
no original texture PNG payloads are distributed. Normal-size warp digits adapt
the embedded `warp_digits_png_buffer` added by Rangi in commit
8f457b682f35388bb143cfd7c36fc6c109b30898 (2022-03-28), under the repository's
LGPL-3.0 license. Decoded pixel patterns are stored as row masks; original
attribution and LGPL-3.0-only terms remain. See `docs/assets.md` for hashes,
history, verification and the distinction from unaudited toolbar icons.
External script dispatch and known Windows editor line arguments adapt
`Main_Window::view_event_script` in master `main-window.cpp`; native calls,
worker lifetime, validation and error reporting are Rust implementations.
Event cursor and Game Boy guide placement/borders adapt `map-buttons.cpp`,
`map.cpp`, `main-window.cpp` and `gameboy-screen.cpp`. The guide's uniformly
colored source image is represented by a rectangle and its RGBA constant;
the original image bytes are not distributed.
Map ruler rectangles, tick/label placement, coordinate limits and paired
highlighting adapt master `ruler.cpp`, `main-window.cpp` and `map.h`.

## Assets and fixtures

No upstream game graphics, example maps, native font files, or toolbar icons are currently
redistributed. Tests create synthetic data. The original About dialog credits
some icons to Yusuke Kamiyamane; those assets require a separate provenance
review before inclusion. Reference checkouts are ignored and not shipped.

Rust dependencies retain their own licenses. Dependency notices must accompany
binary releases. This notice does not relicense third-party assets.

egui's default fonts include Ubuntu Light, Hack, Noto Emoji, emoji-icon-font,
and egui icons. They remain under their respective Ubuntu Font Licence, SIL OFL,
MIT, and component notices, copied from epaint_default_fonts 0.37.0 into
`LICENSES/egui-fonts/`. The Ubuntu Font Licence permits embedding with its
notice and original license retained. These fonts are not relicensed as LGPL.

Windows image clipboard support includes clipboard-win 5.4.3 and error-code
3.4.0 by Douman, under the Boost Software License 1.0. Its complete text is
retained in `LICENSES/BSL-1.0.txt`. These dependencies retain their own license;
see `docs/dependencies.md` for the scoped review and release requirements.
