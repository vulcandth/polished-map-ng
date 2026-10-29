"""Compile unchanged master map-size selection methods with inert dropdown storage.

Generated C++ is LGPL-3.0-only. This observes selection math, not native UI.
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

    def source(path):
        return subprocess.check_output(
            [args.git, "-C", str(args.checkout), "show", f"{REVISION}:{path}"]
        ).decode("utf-8").replace("\r\n", "\n")

    cpp_source = source("src/option-dialogs.cpp")
    functions = []
    for signature in ["int Map_Options_Dialog::add_map_size(", "void Map_Options_Dialog::add_valid_sizes("]:
        start = cpp_source.index(signature)
        end = cpp_source.index("\n}\n", start) + 2
        functions.append(cpp_source[start:end])
    header = source("src/option-dialogs.h")
    predicate = next(line.strip() for line in header.splitlines() if "static inline bool valid_size(" in line)
    harness = r'''
// Copyright Rangi42/Polished Map contributors; harness: Polished Map NG. LGPL-3.0-only.
#include <algorithm>
#include <cstdio>
#include <fstream>
#include <vector>
#define RANGE(v) (v).begin(), (v).end()
struct Dropdown { void add(const char *) {} };
struct Map_Options_Dialog {
    std::vector<std::pair<int, int>> _valid_sizes;
    Dropdown dropdown, *_map_sizes = &dropdown;
    int selected = -1;
    int add_map_size(int, int);
    void add_valid_sizes(size_t);
    void select_map_size(int i) { selected = i; }
'''
    driver = r'''
static void word(std::ofstream &out, int value) {
    out.put(static_cast<char>(value & 255));
    out.put(static_cast<char>((value >> 8) & 255));
}
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    std::ofstream out(argv[1], std::ios::binary);
    for (size_t n = 0; n <= 65026; ++n) {
        Map_Options_Dialog dialog;
        dialog.add_valid_sizes(n);
        word(out, dialog.selected);
        word(out, static_cast<int>(dialog._valid_sizes.size()));
        for (auto pair : dialog._valid_sizes) { word(out, pair.first); word(out, pair.second); }
    }
    return out.good() ? 0 : 3;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + predicate + "\n};\n" + "\n".join(functions) + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION,
        "sources": ["src/option-dialogs.cpp", "src/option-dialogs.h"],
        "functions": ["add_map_size", "add_valid_sizes", "valid_size"],
        "source_sha256": hashlib.sha256(("\n".join(functions) + predicate).encode()).hexdigest(),
        "changes_to_functions": [],
        "harness_scope": "every decoded length 0..65026; dropdown storage/callback stubbed",
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
