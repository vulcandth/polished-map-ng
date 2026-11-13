"""Select offline user documentation and rebase source-only links in binaries.

Source archives retain the original documents. Binary copies keep local links
when the destination ships, and link to the exact source revision otherwise.
This handles the inline/reference Markdown links used by this repository, not
arbitrary HTML or a general Markdown parser.
"""
import os
from pathlib import Path
import re
import shutil
from urllib.parse import quote, unquote, urlsplit


USER_DOCS = (
    "usage.md", "menus.md", "preferences.md", "cli.md",
    "testing.md", "testing-plusplus.md", "testing-unified.md",
)
LINK = re.compile(
    r"(?P<prefix>!?\[[^\]\n]*\]\()(?P<url><[^>\n]+>|[^\s)]+)"
    r"|(?P<reference>^\s{0,3}\[[^\]\n]+\]:\s*)(?P<ref_url><[^>\n]+>|\S+)",
    re.MULTILINE,
)


def rewrite_links(text, source, destination, copies, commit, root=Path(".")):
    """Rebase links without changing headings, prose or licensing statements."""
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ValueError("Documentation needs a full source commit")
    root = root.resolve()

    def replace(match):
        prefix = match["prefix"] or match["reference"]
        original = match["url"] or match["ref_url"]
        target = original.removeprefix("<").removesuffix(">")
        url = urlsplit(target)
        if url.scheme or url.netloc or not url.path:
            return match[0]
        resolved = (root / source.parent / unquote(url.path)).resolve()
        relative = resolved.relative_to(root)
        if not resolved.exists():
            raise ValueError(f"Broken documentation link in {source}: {target}")
        if relative in copies:
            link = Path(os.path.relpath(copies[relative], destination.parent)).as_posix()
            link = quote(link, safe="/.-_")
        else:
            kind = "tree" if resolved.is_dir() else "blob"
            link = f"https://github.com/vulcandth/polished-map-ng/{kind}/{commit}/"
            link += quote(relative.as_posix(), safe="/.-_")
        if url.query:
            link += "?" + url.query
        if url.fragment:
            link += "#" + url.fragment
        return prefix + (f"<{link}>" if original.startswith("<") else link)

    return LINK.sub(replace, text)


def copy_documentation(destination, commit):
    """Copy the explicit binary help/attribution set, never internal evidence."""
    copies = {Path(name): Path(name) for name in ("INSTALL.md", "NOTICE.md", "vendor/NOTICE.md")}
    copies.update({Path("docs") / name: Path("docs") / name for name in USER_DOCS})
    copies[Path("docs/rebuild.md")] = Path("REBUILD.md")
    copies.update({p: p for p in Path("crates/gui/assets").rglob("NOTICE.md")})
    # Licenses keep their exact bytes; supply their locations to the link mapper.
    exact = [Path("LICENSE.md"), *Path("vendor/licenses").rglob("*")]
    exact += [p for p in Path("LICENSES").rglob("*")
              if "dependency-supplements" not in p.parts]
    for source in exact:
        if source.is_file():
            copies[source] = source
            target = destination / source
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
    for source, relative in copies.items():
        if source in exact:
            continue
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        text = source.read_bytes().decode("utf-8")
        target.write_bytes(rewrite_links(text, source, relative, copies, commit).encode("utf-8"))
