# RustPG Asset Studio
Independent of the Sunrise/Horizon runtime. Private repository; no public MCP ports, tokens, third-party asset upload or paid image API needed.

## Codespaces (existing session)
In the repository terminal, run `git pull --ff-only` (first commit/stash your own edits if necessary), then `bash scripts/asset_studio_setup.sh`. For newly created/rebuilt Codespaces the devcontainer runs the installer automatically. Requires Node >=23.6 for optional upstream WebGPU rendering; use Node 24. Blender and FFmpeg installed with apt; Pillow and numpy in isolated venv. Blockbench MCP source pinned to 97a8a80d3a37bf95c5e136d93d2c7da399fff753, GPL-3.0, installed separately in ignored .ai-tools; no upstream code copied into RustPG.

## Agents
Run `bash tools/asset_studio/bb.sh --list`. Local VS Code and project MCP config use stdio with this wrapper. Approve the MCP server in your own client. ChatGPT's GitHub connector cannot attach directly to Codespaces stdio: agents without terminal use GitHub Actions, with scripts committed to an appropriate branch and artifacts as evidence. Workflow does not expose an arbitrary public shell and never writes to the repo.

```bash
bash tools/asset_studio/bb.sh bbmodel_info '{"file":"creatures/ashen_sentinel.bbmodel"}'
bash tools/asset_studio/bb.sh bbmodel_validate '{"file":"creatures/ashen_sentinel.bbmodel","self_test":true}'
bash tools/asset_studio/bb.sh bbmodel_validate_animations '{"file":"creatures/ashen_sentinel.bbmodel"}'
bash tools/asset_studio/smoke.sh
```

Write edits using `bbmodel_edit` with the current `expected_revision`; create, add texture, set keyframe, export geometry and animations via tools listed in --list. Prefer a separate branch and new model file; do not overwrite current game assets without approval of its pipeline. Root is project/models with project/assets also accessible; path traversal is rejected by upstream. Keep the desktop HTTP plugin disabled unless separately authenticated; it is not required.

## 2D and previews
```bash
P=.ai-tools/asset-studio/venv/bin/python
$P tools/asset_studio/paint.py tools/asset_studio/example_texture.json .ai-tools/asset-studio/example.png
$P tools/asset_studio/export_mesh.py --model project/models/creatures/ashen_sentinel.bbmodel --output .ai-tools/asset-studio/mesh.json
blender -b -t 2 --python-exit-code 1 -P tools/asset_studio/blender_render.py -- .ai-tools/asset-studio/mesh.json project/assets/rustpg/textures/entity/ashen_sentinel.png .ai-tools/asset-studio/preview.png
```
Pillow recipe supports RGBA layers, opacity, rectangles, polygons, lines, ellipses, gradients; suitable for HUD sprites, icons, atlases and particles. Author atlas based on actual UV layout, not arbitrary repeated tiles. Existing CPU inspector and Blender bridge support single-texture cuboids, explicit non-rotated UVs and static authored pose; generated rigs can use --creature --clip --time, with legacy full-face UV preview only. Full upstream MCP supports additional mesh operations and animation sampling; its optional GPU render requires compatible WebGPU and is not assumed available. FFmpeg can assemble numbered frames into animation video; smoke video is a still, not proof of animated rendering. PBR and particle export formats do not imply Sunrise runtime supports them. Never label Blender/offline images as game screenshots.

GitHub Actions: RustPG Asset Studio, evidence includes tool list, model, PNG, animation validation, independent CPU render, Blender render, MP4 and diagnostic reports. No installation is declared successful until logs pass. Existing Codespace needs the above terminal command; this repo change does not execute inside that existing VM.
