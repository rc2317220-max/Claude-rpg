#!/usr/bin/env python3
"""Verify the complete scene matrix before publishing a UI lab capture."""
import argparse
import struct
from pathlib import Path
from xml.etree import ElementTree

SIZES = {"compact": (320, 180), "phone": (380, 240), "wide": (640, 360), "ultrawide": (820, 360)}
PAGES = "overview status trees bestiary profs equipment inventory enchant history attributes".split()
HUD = "normal low_hp exhausted target boss damage critical".split()


def dimensions(path):
    if path.suffix == ".svg":
        svg = ElementTree.parse(path).getroot()
        if svg.tag != "{http://www.w3.org/2000/svg}svg":
            raise ValueError(f"{path} is not an SVG")
        for image in svg.iter("{http://www.w3.org/2000/svg}image"):
            href = image.get("{http://www.w3.org/1999/xlink}href", image.get("href", ""))
            if not href.startswith("data:image/png;base64,"):
                raise ValueError(f"{path} links to an external or missing image")
        return float(svg.attrib["width"]), float(svg.attrib["height"])
    with path.open("rb") as source:
        header = source.read(24)
    if len(header) != 24 or header[:16] != b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR":
        raise ValueError(f"{path} has no PNG header")
    return struct.unpack(">II", header[16:24])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("format", choices=("svg", "png"))
    parser.add_argument("--publish", type=Path)
    args = parser.parse_args()
    scenes = PAGES + [f"hud-{variant}" for variant in HUD]
    expected = {f"{scene}-{size}.{args.format}": dims for size, dims in SIZES.items() for scene in scenes}
    files = {path.name for path in args.directory.glob(f"*.{args.format}")}
    if files != expected.keys():
        missing, extra = sorted(expected.keys() - files), sorted(files - expected.keys())
        parser.exit(1, f"Incomplete scene matrix in {args.directory}. Missing {missing}. Unexpected {extra}.\n")
    for name, size in expected.items():
        path = args.directory / name
        actual = dimensions(path)
        if actual != size:
            parser.exit(1, f"{path} has dimensions {actual}, expected {size}.\n")
    if args.publish:
        # Only replace our own scenes. A shared destination with foreign images
        # must not silently produce a misleading contact sheet.
        unexpected = {p.name for p in args.publish.glob(f"*.{args.format}")} - expected.keys()
        if unexpected:
            parser.exit(1, f"Unexpected images in {args.publish}: {sorted(unexpected)}\n")
        for name in expected:
            (args.directory / name).replace(args.publish / name)
    print(f"Verified {len(expected)} {args.format.upper()} captures in {args.publish or args.directory}")


if __name__ == "__main__":
    main()
