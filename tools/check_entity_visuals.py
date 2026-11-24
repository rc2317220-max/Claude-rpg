#!/usr/bin/env python3
"""Validate a full Horizon entity capture and measure silhouette and lighting."""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from remaster_handcrafted_creatures import decode


def measure(path: Path, require_transparency: bool = False):
    width, height, color, rows = decode(path.read_bytes())
    if not width or width != height:
        raise ValueError(f"expected a square frame, got {width}x{height}")
    stride = 4 if color == 6 else 3
    if require_transparency:
        if color != 6 or any(row[index] != 0 for row in (rows[0], rows[-1]) for index in (3, -1)):
            raise ValueError("expected transparent margins for the UI portrait")
    opaque_clear = color == 2 or rows[0][:4] == bytes((0, 0, 0, 255))
    count = 0
    total = 0.0
    histogram = [0] * 256
    left, top, right, bottom = width, height, 0, 0
    for y, row in enumerate(rows):
        for x in range(width):
            at = x * stride
            r, g, b = row[at:at + 3]
            # Older Vulkan captures cleared to opaque black, even in RGBA.
            if (color == 6 and row[at + 3] <= 4) or (opaque_clear and r + g + b <= 6):
                continue
            luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b
            histogram[round(luminance)] += 1
            total += luminance
            count += 1
            left, top = min(left, x), min(top, y)
            right, bottom = max(right, x + 1), max(bottom, y + 1)
    if not count:
        raise ValueError("empty silhouette")
    cumulative = 0
    low = None
    high = 0
    for value, frequency in enumerate(histogram):
        cumulative += frequency
        if low is None and cumulative >= count * 0.1:
            low = value
        if cumulative >= count * 0.9:
            high = value
            break
    return {
        "luminance": round(total / count, 2),
        "luminance_p10_p90": [low, high],
        "coverage": round(count / (width * height), 4),
        "bbox": [left, top, right, bottom],
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("root", type=Path)
    ap.add_argument("--output", type=Path)
    ap.add_argument("--strict", action="store_true", help="also fail on low luminance or coverage")
    args = ap.parse_args()
    registered = {p.stem for p in (ROOT / "project/data/rustpg/horizon/entity_type").glob("*.json")}
    manifest = json.loads((args.root / "manifest.json").read_text(encoding="utf-8"))
    ids = [entry["id"] for entry in manifest["creatures"]]
    if not registered or len(ids) != len(set(ids)) or set(ids) != registered:
        raise SystemExit(f"Capture roster differs from the entity pack: expected {sorted(registered)}, got {ids}")
    poses = [f"{pose}-{view}" for pose in ("idle", "move") for view in ("front", "three_quarter", "side")]
    poses.append("action-three_quarter")
    report, warnings = {}, []
    for creature in sorted(registered):
        shots = {}
        for pose in poses:
            png = args.root / creature / f"{pose}.png"
            if not png.is_file():
                raise SystemExit(f"Missing entity capture: {png}")
            try:
                frame = measure(png, require_transparency=manifest.get("background") == "transparent")
            except (ValueError, IndexError) as exc:
                raise SystemExit(f"Invalid entity capture {png}: {exc}") from exc
            shots[pose] = frame
            if frame["coverage"] < 0.035 or frame["luminance"] < 28.0:
                warnings.append(f"{creature}/{png.name}: lum={frame['luminance']} cov={frame['coverage']}")
        report[creature] = shots
    text = json.dumps(report, indent=2, ensure_ascii=False)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(text + "\n", encoding="utf-8")
    print(text)
    if warnings:
        print("\nVisual QA warnings:", *warnings, sep="\n  ", file=sys.stderr)
        if args.strict:
            raise SystemExit(2)


if __name__ == "__main__":
    main()
