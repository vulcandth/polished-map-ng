# Dependency notice supplements

Some published Cargo packages omit their workspace-level license files.
These unmodified upstream notices supplement the files present in the locked
registry archives; they do not change any dependency's license.

`docs/dependency-notices.json` maps exact package versions to repository revisions,
original paths/URLs and SHA-256 hashes. Most revisions come from the published
crate's `.cargo_vcs_info.json`. egui's recorded crate commits were unavailable
through the upstream API, so its license texts are pinned to the immutable
upstream **0.37.0 release tag** revision instead; both revisions are recorded.

Recent objc2 workspace LICENSE.md files link to license terms rather than
reproducing them. We retain those documents and select their MIT option, including
the standard MIT text retained by AccessKit. Older objc2 notices retain Steven
Sheldon's copyright statement unchanged. The dispatch 0.2.0 crate/upstream omit
a license file entirely: its published Cargo.toml identifies Steven Sheldon as
author and declares MIT. We retain that exact manifest together with the standard
MIT terms. These source distinctions are explicit in the provenance manifest.

Binary packages copy these notices into the relevant per-package license folder,
retain original authors/repository/license metadata in the dependency inventory,
and include this complete supplement directory. Font, icon and original
application-artwork notices remain separate in the other LICENSES directories
and the corresponding asset directories. Packages fail if a supplement's bytes
no longer match its recorded hash or a resolved registry package has no notices.
