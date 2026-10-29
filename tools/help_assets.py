"""Verify/rebuild the complete pinned master help document, without network access.

The bundled original is a C++ concatenation of literals and version constants.
Expand only that known grammar, then parse its HTML subset into display blocks.
No JavaScript, images, remote resources or runtime HTML parser are involved.
LGPL-3.0-only; original text Copyright 2021 Rangi and Polished Map contributors.
"""
import argparse
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import subprocess
import tempfile
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "crates/gui/assets/help"


def expand(source, windows):
    # C++ translation normalizes physical CRLF source lines before raw literals.
    source = source.replace("\r\n", "\n")
    constants = {
        "PROGRAM_NAME": "Polished Map", "CURRENT_YEAR": "2021",
        "PROGRAM_AUTHOR": "Rangi", "PROGRAM_EXE": "polishedmap.exe" if windows else "polishedmap",
        "DIR_SEP": "\\" if windows else "/",
    }
    output = []
    while source.strip():
        source = source.lstrip()
        if source.startswith('R"('):
            end = source.index(')"', 3)
            output.append(source[3:end])
            source = source[end + 2:]
        elif source.startswith('"'):
            match = re.match(r'"(?:[^"\\]|\\.)*"', source)
            if not match:
                raise ValueError("Invalid quoted literal")
            output.append(json.loads(match[0]))
            source = source[match.end():]
        else:
            match = re.match(r"[A-Z_]+", source)
            if not match or match[0] not in constants:
                raise ValueError(f"Unknown C++ token: {source[:40]}")
            output.append(constants[match[0]])
            source = source[match.end():]
    return "".join(output)


class Document(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.blocks = []
        self.block = None
        self.tags = []
        self.in_body = False

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "body":
            self.in_body = True
            return
        if not self.in_body:
            if tag not in {"html", "head", "title"}:
                raise ValueError(f"Unexpected metadata: {tag}")
            return
        if tag in {"h1", "p", "li"}:
            if self.block is not None:
                raise ValueError("Nested display block")
            self.block = {"kind": {"h1": "heading", "p": "paragraph", "li": "bullet"}[tag], "spans": []}
        elif tag == "hr":
            self.blocks.append({"kind": "rule", "spans": []})
        elif tag == "br":
            self.add_text("\n")
        elif tag == "ul":
            pass
        elif tag in {"a", "b", "i", "var", "kbd", "font", "span"}:
            if tag == "a" and urlparse(attrs.get("href", "")).scheme not in {"http", "https"}:
                raise ValueError("Only explicit external HTTP(S) links are supported")
            if tag == "font" and attrs != {"size": "2"}:
                raise ValueError("Unknown font directive")
            self.tags.append((tag, attrs))
        else:
            raise ValueError(f"Unsupported help HTML: {tag}")

    def handle_endtag(self, tag):
        if tag == "body":
            self.in_body = False
        elif not self.in_body:
            return
        elif tag in {"p", "h1", "li"}:
            if not self.block or self.tags:
                raise ValueError("Unbalanced display block")
            spans = self.block["spans"]
            if spans:
                spans[0]["text"] = spans[0]["text"].lstrip(" \t\r\n")
                spans[-1]["text"] = spans[-1]["text"].rstrip(" \t\r\n")
            self.blocks.append(self.block)
            self.block = None
        elif tag == "ul":
            pass
        elif self.tags and self.tags[-1][0] == tag:
            self.tags.pop()
        else:
            raise ValueError(f"Unbalanced tag: {tag}")

    def add_text(self, text):
        if not self.block:
            if text.strip():
                raise ValueError("Text outside display block")
            return
        tags = {tag for tag, _ in self.tags}
        span = {"text": text, "bold": "b" in tags, "italic": bool(tags & {"i", "var"}),
                "mono": "kbd" in tags, "small": "font" in tags,
                "link": next((attrs["href"] for tag, attrs in self.tags if tag == "a"), None)}
        previous = self.block["spans"][-1:]
        if previous and all(previous[0][key] == value for key, value in span.items() if key != "text"):
            previous[0]["text"] += text
        else:
            self.block["spans"].append(span)

    def handle_data(self, data):
        if self.in_body:
            self.add_text(re.sub(r"[ \t\r\n]+", " ", data))


def compare_cpp(checkout, cxx, git, provenance, source):
    def blob(path):
        return subprocess.check_output([git, "-C", str(checkout), "show", f"{provenance['commit']}:{path}"])

    original = blob("res/help.html")
    if original != source:
        raise ValueError("Bundled help differs from the immutable Git blob")
    scratch = ROOT / ".scratch"
    scratch.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="help-reference-", dir=scratch) as directory:
        folder = Path(directory)
        (folder / "version.h").write_bytes(blob("src/version.h"))
        (folder / "help.html").write_bytes(original)
        driver = folder / "main.cpp"
        driver.write_text('''#include <iostream>
#undef _WIN32
#ifdef HELP_WINDOWS
#define _WIN32 1
#define DIR_SEP "\\\\"
#else
#define DIR_SEP "/"
#endif
#include "version.h"
int main() { const char* body =
#include "help.html"
; std::cout << body; }
''', encoding="utf8")
        for windows in [False, True]:
            output = folder / ("windows.exe" if windows else "unix.exe")
            command = [cxx, "-std=c++17", "-finput-charset=UTF-8", "-fexec-charset=UTF-8",
                       driver.as_posix(), "-o", output.as_posix()]
            if windows:
                command.append("-DHELP_WINDOWS")
            subprocess.run(command, check=True)
            actual = subprocess.check_output([str(output)]).replace(b"\r\n", b"\n")
            expected = expand(source.decode("utf8"), windows).encode("utf8")
            if actual != expected:
                raise ValueError("C++ and Python help expansions differ")
            print(f"Original C++ {'Windows' if windows else 'Unix'} expansion: {len(actual)} matching bytes")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--reference-checkout", type=Path)
    parser.add_argument("--cxx", default="c++")
    parser.add_argument("--git", default="git")
    args = parser.parse_args()
    provenance = json.loads((ASSETS / "provenance.json").read_text(encoding="utf8"))
    source = (ASSETS / "master-help.html.inc").read_bytes()
    if hashlib.sha256(source).hexdigest() != provenance["sha256"]:
        raise ValueError("Pinned help source hash mismatch")
    if args.reference_checkout:
        compare_cpp(args.reference_checkout, args.cxx, args.git, provenance, source)
    for platform in ["windows", "unix"]:
        document = Document()
        document.feed(expand(source.decode("utf8"), platform == "windows"))
        document.close()
        if document.block or document.tags or document.in_body:
            raise ValueError("Unclosed document")
        data = json.dumps(document.blocks, ensure_ascii=False, indent=2) + "\n"
        path = ASSETS / f"{platform}.json"
        if args.write:
            path.write_text(data, encoding="utf8", newline="\n")
        elif path.read_text(encoding="utf8") != data:
            raise ValueError(f"Rebuild {path.name} using --write")
        links = sum(span["link"] is not None for block in document.blocks for span in block["spans"])
        print(f"Verified {platform}: {len(document.blocks)} blocks, {links} linked spans")


if __name__ == "__main__":
    main()
