#!/usr/bin/env python3
"""Assemble development binaries or complete, offline-rebuildable source.

Run from the repository root. Uses only Python's standard library and Cargo/Git.
Never includes ignored working projects or external reference checkouts.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess
import tarfile
import tempfile


def run(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True, encoding="utf-8")


def metadata():
    return json.loads(run("cargo", "metadata", "--locked", "--format-version", "1"))


def notices(destination, packages):
    """Keep dependency license files and full lockfile-wide license metadata."""
    inventory = []
    for package in packages:
        root = Path(package["manifest_path"]).parent
        files = set()
        if package.get("license_file"):
            files.add(root / package["license_file"])
        # Includes nested font/third-party notices, not only the crate's license.
        for path in root.rglob("*"):
            if path.is_file() and path.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "COPYRIGHT", "NOTICE", "AUTHORS")):
                files.add(path)
        name = f'{package["name"]}-{package["version"]}'
        copied = []
        for path in sorted(files):
            relative = path.relative_to(root)
            target = destination / "dependencies" / name / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
            copied.append(relative.as_posix())
        inventory.append({"name": package["name"], "version": package["version"],
                          "license": package["license"], "source": package["source"],
                          "repository": package["repository"], "notices": copied})
    (destination / "dependency-inventory.json").write_text(
        json.dumps(inventory, indent=2) + "\n", encoding="utf-8")


def common(destination, info):
    for name in ["LICENSE.md", "NOTICE.md", "INSTALL.md", "Cargo.lock", "rust-toolchain.toml", "upstream.lock.toml"]:
        shutil.copy2(name, destination / name)
    shutil.copytree("LICENSES", destination / "LICENSES", dirs_exist_ok=True)
    shutil.copytree("docs", destination / "docs", dirs_exist_ok=True)
    # Original source art, converted pixels and notices travel with the binaries.
    shutil.copytree("crates/gui/assets", destination / "crates/gui/assets", dirs_exist_ok=True)
    shutil.copy2("docs/rebuild.md", destination / "REBUILD.md")
    shutil.copy2("docs/testing.md", destination / "TESTING.md")
    (destination / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=["source", "binary"])
    parser.add_argument("--target")
    parser.add_argument("--output", type=Path, default=Path(".scratch/dist"))
    args = parser.parse_args()
    if args.kind == "binary" and not args.target:
        parser.error("binary requires --target")
    commit = (json.loads(Path("BUILD-INFO.json").read_text(encoding="utf-8"))["commit"]
              if Path("BUILD-INFO.json").is_file() else run("git", "rev-parse", "HEAD").strip())
    stem = "polished-map-ng-dev-" + commit[:12]
    args.output.mkdir(parents=True, exist_ok=True)
    output = args.output.resolve()
    info = {"commit": commit, "development": True, "target": args.target,
            "rustc": run("rustc", "-Vv").strip(), "build_host": platform.platform(),
            "source_archive": stem + "-source.tar.gz",
            "signing": "ad-hoc only" if args.target and "apple" in args.target else "unsigned"}
    with tempfile.TemporaryDirectory(prefix="polished-map-package-") as temporary:
        root = Path(temporary)
        name = stem + ("-source" if args.kind == "source" else "-" + args.target)
        stage = root / name
        stage.mkdir()
        if args.kind == "source":
            archive = root / "tracked.tar"
            subprocess.run(["git", "archive", "--format=tar", "HEAD", "-o", str(archive)], check=True)
            with tarfile.open(archive) as tar:
                tar.extractall(stage, filter="data")
            # Preserve the patched native crates in vendor/; registry dependencies
            # are separate. Cargo emits the exact source replacement configuration.
            config = run("cargo", "vendor", "--locked", "--versioned-dirs", "third-party", cwd=stage)
            (stage / ".cargo").mkdir(exist_ok=True)
            with (stage / ".cargo/config.toml").open("a", encoding="utf-8") as file:
                file.write("\n" + config)
            subprocess.run(["cargo", "metadata", "--offline", "--locked", "--format-version", "1"],
                           cwd=stage, check=True, stdout=subprocess.DEVNULL)
            shutil.copy2("docs/rebuild.md", stage / "REBUILD.md")
            (stage / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
        else:
            common(stage, info)
            notices(stage / "LICENSES", metadata()["packages"])
            suffix = ".exe" if "windows" in args.target else ""
            binaries = Path(os.environ.get("CARGO_TARGET_DIR", "target")) / args.target / "release"
            shutil.copy2(binaries / ("polished-map" + suffix), stage / ("polished-map" + suffix))
            gui = stage / ("polished-map-gui" + suffix)
            if "apple" in args.target:
                bundle = stage / "Polished Map NG.app"
                contents = bundle / "Contents"
                (contents / "MacOS").mkdir(parents=True)
                gui = contents / "MacOS/polished-map-gui"
                with (contents / "Info.plist").open("wb") as file:
                    plistlib.dump({"CFBundleName": "Polished Map NG", "CFBundleDisplayName": "Polished Map NG",
                                  "CFBundleIdentifier": "io.github.vulcandth.polished-map-ng",
                                  "CFBundleExecutable": "polished-map-gui", "CFBundlePackageType": "APPL",
                                  "CFBundleVersion": "0.1.0", "CFBundleShortVersionString": "0.1.0",
                                  "LSMinimumSystemVersion": "11.0", "NSHighResolutionCapable": True}, file)
            shutil.copy2(binaries / ("polished-map-gui" + suffix), gui)
            if "apple" in args.target:
                subprocess.run(["codesign", "--force", "--sign", "-", str(bundle)], check=True)
                subprocess.run(["codesign", "--verify", "--deep", "--strict", str(bundle)], check=True)
            # Run the shipped CLI, not a separate debug artifact.
            subprocess.run([str(stage / ("polished-map" + suffix)), "--help"], check=True, stdout=subprocess.DEVNULL)
        fmt = "zip" if args.target and ("windows" in args.target or "apple" in args.target) else "gztar"
        archive = Path(shutil.make_archive(str(output / name), fmt, root, name))
        with archive.open("rb") as file:
            digest = hashlib.file_digest(file, "sha256").hexdigest()
        archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n", encoding="utf-8")
        print(f"Packaged {archive.name} ({archive.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
