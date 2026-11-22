#!/usr/bin/env python3
"""Build two native source bundles from the exact same Claude RPG project.

Starling's flat native package accepts only Cargo source, assets and data.
The editor and .bbmodel originals stay in Git, not in the installable archive.
The mixin bundle is source-only until a whole-game build can apply it.
The core bundle compiles on the flat/native Compile-v2 path but does not pretend
that game-function mixins are present: their skill nodes are purchase-locked.
"""
from __future__ import annotations

import pathlib
import re
import sys
import tomllib
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT = ROOT / "project"
RELEASES = ROOT / "releases"
FULL_NAME = "Claude-RPG-dev.hmod"
CORE_NAME = "Claude-RPG-core.hmod"
ROOT_ALLOWED = {
    "Cargo.toml", "Cargo.lock", "build.rs", "horizon-mod.toml",
    "layouts.json", "icon.png",
}
NATIVE_GROUP = re.compile(r"(?m)^\[\[native\.(?:bridge|mixin|module|state|injection)\]\]\s*$")


def accepted(name: str) -> bool:
    if name in ROOT_ALLOWED:
        return True
    if "/" not in name and name.startswith(("README", "LICENSE", "LICENCE", "COPYING", "NOTICE")):
        return True
    if name.startswith("src/") and name.endswith((".rs", ".toml")):
        return True
    if name.startswith(("assets/", "data/")):
        return True
    return False


def contents() -> dict[str, bytes]:
    result: dict[str, bytes] = {}
    for item in PROJECT.rglob("*"):
        if not item.is_file() or item.is_symlink():
            continue
        name = item.relative_to(PROJECT).as_posix()
        if (set(item.relative_to(PROJECT).parts) & {".git", ".horizon", "target", "__pycache__"}):
            continue
        if accepted(name):
            result[name] = item.read_bytes()
    if not {"horizon-mod.toml", "Cargo.toml", "src/lib.rs"} <= result.keys():
        raise SystemExit("Missing manifest, Cargo.toml, or src/lib.rs")
    return result


def prepare_core(full: dict[str, bytes]) -> dict[str, bytes]:
    core = full.copy()
    manifest = full["horizon-mod.toml"].decode("utf-8")
    chunks = NATIVE_GROUP.split(manifest, maxsplit=1)
    if len(chunks) != 2:
        raise SystemExit("Full source has no mixin/bridge definitions to strip")
    text = chunks[0].rstrip() + "\n"
    doc = tomllib.loads(text)
    if not doc["native"]["horizon"] or not doc["native"]["mixins"]:
        raise SystemExit("native engine/mixins pins missing")
    if any(k in doc["native"] for k in ("mixin", "bridge", "module", "injection", "state")):
        raise SystemExit("Core unexpectedly still has game patches")
    core["horizon-mod.toml"] = text.encode("utf-8")
    cargo = full["Cargo.toml"].decode("utf-8")
    flag = 'default = ["whole_game_mixins"]'
    if cargo.count(flag) != 1:
        raise SystemExit("The full source must default to whole_game_mixins")
    cargo = cargo.replace(flag, "default = []")
    if tomllib.loads(cargo)["features"]["default"] != []:
        raise SystemExit("Core did not disable the mixins feature")
    core["Cargo.toml"] = cargo.encode("utf-8")
    return core


def write(name: str, files: dict[str, bytes]) -> pathlib.Path:
    RELEASES.mkdir(exist_ok=True)
    dest = RELEASES / name
    with zipfile.ZipFile(dest, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=7) as z:
        for path in sorted(files):
            z.writestr(path, files[path])
    with zipfile.ZipFile(dest) as z:
        assert z.testzip() is None
        assert set(z.namelist()) == set(files)
        assert all(accepted(path) for path in z.namelist())
    return dest


def main() -> None:
    full = contents()
    native = tomllib.loads(full["horizon-mod.toml"].decode("utf-8"))
    if not native.get("native", {}).get("mixin"):
        raise SystemExit("Expected mixin entries in the full variant")
    core = prepare_core(full)
    full_dest = write(FULL_NAME, full)
    core_dest = write(CORE_NAME, core)
    print(f"Full source, requires whole-game mixins (NOT Compile v2): {full_dest}")
    print(f"API-only core, Compile v2 without mixin-only skills: {core_dest}")
    print(f"Bundled native files: {len(full)}")


if __name__ == "__main__":
    main()
