"""Verify the pinned native-adapter source trees and recorded patches.

Use --archives DIR to also compare pristine file hashes against the original
Cargo .crate archives. Ordinary verification is offline and needs only this repo.
"""
import argparse
import difflib
import hashlib
import json
from pathlib import Path
import tarfile

ROOT = Path(__file__).resolve().parents[1]
VENDOR = ROOT / "vendor"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archives", type=Path)
    parser.add_argument("--refresh", action="store_true", help="Regenerate reviewed patches/hashes from the supplied original archives")
    args = parser.parse_args()
    if args.refresh and not args.archives:
        parser.error("--refresh requires --archives")
    manifest = json.loads((VENDOR / "sources.json").read_text(encoding="utf-8"))
    assert manifest["format"] == 1
    count = 0
    for crate in manifest["crates"]:
        folder = VENDOR / crate["directory"]
        actual = {p.relative_to(folder).as_posix() for p in folder.rglob("*") if p.is_file()}
        original = None
        if args.archives:
            archive = args.archives / f'{crate["directory"]}.crate'
            assert digest(archive.read_bytes()) == crate["archive_sha256"]
            with tarfile.open(archive, "r:gz") as tar:
                original = {member.name.removeprefix(crate["directory"] + "/"): tar.extractfile(member).read()
                            for member in tar.getmembers() if member.isfile()}
            assert set(original) <= actual, "Pristine source files must not be removed"
            patches = []
            records = {}
            for name in sorted(actual):
                data = (folder / name).read_bytes()
                before = original.get(name)
                records[name] = {"sha256": digest(data), "original_sha256": digest(before) if before is not None else None}
                if data != before:
                    patches.extend(difflib.unified_diff((before or b"").decode("utf-8").splitlines(keepends=True),
                        data.decode("utf-8").splitlines(keepends=True), fromfile=f'a/{crate["directory"]}/{name}', tofile=f'b/{crate["directory"]}/{name}'))
            patch = "".join(patches).encode("utf-8")
            if args.refresh:
                crate["files"] = records
                (VENDOR / crate["patch"]).write_bytes(patch)
                crate["patch_sha256"] = digest(patch)
            else:
                assert records == crate["files"], "Source/patch records differ from the pristine archive"
                assert patch == (VENDOR / crate["patch"]).read_bytes(), "Patch does not describe these source changes"
        # A build must never write into these vendored source trees.
        assert actual == set(crate["files"]), f"{folder}: missing/unrecorded files"
        assert digest((VENDOR / crate["patch"]).read_bytes()) == crate["patch_sha256"]
        for name, record in crate["files"].items():
            assert digest((folder / name).read_bytes()) == record["sha256"], f"Changed vendor source: {folder / name}"
            count += 1
    if args.refresh:
        (VENDOR / "sources.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    adaptations_path = VENDOR / "adaptations.json"
    adaptations = json.loads(adaptations_path.read_text(encoding="utf-8"))
    assert adaptations["format"] == 1
    for source in adaptations["sources"]:
        data = (VENDOR / source["path"]).read_bytes()
        license_data = (VENDOR / source["license_path"]).read_bytes()
        patch_path = VENDOR / source["patch"]
        if args.archives:
            archive = args.archives / source["archive"]
            assert digest(archive.read_bytes()) == source["archive_sha256"]
            with tarfile.open(archive, "r:gz") as tar:
                before = tar.extractfile(source["original_path"]).read()
                license_before = tar.extractfile(source["original_license_path"]).read()
            assert digest(before) == source["original_sha256"]
            assert license_data == license_before, "Original license must be retained verbatim"
            patch = "".join(difflib.unified_diff(before.decode("utf-8").splitlines(keepends=True),
                data.decode("utf-8").splitlines(keepends=True),
                fromfile="a/" + source["original_path"], tofile="b/" + source["path"])).encode("utf-8")
            if args.refresh:
                source["sha256"] = digest(data)
                source["patch_sha256"] = digest(patch)
                patch_path.write_bytes(patch)
            else:
                assert patch == patch_path.read_bytes(), "Adaptation patch differs from original source"
        assert digest(data) == source["sha256"]
        assert digest(license_data) == source["license_sha256"]
        assert digest(patch_path.read_bytes()) == source["patch_sha256"]
    if args.refresh:
        adaptations_path.write_text(json.dumps(adaptations, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"Verified {len(manifest['crates'])} pinned native adapters, {count} files and their patches")
    print(f"Verified {len(adaptations['sources'])} separately licensed source adaptations")


if __name__ == "__main__":
    main()
