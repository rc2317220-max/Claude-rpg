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
    def walk(nodes):
        for n in nodes:
            if isinstance(n,dict):
                result[n.get('name','')]=n
                walk(n.get('children',[]))
    walk(model.get('outliner',[]))
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
old=set().union(*(descendants(bg[n]) for n in targets))
new_ids=set().union(*(descendants(dg[n]) for n in targets))
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
for n in targets:
    assert bg[n]['origin']==dg[n]['origin'] or n.startswith('Wing_'),(n,bg[n]['origin'],dg[n]['origin'])
    # Each element retains its own absolute origin, while original group animation pivots stay intact.
    bg[n]['children']=copy.deepcopy(dg[n]['children'])
result['name']='Avatar Toon V18 Hybrid'
result['model_identifier']='avatar_toon_v18_hybrid'
assert all(e in result['elements'] for e in base['elements'] if e['uuid'] not in old)
assert result['textures'][:offset]==base['textures']
assert len(set(e['uuid'] for e in result['elements']))==len(result['elements'])
a.output.parent.mkdir(parents=True,exist_ok=True)
a.output.write_text(json.dumps(result,separators=(',',':')))
print('MERGED',len(old_elements),'old accessory meshes replaced by',len(donor_elements),'donor meshes; other elements and textures unchanged')
