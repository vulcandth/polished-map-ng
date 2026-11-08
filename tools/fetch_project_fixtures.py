#!/usr/bin/env python3
"""Fetch immutable external test projects; never include these assets in packages."""
import argparse
import re
import subprocess
import tomllib
from pathlib import Path


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], text=True, encoding="utf-8").strip()


def main() -> None:
    repo = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, default=repo / ".reference")
    args = parser.parse_args()
    fixtures = tomllib.loads((repo / "upstream.lock.toml").read_text(encoding="utf-8"))["fixtures"]
    for name, fixture in fixtures.items():
        if not re.fullmatch(r"[a-z0-9_-]+", name) or not re.fullmatch(r"[0-9a-f]{40}", fixture["commit"]):
            raise ValueError(f"Invalid fixture name or immutable commit: {name}")
        target = args.destination.resolve() / name
        if target.exists():
            if git("-C", str(target), "rev-parse", "HEAD") != fixture["commit"]:
                raise ValueError(f"{target}: existing checkout differs from lock; left untouched")
            if git("-C", str(target), "status", "--porcelain", "--untracked-files=no"):
                raise ValueError(f"{target}: tracked fixture files are modified; left untouched")
        else:
            git("init", str(target))
            git("-C", str(target), "fetch", "--depth", "1", fixture["repository"], fixture["commit"])
            git("-C", str(target), "checkout", "--detach", "FETCH_HEAD")
            assert git("-C", str(target), "rev-parse", "HEAD") == fixture["commit"]
        print(f"{name}: {fixture['commit']}")


if __name__ == "__main__":
    main()
