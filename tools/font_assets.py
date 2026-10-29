"""Verify the unchanged, separately licensed menu-symbol font and its notice."""
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FILES = {
    "crates/gui/assets/fonts/NotoSansSymbols2-Regular.ttf": "7d5fb73b7ca67a6798101741f5d280a3d016a56a197afcd4199dbb57b4b82a21",
    "LICENSES/egui-fonts/NotoSansSymbols2-OFL.txt": "b118dd41337806a5d4797052c77caf3bd096aed783e5eb21b4d11154351e1ac0",
}

for name, expected in FILES.items():
    actual = hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
    if actual != expected:
        raise SystemExit(f"Original font/license hash mismatch: {name}")
print("Verified unchanged Noto menu-symbol font and OFL license")
