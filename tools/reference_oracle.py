"""Build an oracle using an unmodified function extracted from pinned upstream.

Usage: python tools/reference_oracle.py --checkout .reference/master
       --output .scratch/reference-2bpp --cxx g++
The generated source is a derivative of Polished Map (LGPL-3.0-only).
Only valid inputs are compared; this is not a full application parity test.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

REVISION = "8b9bdc2b8aac8c865809516c2123af69c498da0b"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cxx", default="g++")
    parser.add_argument("--git", default="git")
    args = parser.parse_args()
    source = subprocess.check_output([
        args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/tiled-image.cpp"
    ], text=True)
    start = source.index("static void convert_2bytes_to_8hues(")
    end = source.index("\n}\n", start) + 2
    function = source[start:end]
    header = """// Original function: Copyright Rangi42/Polished Map contributors.
// Driver: Copyright 2026 Polished Map NG contributors. LGPL-3.0-only.
#include <cstdint>
#include <fstream>
using uchar = unsigned char;
enum class Hue { WHITE, DARK, LIGHT, BLACK };
"""
    driver = """
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    std::ofstream output(argv[1], std::ios::binary);
    if (!output) return 3;
    for (unsigned a=0; a<256; ++a) {
        for (unsigned b=0; b<256; ++b) {
            Hue hues[8];
            convert_2bytes_to_8hues((uchar)a, (uchar)b, hues);
            for (Hue hue : hues) output.put((char)hue);
        }
    }
    return output.good() ? 0 : 4;
}
"""
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(header + function + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION,
        "source": "src/tiled-image.cpp",
        "function": "convert_2bytes_to_8hues",
        "function_sha256": hashlib.sha256(function.encode()).hexdigest(),
        "changes_to_function": [],
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
