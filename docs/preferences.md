# Preferences

Settings are saved at normal shutdown; periodic autosave is disabled. Canceling
exit keeps the session running. Each launcher has independent preferences,
recent maps and native window state.

## Stored settings and defaults

| Setting | Default |
|---|---|
| Theme | System light/dark on first launch; light if detection is unavailable |
| Transparent / Full Screen | Off |
| Editing mode | Blocks |
| Grid | On |
| Rulers, zoom, block IDs, hexadecimal, Game Boy screen | Off |
| Show Events, Show Warp IDs | On |
| Selected palette set | Day |
| Roof Palettes | Morn + Day, Night |
| Auto-Load Events, Special Palettes and Roof Colors | On |
| Resize anchor | Center |
| Classic: Tile Priority | Off; reading PRIORITY_* palette colors enables it |
| Classic: Monochrome, 256 Tiles | Off |
| Attributed: 512 Tiles | Off; loaded graphics/attributes may enable it |
| Attributed: $0:80-FF Before $1:80-FF | Off; bank 1 first in the upper half |
| Show Priority | On; classic controls require Tile Priority |
| Recent maps | Empty; at most ten paths |
| Classic 128-block warning acknowledgment | Off |
| Overworld Map Size | 1300 bytes; range 1–4096 |
| Drag and Drop | On; internal block, tile, roof and palette transfers |
| Print Grid, IDs, Priority, Events, Warp IDs | All off; independent of editor views |

Saved settings override system theme detection. Choosing a theme manually cancels
any pending startup lookup; later OS theme changes are not followed automatically.

In unified, a missing palette map makes only the current document monochrome.
An explicit Monochrome choice is saved and affects subsequent opens. Fixed master
retains upstream's persistent missing-palette fallback. The two tile-control
groups in unified retain their values independently of which format is active.

Auto-load toggles affect subsequent opens, not the current map. Special palettes
and roof colors have independent toggles; the default project palette import
still runs. Manual imports remain available. Unified Open/New may infer Roof
Palettes or accept an override; it commits the layout only on a successful open.
The global setting is the fallback when inference is ambiguous.

Overworld Map Size warns after a successful dirty-map write or forced Save As
when `(width + 6) * (height + 6)` exceeds the chosen capacity. Equality is allowed.
It does not resize the map or change game code. The separate warning for a classic
collision blockset larger than 128 blocks is suppressed after acknowledgment.

Recent maps record completed opens and Save As, preserving exact path spelling.
Canceled/failed operations, New and ordinary Save do not add entries. Clear Recent
changes only the list. Startup neither probes the paths nor reopens a map.

Documents, unsaved colors, undo history, clipboard contents, block digit bindings,
selection, scrolling and open dialogs are not saved. The selected palette set is
saved, but its colors start from defaults next time. Print options are retained
even when their dialog or subsequent file chooser is canceled.

Native window geometry, maximized state and fullscreen use eframe's separate
record. Normal geometry is restored before fullscreen/maximization; minimized or
child-window geometry must not replace it. Transparent stores intent separately:
an unsuccessful startup request reports an error without erasing that preference.
Actual placement and opacity depend on the desktop/compositor.

## Format and location

Substitute the launcher's app ID into these paths:

| Launcher | App ID | JSON storage key |
|---|---|---|
| Unified | `polished-map-ng-unified` | `polished-map-ng-unified.preferences` |
| Fixed master | `polished-map-ng` | `polished-map-ng.preferences` |
| Fixed plusplus | `polished-map-ng-plusplus` | `polished-map-ng-plusplus.preferences` |

| Platform | Default settings file |
|---|---|
| Windows | `%APPDATA%\APP-ID\data\app.ron` |
| Linux | `$XDG_DATA_HOME/APP-ID/app.ron`, or `~/.local/share/APP-ID/app.ron` when XDG_DATA_HOME is not absolute |
| macOS | `~/Library/Application Support/APP-ID/app.ron` |

To reset settings, close the application and back up or rename its `app.ron`.
No launcher falls back to another NG launcher's store. **Unified does not import
original Polished Map preferences.** The legacy import below applies only to the
fixed parity launchers.

The application key contains version-1 JSON. Missing fields receive defaults.
Unknown versions/fields, invalid values or records larger than 4 MiB restore
defaults and disable writes to that key for the session, preserving the original
record. Recent paths support native Unix bytes and Windows UTF-16; foreign native
encodings are rejected. This internal schema is not a stable automation API.

The outer RON file is owned by eframe. Its read/write failures use logged,
best-effort behavior; settings storage is not a crash-recovery journal.

## Legacy import

With no existing NG `app.ron`, fixed master reads the first original store found:

1. `polishedmap.prefs` beside the NG executable.
2. `Polished Map.prefs` beside that executable.
3. `Rangi/Polished Map.prefs` under the platform root below.

| Platform | Original user-store root |
|---|---|
| Windows | Redirected roaming AppData from the shell |
| macOS | `~/Library/Preferences` |
| Linux | `$XDG_CONFIG_HOME`, otherwise `~/.config`; `~/.fltk` only when the modern `Rangi` directory is absent and the old one exists |

Legacy files are read-only. Existing NG storage wins even if malformed or newer;
there is no merging or continuous synchronization. To import an executable-local
store, copy it beside NG before first launch. Other installations are not searched.

The reader maps supported settings, recent paths and window state, preserving
FLTK escape and integer-prefix conventions. Unsupported themes fall back to light
with a startup message. Invalid headers, escapes, overflow, oversized files or
out-of-range settings reject the whole import and disable application-key writes
for that session. Eframe may still create its window record: to retry, correct
the source, close NG and back up/rename its `app.ron`.

## Separate plusplus settings

Fixed plusplus uses `polishedmap-plusplus.prefs`, then `Polished Map++.prefs`
beside the executable, then `Rangi/Polished Map++.prefs` under the same platform
root. It imports `all512` and `swap01`; master-only options are ignored. It has
eight game palettes and no editable ninth monochrome palette. The unified
launcher retains both models' settings but does not run either legacy importer.

## Remaining parity and verification

The [native lifecycle probe](compatibility.md#native-profile-preference-lifecycle)
checks real storage, shutdown/restart and normal size restoration in isolated
stores. Fullscreen, maximization, transparency and mixed-monitor placement still
need integrated human acceptance. See the [roadmap](roadmap.md) and
[preference implementation](../crates/gui/src/editor/preferences.rs) for current scope.
