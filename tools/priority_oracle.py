"""Extract pinned original priority PNGs for independent Rust image-decoder checks.

Artwork: Rangi/Polished Map contributors, LGPL-3.0-only. Outputs stay ignored.
No image library is required here; Rust's image decoder checks the source pixels.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

REVISION = "8b9bdc2b8aac8c865809516c2123af69c498da0b"
HASHES = {
    "small": "19c7a689eed9e04b090fb16ff9cfc00c9c58b8123b1a11c8cd8c7bbb8a3085e6",
    "large": "eaee27e74a3b6c0edb1695a4023ab9171a99043a4b238e892f01539e6cd25f92",
    "chip": "6bf3dfe075b20ed46a9c714a101d4885f4617b0bce1ed1f0c0679e9c4e3534c3",
}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--git", default="git")
    args = parser.parse_args()
    source = subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/tile.cpp"], text=True, encoding="utf-8")
    args.output.mkdir(parents=True, exist_ok=True)
    for name, expected in HASHES.items():
        body = re.search(name + r"_priority_png_buffer\[\] = \{(.*?)\};", source, re.S).group(1)
        png = bytes(int(n, 16) for n in re.findall(r"0x([0-9a-f]{2})", body))
        if hashlib.sha256(png).hexdigest() != expected:
            raise ValueError(f"Unexpected {name} PNG hash")
        (args.output / f"{name}.png").write_bytes(png)
    (args.output / "provenance.json").write_text(json.dumps({"revision": REVISION, "source": "src/tile.cpp", "sha256": HASHES, "changes": []}, indent=2) + "\n", encoding="utf-8")
    print("Extracted three hash-verified original priority PNGs")

if __name__ == "__main__":
    main()
