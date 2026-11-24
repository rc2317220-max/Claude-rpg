"""Regression: actual pixel occupancy, unpainted faces and emissive alpha."""
import tempfile
from pathlib import Path
from PIL import Image
from studio_v2 import uv_check
def test():
    with tempfile.TemporaryDirectory() as tmp:
        out=Path(tmp);d={"resolution":{"width":8,"height":8},"elements":[{"name":"cube","from":[0,0,0],"to":[2,2,2],"faces":{"north":{"uv":[1,1,3,3],"texture":0}}}]}
        tex=Image.new("RGBA",(8,8));tex.paste((80,40,20,255),(1,1,3,3));glow=Image.new("RGBA",(8,8));glow.putpixel((1,1),(255,100,0,255))
        r=uv_check(d,tex,glow,out);assert r["faces"]==1 and r["occupied_fraction"]==4/64 and r["emissive_fraction"]==.25
        glow.putpixel((7,7),(255,0,0,255))
        try:uv_check(d,tex,glow,out)
        except ValueError as e:assert "emissive outside" in str(e)
        else:raise AssertionError("bad emissive passed")
        glow.putpixel((7,7),(0,0,0,0));d["elements"].append(dict(d["elements"][0],name="second"))
        try:uv_check(d,tex,glow,out)
        except ValueError as e:assert "overlapping" in str(e)
        else:raise AssertionError("overlap passed")
if __name__=="__main__":test()
