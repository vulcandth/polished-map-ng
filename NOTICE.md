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

The compressed-graphics reference harness extracts unchanged master
`tiled-image.cpp` LZ/2bpp loader functions and constants, relevant header
declarations and `utils.cpp::file_size`, under LGPL-3.0-only. Generated source
and per-snippet hashes stay in ignored scratch storage; the checked-in driver
and synthetic corpus document these sources without distributing game graphics.

Oversized map prefix loading and its acknowledgment warning adapt master
`Map::{read_blocks,read_asm_blocks}`, `Main_Window::{open_map,save_map}` and
`Modal_Dialog`, under LGPL-3.0-only. Structured load reports and whole-source
conflict hashes are additions; strict library opening remains available.

Installed GUI font preferences adapt master `themes.cpp::use_native_fonts`;
Consolas-specific text sizes adapt `ruler.cpp` and `event.cpp`, under
LGPL-3.0-only. Backend default names follow pinned FLTK 1.4.5's GDI, Quartz and
Xlib/Xft font tables under the retained FLTK terms below. Fontique discovers local
faces; no native font files are redistributed. Bundled fallback fonts retain
their separate licenses even when used through these font-role aliases.

The complete toolbar order, control extents, grouping and availability rules
adapt master `Main_Window::{initialize,update_active_controls}` and
`widgets.cpp::Toolbar::draw`, under LGPL-3.0-only. Text placeholders for
unreviewed artwork contain no imported image pixels.

The five status fields, fixed measurement samples and hover formatting adapt
master `Main_Window::{initialize,update_status,update_event_cursor}`. Their
Greybird/Dark frames and separators adapt `themes.cpp`; label insets follow
FLTK 1.4.5 `fl_labeltype.cxx` under the retained FLTK terms described below.

Sidebar press/release selection and block-editor entry adapt master
`Metatile_Button::handle` in `map-buttons.cpp` and
`Main_Window::select_metatile_cb` in `main-window.cpp`, under LGPL-3.0-only.
Numeric block assignment, selection reveal and digit badges adapt
`Main_Window::{handle_hotkey,select_metatile,force_add_sub_metatiles,close_cb}`,
`Metatile_Button::draw` and `utils.cpp::draw_outlined_text`. Map picking adapts
`Block::handle` and `Main_Window::change_block_cb`, including press-time selection.

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
The Help window adapts `help-window.cpp` and `widgets.cpp::HTML_View`. Its
bundled original documentation is Copyright © 2021 Rangi under LGPL-3.0-only;
see `crates/gui/assets/help/NOTICE.md` for the immutable source, reproducible
Windows/Unix conversion and preserved upstream attribution.
About adapts `Main_Window::initialize/about_cb`, `Modal_Dialog::refresh` and
the platform sizes in `themes.h`. Its text distinguishes NG's own identity from
the retained Rangi/Fugue notices. Unverified application artwork is not included.
Root-window titles and Print filename suggestions adapt
`Main_Window::{open_map,save_as_cb,close_cb}` and `NEW_MAP_NAME` under LGPL-3.0-only,
retaining a separate NG development identity and successful-save publication.
The initial Theme submenu adapts `Main_Window::initialize`, `greybird_theme_cb`
and `dark_theme_cb`, retaining the source radio behavior under LGPL-3.0-only.
Popup panel borders adapt the Greybird/Dark panel functions in `themes.cpp`;
divider drawing and gray-ramp colors adapt FLTK 1.4.5 `Fl_Menu.cxx`,
`Fl_get_system_colors.cxx` and `Enumerations.H`, under the retained FLTK terms.
Popup sizing adapts the pinned master's menu labels/flags and FLTK 1.4.5
`Fl_Menu.cxx` measurement/drawing calculations. Polished Map NG is based in part
on the work of the FLTK project (<https://www.fltk.org>), Copyright 1998-2024
Bill Spitzak and others. The original FLTK license and exceptions are retained
in `LICENSES/FLTK.txt`; the Rust adaptation follows LGPL-3.0 under its
version-selection provision. Menu indicators also adapt master's Greybird/Dark
theme drawing functions and FLTK's `fl_draw_check`, `fl_draw_radio`, color average
and CIELAB contrast rules. The compiled comparison records unchanged source
functions and hashes; native arc/polygon rasterization is not claimed equivalent.
Whole-window transparency adapts master `Main_Window::apply_transparency`,
its F10/menu callback and startup/exit preference handling under LGPL-3.0-only.
The native Rust boundary preserves the Win32/X11 alpha encodings and uses
NSWindow's equivalent on macOS; checked errors and LP64-safe Xlib storage are
port corrections. Wayland integration remains incomplete.
The unchanged Noto Sans Symbols 2 font is Copyright 2022 The Noto Project
Authors and remains OFL-1.1. Its separate family supplies Mac menu modifiers;
see `crates/gui/assets/fonts/NOTICE.md` and the full copyright/license in
`LICENSES/egui-fonts/NotoSansSymbols2-OFL.txt`.
Event cursor and Game Boy guide placement/borders adapt `map-buttons.cpp`,
`map.cpp`, `main-window.cpp` and `gameboy-screen.cpp`. The guide's uniformly
colored source image is represented by a rectangle and its RGBA constant;
the original image bytes are not distributed.
Map ruler rectangles, tick/label placement, coordinate limits and paired
highlighting adapt master `ruler.cpp`, `main-window.cpp` and `map.h`.
Zoom offset scaling and sidebar resets adapt `Main_Window::update_zoom/update_layout`.
Recent-file ordering, labels and open/save lifecycle adapt
`Main_Window::store_recent_map/update_recent_maps/open_recent`; native path
serialization and validation are Rust implementations.
Block-ID overlays and decimal/hexadecimal hover formats adapt master
`map-buttons.cpp`, `Main_Window::update_status/update_event_cursor` and
`utils.cpp::draw_outlined_text`. Native font selection/weight remains unfinished.
CPU Print composition adapts `Main_Window::print_map`, `Block::print`,
`Event::print` and `draw_outlined_text`, with font coverage supplied separately.
Its differential harness compiles those functions unchanged, plus the original
warp-label placement and event metadata, in ignored scratch storage. The harness
uses extracted event/priority/warp images solely as test inputs; it does not
distribute additional assets or native fonts.
The Print Options layout and preference/cancellation semantics adapt
`Print_Options_Dialog` in `option-dialogs.cpp` and `Main_Window::print_cb`.
Worker dispatch, observed PNG replacement and clipboard handoff are Rust
implementations; native frame/font/clipboard fidelity remains under verification.
Priority zigzag artwork by Rangi originates in Polished Map++ commit
`d090450b5649b5cfcc7645884d44f6414c3f0a96`, ported to master with the toolbar
icon in `0a627967a3ff6e68ca062aef69184065e16a4efe`. The three source PNG patterns
are represented as periodic RGBA masks under LGPL-3.0-only; source PNG payloads
are not distributed. Drawing adapts `Tile::draw_with_priority` and
`Metatileset::draw_metatile`. See `docs/assets.md` for source hashes and pixel checks.
Map/sidebar grids and shared map/event/editor selection rings adapt
`map-buttons.cpp::draw_map_button/draw_selection_border`. The portable grid colors
come from the Greybird and Dark inactive colors in master `themes.cpp`.

## Assets and fixtures

No upstream game graphics, example maps or native font files are redistributed.
Tests create synthetic data. Twenty-seven reviewed toolbar/editor icon images are
included with unchanged XPM sources, decoded RGBA pixels and provenance hashes.
Some adapt **Fugue Icons**, copyright © 2013 Yusuke Kamiyamane, under **CC BY 3.0**:
<https://p.yusukekamiyamane.com/> and <https://creativecommons.org/licenses/by/3.0/>.
Polished Map simplified/recolored that artwork; its adaptations remain attributed
to Rangi/Polished Map under LGPL-3.0-only. NG preserves source pixels, converts
storage and generates disabled appearances. The original asset terms are not
replaced by this code license. See `crates/gui/assets/icons/NOTICE.md`,
`docs/assets.md` and `LICENSES/CC-BY-3.0.txt`, also applicable to icon-containing
GUI snapshots. Unreviewed icons remain excluded. Reference checkouts are not shipped.

Rust dependencies retain their own licenses. Dependency notices must accompany
binary releases. This notice does not relicense third-party assets.
Print font enumeration uses Fontique, and CPU outline processing/rasterization
uses Skrifa and Vello CPU, under their retained MIT/Apache-2.0 terms. Installed
OS fonts are read locally and are not included in project assets or releases.
Explicit caller-supplied font data remains subject to its own license. Existing
epaint_default_fonts faces are reused only as test inputs for this adapter.
Variable-instance tests add minimal synthetic fvar/gvar tables to the already
licensed Hack face in memory, with controlled outline/advance deltas. No modified
font file is bundled or written; Hack's original dependency notices remain.

egui's default fonts include Ubuntu Light, Hack, Noto Emoji, emoji-icon-font,
and egui icons. They remain under their respective Ubuntu Font Licence, SIL OFL,
MIT, and component notices, copied from epaint_default_fonts 0.37.0 into
`LICENSES/egui-fonts/`. The Ubuntu Font Licence permits embedding with its
notice and original license retained. These fonts are not relicensed as LGPL.

Windows image clipboard support includes clipboard-win 5.4.3 and error-code
3.4.0 by Douman, under the Boost Software License 1.0. Its complete text is
retained in `LICENSES/BSL-1.0.txt`. These dependencies retain their own license;
see `docs/dependencies.md` for the scoped review and release requirements.
