"""Compile pinned master Config asset-path rules without a GUI.

Generated source is LGPL-3.0-only. The adapter covers readable ASCII fixture
files/directories; native access permissions and filesystem case rules are not
emulated. No game assets are included.
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

    def read(path):
        return subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{REVISION}:{path}"]).decode("utf-8").replace("\r\n", "\n")

    def function(source, signature):
        start = source.index(signature)
        return source[start:source.index("\n}", start) + 2]

    config = read("src/config.cpp")
    utils = read("src/utils.cpp")
    signatures = [
        "void Config::gfx_tileset_dir(", "void Config::gfx_roof_dir(",
        "static void _palette_map_path(", "void Config::palette_map_path(",
        "void Config::tileset_path(", "bool Config::tileset_before_path(",
        "bool Config::tileset_after_path(", "void Config::tileset_png_paths(",
        "void Config::roof_path(", "void Config::roof_png_path(",
        "static void _metatileset_path(", "void Config::metatileset_path(",
        "static bool _collisions_path(", "bool Config::collisions_path(",
        "void Config::event_script_path(", "void Config::tileset_constants_path(",
        "void Config::bg_tiles_pal_path(", "void Config::roofs_pal_path(",
        "void Config::special_pal_path(",
    ]
    snippets = [function(utils, f"void {name}(") for name in ["remove_suffix", "before_suffix", "after_suffix"]]
    snippets += [function(config, signature) for signature in signatures]
    declarations = "\n".join("static " + function(config, s).split(" {")[0].replace("Config::", "") + ";" for s in signatures if "Config::" in s)
    harness = r'''
// Copyright Rangi42/Polished Map contributors; harness: Polished Map NG. LGPL-3.0-only.
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <iomanip>
#include <iostream>
#include <string>
#define FL_PATH_MAX 4096
#define DIR_SEP "/"
bool file_exists(const char *path) { return std::filesystem::exists(path); }
void fl_filename_setext(char *path, int, const char *ext) {
    char *slash = strrchr(path, '/'), *dot = strrchr(path, '.');
    if (dot && (!slash || dot > slash)) *dot = '\0';
    strcat(path, ext);
}
struct Config {
    static bool monochrome() { return false; }
'''
    driver = r'''
int main(int argc, char **argv) {
    if (argc != 5) return 2;
    const std::string root = std::string(argv[1]) + "/";
    const char *r = root.c_str(), *name = argv[2];
    char path[FL_PATH_MAX] = {}, before[FL_PATH_MAX] = {}, after[FL_PATH_MAX] = {};
    bool first = true;
    std::cout << "{";
    auto emit = [&](const char *key, const char *value) {
        if (!first) std::cout << ",";
        first = false;
        std::cout << std::quoted(key) << ":" << std::quoted(value);
    };
    Config::tileset_path(path, r, name); emit("graphics", path);
    path[0] = '\0'; Config::tileset_before_path(path, r, name); emit("before", path);
    path[0] = '\0'; Config::tileset_after_path(path, r, name); emit("after", path);
    Config::tileset_png_paths(path, before, after, r, name);
    emit("png", path); emit("before_png", before); emit("after_png", after);
    Config::palette_map_path(path, r, name); emit("palette_map", path);
    Config::metatileset_path(path, r, name); emit("blocks", path);
    bool binary = Config::collisions_path(path, r, name); emit("collisions", path);
    emit("binary_collisions", binary ? "true" : "false");
    Config::roof_path(path, r, "roof"); emit("roof", path);
    Config::roof_png_path(path, r, "roof"); emit("roof_png", path);
    Config::event_script_path(path, r, "Town"); emit("events", path);
    Config::tileset_constants_path(path, r); emit("constants", path);
    Config::bg_tiles_pal_path(path, r); emit("bg", path);
    Config::roofs_pal_path(path, r); emit("roof_colors", path);
    Config::special_pal_path(path, r, strcmp(argv[3], "-") ? argv[3] : nullptr,
        strcmp(argv[4], "-") ? argv[4] : nullptr, name); emit("special", path);
    std::cout << "}\n";
}
'''
    body = "\n".join(snippets)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + declarations + "\n};\n" + body + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION, "sources": ["src/config.cpp", "src/utils.cpp"],
        "snippet_sha256": hashlib.sha256(body.encode()).hexdigest(),
        "changes_to_functions": [],
        "scope": "readable ASCII asset paths; standard filesystem adapter; monochrome false",
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
