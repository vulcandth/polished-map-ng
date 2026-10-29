"""Compile unchanged pinned master assembly parsing and CHR image loading.

Generated C++ and snippet hashes stay in ignored scratch storage. LGPL-3.0-only.
The only filesystem adapter opens synthetic ASCII paths with std::ifstream.
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
    p.add_argument("--cxx", default="g++")
    p.add_argument("--git", default="git")
    args = p.parse_args()
    names = ["parse-asm.cpp", "parse-asm.h", "utils.cpp", "utils.h",
             "tiled-image.cpp", "tiled-image.h", "tile.h", "palette-map.h", "colors.h"]
    sources = {name: subprocess.check_output([
        args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/{name}"
    ]).decode("utf-8") for name in names}
    evidence = []

    def snippet(name, start, end):
        source = sources[name]
        a = source.index(start)
        b = source.index(end, a) + len(end)
        text = source[a:b]
        evidence.append({"source": f"src/{name}", "start": start,
                         "sha256": hashlib.sha256(text.encode()).hexdigest(), "changes": []})
        return text

    pieces = [snippet("utils.h", "#define RANGE", "\n"),
              snippet("utils.cpp", "const std::string whitespace", ";"),
              snippet("utils.h", "void trim(", ";"),
              snippet("utils.h", "bool leading_macro(", ";")]
    for start in ["static bool cmp_ignore_case(", "bool equals_ignore_case(",
                  "void trim(", "bool leading_macro(", "void remove_comment("]:
        pieces.append(snippet("utils.cpp", start, "\n}"))
    pieces.extend([snippet("parse-asm.h", "class Parsed_Asm", "\n};"),
                   snippet("parse-asm.cpp", "Parsed_Asm::Parsed_Asm", "\n}"),
                   snippet("parse-asm.cpp", "static uint32_t parse_value(", "\n}"),
                   snippet("parse-asm.cpp", "Parsed_Asm::Result Parsed_Asm::parse_asm(", "\n}")])
    for name, start in [("tile.h", "#define TILE_SIZE "), ("tile.h", "#define TILE_AREA "),
                        ("palette-map.h", "#define MAX_NUM_TILES "),
                        ("tiled-image.h", "#define BYTES_PER_2BPP_TILE ")]:
        pieces.append(snippet(name, start, "\n"))
    pieces.append(snippet("colors.h", "enum class Hue", ";"))
    result = snippet("tiled-image.h", "enum class Result", ";")
    pieces.append("struct Tiled_Image {\n" + result + """
    std::vector<Hue> _tile_hues;
    size_t _num_tiles = 0;
    Result _result = Result::IMG_NULL;
    Result read_asm_graphics(const char*);
};
""")
    pieces.append(snippet("tiled-image.cpp", "static void convert_2bytes_to_8hues(", "\n}"))
    pieces.append(snippet("tiled-image.cpp", "Tiled_Image::Result Tiled_Image::read_asm_graphics(", "\n}"))
    header = r'''
// Copyright Rangi42/Polished Map contributors; driver: Polished Map NG. LGPL-3.0-only.
#include <algorithm>
#include <cctype>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>
#include <string_view>
#include <vector>
using uchar = unsigned char;
void open_ifstream(std::ifstream &ifs, const char *path) { ifs.open(path); }
'''
    driver = r'''
bool read32(std::istream &in, uint32_t &value) {
    unsigned char b[4]; if (!in.read((char*)b,4)) return false;
    value = uint32_t(b[0]) | uint32_t(b[1])<<8 | uint32_t(b[2])<<16 | uint32_t(b[3])<<24;
    return true;
}
void write32(std::ostream &out, uint32_t value) {
    for(int i=0;i<4;++i) out.put(char(value>>(8*i)));
}
int main(int argc, char **argv) {
    if(argc!=4) return 2;
    std::ifstream input(argv[1], std::ios::binary);
    std::ofstream output(argv[2], std::ios::binary);
    uint32_t count=0; if(!read32(input,count) || count>10000) return 3;
    for(uint32_t index=0;index<count;++index) {
        uint32_t size=0; if(!read32(input,size) || size>1024*1024) return 4;
        std::vector<char> bytes(size); if(!input.read(bytes.data(),size)) return 5;
        { std::ofstream file(argv[3],std::ios::binary|std::ios::trunc);
          file.write(bytes.data(),size); if(!file.good()) return 6; }
        Parsed_Asm parsed(argv[3]);
        write32(output,uint32_t(parsed.result()));
        write32(output,uint32_t(parsed.size()));
        for(size_t i=0;i<parsed.size();++i) output.put(char(parsed.get(i)));
        Tiled_Image image;
        write32(output,uint32_t(image.read_asm_graphics(argv[3])));
        write32(output,uint32_t(image._num_tiles));
        for(Hue hue:image._tile_hues) output.put(char(hue));
        if(!output.good()) return 7;
    }
    if(input.peek()!=EOF) return 8;
    return 0;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(header + "\n".join(pieces) + driver, encoding="utf-8", newline="\n")
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION, "functions_and_constants": evidence,
        "adapters": ["open_ifstream uses std::ifstream for synthetic ASCII paths",
                     "minimal Tiled_Image declaration exposes original result and hue buffers"]
    }, indent=2) + "\n", encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)


if __name__ == "__main__":
    main()
