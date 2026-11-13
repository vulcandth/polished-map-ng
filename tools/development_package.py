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
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile
from package_docs import copy_documentation
from windows_icon_resources import verify as verify_windows_icons


PROFILES = {
    "master": ("polished-map-gui", "Polished Map NG", "io.github.vulcandth.polished-map-ng"),
    "plusplus": ("polished-map-plusplus", "Polished Map NG++", "io.github.vulcandth.polished-map-ng.plusplus"),
    "unified": ("polished-map-unified", "Polished Map NG Unified", "io.github.vulcandth.polished-map-ng.unified"),
}


def package_profile(requested, source_info):
    # Archives predating profiles were master builds; new packages default to main.
    inherited = source_info.get("profile", "master" if source_info else "unified")
    if inherited not in PROFILES:
        raise ValueError(f"Unknown source profile: {inherited}")
    if requested is not None and requested not in PROFILES:
        raise ValueError(f"Unknown requested profile: {requested}")
    if source_info and requested is not None and requested != inherited:
        raise ValueError("Binary profile must match its source archive")
    return requested or inherited


def profile_suffix(profile):
    package_profile(profile, {})
    return "" if profile == "master" else "-" + profile


def package_stem(commit, profile):
    return "polished-map-ng" + profile_suffix(profile) + "-dev-" + commit[:12]


def copy_gui(destination, binaries, target, profile, version):
    """Copy the selected launcher without falling back to a different profile."""
    executable, display_name, identifier = PROFILES[profile]
    suffix = ".exe" if "windows" in target else ""
    gui = destination / (executable + suffix)
    bundle = None
    if "apple" in target:
        bundle = destination / (display_name + ".app")
        contents = bundle / "Contents"
        (contents / "MacOS").mkdir(parents=True)
        gui = contents / "MacOS" / executable
        with (contents / "Info.plist").open("wb") as file:
            plistlib.dump({"CFBundleName": display_name, "CFBundleDisplayName": display_name,
                          "CFBundleIdentifier": identifier, "CFBundleExecutable": executable,
                          "CFBundlePackageType": "APPL", "CFBundleVersion": version,
                          "CFBundleShortVersionString": version, "LSMinimumSystemVersion": "11.0",
                          "NSHighResolutionCapable": True}, file)
    shutil.copy2(binaries / (executable + suffix), gui)
    return bundle


def run(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True, encoding="utf-8")


def select_packages(packages, graph):
    """Fail closed if Cargo's selected graph cannot be mapped to package sources."""
    selected = set(re.findall(r"^(\S+) v(\S+)", graph, re.MULTILINE))
    if not selected:
        raise ValueError("Empty production dependency graph")
    found = {}
    for package in packages:
        key = package["name"], package["version"]
        if key in selected:
            if key in found:
                raise ValueError(f"Ambiguous dependency source: {key}")
            found[key] = package
    if missing := selected - found.keys():
        raise ValueError(f"Missing production dependency sources: {sorted(missing)}")
    return [found[key] for key in sorted(found)]


def production_packages(target):
    # Include build generators/proc macros conservatively: generated code may be
    # embedded. Exclude dev-only crates, optional test features and other targets.
    graph = run("cargo", "tree", "--locked", "--offline", "--target", target,
                "-p", "polished-map-gui", "-p", "polished-map-cli",
                "--edges", "normal,build", "--prefix", "none", "--format", "{p}")
    # Metadata's platform filter can omit host build dependencies during cross
    # compilation. Let the tree select the host/target graph from full metadata.
    data = json.loads(run("cargo", "metadata", "--locked", "--offline",
                          "--format-version", "1"))
    return select_packages(data["packages"], graph)


def source_commit():
    """A Git package must describe committed source, not a mixed working tree."""
    if run("git", "status", "--porcelain", "--untracked-files=normal").strip():
        raise ValueError("Commit or remove working-tree changes before packaging")
    return run("git", "rev-parse", "HEAD").strip()


def supplemental_notices(root=Path(".")):
    records = json.loads((root / "docs/dependency-notices.json").read_text(encoding="utf-8"))
    for record in records:
        for file in record["files"]:
            actual = hashlib.sha256((root / file["path"]).read_bytes()).hexdigest()
            if actual != file["sha256"]:
                raise ValueError(f'Changed dependency notice: {file["path"]}')
    return {(r["name"], r["version"]): r for r in records}


def archive_tree(output, root, name, fmt):
    if fmt != "zip":
        return Path(shutil.make_archive(str(output / name), fmt, root, name))
    archive = output / (name + ".zip")
    # Cargo registry sources use reproducible epoch timestamps, predating ZIP's
    # 1980 limit. Clamp only archive metadata; retain exact license/asset bytes
    # and Unix executable modes (needed when macOS extracts the app bundle).
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED,
                         strict_timestamps=False) as zipped:
        for path in [root / name, *sorted((root / name).rglob("*"))]:
            zipped.write(path, path.relative_to(root))
    return archive


def notices(destination, packages):
    """Keep selected dependencies' original notice bytes and pinned supplements."""
    inventory = []
    supplements = supplemental_notices()
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
        supplement = supplements.get((package["name"], package["version"]))
        if supplement:
            supplement = dict(supplement, files=[])
            original = supplements[(package["name"], package["version"])]
            for file in original["files"]:
                # Many crates share one upstream notice. Keep it once, with an
                # inventory path relative to LICENSES (not a missing source path).
                relative = Path(file["path"]).relative_to("LICENSES")
                target = destination / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(file["path"], target)
                supplement["files"].append(dict(file, path=relative.as_posix()))
        if package["source"] and not copied and not supplement:
            raise ValueError(f"No dependency notices collected for {name}")
        inventory.append({"name": package["name"], "version": package["version"],
                          "license": package["license"], "source": package["source"],
                          "authors": package["authors"], "repository": package["repository"],
                          "supplement": supplement, "notices": copied})
    (destination / "dependency-inventory.json").write_text(
        json.dumps(inventory, indent=2) + "\n", encoding="utf-8")


def common(destination, info):
    copy_documentation(destination, info["commit"])
    guide = "testing" + profile_suffix(package_profile(None, info)) + ".md"
    # Keep the guide in docs/ so its relative links work offline. The package
    # root provides a short entry point instead of relocating the document.
    (destination / "TESTING.md").write_text(
        f"# Testing development builds\n\nStart with the [tester guide](docs/{guide}) "
        "for platform requirements, test steps and reporting instructions.\n",
        encoding="utf-8")
    (destination / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=["source", "binary"])
    parser.add_argument("--target")
    parser.add_argument("--profile", choices=PROFILES)
    parser.add_argument("--output", type=Path, default=Path(".scratch/dist"))
    args = parser.parse_args()
    if args.kind == "binary" and not args.target:
        parser.error("binary requires --target")
    source_info = (json.loads(Path("BUILD-INFO.json").read_text(encoding="utf-8"))
                   if Path("BUILD-INFO.json").is_file() else {})
    profile = package_profile(args.profile, source_info)
    commit = source_info["commit"] if source_info else source_commit()
    version = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]
    stem = package_stem(commit, profile)
    args.output.mkdir(parents=True, exist_ok=True)
    output = args.output.resolve()
    info = {"commit": commit, "version": version, "development": True, "profile": profile, "target": args.target,
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
            shutil.copy2(stage / "docs/rebuild.md", stage / "REBUILD.md")
            (stage / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
        else:
            common(stage, info)
            notices(stage / "LICENSES", production_packages(args.target))
            suffix = ".exe" if "windows" in args.target else ""
            binaries = Path(os.environ.get("CARGO_TARGET_DIR", "target")) / args.target / "release"
            shutil.copy2(binaries / ("polished-map" + suffix), stage / ("polished-map" + suffix))
            bundle = copy_gui(stage, binaries, args.target, profile, version)
            if "windows" in args.target:
                verify_windows_icons(stage / (PROFILES[profile][0] + ".exe"))
            if bundle is not None:
                subprocess.run(["codesign", "--force", "--sign", "-", str(bundle)], check=True)
                subprocess.run(["codesign", "--verify", "--deep", "--strict", str(bundle)], check=True)
            # Run the shipped CLI, not a separate debug artifact.
            subprocess.run([str(stage / ("polished-map" + suffix)), "--help"], check=True, stdout=subprocess.DEVNULL)
        fmt = "zip" if args.target and ("windows" in args.target or "apple" in args.target) else "gztar"
        archive = archive_tree(output, root, name, fmt)
        with archive.open("rb") as file:
            digest = hashlib.file_digest(file, "sha256").hexdigest()
        archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n", encoding="utf-8")
        print(f"Packaged {archive.name} ({archive.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
