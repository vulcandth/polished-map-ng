# Repository tools

Run Python tools from the repository root with Python 3.12 or newer. They use
the standard library; Cargo, Git and external compilers/SDKs are needed where
specified by each tool's `--help` and [CI](../.github/workflows/ci.yml). Generated
output and fetched fixtures belong in `.scratch/` and `.reference/`.

| Task | Entry points |
|---|---|
| Package corresponding source or a platform binary | [development_package.py](development_package.py); [package_docs.py](package_docs.py) selects offline help |
| Verify dependency license coverage | [license_report.py](license_report.py), [license configuration](../about.toml) |
| Verify production native adapters | [native_features.py](native_features.py) |
| Verify native patches against original crate archives | [vendor_sources.py](vendor_sources.py) |
| Verify embedded artwork, icons, fonts and Help | `artwork_assets.py`, `icon_assets.py`, `font_assets.py`, `help_assets.py` |
| Inspect Fugue-derived icon compositions | [icon_components.py](icon_components.py) |
| Verify Windows PE icon resources | [windows_icon_resources.py](windows_icon_resources.py) |
| Fetch pinned game-project fixtures for local tests | [fetch_project_fixtures.py](fetch_project_fixtures.py) |
| Compare with upstream code independently | `*_oracle.py`; [reference job](../.github/workflows/ci.yml) defines inputs and paired Rust tests |
| Verify Wayland wire protocols without the user's desktop | `wayland_*_probe.py`; [native boundary evidence](../docs/compatibility.md) |

The oracle scripts compile small drivers against immutable upstream sources;
they are verification tools, not runtime dependencies. The Wayland probes use
isolated protocol peers and bounded subprocess lifetimes. Do not substitute
these checks for human acceptance on real desktops.

Run tool regressions with:

```text
python -m unittest discover -s tools -p "test_*.py"
```

Packaging requires a clean, committed checkout, or (for binary packaging) an
extracted source archive. It never packages ignored worktrees or game fixtures.
See [release policy](../docs/releases.md) for the binary content policy and
[rebuild instructions](../docs/rebuild.md) for source/relink delivery.
