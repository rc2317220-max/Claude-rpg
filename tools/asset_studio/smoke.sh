#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
D="$PWD/.ai-tools/asset-studio"
PY="$D/venv/bin/python"
OUT="$PWD/.ai-tools/asset-studio/smoke"
rm -rf "$OUT"
mkdir -p "$OUT"
export RUSTPG_ASSET_ROOT="$OUT"
BB=tools/asset_studio/bb.sh
bash "$BB" --list > "$OUT/tools.txt"
bash "$BB" bbmodel_create '{"file":"studio-test.bbmodel","format":"free","name":"Studio Test","box_uv":true,"resolution":{"width":64,"height":64}}' > "$OUT/create.json"
bash "$BB" bbmodel_edit '{"file":"studio-test.bbmodel","operations":[{"op":"add_group","name":"root","origin":[0,0,0]},{"op":"add_cube","name":"body","parent":"root","from":[-4,0,-4],"to":[4,12,4],"box_uv":true,"uv_offset":[0,0]}]}' > "$OUT/edit.json"
"$PY" tools/asset_studio/paint.py tools/asset_studio/example_texture.json "$OUT/texture.png"
bash "$BB" bbmodel_add_texture '{"file":"studio-test.bbmodel","image":"texture.png","assign_to":["body"]}' > "$OUT/texture-result.json"
bash "$BB" bbmodel_info '{"file":"studio-test.bbmodel"}' > "$OUT/info.json"
bash "$BB" bbmodel_validate '{"file":"studio-test.bbmodel"}' > "$OUT/validate.json"
bash "$BB" bbmodel_edit '{"file":"studio-test.bbmodel","operations":[{"op":"add_animation","name":"idle","length":1,"loop":"loop","snapping":20},{"op":"set_keyframe","animation":"idle","bone":"root","channel":"rotation","time":0,"value":[0,0,0],"interpolation":"linear"},{"op":"set_keyframe","animation":"idle","bone":"root","channel":"rotation","time":0.5,"value":[0,15,0],"interpolation":"linear"},{"op":"set_keyframe","animation":"idle","bone":"root","channel":"rotation","time":1,"value":[0,0,0],"interpolation":"linear"}]}' > "$OUT/animation-edit.json"
bash "$BB" bbmodel_validate_animations '{"file":"studio-test.bbmodel"}' > "$OUT/animation-validation.json"
"$PY" tools/asset_studio/export_mesh.py --model "$OUT/studio-test.bbmodel" --output "$OUT/mesh.json"
"$PY" tools/creature_inspector.py --model "$OUT/studio-test.bbmodel" --texture "$OUT/texture.png" --output "$OUT/cpu"
blender -b -t 2 --python-exit-code 1 -P tools/asset_studio/blender_render.py -- "$OUT/mesh.json" "$OUT/texture.png" "$OUT/blender.png" > "$OUT/blender.log" 2>&1
ffmpeg -hide_banner -loglevel error -y -loop 1 -i "$OUT/blender.png" -t 2 -r 24 -pix_fmt yuv420p "$OUT/preview.mp4"
"$PY" - "$OUT" <<'PY'
from pathlib import Path
from PIL import Image
import json,sys
p=Path(sys.argv[1]);m=json.loads((p/"studio-test.bbmodel").read_text())
assert len(m["elements"])==1 and len(m["textures"])==1 and len(m["animations"])==1
im=Image.open(p/"blender.png").convert("RGB")
assert im.size==(512,512) and len(im.getcolors(262145))>16
assert (p/"preview.mp4").stat().st_size>1000
print("Smoke test: created model, embedded PNG, animation, independent CPU and Blender renders, video.")
PY
# Preserve diagnostic reports from the actual project, without modifying its models.
unset RUSTPG_ASSET_ROOT
bash "$BB" bbmodel_info '{"file":"creatures/ashen_sentinel.bbmodel"}' > "$OUT/ashen-sentinel-info.json"
bash "$BB" bbmodel_validate '{"file":"creatures/ashen_sentinel.bbmodel"}' > "$OUT/ashen-sentinel-validate.json"
bash "$BB" bbmodel_validate_animations '{"file":"creatures/ashen_sentinel.bbmodel"}' > "$OUT/ashen-sentinel-animations.json"
