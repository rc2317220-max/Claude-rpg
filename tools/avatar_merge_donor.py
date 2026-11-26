#!/usr/bin/env python3
"""Merge approved donor accessory meshes into a COPY of an existing V17 free model."""
import argparse, copy, json, pathlib
p=argparse.ArgumentParser()
p.add_argument('base',type=pathlib.Path);p.add_argument('donor',type=pathlib.Path);p.add_argument('output',type=pathlib.Path)
a=p.parse_args()
base=json.loads(a.base.read_text()); donor=json.loads(a.donor.read_text())
assert a.output.resolve()!=a.base.resolve(),'Never overwrite the original'
result=copy.deepcopy(base)
def groups(model):
    result={}
    raw=model.get('groups',[])
    if isinstance(raw,dict):raw=list(raw.values())
    lookup={g['uuid']:g for g in raw if isinstance(g,dict) and 'uuid' in g}
    def walk(nodes):
        for n in nodes:
            if not isinstance(n,dict):continue
            meta=lookup.get(n.get('uuid'),n)
            name=meta.get('name') or n.get('name')
            if name:result[name]=(n,meta)
            walk(n.get('children',[]))
    walk(model.get('outliner',[]))
    print('MODEL STRUCTURE',model.get('meta'),list(result),flush=True)
    return result
bg,dg=groups(result),groups(donor)
targets=['Head','Wing_L','Wing_R']
assert all(n in bg and n in dg for n in targets), (list(bg),list(dg))
def descendants(g):
    ids=set()
    for c in g.get('children',[]):
        if isinstance(c,str):ids.add(c)
        elif isinstance(c,dict):ids.update(descendants(c))
    return ids
all_old=set().union(*(descendants(bg[n][0]) for n in targets))
# Preserve hidden spinal mount geometry; do not replace it with cut donor edges.
roots={e['uuid'] for e in result['elements'] if e['uuid'] in all_old and e['name'].startswith('wing_hidden_spinal_root_')}
head_base={e['uuid'] for e in result['elements'] if e['uuid'] in all_old and e['name']=='head'}
old=all_old-roots-head_base
assert len(roots)==2 and len(head_base)==1,(roots,head_base)
new_ids=set().union(*(descendants(dg[n][0]) for n in targets))
assert old, 'Base accessory groups empty'
old_elements=[e for e in result['elements'] if e['uuid'] in old]
donor_elements=[copy.deepcopy(e) for e in donor['elements'] if e['uuid'] in new_ids]
assert len(donor_elements)==3
# Keep all non-accessory data verbatim, including the original cape photograph.
result['elements']=[e for e in result['elements'] if e['uuid'] not in old]
offset=len(result.get('textures',[]))
for i,t in enumerate(copy.deepcopy(donor['textures'])):
    t['id']=str(offset+i)
    result.setdefault('textures',[]).append(t)
for e in donor_elements:
    for f in e.get('faces',{}).values():
        if isinstance(f.get('texture'),int):f['texture']+=offset
    result['elements'].append(e)
for e in result['elements']:
    if e['uuid'] in roots:
        n=next(n for n in ('Wing_L','Wing_R') if e['uuid'] in descendants(bg[n][0]))
        pivot=bg[n][1]['origin']
        origin=e['origin']
        # Shorten the protruding mount around its existing spinal pivot.
        for v in e['vertices'].values():
            for j in range(3):
                v[j]=round(pivot[j]+(origin[j]+v[j]-pivot[j])*.60-origin[j],5)
    if e['uuid'] in head_base:
        # Keep exact vanilla 8x8x8 head as a non-rendered rig/size reference.
        e['visibility']=False
        e['export']=False
for n in targets:
    assert bg[n][1]['origin']==dg[n][1]['origin'] or n.startswith('Wing_'),(n,bg[n][1]['origin'],dg[n][1]['origin'])
    # Each element retains its own absolute origin, while original group animation pivots stay intact.
    retained=[c for c in bg[n][0]['children'] if isinstance(c,str) and c in roots|head_base]
    # Donor labels were camera left/right; original groups use player left/right.
    donor_name={'Head':'Head','Wing_L':'Wing_R','Wing_R':'Wing_L'}[n]
    bg[n][0]['children']=retained+copy.deepcopy(dg[donor_name][0]['children'])
    for e in donor_elements:
        if e['uuid'] in descendants(dg[donor_name][0]):
            if n.startswith('Wing_'):
                xs=[v[0]+e['origin'][0] for v in e['vertices'].values()]
                assert (sum(xs)/len(xs))*bg[n][1]['origin'][0]>0
            e['name']=n+'_Tripo_donor'
result['name']='Avatar Toon V18 Hybrid'
result['model_identifier']='avatar_toon_v18_hybrid'
assert all(e in result['elements'] for e in base['elements'] if e['uuid'] not in old|head_base|roots)
assert result['textures'][:offset]==base['textures']
assert len(set(e['uuid'] for e in result['elements']))==len(result['elements'])
a.output.parent.mkdir(parents=True,exist_ok=True)
a.output.write_text(json.dumps(result,separators=(',',':')))
print('MERGED',len(old_elements),'old accessory meshes replaced by',len(donor_elements),'donor meshes; other elements and textures unchanged')
