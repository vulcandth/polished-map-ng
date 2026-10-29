"""Compile pinned FLTK numeric sorting and UTF-8 decoding without a GUI.

The unchanged functions retain FLTK's license and copyright; see LICENSES/FLTK.txt.
Only the case-sensitive path used by master fl_filename_list is exercised.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

REVISION = "a9b1113516ffd15fc7602a6d425a317df30f4720"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cxx", default="g++")
    parser.add_argument("--git", default="git")
    args = parser.parse_args()

    def read(path):
        return subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{REVISION}:{path}"]).decode("utf-8").replace("\r\n", "\n")

    def function(source, signature):
        start = source.index(signature)
        return source[start:source.index("\n}", start) + 2]

    sorting = read("src/numericsort.c")
    utf8 = read("src/fl_utf8.cxx")
    comparator = function(sorting, "static int numericsort(")
    decoder = function(utf8, "unsigned fl_utf8decode(")
    start = utf8.index("static unsigned short cp1252[32]")
    table = utf8[start:utf8.index("\n};", start) + 3]
    harness = r'''
// Copyright 1998-2018 Bill Spitzak and others. FLTK terms: LICENSES/FLTK.txt.
#include <cctype>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>
#define ERRORS_TO_CP1252 1
#define ERRORS_TO_ISO8859_1 1
#define STRICT_RFC3629 0
struct dirent { char d_name[1024] = {}; };
// Not exercised: the driver always requests the case-sensitive branch.
static unsigned fl_tolower(unsigned c) { return c; }
'''
    driver = r'''
int main(int argc, char **argv) {
    if (argc != 3) return 2;
    std::ifstream input(argv[1], std::ios::binary);
    std::vector<dirent> entries;
    std::string line;
    while (std::getline(input, line)) {
        if (line.size() > 1000) return 3;
        dirent entry;
        memcpy(entry.d_name, line.data(), line.size());
        entries.push_back(entry);
    }
    std::ofstream output(argv[2], std::ios::binary);
    for (auto &a : entries) for (auto &b : entries) {
        dirent *pa = &a, *pb = &b;
        output.put(static_cast<char>(numericsort(&pa, &pb, 1)));
    }
    return output.good() ? 0 : 4;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    snippets = table + "\n" + decoder + "\n" + comparator
    cpp.write_text(harness + snippets + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION,
        "sources": ["src/numericsort.c", "src/fl_utf8.cxx"],
        "functions": ["numericsort", "fl_utf8decode"],
        "snippet_sha256": hashlib.sha256(snippets.encode()).hexdigest(),
        "changes_to_functions": [],
        "scope": "case-sensitive comparison of valid UTF-8 filenames; not directory enumeration",
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
