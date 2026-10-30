"""Check native GUI adapters in Cargo's production graph, excluding dev edges.

This verifies compile-time wiring, not OS clipboard or browser operation. Run
on each CI host, or supply --target to inspect another supported target graph.
"""
import argparse
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", help="Rust target triple (default: rustc host)")
    args = parser.parse_args()
    target = args.target or next(
        line.removeprefix("host: ")
        for line in run("rustc", "-vV").splitlines()
        if line.startswith("host: ")
    )
    graph = run(
        "cargo", "tree", "--locked", "-p", "polished-map-gui",
        "--target", target, "--edges", "normal,build", "--prefix", "none",
        "--format", "{p}|{f}",
    )
    packages = {}
    for line in graph.splitlines():
        package, features = line.split("|", 1)
        name = package.split()[0]
        packages.setdefault(name, set()).update(
            features.removesuffix(" (*)").split(",")
        )
    required = {
        "eframe": {"clipboard", "links"},
        "egui-winit": {"clipboard", "arboard", "links", "webbrowser"},
        "arboard": {"image-data"},
        "webbrowser": set(),
    }
    if "linux" in target:
        required["egui-winit"].add("smithay-clipboard")
        required["arboard"].add("wayland-data-control")
        required["smithay-clipboard"] = set()
    errors = []
    for name, version in {"winit": "0.30.13", "egui-winit": "0.37.0"}.items():
        entries = [line.split("|", 1)[0] for line in graph.splitlines() if line.startswith(name + " ")]
        if not entries or any(not entry.startswith(f"{name} v{version} ")
                              or f"/vendor/{name}-{version}" not in entry.replace("\\", "/")
                              for entry in entries):
            errors.append(f"{name} must use the audited local {version} native adapter")
    for name, features in required.items():
        if name not in packages:
            errors.append(f"Missing production dependency: {name}")
        elif missing := features - packages[name]:
            errors.append(f"{name} missing features: {', '.join(sorted(missing))}")
    if errors:
        raise SystemExit(f"Native adapter graph failed for {target}:\n" + "\n".join(errors))
    print(f"Verified production clipboard/image/browser and pinned file-drop adapters for {target}")


if __name__ == "__main__":
    main()
