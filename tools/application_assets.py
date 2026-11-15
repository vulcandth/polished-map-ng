"""Package the maintainer-supplied single-tree artwork for desktop applications.

Regeneration requires Pillow 11.2.1. Normal verification is standard-library-only.
Original attachments remain byte-for-byte intact; scaling uses nearest-neighbor
pixels, never repetition/tiling of the complete image.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'crates/gui/assets/application'
SOURCES = {
    32: ('cherry-tree-32.png', '2c0a1476fe6e23d8be72eb7bb9f0c099b1fa609727fd267e7628fb272c7290cf'),
    48: ('cherry-tree-48.png', '27b41271b2f8eb3b6ec8800b0cf0b21c8f6840902910f379c97fa86f82b3b10f'),
}
ICON_SIZES = (16, 24, 32, 48, 64, 128, 256)
OUTPUTS = {'app.ico', 'app.icns', 'app.png', 'window.rgba', 'about.rgba'}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def frames():
    from PIL import Image
    originals = {size: Image.open(DEST / name).convert('RGBA') for size, (name, _) in SOURCES.items()}
    result = {}
    for size in sorted(set(ICON_SIZES) | {512, 1024}):
        # Preserve both supplied sizes exactly. Integer resampling gives crisp
        # 16/24/64 icons; larger display sizes use the more detailed 48px source.
        source = originals[32 if size in (16, 32, 64) else 48]
        result[size] = source.resize((size, size), Image.Resampling.NEAREST)
    return result


def write():
    images = frames()
    images[256].save(DEST / 'app.ico', sizes=[(s, s) for s in ICON_SIZES],
                     append_images=[images[s] for s in ICON_SIZES])
    # Pillow leaves PNG-frame color planes unspecified (zero). Windows rc.exe
    # records one plane in RT_GROUP_ICON; write that explicitly so the ICO and
    # executable resource metadata match as well as their image payloads.
    ico = bytearray((DEST / 'app.ico').read_bytes())
    for index in range(len(ICON_SIZES)):
        struct.pack_into('<H', ico, 6 + index * 16 + 4, 1)
    (DEST / 'app.ico').write_bytes(ico)
    images[1024].save(DEST / 'app.icns', append_images=list(images.values()))
    images[256].save(DEST / 'app.png')
    (DEST / 'window.rgba').write_bytes(images[64].tobytes())
    (DEST / 'about.rgba').write_bytes(images[48].tobytes())
    # Check the container writer actually used every supplied frame rather than
    # silently regenerating small representations from its largest image.
    from PIL import Image
    with Image.open(DEST / 'app.ico') as icon:
        assert icon.ico.sizes() == {(s, s) for s in ICON_SIZES}
        for size in ICON_SIZES:
            assert icon.ico.getimage((size, size)).convert('RGBA').tobytes() == images[size].tobytes()
    hashes = {name: digest(DEST / name) for name in sorted(OUTPUTS)}
    (DEST / 'hashes.json').write_text(json.dumps(hashes, indent=2) + '\n', encoding='utf-8', newline='\n')


def verify():
    for name, expected in SOURCES.values():
        assert digest(DEST / name) == expected, f'Supplied artwork changed: {name}'
    hashes = json.loads((DEST / 'hashes.json').read_text(encoding='utf-8'))
    assert hashes.keys() == OUTPUTS, 'Incomplete application icon inventory'
    for name, expected in hashes.items():
        assert digest(DEST / name) == expected, name
    assert (DEST / 'window.rgba').stat().st_size == 64 * 64 * 4
    assert (DEST / 'about.rgba').stat().st_size == 48 * 48 * 4
    print('Verified application assets derived from the two supplied single-tree images.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    if args.write:
        write()
    verify()
