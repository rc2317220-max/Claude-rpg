import test from 'node:test';import assert from 'node:assert/strict';import {buildEdits} from './vision-agent.mjs';
const doc={elements:[{name:'belly',uuid:'a',from:[-2,4,-4],to:[2,8,-1],origin:[0,6,-2.5]},{name:'club_handle',from:[0,0,0],to:[1,1,1]}]};
test('bounded actual cube edit',()=>assert.deepEqual(buildEdits({edits:[{id:'belly',scale:[1.2,1,1],translate:[0,0,-1]}]},doc),[{id:'a',from:[-2.4,4,-5],to:[2.4,8,-2],origin:[0,6,-3.5]}]));
test('reject invented id',()=>assert.throws(()=>buildEdits({edits:[{id:'fake'}]},doc)));
test('protect weapon contact',()=>assert.throws(()=>buildEdits({edits:[{id:'club_handle'}]},doc)));
test('reject excessive scaling',()=>assert.throws(()=>buildEdits({edits:[{id:'belly',scale:[4,1,1]}]},doc)));
test('reject duplicate edits',()=>assert.throws(()=>buildEdits({edits:[{id:'belly'},{id:'belly'}]},doc)));
