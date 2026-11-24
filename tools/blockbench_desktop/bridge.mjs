// MCP client for actual Blockbench Desktop. No model geometry synthesized here.
// Only repo-reviewed JSON requests are accepted; no public HTTP command endpoint.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
const lab=process.env.BB_LAB,out=process.env.BB_OUT,root=process.env.BB_ROOT;
if(!lab||!out||!root)throw Error('run through launcher');
const upstream=path.join(lab,'mcp');
const req=createRequire(path.join(upstream,'package.json'));
const {WebSocket}=req('ws');
const {Client}=await import(pathToFileURL(req.resolve('@modelcontextprotocol/sdk/client/index.js')));
const {StdioClientTransport}=await import(pathToFileURL(req.resolve('@modelcontextprotocol/sdk/client/stdio.js')));
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
const transport=new StdioClientTransport({command:process.execPath,args:[path.join(upstream,'dist/mcp-server.js')],env:{...process.env,BB_HEADLESS:'0',BB_BRIDGE_PORT:'8188'},stderr:'pipe'});
const client=new Client({name:'rustpg-desktop-lab',version:'1.0.0'});
let cdp;
const transcript=[];let connected=false;
function slug(v){if(!/^[a-zA-Z0-9_-]{1,80}$/.test(v))throw Error('invalid label '+v);return v}
async function write(file,value){await fs.writeFile(path.join(out,file),JSON.stringify(value,null,2));}
async function cdpConnect(){
  const deadline=Date.now()+90000;
  while(Date.now()<deadline){
    try{
      const pages=await (await fetch('http://127.0.0.1:9222/json/list')).json();
      const page=pages.find(p=>p.type==='page'&&p.webSocketDebuggerUrl&&/blockbench|index\.html/i.test(p.url+' '+p.title));
      if(page){
        const ws=new WebSocket(page.webSocketDebuggerUrl);await new Promise((res,rej)=>{ws.once('open',res);ws.once('error',rej)});
        let id=0;const pending=new Map();
        ws.on('message',data=>{const m=JSON.parse(String(data));const p=pending.get(m.id);if(p){pending.delete(m.id);clearTimeout(p.timer);m.error?p.reject(Error(JSON.stringify(m.error))):p.resolve(m.result);}});
        return {close:()=>ws.close(),call:(method,params={})=>new Promise((resolve,reject)=>{const i=++id;const timer=setTimeout(()=>{pending.delete(i);reject(Error('CDP timeout '+method));},60000);pending.set(i,{resolve,reject,timer});ws.send(JSON.stringify({id:i,method,params}));})};
      }
    }catch(e){console.log('Waiting for desktop renderer:',String(e).slice(0,150));}
    await sleep(1500);
  }
  throw Error('Blockbench renderer never became ready');
}
async function evaluate(expression){
  const r=await cdp.call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true,userGesture:true});
  if(r.exceptionDetails)throw Error(JSON.stringify(r.exceptionDetails).slice(0,3000));
  return r.result?.value;
}
async function bootPlugin(){
  cdp=await cdpConnect();
  let ready=false;
  for(let i=0;i<60;i++){
    if(await evaluate('typeof Plugin === "function" && typeof Plugins === "object" && typeof Blockbench === "object" && document.readyState === "complete"')){ready=true;break}
    await sleep(1000);
  }
  if(!ready)throw Error('Blockbench API unavailable');
  const p=JSON.stringify(path.join(upstream,'dist/blockbench_mcp.js'));
  // The standard local-file loader runs this exact pinned plugin in the real app.
  const result=await evaluate('(async()=>{if(window.BlockbenchMCP)return {already:true};const p='+p+';const plugin=new Plugin();await plugin.loadFromFile({path:p,name:"blockbench_mcp.js"},false);return {id:plugin.id,installed:plugin.installed,version:window.BlockbenchMCP?.version};})()');
  await write('plugin-install.json',result);
  if(!result?.version&&!result?.already)throw Error('plugin installation did not expose bridge');
}
function substitute(v){
  if(typeof v==='string')return v.replaceAll('$CHECKPOINT',path.join(out,'checkpoint')).replaceAll('$INPUT',path.join(lab,'input')).replaceAll('$REPO',root).replaceAll('$OUT',out);
  if(Array.isArray(v))return v.map(substitute);
  if(v&&typeof v==='object')return Object.fromEntries(Object.entries(v).map(([k,x])=>[k,substitute(x)]));
  return v;
}
function decodeText(result){
  const s=result.content?.filter(x=>x.type==='text').map(x=>x.text).join('\n')??'';
  try{return JSON.parse(s)}catch{return s}
}
async function call(name,args={},label=name,expectImages=false){
  label=slug(label);
  if(!allowed.has(name))throw Error('tool not allowed in batch: '+name);
  const result=await client.callTool({name,arguments:substitute(args)},undefined,{timeout:150000});
  const payload=decodeText(result);const images=[];
  for(const item of result.content??[]){
    if(item.type==='image'){
      const ext=item.mimeType==='image/jpeg'?'jpg':item.mimeType==='image/png'?'png':null;
      if(!ext)throw Error('unexpected image MIME '+item.mimeType);
      const data=Buffer.from(item.data,'base64');
      if(data.length<500)throw Error('screenshot suspiciously small');
      const file=label+'-'+String(images.length+1).padStart(2,'0')+'.'+ext;
      await fs.writeFile(path.join(out,file),data);images.push({file,bytes:data.length,sha256:crypto.createHash('sha256').update(data).digest('hex')});
    }
  }
  transcript.push({name,label,args:substitute(args),isError:!!result.isError,result:payload,images});
  await write('transcript.json',transcript);
  console.log(name,label,result.isError?'ERROR':'OK',images.length+' images');
  if(result.isError)throw Error(name+': '+JSON.stringify(payload).slice(0,1500));
  if(expectImages&&!images.length)throw Error('no genuine MCP image for '+name);
  return {payload,images};
}
const allowed=new Set([
 'get_status','list_formats','create_project','get_project_info','set_project_settings','project_file',
 'add_groups','add_cubes','add_planes','add_locators','list_outline','get_element','update_elements','delete_elements','duplicate_elements','mirror_elements','select_elements',
 'create_texture','generate_texture_template','list_textures','get_texture','import_texture','apply_texture','paint_texture','paint_faces','bake_texture','blend_seams','palette','texture_layers','resize_texture','set_texture_resolution','uv',
 'create_animation','list_animations','get_animation','update_animation','delete_animation','set_keyframes','edit_keyframes','mirror_keyframes','add_effect_keyframes','preview_animation','render_animation',
 'add_ik_controllers','bake_ik_animation','validate_model','query_geometry','capture_screenshot','capture_multi_view','compare_reference','project_reference','reference_images','preview_models','undo','redo'
]);
try{
  await client.connect(transport);connected=true;
  if(transport.stderr)transport.stderr.on('data',d=>{console.error(String(d));});
  await bootPlugin();
  const listing=await client.listTools();await write('tools.json',listing);
  console.log('MCP tools',listing.tools.length);
  for(const tool of ['get_status','create_project','add_groups','add_cubes','capture_screenshot','set_keyframes','project_file'])if(!listing.tools.some(x=>x.name===tool))throw Error('missing required tool '+tool);
  let status;
  for(let i=0;i<30;i++){
    try{
      const result=await client.callTool({name:'get_status',arguments:{}},undefined,{timeout:15000});
      status=decodeText(result);
      if(!result.isError&&status?.connected===true)break;
    }catch(e){console.log('waiting MCP plugin:',String(e).slice(0,100))}
    await sleep(1000);
  }
  await write('status.json',status);
  if(!status?.connected)throw Error('Desktop MCP did not connect: '+JSON.stringify(status));
  console.log('Desktop connected:',JSON.stringify(status).slice(0,700));
  const queue=JSON.parse(await fs.readFile(process.argv[2],'utf8'));
  if(queue.schema!==1||!Array.isArray(queue.commands)||queue.commands.length>120)throw Error('invalid queue');
  await write('queue.json',queue);
  let snapshots=[];
  for(const step of queue.commands){
    const label=slug(step.label??step.tool);
    if(step.tool==='desktop_capture'){
      const r=await cdp.call('Page.captureScreenshot',{format:'png',fromSurface:true});
      await fs.writeFile(path.join(out,label+'.png'),Buffer.from(r.data,'base64'));await sleep(700);continue;
    }
    if(!listing.tools.some(x=>x.name===step.tool))throw Error('tool absent from schema: '+step.tool);
    const result=await call(step.tool,step.args??{},label,!!step.images);
    if(step.assert){
      for(const [key,expected] of Object.entries(step.assert)){
        const got=key.split('.').reduce((obj,k)=>obj?.[k],result.payload);
        if(got!==expected)throw Error(label+' expected '+key+'='+JSON.stringify(expected)+', got '+JSON.stringify(got));
      }
    }
    if(step.tool==='capture_screenshot'||step.tool==='preview_animation')snapshots.push(...result.images);
  }
  if(queue.distinct_pose_images&&new Set(snapshots.map(x=>x.sha256)).size<2)throw Error('pose images are identical');
  // Save checkpoint in the app itself. Reopen smoke proves persisted model can be edited in next run.
  const model=path.join(out,'checkpoint',slug(queue.checkpoint??'session')+'.bbmodel');
  await call('project_file',{action:'save',path:model},'save-checkpoint');
  const bytes=await fs.readFile(model);const doc=JSON.parse(bytes.toString());
  if(!doc.elements?.length)throw Error('checkpoint missing geometry');
  await write('checkpoint/manifest.json',{model:path.basename(model),sha256:crypto.createHash('sha256').update(bytes).digest('hex'),elements:doc.elements.length,animations:doc.animations?.length??0,textures:doc.textures?.length??0,run:process.env.GITHUB_RUN_ID,commit:process.env.GITHUB_SHA});
  await write('summary.json',{passed:true,desktop:true,mcp_tools:listing.tools.length,commands:transcript.length,checkpoint:path.basename(model),snapshot_count:snapshots.length});
}catch(e){await write('failure.json',{error:String(e),stack:e.stack});throw e}
finally{cdp?.close();if(connected)await client.close().catch(()=>{});}
