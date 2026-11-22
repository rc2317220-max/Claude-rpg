#!/usr/bin/env python3
"""Source migration to RustPG (brand, identifiers, resources, and mobile inspector).

Run from repository root. Requires Python 3.11+. Original folders are intentionally
renamed; existing saves, registry keys, network IDs are not kept backwards-compatible.
"""
from __future__ import annotations
import base64
import json
import pathlib
import re
import shutil
import sys
import tomllib

ROOT = pathlib.Path(sys.argv[1]).resolve() if len(sys.argv)>1 else pathlib.Path.cwd()
PROJECT = ROOT / 'project' if (ROOT / 'project').is_dir() else ROOT
SRC = PROJECT / 'src'

def update(path: pathlib.Path, old: str, new: str):
    text = path.read_text(encoding='utf-8')
    if old not in text: raise RuntimeError(f'{path}: missing expected pattern {old[:120]}')
    path.write_text(text.replace(old, new), encoding='utf-8')

def edit_inspector():
    p = SRC / 'crpg_inspector.rs'
    s = p.read_text()
    old = '''const GAP:f32=7.0;
pub fn main_area(full:Rect)->Rect {
    if full.w < 455.0 {return full;}
    let sidebar=(full.w*0.305).clamp(152.0,280.0);
    Rect{x:full.x,y:full.y,w:full.w-sidebar-GAP,h:full.h}
}
pub fn inspector_area(full:Rect)->Option<Rect>{
    if full.w<455.0 {return None;}
    let main=main_area(full);
    Some(Rect{x:main.x+main.w+GAP,y:full.y,w:(full.x+full.w-main.x-main.w-GAP).max(0.0),h:full.h})
}'''
    new = '''const GAP:f32=7.0;
// The inspector is part of the real touch layout, not just a desktop hover.
// In narrow screens it becomes a compact bottom dock. The same geometries are
// used for drawing, scrolling and pointer handling.
fn mobile_dock_height(full:Rect)->f32 {
    (full.h*0.31).clamp(76.0,158.0).min((full.h-100.0).max(0.0))
}
pub fn main_area(full:Rect)->Rect {
    if full.w < 455.0 {
        let dock=mobile_dock_height(full);
        return Rect{x:full.x,y:full.y,w:full.w,h:(full.h-dock-GAP).max(25.0)};
    }
    let sidebar=(full.w*0.305).clamp(152.0,280.0);
    Rect{x:full.x,y:full.y,w:full.w-sidebar-GAP,h:full.h}
}
pub fn inspector_area(full:Rect)->Option<Rect>{
    if full.w<455.0 {
        let dock=mobile_dock_height(full);
        return (dock>=55.0).then_some(Rect{
            x:full.x,y:full.y+full.h-dock,w:full.w,h:dock});
    }
    let main=main_area(full);
    Some(Rect{x:main.x+main.w+GAP,y:full.y,w:(full.x+full.w-main.x-main.w-GAP).max(0.0),h:full.h})
}'''
    if old not in s:raise RuntimeError('inspector geometry changed; cannot safely edit')
    s=s.replace(old,new)
    old='''    let preview_space=if info.entity_kind.is_some() && rect.h>=240.0 {103.0}
        else if info.key.starts_with("node:") {27.0} else {0.0};
    let body=Rect{x:rect.x+10.0,y:after_title+9.0,w:rect.w-20.0,
        h:(rect.y+rect.h-20.0-preview_space-(after_title+9.0)).max(10.0)};'''
    new='''    // A mobile bestiary preview is placed beside the text instead of below it.
    let mobile_preview=info.entity_kind.is_some() && rect.w<455.0 && rect.w>=185.0;
    let preview_side=if mobile_preview {(rect.w*0.28).clamp(65.0,105.0)} else {0.0};
    let preview_space=if mobile_preview {0.0}
        else if info.entity_kind.is_some() && rect.h>=240.0 {103.0}
        else if info.key.starts_with("node:") {27.0} else {0.0};
    let body=Rect{x:rect.x+10.0,y:after_title+9.0,w:rect.w-20.0-preview_side,
        h:(rect.y+rect.h-20.0-preview_space-(after_title+9.0)).max(10.0)};'''
    if old not in s:raise RuntimeError('body geometry changed')
    s=s.replace(old,new)
    old='''    let preview_rect=if info.entity_kind.is_some() && preview_space>0.0 {
        let x0=rect.x+16.0;let y0=rect.y+rect.h-112.0;'''
    new='''    let preview_rect=if mobile_preview {
        let x0=rect.x+rect.w-preview_side-4.0;
        let y0=(after_title+6.0).min(rect.y+rect.h-52.0);
        let ph=(rect.y+rect.h-y0-12.0).max(24.0);
        l.rect(Rect{x:x0,y:y0,w:preview_side-7.0,h:ph},0x780C1C2A);
        l.border(Rect{x:x0,y:y0,w:preview_side-7.0,h:ph},0x6659A2BC);
        Some([x0+2.0,y0+2.0,x0+preview_side-10.0,y0+ph-2.0])
    } else if info.entity_kind.is_some() && preview_space>0.0 {
        let x0=rect.x+16.0;let y0=rect.y+rect.h-112.0;'''
    if old not in s:raise RuntimeError('preview block changed')
    s=s.replace(old,new)
    s=s.replace('''        assert!(inspector_area(Rect{x:4.0,y:4.0,w:380.0,h:240.0}).is_none());''','''        let phone=Rect{x:4.0,y:4.0,w:380.0,h:240.0};
        let main=main_area(phone);
        let dock=inspector_area(phone).expect("mobile inspector must be visible");
        assert!(main.y+main.h<dock.y);
        assert!(dock.h>=70.0);
        assert_eq!(dock.w,phone.w);''')
    marker='''    #[test] fn contextual_fallback_for_every_page(){'''
    test='''    #[test] fn bestiary_description_typewriter_on_mobile(){
        use super::super::crpg_ui_window::SpeciesRow;
        let area=Rect{x:4.0,y:4.0,w:380.0,h:310.0};
        let m=MenuModel{kind:"bestiary",bestiary:vec![SpeciesRow{
            name:"minecraft:zombie".into(),kills:3,level:4,best:7.5,
            tiers:"Normal".into(),drops:"minecraft:rotten_flesh".into(),
            discovery:0.02}],..Default::default()};
        let selected=inspect(&m);
        assert!(selected.key.starts_with("species:"));
        assert!(selected.paragraphs.iter().any(|p|p.contains("Abates registrados: 3")));
        let rect=inspector_area(area).unwrap();
        let early=presentation(rect,&selected,0,0);
        let late=presentation(rect,&selected,0,1000);
        assert!(late.preview_rect.is_some());
        let count=|p:&Presentation|p.layer.prims.iter().filter_map(|prim|match prim{
            super::super::crpg_ui_system::Prim::Text{text,..}=>Some(text.chars().count()),
            _=>None}).sum::<usize>();
        assert!(count(&late)>count(&early),"paragraphs must be typed over time");
    }
'''
    if marker not in s:raise RuntimeError('test insert location missing')
    s=s.replace(marker,test+marker)
    p.write_text(s)

def edit_brand_header():
    p=SRC/'crpg_ui_window.rs';s=p.read_text()
    old='''    l.text("// Rust Pg", h.x + 76.0, h.y + 1.0, alpha(MUTED, 0x70));'''
    if old not in s:raise RuntimeError('header branding not found')
    new='''    // Native sprite instead of a single-size font: small "Rust" + big "PG".
    l.sprite("claude_rpg:gui/branding/rustpg_logo",[0.0,0.0,384.0,110.0],
        Rect{x:h.x+75.0,y:h.y-5.0,w:64.0,h:18.3});'''
    s=s.replace(old,new)
    p.write_text(s)

def make_logo():
    from PIL import Image,ImageDraw,ImageFont
    im=Image.new('RGBA',(384,110),(0,0,0,0));d=ImageDraw.Draw(im)
    font='/usr/share/fonts/truetype/dejavu/DejaVuSansCondensed-Bold.ttf'
    if not pathlib.Path(font).exists(): font='/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf'
    small=ImageFont.truetype(font,43);large=ImageFont.truetype(font,91)
    # Make every letter aligned to the same baseline, not just same top.
    d.text((7,48),'Rust',font=small,fill=(104,218,242,255),stroke_width=1,stroke_fill=(5,22,38,240))
    w=d.textbbox((0,0),'Rust',font=small)[2]
    d.text((w+14,9),'PG',font=large,fill=(244,250,255,255),stroke_width=2,stroke_fill=(8,32,52,255))
    p=PROJECT/'assets/claude_rpg/textures/gui/branding/rustpg_logo.png'
    p.parent.mkdir(parents=True,exist_ok=True);im.save(p)
    return p

def normalize_identifiers():
    mapping=[
       ('ClaudeRpg','RustPGMod'),('CLAUDE_RPG','RUSTPG'),
       ('Claude RPG','RustPG'),('Claude-RPG','RustPG'),('Claude_RPG','RustPG'),('CLAUDE','RUSTPG'),
       ('claude_rpg','rustpg'),('claude-rpg','rustpg'),
       ('Rust Pg','RustPG'),('Rust-Pg','RustPG'),('rust_pg','rustpg'),('rust-pg','rustpg'),
       ('crpg','rustpg'),('Crpg','RustPG'),('CRPG','RUSTPG')
    ]
    # Includes standalone Rust/Python, translations, data JSON, models and tools;
    # never attempts to decode/re-encode PNGs or platform binaries.
    allowed={'.rs','.toml','.json','.md','.py','.sh','.yml','.yaml','.html','.txt','.crpg','.crpgx','.bbmodel','.patch','.gitignore','.js'}
    files=[p for p in ROOT.rglob('*') if p.is_file() and p.name!='rustpg_migrate.py' and not any(x in p.parts for x in ('.git','target','.ci','.horizon','.venv','__pycache__','releases')) and (p.suffix in allowed or p.name=='.gitignore')]
    for f in files:
        try:s=f.read_text(encoding='utf-8')
        except UnicodeError:continue
        n=s
        for before,after in mapping:n=n.replace(before,after)
        if n!=s:f.write_text(n,encoding='utf-8')
    # Rename deepest objects first; all file path references were rewritten above.
    targets=[p for p in ROOT.rglob('*') if p.exists() and (p.is_file() or p.is_dir())
      and not any(x in p.parts for x in ('.git','target','.ci','.horizon','.venv','__pycache__','releases'))]
    for p in sorted(targets,key=lambda x:len(x.parts),reverse=True):
        if not p.exists():continue
        new=p.name
        for before,after in mapping:new=new.replace(before,after)
        if new!=p.name:
            dest=p.with_name(new)
            if dest.exists():raise RuntimeError(f'rename collision {p} -> {dest}')
            p.rename(dest)


def update_manifest():
    m=PROJECT/'horizon-mod.toml';s=m.read_text()
    s=re.sub(r'^id\s*=.*$', 'id = "rustpg"',s,count=1,flags=re.M)
    s=re.sub(r'^name\s*=.*$', 'name = "RustPG"',s,count=1,flags=re.M)
    s=re.sub(r'^version\s*=.*$', 'version = "5.2.0-alpha"',s,count=1,flags=re.M)
    m.write_text(s)
    c=PROJECT/'Cargo.toml';s=c.read_text()
    s=re.sub(r'^name\s*=.*$', 'name = "mod-rustpg"',s,count=1,flags=re.M)
    s=re.sub(r'^version\s*=.*$', 'version = "5.2.0-alpha"',s,count=1,flags=re.M)
    c.write_text(s)
    lib=SRC/'lib.rs';s=lib.read_text()
    s=s.replace('const VERSION_STR: &str = "5.1.1-starling090-alpha";', 'const VERSION_STR: &str = "5.2.0-starling090-alpha";')
    lib.write_text(s)
    m.write_text(m.read_text().replace('RustPG 5.1.1', 'RustPG 5.2.0'))
    tomllib.loads(c.read_text());tomllib.loads(m.read_text())


def audit():
    assert (SRC/'rustpg_inspector.rs').exists()
    assert (PROJECT/'assets/rustpg/textures/gui/branding/rustpg_logo.png').exists()
    for path in PROJECT.rglob('*'):
      if not path.is_file() or path.suffix not in ('.rs','.json','.toml','.html','.crpg','.crpgx','.rustpg','.rustpgx','.bbmodel'):continue
      s=path.read_text(encoding='utf-8')
      assert all(old not in s for old in ('claude_rpg','Claude RPG','Claude-RPG','Rust Pg','crpg_')),path
    lib=(SRC/'lib.rs').read_text()
    mods=re.findall(r'(?m)^\s*(?:pub\s+)?mod\s+([a-zA-Z0-9_]+)\s*;',lib)
    missing=[m for m in mods if not (SRC/f'{m}.rs').exists() and not (SRC/m/'mod.rs').exists()]
    assert not missing,f'missing Rust modules {missing}'
    # Check that resource URLs still refer to the renamed folder.
    assert '"rustpg:action"' in lib
    assert '"rustpg:gui/branding/rustpg_logo"' in (SRC/'rustpg_ui_window.rs').read_text()
    print('Audit RustPG:',len(mods),'module declarations; RustPG namespaces and resource paths OK')


def main():
    edit_inspector();edit_brand_header();logo=make_logo()
    normalize_identifiers();update_manifest();audit()
    print('Migrated to RustPG; brand asset:',PROJECT/'assets/rustpg/textures/gui/branding/rustpg_logo.png')
if __name__=='__main__':main()