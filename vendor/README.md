# Pinned native window adapters

These are the original Cargo archives for winit 0.30.13 and egui-winit 0.37.0,
with the changes listed in [NOTICE](NOTICE.md). The root Cargo patch table keeps
eframe, its renderer and the editor on the same two adapter instances.

[sources.json](sources.json) records the original archive checksums from the
previous registry lockfile, every pristine/bundled file hash and each complete
patch. [patches](patches) contains the readable differences from those archives.
No source file from either archive has been removed. Cargo's local extraction
marker is not distributed. Original examples/docs are retained as source, not
installed as application assets. The separately licensed SCTK frame adaptation
is tracked by [adaptations.json](adaptations.json), its own readable patch and
the original MIT license; see [NOTICE](NOTICE.md).

Run `python tools/vendor_sources.py` from the repository root. To verify the
pristine inputs too, supply `--archives` pointing to a directory containing the
two original `.crate` files plus `smithay-client-toolkit-0.19.2.crate`.
After deliberately editing a patch, use
`--refresh --archives DIR`, review the regenerated patch and manifest, then run
the ordinary verification and workspace checks. Do not refresh automatically in
CI. Check native adapters on every supported OS; Windows alone cannot type-check
the macOS, X11 and Wayland implementations.

The GUI tests exercise coordinate conversion, viewport isolation, target bounds
and guarded Open. The platform OLE test includes the actual patched callback and
vtable sources, creates an unseen window, supplies real CF_HDROP media, and checks
native coordinate translation, path bytes and release ownership. Unix tests
exercise the exact URI-list parser. These tests do not replace file-manager,
compositor, clipboard and DPI acceptance on real desktops.
On Linux, `python tools/wayland_opacity_probe.py` runs the real native adapter
against a private socket peer, checking six surface multipliers, synchronized
commits, capability rejection/loss and destruction order. It uses protocol XML
from locked registry packages and requires no desktop/GPU. This is protocol
acceptance, not compositor pixel verification.
`python tools/wayland_file_drop_probe.py` uses the same private-peer harness to
exercise real data-device events and transferred pipe descriptors. It checks
ordered paths/scale, overlapping receives, leave, unsupported data/actions,
bounds, timeout, seat/window removal and clipboard non-interference at the
protocol level. Real file-manager and clipboard workflows still need acceptance.

Keep this patch focused on native transfer/opacity integration. Re-evaluate it
when a compatible egui/winit release supplies these capabilities; remove it only
after the same behavioral gates pass. Preserve upstream copyrights and license
texts when updating or distributing the patched sources.
