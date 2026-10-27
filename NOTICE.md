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

## Assets and fixtures

No upstream game graphics, example maps, fonts, or toolbar icons are currently
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
