"""Build desktop icon containers from the original upstream cherry-tree ICO.

Uses the existing size-specific artwork without redrawing it. Pillow is needed
only for regeneration; normal CI verifies the source and generated hashes.
"""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'crates/gui/assets/artwork/app.ico'
DEST = ROOT / 'crates/gui/assets/application'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write():
    from PIL import Image
    DEST.mkdir(parents=True, exist_ok=True)
    with Image.open(SOURCE) as source:
        frames = {s: source.ico.getimage((s, s)).convert('RGBA') for s in [32, 64]}
    for size in [128, 256, 512, 1024]:
        frames[size] = frames[64].resize((size, size), Image.Resampling.NEAREST)
    frames[256].save(DEST/'app.png')
    (DEST/'window.rgba').write_bytes(frames[64].tobytes())
    frames[1024].save(DEST/'app.icns', append_images=list(frames.values()))
    hashes = {p.name: digest(p) for p in [DEST/'app.png', DEST/'app.icns', DEST/'window.rgba']}
    hashes['upstream_ico'] = digest(SOURCE)
    (DEST/'hashes.json').write_text(json.dumps(hashes, indent=2)+'\n', encoding='utf-8')


def verify():
    hashes = json.loads((DEST/'hashes.json').read_text(encoding='utf-8'))
    assert digest(SOURCE) == hashes.pop('upstream_ico'), 'upstream ICO changed'
    for name, expected in hashes.items():
        assert digest(DEST/name) == expected, name
    assert (DEST/'window.rgba').stat().st_size == 64*64*4
    print('Verified desktop containers derived from the original cherry-tree ICO.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    if args.write: write()
    verify()
