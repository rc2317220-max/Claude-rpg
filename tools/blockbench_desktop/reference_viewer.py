#!/usr/bin/env python3
"""Local, non-networked X11 image viewer. Never executes reference content."""
import hashlib
import json
import os
import sys
import tkinter as tk
from pathlib import Path
from PIL import Image, ImageTk, ImageOps

source, out = map(Path, sys.argv[1:3])
Image.MAX_IMAGE_PIXELS = 40_000_000
if source.stat().st_size > 25 * 1024 * 1024:
    raise ValueError("reference exceeds 25 MiB")
with Image.open(source) as probe:
    if probe.format not in {"PNG", "JPEG", "WEBP"}:
        raise ValueError("reference must be PNG, JPEG or WEBP")
    if probe.width * probe.height > 40_000_000:
        raise ValueError("reference exceeds 40 megapixels")
    probe.verify()
with Image.open(source) as original:
    img = ImageOps.exif_transpose(original).convert("RGB")
out.mkdir(parents=True, exist_ok=True)
img.save(out / "reference-source.png")
state = out / "reference-state.json"
ack = out / "reference-ack.json"
def write_atomic(p, obj):
    tmp = p.with_suffix(".tmp")
    tmp.write_text(json.dumps(obj), encoding="utf-8")
    os.replace(tmp, p)
write_atomic(out / "reference-source.json", {
    "source": source.name, "sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
    "width": img.width, "height": img.height
})
root = tk.Tk()
root.title("RustPG Reference Studio")
root.geometry("880x1140+1510+15")
root.configure(bg="#20252d")
title = tk.Label(root, text="REFERENCE | full image", font=("DejaVu Sans", 15, "bold"), fg="#e7e9ee", bg="#20252d")
title.pack(pady=12)
canvas = tk.Label(root, bg="#20252d")
canvas.pack(fill="both", expand=True, padx=10, pady=10)
last = None
def refresh():
    global last
    try:
        cfg = json.loads(state.read_text()) if state.exists() else {"id":"initial","label":"Full reference","crop":[0,0,1,1]}
        size = (max(100, canvas.winfo_width()), max(100, canvas.winfo_height()))
        key = (json.dumps(cfg, sort_keys=True), size)
        if key != last:
            x0,y0,x1,y1 = cfg.get("crop", [0,0,1,1])
            if not (0 <= x0 < x1 <= 1 and 0 <= y0 < y1 <= 1):
                raise ValueError("invalid crop")
            region = img.crop((round(x0*img.width),round(y0*img.height),round(x1*img.width),round(y1*img.height)))
            region = ImageOps.contain(region, size, Image.Resampling.LANCZOS)
            pic = ImageTk.PhotoImage(region)
            canvas.configure(image=pic)
            canvas.image = pic
            title.configure(text="REFERENCE | " + str(cfg.get("label","Full reference"))[:60])
            root.update_idletasks()
            write_atomic(ack, {"id":cfg["id"], "crop":[x0,y0,x1,y1],"rendered_size":list(region.size)})
            last = key
    except Exception as e:
        write_atomic(out / "reference-viewer-error.json", {"error":str(e)})
    root.after(150,refresh)
root.after(250,refresh)
root.mainloop()
