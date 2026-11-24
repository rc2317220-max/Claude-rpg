import test from 'node:test';import assert from 'node:assert/strict';import {buildEdits} from './vision-agent.mjs';
const doc={elements:[{name:'belly',uuid:'a',from:[-2,4,-4],to:[2,8,-1],origin:[0,6,-2.5]},{name:'club_handle',from:[0,0,0],to:[1,1,1]}]};
test('bounded actual cube edit',()=>assert.deepEqual(buildEdits({edits:[{id:'belly',scale:[1.2,1,1],translate:[0,0,-1]}]},doc),[{id:'a',from:[-2.4,4,-5],to:[2.4,8,-2],origin:[0,6,-3.5]}]));
test('reject invented id',()=>assert.throws(()=>buildEdits({edits:[{id:'fake'}]},doc)));
test('protect weapon contact',()=>assert.throws(()=>buildEdits({edits:[{id:'club_handle'}]},doc)));
test('reject excessive scaling',()=>assert.throws(()=>buildEdits({edits:[{id:'belly',scale:[4,1,1]}]},doc)));
test('reject duplicate edits',()=>assert.throws(()=>buildEdits({edits:[{id:'belly'},{id:'belly'}]},doc)));

import {buildAdditions,buildPaint,buildMotion,buildEffects,critiqueAccept} from './vision-agent.mjs';
const faces=Object.fromEntries(['north','south','east','west','up','down'].map(k=>[k,{uv:[0,0,2,2],texture:0}]));
const rig={elements:[{name:'belly_core',uuid:'belly-id',from:[-2,4,-4],to:[2,8,-1],origin:[0,6,-2.5],faces},{name:'ground_center',type:'locator'}],textures:[{uuid:'tex'}],groups:[{name:'belly',uuid:'g',origin:[0,6,0]},{name:'chest',uuid:'c',origin:[0,10,0]},{name:'head',uuid:'h',origin:[0,14,0]}],outliner:[{uuid:'g',children:['belly-id']}]};
test('attached detail reuses explicit UVs',()=>{const a=buildAdditions({add:[{anchor:'belly_core',offset:[1,0,0],size:[1,1,1]}]},rig,1)[0];assert.equal(a.parent,'belly');assert.equal(a.faces.north.texture,'tex');});
test('reject detached new cube',()=>assert.throws(()=>buildAdditions({add:[{anchor:'belly_core',offset:[5,5,5],size:[.5,.5,.5]}]},rig,1)));
test('bounded face paint',()=>assert.equal(buildPaint({paint:[{id:'belly_core',face:'north',type:'line',from:[0,0],to:[1,1],color:'#aa8866'}]},rig)[0].target.element,'belly_core'));
test('reject arbitrary texture target',()=>assert.throws(()=>buildPaint({paint:[{id:'*',face:'north',type:'line',color:'#aa8866'}]},rig)));
const clip={length:1,loop:true,bones:['belly','chest','head'].map(bone=>({bone,channel:'rotation',keys:[[0,0,0,0],[.5,10,0,0],[1,0,0,0]]}))};
test('valid motion loop',()=>assert.equal(buildMotion(clip,rig,'test').keys.bones.length,3));
test('reject broken loop',()=>{const p=structuredClone(clip);p.bones[0].keys[2][1]=2;assert.throws(()=>buildMotion(p,rig,'test'));});
test('effect timing and locator bounded',()=>{assert.equal(buildEffects({effects:[{effect:'rustpg:club_dust',locator:'ground_center',time:.5}]},rig,1)[0].channel,'particle');assert.throws(()=>buildEffects({effects:[{effect:'rustpg:club_dust',locator:'ground_center',time:4}]},rig,1));});
test('critic cannot accept a declared regression',()=>assert.equal(critiqueAccept({accept:true,regression:true,reason:'obviously worse in the side view'}),false));

test('detail names use the iteration index without shadowing the rounding function',()=>{const a=buildAdditions({add:[{anchor:'belly_core',offset:[1,0,0],size:[1,1,1]}]},rig,3)[0];assert.equal(a.name,'agent_detail_3_0');assert.deepEqual(a.from,[.5,5.5,-3]);});
