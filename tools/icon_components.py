"""Reproduce the reviewed Open/Load/Load Palettes component comparisons.

Requires Pillow and the independently downloaded Fugue 3.5.6 archive. It neither
downloads nor imports artwork. Full-image/source verification remains in
icon_assets.py; these comparisons support, but do not replace, the human review
and attribution chain documented in docs/assets.md.
"""
import argparse
import hashlib
from io import BytesIO
import json
from pathlib import Path
import zipfile

from PIL import Image
from icon_assets import ASSETS, ROOT, decode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    args = parser.parse_args()
    raw = args.archive.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == "59d54804343b9add8c96ea03bfb2fa3ff48f692459cf426c920a5612e771c37a"
    inventory = json.loads((ROOT / "docs/icon-inventory.json").read_text(encoding="utf-8"))
    components = {}
    for icon in inventory["icons"]:
        for component in icon.get("distribution", {}).get("fugue_components", []):
            components[component["file"]] = component["sha256"]
    with zipfile.ZipFile(BytesIO(raw)) as archive:
        for name, digest in components.items():
            assert hashlib.sha256(archive.read(name)).hexdigest() == digest, name

        def source_mask(name, threshold, dx, dy):
            image = Image.open(BytesIO(archive.read("icons/" + name + ".png"))).convert("RGBA")
            return {(x + dx, y + dy) for y in range(16) for x in range(16)
                    if image.getpixel((x, y))[3] > threshold}

        def pixels(stem):
            raw = decode((ASSETS / (stem + ".xpm")).read_bytes())
            return {(x, y): tuple(raw[(y * 16 + x) * 4:(y * 16 + x + 1) * 4])
                    for y in range(16) for x in range(16)}

        opened, loaded, palettes = [pixels(stem) for stem in ["open", "load", "load-palettes"]]
        blue = {(0x22, 0x5d, 0x8d, 255), (0x1c, 0x79, 0xc1, 255), (0x38, 0xc5, 0xff, 255)}
        arrow = {pos for pos, color in opened.items() if color in blue}
        assert len(arrow) == 28
        assert arrow == source_mask("arrow-000-small", 127, 4, 3)

        # Isolate the small bulb's non-document colors plus its white highlights.
        background = {(255, 255, 255, 255), (0x80, 0x91, 0x91, 255)}
        bulb = {pos for pos, color in palettes.items()
                if 8 <= pos[0] <= 12 and 4 <= pos[1] <= 12 and color not in background}
        bulb |= {(9, 5), (10, 11)}
        assert len(bulb) == 32
        assert bulb == source_mask("light-bulb-small", 74, 3, 1)

        # Load reuses Open's complete folder/document with a three-color recolor.
        # The modified arrow is excluded; no claim of an unchanged Fugue arrow.
        recolor = {(0x8b, 0x5c, 0, 255): (0x5a, 0x7c, 0x9d, 255),
                   (0xa9, 0x7d, 0x26, 255): (0x72, 0x94, 0xad, 255),
                   (0xfa, 0xd8, 0x94, 255): (0xa1, 0xc3, 0xd9, 255)}
        compared = 0
        for pos, color in opened.items():
            if color not in blue and loaded[pos] not in blue:
                assert loaded[pos] == recolor.get(color, color), pos
                compared += 1
            if pos[0] <= 6:
                assert palettes[pos] == color, pos
        print(f"Verified {len(components)} source component hashes, 28 arrow/32 bulb mask pixels, "
              f"{compared} Open/Load base pixels and the 112-pixel shared folder strip")


if __name__ == "__main__":
    main()
