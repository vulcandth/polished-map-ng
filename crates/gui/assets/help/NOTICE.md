# Bundled Help and upstream references

Original documentation: Copyright © 2021 Rangi and Polished Map contributors.
Master source: https://github.com/Rangi42/polished-map/blob/8b9bdc2b8aac8c865809516c2123af69c498da0b/res/help.html

Plusplus source: https://github.com/Rangi42/polished-map/blob/27c1f8479107c7495ac0c1818900190a137da6e1/res/help.html

Distributed under LGPL-3.0-only, like the upstream repository. Preserve this
notice, the original `master-help.html.inc` and `plusplus-help.html.inc`, their
`provenance.json` and `plusplus-provenance.json` records, and the repository
LGPL/GPL license texts when distributing this documentation or its adaptations.

Both `.inc` files retain the complete pinned C++ literal sources, including
their CRLF bytes protected from Git newline conversion by `.gitattributes`. The four
original-reference JSON files are generated adaptations by Polished Map NG contributors, © 2026:
version/path constants are expanded for Windows/Unix, HTML whitespace/entities
are decoded, and paragraphs, list items, styles and links become display blocks.
No source paragraph or external link is intentionally removed. No images,
external page contents or font files are included in this asset directory.

The displayed guide is `ng-help.html`, adapted for the current Rust application
by vulcandth and Polished Map NG contributors, © 2026, under LGPL-3.0-only.
It retains upstream attribution and shared controls, corrects obsolete commands,
describes both storage models, and replaces legacy installer/product claims.
`ng.json` is its generated display representation. The four original-reference
JSON files remain source evidence and are not embedded in the application.

Run `python3 tools/help_assets.py` to verify the original SHA-256 values and all
generated documents. `--write` rebuilds them without network access.
