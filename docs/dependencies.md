# Dependency review

The project adapter directly uses the already-locked `png` 0.18 encoder to retain
upstream's two-bit grayscale tileset output. The `image` convenience writer emits
eight-bit channels; `image` remains responsible for bounded import decoding.
This adds no new package to the dependency graph. The encoder is MIT/Apache-2.0.

The GUI directly uses the already-locked MIT/Apache-2.0 `arboard` 3.7 for an
explicit image-paste button/shortcut. egui exposes image-copy output and incoming
paste events but no request to read an image on button click. Image/Wayland data
control features match the existing egui-winit dependency graph. Clipboard reads
occur only on that user action; native transport still needs platform checks.

The locked eframe 0.37 stack includes three Linux D-Bus maintenance advisories:
RUSTSEC-2026-0370 (zvariant), 0371 (zbus_names), and 0372 (zvariant_derive).
The upstream crates merged into zbus 6; accesskit/atspi and file-dialog dependencies
still use zbus 5. Forcing a major update would break those APIs. Targeted ignores
record this constraint, preserve all other advisory checks, and require review
on every eframe update and by 2026-12-01. No vulnerability advisory is suppressed.

Default egui fonts require Ubuntu-font-1.0 in addition to MIT/Apache/OFL terms.
Only epaint_default_fonts receives that exception; bundled license texts and
notices are retained. Native/system fonts are not copied into the repository.

Duplicate dependency versions are reported as warnings. Do not use broad ignore
rules or disable accessibility merely to obtain a green dependency check.
