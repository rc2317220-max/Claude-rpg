#!/usr/bin/env python3
"""Create a portable before/after viewer from two extracted visual-lab runs."""
import argparse
import base64
import json
import struct
from pathlib import Path

from check_ui_capture import HUD, PAGES, SIZES


def png_uri(path, expected=None):
    raw = path.read_bytes()
    if raw[:16] != b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR":
        raise ValueError(f"Invalid capture: {path}")
    size = struct.unpack(">II", raw[16:24])
    if expected and size != expected:
        raise ValueError(f"{path} has size {size}, expected {expected}")
    return "data:image/png;base64," + base64.b64encode(raw).decode("ascii")


def captures(root):
    result = {}
    for size, dimensions in SIZES.items():
        for scene in PAGES + [f"hud-{v}" for v in HUD]:
            name = f"{scene}-{size}"
            result[f"UI / {size} / {scene}"] = png_uri(root / "ui-png" / f"{name}.png", dimensions)
    for path in sorted((root / "rustpg-entities").glob("*/*.png")):
        result[f"3D / {path.parent.name} / {path.stem}"] = png_uri(path)
    return result


VIEW = """<!doctype html><html lang="pt-BR"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>RustPG · Revisão visual</title>
<style>
*{box-sizing:border-box}body{margin:auto;max-width:1600px;padding:clamp(14px,3vw,46px);background:#101818;color:#f3eedc;font:16px/1.4 system-ui,sans-serif}
header{display:flex;align-items:end;justify-content:space-between;flex-wrap:wrap;gap:18px;border-bottom:1px solid #536963;padding-bottom:18px}
h1{font:bolder clamp(24px,4vw,40px)/1.1 Georgia,serif;margin:0}label,figcaption{color:#dcc89d}select{display:block;max-width:100%;margin-top:4px;padding:6px;background:#253532;color:#fff;border:1px solid #8b9c91;border-radius:4px;font:inherit}
main{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,365px),1fr));gap:18px}figure{margin:18px 0 0}figcaption{margin-bottom:8px;overflow-wrap:anywhere}img{display:block;width:100%;image-rendering:pixelated;background:#192629}figure.mob img{max-width:480px}
</style><header><h1>RustPG · Revisão visual</h1><label>Cena<select id="scene"></select></label></header>
<main><figure><figcaption id="old-title"></figcaption><img id="old" alt="Captura anterior"></figure>
<figure><figcaption id="new-title"></figcaption><img id="new" alt="Captura atual"></figure></main>
<script type="application/json" id="captures">DATA_SLOT</script>
<script>
const data=JSON.parse(document.getElementById("captures").textContent);
const menu=document.getElementById("scene");
const keys=Object.keys(data.pairs);
for(const key of keys) menu.add(new Option(key,key));
document.getElementById("old-title").textContent=data.labels[0];
document.getElementById("new-title").textContent=data.labels[1];
function draw(){
  const key=menu.value, pair=data.pairs[key];
  for(const [i,id] of ["old","new"].entries()){
    const img=document.getElementById(id);
    img.hidden=!pair[i];
    if(pair[i]) img.src=pair[i]; else img.removeAttribute("src");
    img.parentElement.className=key.startsWith("3D")?"mob":"";
    document.getElementById(id+"-title").textContent=data.labels[i]+(pair[i]?"":" · Sem captura nesta rodada");
  }
}
menu.value=keys.includes("UI / phone / overview")?"UI / phone / overview":keys[0];
menu.addEventListener("change",draw);draw();
</script></html>"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("previous", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--labels", nargs=2, default=["Anterior", "Atual"])
    args = parser.parse_args()
    previous, current = captures(args.previous), captures(args.current)
    pairs = {key: (previous.get(key), current.get(key)) for key in dict.fromkeys([*current, *previous])}
    data = json.dumps({"labels": args.labels, "pairs": pairs}, separators=(",", ":")).replace("<", "\\u003c")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(VIEW.replace("DATA_SLOT", data), encoding="utf-8")
    print(f"{len(pairs)} pairs in {args.output}")


if __name__ == "__main__":
    main()
