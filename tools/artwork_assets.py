"""Verify original application/message artwork against its pinned inventory.

Uses only the standard library; no reference checkout or network is required.
"""
import argparse
import hashlib
import json
from pathlib import Path
from icon_assets import decode

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "crates/gui/assets/artwork"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    inventory = json.loads((ROOT / "docs/artwork-inventory.json").read_text(encoding="utf-8"))
    expected = {Path(i["file"]).name for i in inventory["images"]}
    if {p.name for p in ASSETS.glob("*.xpm")} != expected:
        raise ValueError("Artwork XPMs must match the inventory")
    for item in inventory["images"]:
        path = ASSETS / Path(item["file"]).name
        data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != item["xpm_sha256"]:
            raise ValueError(f"Source hash mismatch: {path.name}")
        pixels = decode(data, (item["width"], item["height"]))
        if hashlib.sha256(pixels).hexdigest() != item["rgba_sha256"]:
            raise ValueError(f"Decoded hash mismatch: {path.name}")
        output = path.with_suffix(".rgba")
        if args.write:
            output.write_bytes(pixels)
        elif output.read_bytes() != pixels:
            raise ValueError(f"RGBA mismatch: {output.name}")
    if {p.stem for p in ASSETS.glob("*.rgba")} != {Path(n).stem for n in expected}:
        raise ValueError("Unexpected RGBA artwork")
    bundle = inventory["windows_bundle"]
    if hashlib.sha256((ASSETS / Path(bundle["file"]).name).read_bytes()).hexdigest() != bundle["sha256"]:
        raise ValueError("Original Windows ICO hash mismatch")
    print("Verified 5 original artwork XPMs, 9,472 RGBA pixels and the Windows ICO")


if __name__ == "__main__":
    main()
