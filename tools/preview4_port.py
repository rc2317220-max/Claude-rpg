#!/usr/bin/env python3
"""Create a disposable Preview 4 candidate, never mutate project/ or releases/.
This is an initial mechanical port, not proof that all APIs are compatible."""
import argparse, pathlib, re, shutil, tomllib, zipfile
P=pathlib.Path
ap=argparse.ArgumentParser();ap.add_argument("--project",type=P,default=P("project"));ap.add_argument("--source",type=P,required=True);ap.add_argument("--out",type=P,required=True);a=ap.parse_args()
a.out.mkdir(parents=True,exist_ok=True)
stage=a.out/"project";shutil.copytree(a.project,stage,ignore=shutil.ignore_patterns("target",".horizon",".sunrise"),dirs_exist_ok=True)
pin="cac7bf64590fb38b4badee90f1f09848ce7f8daa"
manifest=stage/"horizon-mod.toml"; original=manifest.read_text()
s=re.sub(r'(?m)^horizon\s*=.*$',f'sunrise = "{pin}"',original)
s=re.sub(r'(?m)^mixins\s*=.*$\n?',"",s)
# sdk pin for native-route is not needed by this public-hook candidate; SDK builder reads native table.
manifest.unlink();(stage/"sunrise-mod.toml").write_text(s)
cargo=stage/"Cargo.toml";cargo.write_text(cargo.read_text().replace("horizon-mod-api","sunrise-mod-api").replace(".horizon/sdk/",".sunrise/sdk/"))
# Only known renamed proc macros. All other differences must be fixed from compiler diagnostics.
for f in (stage/"src").rglob("*.rs"):
    s=f.read_text()
    s=s.replace("mod_api::horizon_mod","mod_api::sunrise_mod").replace("mod_api::horizon_part","mod_api::sunrise_part")
    f.write_text(s)
for ns in (stage/"data").iterdir():
    h=ns/"horizon"
    if h.is_dir():h.rename(ns/"sunrise")
# Install lab instrumentation into staged files only. No source or release is overwritten.
tools=P(__file__).resolve().parent
lib=stage/"src/lib.rs"; text=lib.read_text()
anchor='for usage in ["rustpg", "rustpg open"'
assert text.count(anchor)==1,"command registration changed"
text=text.replace(anchor,'for usage in ["rustpg lab snapshot", "rustpg lab seed", "rustpg lab refill", "rustpg lab buy_fury", "rustpg lab buy_barrier", "rustpg lab buy_siphon", "rustpg", "rustpg open"')
anchor='            "help" => {'
# Only modify run_mod_command's match, not an unrelated help branch.
start=text.index("fn run_mod_command("); index=text.index(anchor,start)
text=text[:index]+(tools/"lab_fixture/commands.rs.inc").read_text()+text[index:]
lib.write_text(text)
guard=stage/"src/rustpg_guard.rs";guard.write_text(guard.read_text()+"\n"+(tools/"lab_fixture/guard.rs.inc").read_text())
# Mark the installable package as strictly a lab artifact, not a stable release.
package=a.out/"RustPG-preview4-LAB-ONLY.smod"
with zipfile.ZipFile(package,"w",zipfile.ZIP_DEFLATED) as z:
    for name in ("Cargo.toml","sunrise-mod.toml","src","data","assets"):
        p=stage/name
        if p.is_file():z.write(p,name)
        elif p.is_dir():
            for f in sorted(p.rglob("*")):
                if f.is_file():z.write(f,f.relative_to(stage))
# Desktop API compile from exact local source, not network SDK guessing.
cargo.write_text(cargo.read_text().replace(".sunrise/sdk/api/crates/sunrise-mod-api",str((a.source/"crates/sunrise-mod-api").resolve())))
print(package)
