"""Compile pinned master event parsing/formatting/warp functions with UI stubs.

The functions and metadata table are extracted unchanged. This tests event
semantics, not FLTK dispatch or file persistence. No game assets are used.
Generated C++ is LGPL-3.0-only; provenance hashes accompany the executable.
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
    functions, provenance = [], []
    for path, names in [
        ("src/utils.cpp", ["void trim(", "void remove_comment("]),
        ("src/event.cpp", ["static int parse_value(", "std::pair<std::string, int> Event::warp_destination(",
                           "static int16_t parse_coord(", "void Event::parse(", "void Event::update_tooltip("]),
    ]:
        source = subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{REVISION}:{path}"], text=True)
        for name in names:
            start = source.index(name)
            function = source[start:source.index("\n}\n", start)+2]
            functions.append(function)
            provenance.append({"source": path, "function": name, "sha256": hashlib.sha256(function.encode()).hexdigest(), "changes": []})
    source = subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/map-events.cpp"], text=True)
    start = source.index("static const std::unordered_map")
    metadata = source[start:source.index("\n};", start)+3]
    provenance.append({"source": "src/map-events.cpp", "table": "events_meta", "sha256": hashlib.sha256(metadata.encode()).hexdigest(), "changes": []})
    harness = r'''
// Copyright Rangi42/Polished Map contributors; harness: Polished Map NG. LGPL-3.0-only.
#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <iomanip>
#include <sstream>
#include <string>
#include <unordered_map>
#define EVENT_MARGIN 6
enum class EventTexture {TX_RED, TX_GREEN, TX_BLUE, TX_PURPLE, TX_ORANGE, TX_AZURE};
struct Event_Meta {
    EventTexture texture; char symbol; uint8_t skip; bool yx; bool id_map;
    bool is_warp() const {return texture == EventTexture::TX_PURPLE;}
};
void trim(std::string &, const std::string & = " \t\r\n");
void remove_comment(std::string &);
struct Event {
    Event_Meta _meta; int16_t _event_x=0, _event_y=0;
    std::string _macro, _prefix, _suffix, _tip;
    bool _prefixed=false, _suffixed=false, _hex_coords=false;
    std::pair<std::string,int> warp_destination() const;
    void parse(std::istringstream &);
    void update_tooltip();
    void tip(const std::string &s) {_tip=s;}
};
'''
    driver = r'''
int main(int argc, char **argv) {
    if(argc != 3) return 2;
    std::ifstream input(argv[1]); std::ofstream output(argv[2], std::ios::binary);
    std::string line;
    while(std::getline(input,line)) {
        std::istringstream stream(line); std::string name;
        stream >> std::ws >> name >> std::ws;
        Event e; e._macro=name; e._meta=events_meta.at(name); e.parse(stream); e.update_tooltip();
        auto dest=e.warp_destination();
        output << e._event_x << '\t' << e._event_y << '\t' << e._hex_coords << '\t'
            << e._meta.is_warp() << '\t' << e._meta.symbol << '\t' << e._tip << '\t'
            << dest.first << '\t' << dest.second << '\n';
    }
    return input.bad() || !output.good() ? 3 : 0;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + metadata + "\n" + "\n".join(functions) + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({"revision": REVISION, "functions": provenance}, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
