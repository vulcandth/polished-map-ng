"""Compile unchanged master binary-map metadata discovery with inert UI storage.

Generated code is LGPL-3.0-only. ASCII fixture paths use standard streams;
native file dialogs, ASM decoding and filesystem case behavior are not simulated.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

REVISION = "8b9bdc2b8aac8c865809516c2123af69c498da0b"


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--checkout", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--windows", action="store_true", help="original case-insensitive global-header matching")
    p.add_argument("--cxx", default="g++")
    p.add_argument("--git", default="git")
    args = p.parse_args()

    def read(path):
        return subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{REVISION}:{path}"]).decode("utf-8").replace("\r\n", "\n")

    def function(source, signature):
        start = source.index(signature)
        return source[start:source.index("\n}", start) + 2]

    utils, config, options = [read(f"src/{name}") for name in ["utils.cpp", "config.cpp", "option-dialogs.cpp"]]
    snippets = [function(utils, signature) for signature in [
        "static bool cmp_ignore_case(", "bool equals_ignore_case(std::string_view",
        "bool starts_with(", "bool ends_with_ignore_case(std::string_view",
        "void trim(", "void lowercase(", "bool leading_macro(", "void remove_comment(", "void remove_dot_ext("]]
    snippets += [function(config, signature) for signature in ["void Config::map_constants_path(", "bool Config::map_headers_path(", "void Config::map_header_path("]]
    snippets += [function(options, signature) for signature in ["static void guess_map_constant(", "bool Map_Options_Dialog::guess_map_size(", "std::string Map_Options_Dialog::guess_map_tileset(", "int Map_Options_Dialog::add_map_size(", "void Map_Options_Dialog::add_valid_sizes("]]
    map_header = read("src/map.h")
    start = map_header.index("struct Map_Attributes {")
    attrs = map_header[start:map_header.index("\n};", start) + 3]
    helpers = "\n".join(line for line in options.splitlines() if line.startswith("inline static bool isupperordigit") or line.startswith("inline static int toconstant"))
    header = read("src/option-dialogs.h")
    valid_size = next(line.strip() for line in header.splitlines() if "static inline bool valid_size(" in line)
    harness = r'''
// Copyright Rangi42/Polished Map contributors; harness: Polished Map NG. LGPL-3.0-only.
#include <algorithm>
#include <cctype>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <regex>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>
#define FL_PATH_MAX 4096
#define DIR_SEP "/"
#define NEW_MAP_NAME "New Map"
#define RANGE(x) std::begin(x), std::end(x)
const std::string whitespace(" \f\n\r\t\v");
void trim(std::string &, const std::string & = whitespace);
bool leading_macro(std::istringstream &, std::string &, const char * = NULL);
const char *fl_filename_name(const char *path) {
    const char *name = path;
    for (const char *p = path; *p; ++p) if (*p == '/' || *p == '\\') name = p + 1;
    return name;
}
void open_ifstream(std::ifstream &stream, const char *path) { stream.open(path); }
bool file_exists(const char *path) { return std::filesystem::is_regular_file(path); }
size_t file_size(const char *path) { return std::filesystem::file_size(path); }
struct Parsed_Asm {
    Parsed_Asm(const char *) { throw std::runtime_error("binary-map oracle only"); }
    size_t size() const { return 0; }
};
struct Widget {
    int v = 0;
    void clear() {} void add(const char *) {} void label(const char *) {} void copy_label(const char *) {}
    void value(int value) { v = value; }
    int value() const { return v; }
};
struct Config {
    static void map_constants_path(char *, const char *);
    static bool map_headers_path(char *, const char *);
    static void map_header_path(char *, const char *, const char *);
};
'''
    dialog = r'''
struct Map_Options_Dialog {
    Widget storage[5], *_map_sizes = &storage[0], *_map_header = &storage[1], *_map_size = &storage[2], *_map_width = &storage[3], *_map_height = &storage[4];
    std::vector<std::pair<int, int>> _valid_sizes;
    bool guess_map_size(const char *, const char *, Map_Attributes &);
    static std::string guess_map_tileset(const char *, const char *, Map_Attributes &);
    int add_map_size(int, int);
    void add_valid_sizes(size_t);
    void select_map_size(int i) { _map_width->value(_valid_sizes[i].first); _map_height->value(_valid_sizes[i].second); }
'''
    platform = "\n#undef _WIN32\n" + ("#define _WIN32 1\n" if args.windows else "")
    driver = r'''
int main(int argc, char **argv) {
    if (argc != 3) return 2;
    std::string root = std::string(argv[1]) + "/";
    const char *path = argv[2];
    Map_Options_Dialog dialog;
    Map_Attributes attrs;
    char constant[FL_PATH_MAX] = {};
    guess_map_constant(fl_filename_name(path), constant);
    bool dimensions = dialog.guess_map_size(path, root.c_str(), attrs);
    std::string hint = dialog.guess_map_tileset(path, root.c_str(), attrs);
    std::cout << "{\"constant\":" << std::quoted(constant) << ",\"dimensions\":";
    if (dimensions) std::cout << "[" << dialog._map_width->value() << "," << dialog._map_height->value() << "]";
    else std::cout << "null";
    std::cout << ",\"group\":" << static_cast<int>(attrs.group)
        << ",\"hint\":" << std::quoted(hint)
        << ",\"environment\":" << std::quoted(attrs.environment)
        << ",\"landmark\":" << std::quoted(attrs.landmark)
        << ",\"palette\":" << std::quoted(attrs.palette) << "}\n";
}
'''
    body = "\n".join(snippets)
    cpp = args.output.with_suffix(".cpp")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp.write_text(harness + attrs + dialog + valid_size + "\n};\n" + helpers + platform + body + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION,
        "sources": ["src/option-dialogs.cpp", "src/option-dialogs.h", "src/config.cpp", "src/utils.cpp", "src/map.h"],
        "snippet_sha256": hashlib.sha256((body + attrs + helpers + valid_size).encode()).hexdigest(),
        "changes_to_functions": [],
        "windows_header_matching": args.windows,
        "scope": "binary map metadata; inert widgets, standard streams, existing ASCII fixture paths",
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
