#!/usr/bin/env bash
set -euo pipefail
# Find the real X11 windows, tile them, and check geometry instead of assuming a resize worked.
for _ in $(seq 1 60); do
  bb=$(wmctrl -l -x | awk 'tolower($3) ~ /blockbench/ {print $1;exit}')
  ref=$(wmctrl -l | awk '/RustPG Reference Studio/ {print $1;exit}')
  if [[ -n "$bb" && -n "$ref" ]]; then break; fi
  sleep .25
done
[[ -n "${bb:-}" && -n "${ref:-}" ]] || { wmctrl -l -x; exit 1; }
for id in "$bb" "$ref"; do
  wmctrl -i -r "$id" -b remove,maximized_vert,maximized_horz
  wmctrl -i -r "$id" -b remove,fullscreen
done
wmctrl -i -r "$bb" -e 0,0,0,1490,1150
wmctrl -i -r "$ref" -e 0,1500,0,880,1150
sleep 1
wmctrl -lG > "$BB_OUT/window-layout.txt"
python3 - "$bb" "$ref" "$BB_OUT/window-layout.txt" <<'PY'
import sys
b,r,p=sys.argv[1:]
rows={v[0].lower():[int(x) for x in v[2:6]] for line in open(p) if len(v:=line.split())>=6}
bx,by,bw,bh=rows[b.lower()]
rx,ry,rw,rh=rows[r.lower()]
assert bw>=1250 and rw>=700 and bh>=900 and rh>=900, rows
assert bx+bw<=rx+15 and rx+rw<=2410, rows
PY
