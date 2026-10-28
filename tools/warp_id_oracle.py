"""Extract pinned warp digit PNG and compile unchanged label-placement functions.

Generated material is LGPL-3.0-only, attributed to Rangi/Polished Map contributors.
Outputs stay in ignored scratch storage. The harness records FLTK drawing calls;
it does not emulate native font rendering.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

REVISION = "8b9bdc2b8aac8c865809516c2123af69c498da0b"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cxx", default="g++")
    parser.add_argument("--git", default="git")
    args = parser.parse_args()
    source = subprocess.check_output([args.git, "-C", str(args.checkout), "show",
                                      f"{REVISION}:src/event.cpp"], text=True)
    functions, provenance = [], []
    for name in ["void Event::draw_warp_id(", "void Event::draw_warp_id_zoomed("]:
        start = source.index(name)
        function = source[start:source.index("\n}\n", start)+2]
        functions.append(function)
        provenance.append({"function": name, "sha256": hashlib.sha256(function.encode()).hexdigest(), "changes": []})
    body = re.search(r"warp_digits_png_buffer\[\] = \{(.*?)\};", source, re.S).group(1)
    png = bytes(int(n, 16) for n in re.findall(r"0x([0-9a-f]{2})", body))
    assert hashlib.sha256(png).hexdigest() == "fc499957baba4177147b982f080dca61d199886b06d85a7fff1748a69678ac6e"
    harness = r'''
// Copyright Rangi/Polished Map contributors; harness Polished Map NG. LGPL-3.0-only.
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>
static std::ostringstream calls;
struct Image {
    void draw(int x, int y, int w, int h, int cx, int cy) const {
        calls << x << ',' << y << ',' << w << ',' << h << ',' << cx << ',' << cy << ';';
    }
};
static Image warp_digits_image;
enum { FL_COURIER, FL_ALIGN_BOTTOM_RIGHT, FL_ALIGN_INSIDE, FL_BLACK, FL_WHITE };
struct OS { static bool is_consolas() {return false;} };
void fl_font(int, int) {}
void draw_outlined_text(const char *text, int, int, int, int, int, int, int) {calls << text;}
struct Event {
    bool _hex_coords; int _warp_id;
    int w() const {return 32;} int h() const {return 32;}
    void draw_warp_id(int x, int y) const;
    void draw_warp_id_zoomed(int x, int y) const;
};
'''
    driver = r'''
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    std::ofstream output(argv[1], std::ios::binary);
    for (bool hex : {false, true}) for (int id=1; id<=4096; ++id) {
        calls.str(""); calls.clear();
        Event e{hex,id}; e.draw_warp_id(0,0);
        calls << '\t'; e.draw_warp_id_zoomed(0,0);
        output << id << '\t' << hex << '\t' << calls.str() << '\n';
    }
    return output.good() ? 0 : 3;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + "\n".join(functions) + driver, encoding="utf-8")
    args.output.with_suffix(".png").write_bytes(png)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION, "source": "src/event.cpp", "functions": provenance,
        "png_sha256": hashlib.sha256(png).hexdigest(),
    }, indent=2), encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)


if __name__ == "__main__":
    main()
