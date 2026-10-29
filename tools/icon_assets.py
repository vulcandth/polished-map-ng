"""Verify imported icon provenance and decode the pinned XPM subset to RGBA8.

No upstream checkout, network or third-party Python packages are needed. The
inventory explicitly lists which artwork has completed review. --write rebuilds
only those RGBA files from their bundled, hash-checked XPM source. Image licenses
and adaptation notices live in crates/gui/assets/icons/NOTICE.md.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "crates/gui/assets/icons"


def decode(data):
    lines = re.findall(r'^"(.*)"[,]?\r?$', data.decode("ascii"), re.M)
    width, height, colors, cpp = map(int, lines[0].split())
    if (width, height, cpp) != (16, 16, 1):
        raise ValueError("Expected the reviewed 16x16, one-character XPM subset")
    palette = {}
    for line in lines[1:colors + 1]:
        if not re.fullmatch(r". c (None|#[0-9a-fA-F]{6})", line):
            raise ValueError("Unexpected XPM color")
        color = line[4:]
        palette[line[0]] = bytes(4) if color == "None" else bytes.fromhex(color[1:]) + b"\xff"
    rows = lines[colors + 1:]
    if len(rows) != height or any(len(row) < width for row in rows):
        raise ValueError("Incorrect XPM row dimensions")
    # Two original XPMs have an extra character in the last row. FLTK reads
    # the declared width; retain the original source and ignore that tail.
    return b"".join(palette[pixel] for row in rows for pixel in row[:width])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    inventory = json.loads((ROOT / "docs/icon-inventory.json").read_text(encoding="utf8"))
    imported = [item for item in inventory["icons"] if item.get("distribution")]
    expected = {Path(item["file"]).name for item in imported}
    if {p.name for p in ASSETS.glob("*.xpm")} != expected:
        raise ValueError("Imported XPM files must exactly match the reviewed inventory")
    for item in imported:
        source = ASSETS / Path(item["file"]).name
        data = source.read_bytes()
        if hashlib.sha256(data).hexdigest() != item["xpm_sha256"]:
            raise ValueError(f"Original XPM hash mismatch: {source.name}")
        pixels = decode(data)
        if hashlib.sha256(pixels).hexdigest() != item["rgba_sha256"]:
            raise ValueError(f"Decoded pixel hash mismatch: {source.name}")
        output = source.with_suffix(".rgba")
        if args.write:
            output.write_bytes(pixels)
        elif output.read_bytes() != pixels:
            raise ValueError(f"Rebuild RGBA pixels with --write: {output.name}")
    if {p.stem for p in ASSETS.glob("*.rgba")} != {Path(n).stem for n in expected}:
        raise ValueError("Unexpected RGBA assets")
    print(f"Verified {len(imported)} licensed XPM sources and {len(imported) * 256} RGBA pixels")


if __name__ == "__main__":
    main()
