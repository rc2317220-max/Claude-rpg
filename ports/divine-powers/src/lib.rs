//! Independent gameplay implementation; not the Java mod or its assets.
use std::{collections::HashMap,sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,AtomicU8,Ordering}}};
use mod_api::{self, Hooks, Registrar, RegistrationContext, ClientHost, ServerHost, BlockState, BlockUpdateFlags, Flow, ScreenRenderPostEvent, ScreenMouseButtonPressedPreEvent,
 PlayerRef, EntityRef, LevelRef, ClientTickPreEvent,
 ServerTickEvent, ServerTickPhase, PlayerLoggedOutEvent, RegisterCommandsEvent,
 Command, CommandSource, CommandInvocation, Text, KeyRegistration, KeyConflictContext,
 IPayloadHandler, IPayloadContext, PayloadBytes, PayloadId, PayloadHandleError,
 PayloadDirection, PayloadProtocol, ParticleKey, ParticleOptions};
const ID:&str="divine_powers";
const WIRE:&str="divine_powers:input";
const MAX_RANGE:f64=36.;
const POWERS:[&str;9]=["materia","telecinese","calor","void","raio","lentidao","tempo","celeste","realidade"];
static PARTICLES:OnceLock<[ParticleKey;5]>=OnceLock::new();
static CLIENT_CAST:AtomicBool=AtomicBool::new(false);
static MENU_SELECTION:AtomicU8=AtomicU8::new(255);
static MENU_ACTIVE:AtomicU8=AtomicU8::new(255);
static MENU_SIZE:OnceLock<Mutex<[f32;2]>>=OnceLock::new();
const LABELS:[&str;9]=["Matéria","Telecinese","Visão térmica","Void","Eletrocinese","Supervelocidade","Tempo","Celeste","Realidade"];
const COLORS:[u32;9]=[0xffba7b55,0xffc44955,0xffef763d,0xff8162c4,0xff70bbee,0xff76cdb4,0xffe3c577,0xffffb65a,0xff81a765];
#[derive(Clone)]
struct TerrainChange{pos:[i32;3],before:BlockState,after:BlockState}
#[derive(Clone)]
struct TerrainWave{level:LevelRef,changed:Vec<TerrainChange>}
#[derive(Clone)]
struct CelestialFlight{level:LevelRef,pos:[f64;3],vel:[f64;3],moon:bool,ticks:u16}
fn layout(size:[f32;2])->([f32;2],f32){([size[0]*0.5,size[1]*0.5],(size[0].min(size[1])*0.33).clamp(45.,112.))}
fn wheel_pick(size:[f32;2],at:[f32;2])->Option<u8>{
 let (c,r)=layout(size);let dx=at[0]-c[0];let dy=at[1]-c[1];let d=(dx*dx+dy*dy).sqrt();
 if d<r*0.35{return Some(4)}if d>r*1.30{return None}
 let a=(dy.atan2(dx)+std::f32::consts::FRAC_PI_2+std::f32::consts::TAU/18.).rem_euclid(std::f32::consts::TAU);
 Some(16+((a/std::f32::consts::TAU*9.).floor() as u8).min(8))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Power{Matter,Grip,Heat,Void,Lightning,Slow,Time,Celestial,Reality}
const ALL:[Power;9]=[Power::Matter,Power::Grip,Power::Heat,Power::Void,Power::Lightning,Power::Slow,Power::Time,Power::Celestial,Power::Reality];
impl Power{
 fn name(self)->&'static str{POWERS[self as usize]}
 fn next(self)->Self{ALL[(self as usize+1)%ALL.len()]}
 fn from_name(name:&str)->Option<Self>{ALL.into_iter().find(|x|x.name()==name)}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Action{Cycle,Cast,Release,Alternate,Clear,Select(Power)}
fn decode(b:&[u8])->Option<Action>{
 if b.len()!=1{return None}
 match b[0]{0=>Some(Action::Cycle),1=>Some(Action::Cast),2=>Some(Action::Release),3=>Some(Action::Alternate),4=>Some(Action::Clear),16..=24=>Some(Action::Select(ALL[(b[0]-16) as usize])),_=>None}
}
#[derive(Clone)]
struct Marker{level:LevelRef,pos:[f64;3],rotation:Option<[f32;2]>,health:f32,clock:Option<i64>,expires:u64}
#[derive(Clone)]
struct Session{power:Option<Power>,next:u64,grip:Option<EntityRef>,slow_until:u64,heat_until:u64,marker:Option<Marker>,celestial_charge:u64,time_until:u64,last_heat:u64,grip_distance:f64,casting:bool}
impl Default for Session{fn default()->Self{Self{power:None,next:0,grip:None,slow_until:0,heat_until:0,marker:None,celestial_charge:0,time_until:0,last_heat:0,grip_distance:4.2,casting:false}}}
#[derive(Default)]
struct State{tick:u64,players:HashMap<u128,Session>,paused:HashMap<EntityRef,u128>,terrain:HashMap<u128,TerrainWave>,flights:HashMap<u128,CelestialFlight>}
static STATE:OnceLock<Mutex<State>>=OnceLock::new();
fn state()->&'static Mutex<State>{STATE.get_or_init(||Mutex::new(State::default()))}
fn lock()->std::sync::MutexGuard<'static,State>{state().lock().unwrap_or_else(|p|p.into_inner())}
#[derive(Default)]
struct Divine;
fn message(host:&mut dyn ServerHost,pl:PlayerRef,s:impl Into<String>){let _=pl.action_bar(host,Text::literal(s.into()));}
fn authorized(host:&dyn ServerHost,pl:PlayerRef)->bool{!pl.is_spectator(host).unwrap_or(true)}
fn empty_hand(host:&dyn ServerHost,p:PlayerRef)->bool{
 let Some(slot)=p.selected_slot(host) else{return false};
 p.with_inventory(host,|v|v.get(slot as u32).and_then(|s|s.project()).map_or(true,|s|s.count()==0)).unwrap_or(false)
}
fn valid_target(host:&dyn ServerHost,e:EntityRef)->bool{
 if e.is_multipart(host).unwrap_or(true){return false}
 let Some(kind)=e.kind(host).and_then(|k|k.id(host).map(|v|v.to_owned())) else{return false};
 if matches!(kind.as_str(),"minecraft:player"|"minecraft:armor_stand"|"minecraft:ender_dragon"|"minecraft:wither"){return false}
 host.living(e).and_then(|l|l.health(host)).is_some_and(|hp|hp>0.)
}
fn add(a:[f64;3],b:[f64;3])->[f64;3]{std::array::from_fn(|i|a[i]+b[i])}
fn sub(a:[f64;3],b:[f64;3])->[f64;3]{std::array::from_fn(|i|a[i]-b[i])}
fn scale(a:[f64;3],s:f64)->[f64;3]{a.map(|x|x*s)}
fn norm2(a:[f64;3])->f64{a.into_iter().map(|v|v*v).sum()}
fn limit(a:[f64;3],max:f64)->[f64;3]{let n=norm2(a).sqrt();if n>max{scale(a,max/n)}else{a}}
fn aim(host:&dyn ServerHost,p:PlayerRef)->Option<(LevelRef,[f64;3],[f64;3])>{
 let e=p.entity();let level=e.level(host)?;let eye=e.eye_position(host)?;let v=e.view_vector(host)?;
 if eye.iter().chain(v.iter()).any(|x|!x.is_finite())||norm2(v)<0.5{return None}
 Some((level,eye,scale(v,1./norm2(v).sqrt())))
}
// Slab ray/AABB intersection. Independent of target list traversal order.
fn ray_box(origin:[f64;3],dir:[f64;3],bb:[[f64;3];2],max:f64)->Option<f64>{
 let mut enter: f64=0.;let mut exit=max;
 for i in 0..3{
  let lo=bb[0][i]-0.12;let hi=bb[1][i]+0.12;
  if dir[i].abs()<1e-9{if origin[i]<lo||origin[i]>hi{return None}continue}
  let a=(lo-origin[i])/dir[i];let b=(hi-origin[i])/dir[i];
  enter=enter.max(a.min(b));exit=exit.min(a.max(b));
  if enter>exit{return None}
 }
 (enter<=max&&exit>=0.).then_some(enter)
}
fn nearby(host:&dyn ServerHost,l:LevelRef,p:[f64;3],r:f64,cap:usize)->Vec<EntityRef>{
 let min=p.map(|x|x-r);let max=p.map(|x|x+r);let mut out=Vec::new();
 let _=l.visit_entities_in(host,min,max,&mut |e|{
  if valid_target(host,e)&&e.position(host).is_some_and(|q|norm2(sub(q,p))<=r*r){out.push(e);}
  if out.len()>=cap{core::ops::ControlFlow::Break(())}else{core::ops::ControlFlow::Continue(())}
 });out
}
fn target(host:&dyn ServerHost,p:PlayerRef,range:f64)->Option<(EntityRef,[f64;3])>{
 let (l,eye,dir)=aim(host,p)?;let to=add(eye,scale(dir,range));
 let min=std::array::from_fn(|i|eye[i].min(to[i])-1.);
 let max=std::array::from_fn(|i|eye[i].max(to[i])+1.);
 let mut best=None;let mut dist=range+1.;
 let _=l.visit_entities_in(host,min,max,&mut |e|{
  if valid_target(host,e){
   if let Some(t)=e.bounding_box(host).and_then(|bb|ray_box(eye,dir,bb,range)){
    let hit=add(eye,scale(dir,t));
    if t<dist&&l.line_of_sight(host,eye,hit).unwrap_or(false){dist=t;best=Some((e,hit));}
   }
  }
  core::ops::ControlFlow::Continue(())
 });best
}
fn fx(host:&mut dyn ServerHost,l:LevelRef,idx:usize,pos:[f64;3],spread:[f64;3],count:i32){
 if let Some(keys)=PARTICLES.get(){let opt=ParticleOptions::simple(keys[idx.min(4)]);
 let _=l.send_particles(host,&opt,None,pos,spread,0.04,count.clamp(0,40));}
}
fn trail(host:&mut dyn ServerHost,l:LevelRef,idx:usize,a:[f64;3],b:[f64;3]){
 for i in 1..=14 {fx(host,l,idx,add(a,scale(sub(b,a),i as f64/14.)),[0.015;3],1)}
}
fn hurt(host:&mut dyn ServerHost,e:EntityRef,p:PlayerRef,amount:f32,damage:&str)->bool{
 if !valid_target(host,e){return false}
 e.hurt(host,damage,amount,None,Some(p.entity())).unwrap_or(false)
}
fn release_pause(host:&mut dyn ServerHost,owner:u128){
 let ids={let mut rt=lock();let a:Vec<_>=rt.paused.iter().filter(|(_,v)|**v==owner).map(|(k,_)|*k).collect();for e in &a{rt.paused.remove(e);}a};
 for e in ids{if let Some(mob)=host.mob(e){let _=mob.set_ai_paused(host,false);}}
}
fn clear(host:&mut dyn ServerHost,owner:u128){
 {let mut rt=lock();rt.players.remove(&owner);rt.flights.remove(&owner);}
 release_pause(host,owner);
}
fn reality(host:&mut dyn ServerHost,p:PlayerRef,level:LevelRef,eye:[f64;3],dir:[f64;3]){
 let uuid=p.uuid();let previous={lock().terrain.remove(&uuid)};
 if let Some(mut wave)=previous{
  if wave.level!=level{lock().terrain.insert(uuid,wave);message(host,p,"Volte à dimensão da onda antes de desfazer");return}
  let flags=BlockUpdateFlags::CLIENTS|BlockUpdateFlags::NEIGHBORS|BlockUpdateFlags::SUPPRESS_DROPS;
  let mut restored=0;let mut pending=Vec::new();
  for c in wave.changed.drain(..).rev(){
   match level.block_state(host,c.pos){
    Some(current) if current==c.after=>{if level.set_block(host,c.pos,c.before,flags).unwrap_or(false){restored+=1;}else{pending.push(c);}},
    Some(_)=>{},None=>pending.push(c)
   }
  }
  if !pending.is_empty(){wave.changed=pending;lock().terrain.insert(uuid,wave);}
  message(host,p,format!("Realidade: {restored} blocos restaurados (não sobrescreve edições posteriores)"));return
 }
 if dir[1]>-0.12{message(host,p,"Olhe para o chão para transformar a realidade");return}
 let mut center=None;
 for n in 1..=48{let at=add(eye,scale(dir,n as f64*0.35));let pos=at.map(|v|v.floor() as i32);
  if !level.line_of_sight(host,eye,at).unwrap_or(false){center=Some(pos);break}
 }
 let Some(c)=center else{message(host,p,"Nenhum terreno ao alcance");return};
 // Full natural blocks only: no inventories, liquids, redstone or structures.
 let pairs=[("minecraft:grass_block","minecraft:mycelium"),("minecraft:dirt","minecraft:coarse_dirt"),("minecraft:stone","minecraft:deepslate"),("minecraft:sand","minecraft:red_sand"),("minecraft:gravel","minecraft:blackstone")];
 let replacements:Vec<_>=pairs.into_iter().filter_map(|(a,b)|Some((host.block_state_id(a)?,host.block_state_id(b)?))).collect();
 let mut changed=Vec::new();let flags=BlockUpdateFlags::CLIENTS|BlockUpdateFlags::NEIGHBORS|BlockUpdateFlags::SUPPRESS_DROPS;
 for radius in 0i32..=7{
  for dz in -radius..=radius{for dx in -radius..=radius{
   if dx.abs().max(dz.abs())!=radius||dx*dx+dz*dz>49{continue}
   for dy in (-3..=3).rev(){
    let pos=[c[0]+dx,c[1]+dy,c[2]+dz];let Some(before)=level.block_state(host,pos)else{continue};
    let Some((_,after))=replacements.iter().find(|(a,_)|before.block()==a.block())else{continue};
    if level.set_block(host,pos,*after,flags).unwrap_or(false){changed.push(TerrainChange{pos,before,after:*after});fx(host,level,0,[pos[0] as f64+0.5,pos[1] as f64+1.0,pos[2] as f64+0.5],[0.2;3],3);}
    break;
   }
  }}
 }
 let count=changed.len();if count>0{lock().terrain.insert(uuid,TerrainWave{level,changed});}
 message(host,p,format!("Realidade: {count} blocos transformados. Use novamente para desfazer nesta sessão."));
}
fn launch_celestial(host:&mut dyn ServerHost,p:PlayerRef,l:LevelRef,eye:[f64;3],dir:[f64;3]){
 let clock=l.clock_time(host).unwrap_or(6000).rem_euclid(24000);
 let moon=(13000..23000).contains(&clock);
 let pos=add(eye,scale(dir,2.));
 lock().flights.insert(p.uuid(),CelestialFlight{level:l,pos,vel:scale(dir,1.8),moon,ticks:48});
 message(host,p,if moon{"Lua lançada (protótipo)"}else{"Sol lançado (protótipo)"});
}
fn tick_celestial(host:&mut dyn ServerHost,uuid:u128,p:PlayerRef,level:LevelRef){
 let Some(mut shot)=({lock().flights.remove(&uuid)})else{return};
 if shot.level!=level{return}
 let next=add(shot.pos,shot.vel);let idx=if shot.moon{3}else{4};
 let targets=nearby(host,level,next,2.,10).into_iter().filter(|e|e.position(host).is_some_and(|q|level.line_of_sight(host,shot.pos,q).unwrap_or(false))).collect::<Vec<_>>();
 let blocked=!level.line_of_sight(host,shot.pos,next).unwrap_or(false);
 if blocked||!targets.is_empty()||shot.ticks<=1{
  let center=if blocked{shot.pos}else{next};
  for e in nearby(host,level,center,6.,24){
   if let Some(at)=e.position(host){if level.line_of_sight(host,center,at).unwrap_or(false){let _=hurt(host,e,p,48.,"minecraft:magic");let _=e.set_velocity(host,limit(add(scale(unit_vector(sub(at,center)),1.35),[0.,0.4,0.]),1.8));}}
  }
  fx(host,level,idx,center,[2.;3],40);fx(host,level,1,center,[1.2;3],30);return
 }
 for k in 0..12{let a=k as f64*std::f64::consts::TAU/12.;let y=((k%5) as f64-2.)*0.32;fx(host,level,idx,add(next,[a.cos()*0.8,y,a.sin()*0.8]),[0.08;3],2);}
 trail(host,level,idx,shot.pos,next);shot.pos=next;shot.ticks-=1;lock().flights.insert(uuid,shot);
}
fn unit_vector(a:[f64;3])->[f64;3]{scale(a,1./norm2(a).sqrt().max(0.001))}
fn grip_motion(delta:[f64;3],velocity:[f64;3])->[f64;3]{limit(sub(scale(delta,0.42),scale(velocity,0.30)),1.45)}
fn charge_ready(now:u64,deadline:u64)->bool{deadline!=0&&now>=deadline&&now.saturating_sub(deadline)<=200}

fn apply(host:&mut dyn ServerHost,p:PlayerRef,action:Action){
 let uuid=p.uuid();
 if action==Action::Clear {clear(host,uuid);message(host,p,"Poder desativado");return}
 if !authorized(host,p){clear(host,uuid);message(host,p,"Poder indisponível no modo espectador");return}
 let now=lock().tick;
 if matches!(action,Action::Cycle|Action::Select(_)){
  let power={let mut rt=lock();let s=rt.players.entry(uuid).or_default();
   let chosen=match action{Action::Select(v)=>v,_=>s.power.map(Power::next).unwrap_or(Power::Matter)};
   *s=Session{power:Some(chosen),next:now+2,..Default::default()};chosen};
  release_pause(host,uuid);
  message(host,p,format!("Poder: {} | O: usar, P: alternativo, I: desligar",power.name()));return
 }
 if !empty_hand(host,p){message(host,p,"Mantenha a mão principal vazia");return}
 let (level,eye,dir)=match aim(host,p){Some(a)=>a,None=>return};
 let mut s={let rt=lock();rt.players.get(&uuid).cloned().unwrap_or_default()};
 let Some(power)=s.power else{message(host,p,"U: escolher poder");return};
 if action==Action::Release{
  s.casting=false;
  if power==Power::Grip{
   if let Some(e)=s.grip.take(){if e.level(host)==Some(level)&&valid_target(host,e){let at=e.position(host).unwrap_or(eye);let _=e.set_velocity(host,add(scale(dir,2.4),[0.,0.28,0.]));trail(host,level,0,eye,at);}}
  }
  if power==Power::Heat{s.heat_until=0;}
  if power==Power::Celestial{if charge_ready(now,s.celestial_charge)&&!lock().flights.contains_key(&uuid){launch_celestial(host,p,level,eye,dir);s.next=now+40;}else if s.celestial_charge!=0{message(host,p,"Carga celestial cancelada");}s.celestial_charge=0;}
  lock().players.insert(uuid,s);return
 }
 if now<s.next{return}
 s.next=now+8;
 if action==Action::Cast{s.casting=true;}
 match power{
 Power::Matter|Power::Void=>{
  let targets=if action==Action::Alternate {
   nearby(host,level,eye,8.,12).into_iter().filter(|e|e.position(host).is_some_and(|at|{let d=sub(at,eye);let n=norm2(d).sqrt().max(0.001);let dot=d.into_iter().zip(dir).map(|(x,y)|x*y).sum::<f64>()/n;dot>0.48&&level.line_of_sight(host,eye,at).unwrap_or(false)})).collect::<Vec<_>>()
  }else{target(host,p,MAX_RANGE).map(|(e,_)|vec![e]).unwrap_or_default()};
  for e in targets{
   let at=e.position(host).unwrap_or(eye);let hp=host.living(e).and_then(|l|l.health(host)).unwrap_or(0.);
   if hp>0.&&hurt(host,e,p,hp.max(1.)*4.+20.,"minecraft:magic"){fx(host,level,if power==Power::Void{2}else{1},add(at,[0.,0.9,0.]),[0.45,0.8,0.45],34);}
  }
  s.next=now+20;
 }
 Power::Grip=>{
  if let Some(e)=s.grip.take(){
   if e.level(host)==Some(level)&&valid_target(host,e){
    if action==Action::Alternate{let at=e.position(host).unwrap_or(eye);let _=hurt(host,e,p,30.,"minecraft:magic");fx(host,level,2,at,[0.4;3],35);}
    else{let _=e.set_velocity(host,add(scale(dir,2.4),[0.,0.25,0.]));}
   }
  }else if let Some((e,hit))=target(host,p,24.){
   if !lock().players.values().any(|v|v.grip==Some(e)){s.grip=Some(e);s.grip_distance=norm2(sub(hit,eye)).sqrt().clamp(3.5,8.0);trail(host,level,0,eye,hit);}
  }
 }
 Power::Heat=>{
  s.heat_until=if action==Action::Alternate{now+40}else{now+200};s.last_heat=0;
 }
 Power::Lightning=>{
  if let Some((e,hit))=target(host,p,MAX_RANGE){
   let _=level.strike_lightning(host,hit,true);let _=hurt(host,e,p,18.,"minecraft:lightning_bolt");fx(host,level,3,hit,[0.25;3],28);
   if action==Action::Alternate{let mut at=hit;let mut seen=vec![e];for _ in 0..4{let next=nearby(host,level,at,7.,24).into_iter().filter(|v|!seen.contains(v)).filter_map(|v|v.position(host).map(|q|(v,q))).filter(|(_,q)|level.line_of_sight(host,at,*q).unwrap_or(false)).min_by(|a,b|norm2(sub(a.1,at)).total_cmp(&norm2(sub(b.1,at))));let Some((v,q))=next else{break};trail(host,level,3,at,q);let _=hurt(host,v,p,10.,"minecraft:lightning_bolt");fx(host,level,3,q,[0.18;3],12);seen.push(v);at=q;}}
  }else if dir[1]>0.65{let _=level.set_weather(host,0,400,true,true);let _=level.strike_lightning(host,add(eye,scale(dir,5.)),true);}
  s.next=now+24;
 }
 Power::Slow=>{
  if s.slow_until>now{s.slow_until=0;release_pause(host,uuid);}
  else{s.slow_until=now+160;}
 }
 Power::Time=>{
  if action==Action::Alternate{s.time_until=if s.time_until>now{0}else{now+160};}
  else if let Some(m)=s.marker.take().filter(|m|m.expires>=now&&m.level==level){
   if p.entity().teleport_to(host,m.pos,m.rotation){
    let _=p.entity().set_velocity(host,[0.;3]);if let Some(l)=host.living(p.entity()){let max=l.max_health(host).unwrap_or(m.health);let _=l.set_health(host,m.health.min(max));}
    if let Some(t)=m.clock{let _=level.set_clock_time(host,t);}
    fx(host,level,3,m.pos,[0.5;3],35);message(host,p,"Marcador pessoal restaurado");
   }
  }else if let (Some(pos),Some(l))=(p.entity().position(host),host.living(p.entity())){
   s.marker=Some(Marker{level,pos,rotation:p.entity().rotation(host),health:l.health(host).unwrap_or(20.),clock:level.clock_time(host),expires:now+600});
   message(host,p,"Marcador pessoal: 30 s. Use novamente para voltar.");
  }
 }
 Power::Celestial=>{
  if lock().flights.contains_key(&uuid){message(host,p,"Corpo celeste ainda em voo");}
  else if s.celestial_charge==0 {
   if dir[1]<0.28{message(host,p,"Olhe para o céu, segure O para carregar e solte para lançar");}
   else{s.celestial_charge=now+20;message(host,p,"Carregando corpo celeste: solte após 1 s");}
  }else{message(host,p,"Solte O para lançar");}
 }
 Power::Reality=>{reality(host,p,level,eye,dir);s.next=now+24;}
 }
 lock().players.insert(uuid,s);
}
fn tick(host:&mut dyn ServerHost){
 let now={let mut rt=lock();rt.tick=rt.tick.wrapping_add(1);rt.tick};
 let owners=lock().players.keys().copied().collect::<Vec<_>>();
 for uuid in owners{
  let Some(p)=host.player(uuid)else{clear(host,uuid);continue};
  if !authorized(host,p)||!empty_hand(host,p){clear(host,uuid);continue}
  let Some((level,eye,dir))=aim(host,p)else{clear(host,uuid);continue};
  let Some(mut s)=lock().players.get(&uuid).cloned()else{continue};
  if let Some(e)=s.grip{
   if e.level(host)!=Some(level)||!valid_target(host,e)||!e.position(host).is_some_and(|q|norm2(sub(q,eye))<36.*36.){s.grip=None;}
   else if let Some(q)=e.position(host){
    let desired=add(eye,add(scale(dir,s.grip_distance),[0.,-0.4,0.]));let delta=sub(desired,q);
    if level.line_of_sight(host,eye,desired).unwrap_or(false)&&level.line_of_sight(host,q,desired).unwrap_or(false){let v=e.velocity(host).unwrap_or([0.;3]);let _=e.set_velocity(host,grip_motion(delta,v));if now%3==0{trail(host,level,0,eye,q)}}
    else{s.grip=None}
   }
  }
  if s.heat_until>now&&now.saturating_sub(s.last_heat)>=5{
   let hit=target(host,p,MAX_RANGE);
   let end=hit.map(|(_,point)|point).unwrap_or_else(||{
    let mut end=eye;for i in 1..=36{let p=add(eye,scale(dir,i as f64));if !level.line_of_sight(host,eye,p).unwrap_or(false){break}end=p;}end
   });
   let right=limit([dir[2],0.,-dir[0]],1.);
   trail(host,level,4,add(eye,scale(right,0.12)),end);trail(host,level,4,add(eye,scale(right,-0.12)),end);
   if let Some((e,_))=hit{let _=hurt(host,e,p,7.,"minecraft:on_fire");}
   s.last_heat=now;
  }
  if s.slow_until>now{
   if now%5==0{
    for e in nearby(host,level,eye,16.,32){
     if let Some(mob)=host.mob(e){if mob.ai_paused(host)==Some(false)&&mob.set_ai_paused(host,true).is_ok(){lock().paused.insert(e,uuid);}}
     if let Some(v)=e.velocity(host){let _=e.set_velocity(host,scale(v,0.20));}
    }
   }
   // Release distant mobs, dimension changes, and stale handles promptly.
   let release=lock().paused.iter().filter(|(e,o)|**o==uuid&&(!e.position(host).is_some_and(|q|norm2(sub(q,eye))<=16.*16.)||e.level(host)!=Some(level))).map(|(e,_)|*e).collect::<Vec<_>>();
   for e in release{if let Some(mob)=host.mob(e){let _=mob.set_ai_paused(host,false);}lock().paused.remove(&e);}
  }else{release_pause(host,uuid);}
  if s.time_until>now&&now%5==0{if let Some(t)=level.clock_time(host){let _=level.set_clock_time(host,t.saturating_add(100));}}
  if s.marker.as_ref().is_some_and(|m|m.expires<now||m.level!=level){s.marker=None;}
  if s.celestial_charge!=0&&now>s.celestial_charge.saturating_add(200){s.celestial_charge=0;s.casting=false;}
  if s.celestial_charge!=0&&s.casting&&now%2==0{let idx=if (13000..23000).contains(&level.clock_time(host).unwrap_or(0).rem_euclid(24000)){3}else{4};fx(host,level,idx,add(eye,scale(dir,2.2)),[0.55;3],12);}
  tick_celestial(host,uuid,p,level);
  lock().players.insert(uuid,s);
 }
}
struct InputHandler;
impl IPayloadHandler for InputHandler{
 fn handle(&self,payload:PayloadBytes,ctx:&mut dyn IPayloadContext)->Result<(),PayloadHandleError>{
  if ctx.flow()!=PayloadDirection::ToServer||ctx.protocol()!=PayloadProtocol::Play{return Ok(())}
  let Some(action)=decode(&payload.0)else{return Ok(())};
  let Ok(p)=ctx.player()else{return Ok(())};let Ok(host)=ctx.server_host()else{return Ok(())};
  apply(host,p,action);Ok(())
 }
}
fn command(host:&mut dyn ServerHost,source:&CommandSource,c:&CommandInvocation)->Option<i32>{
 if c.literals.first().map(String::as_str)!=Some(ID){return None}
 let CommandSource::Player{player,..}=source else{return Some(0)};
 let p=*player;
 match c.literals.get(1).map(String::as_str).unwrap_or("help"){
  "help"=>{let _=p.tell(host,Text::literal("Mão de Deus — adaptação experimental: U abre a roda, O segura/usa, P alternativo, I desliga. /divine_powers <poder>"));},
  "cast"=>apply(host,p,Action::Cast),"alt"=>apply(host,p,Action::Alternate),"clear"=>apply(host,p,Action::Clear),
  "release"=>apply(host,p,Action::Release),"next"=>apply(host,p,Action::Cycle),
  v=>if let Some(power)=Power::from_name(v){apply(host,p,Action::Select(power));}
 };Some(1)
}
#[mod_api::sunrise_mod]
impl Hooks for Divine{
 fn register(&self,r:&mut dyn Registrar,_:&RegistrationContext){
  let _=PARTICLES.set([r.particle_key("minecraft:enchant"),r.particle_key("minecraft:ash"),r.particle_key("minecraft:smoke"),r.particle_key("minecraft:electric_spark"),r.particle_key("minecraft:flame")]);
  for (id,key) in [("choose","key.keyboard.u"),("cast","key.keyboard.o"),("alternate","key.keyboard.p"),("clear","key.keyboard.i")]{
   r.register_key_mapping(KeyRegistration::new(id,key).label(match id{"choose"=>"key.divine_powers.choose","cast"=>"key.divine_powers.cast","alternate"=>"key.divine_powers.alternate",_=>"key.divine_powers.clear"}).category("key.categories.divine_powers").context(KeyConflictContext::InGame));
  }
  let mut event=r.payload_handlers();let mut registry=event.registrar("1");
  registry.play_to_server(PayloadId(WIRE.into()),Arc::new(InputHandler));
 }
 fn register_commands(&self,_:&mut dyn ServerHost,e:&mut RegisterCommandsEvent<'_>){
  for sub in ["help","cast","alt","release","clear","next"].into_iter().chain(POWERS){
   e.commands.push(Command::new(&format!("{ID} {sub}")).executes(command));
  }
 }
 fn client_tick_pre(&self,host:&mut dyn ClientHost,_:&mut ClientTickPreEvent){
  let selected=MENU_SELECTION.swap(255,Ordering::AcqRel);
  if selected!=255&&host.mod_screen()==Some(ID){
   host.mod_screen_close(ID);if selected==4{MENU_ACTIVE.store(255,Ordering::Release);}else{MENU_ACTIVE.store(selected-16,Ordering::Release);}
   let _=host.send_payload_to_server(&PayloadId(WIRE.into()),PayloadBytes(vec![selected]));return
  }
  if host.local_player().is_none(){CLIENT_CAST.store(false,Ordering::Release);MENU_ACTIVE.store(255,Ordering::Release);return}
  let playing=host.playing();
  let cast=playing&&host.mod_key_mapping(ID,"cast").ok().is_some_and(|m|m.is_down(host).unwrap_or(false));
  let was=CLIENT_CAST.swap(cast,Ordering::AcqRel);
  if was&&!cast{let _=host.send_payload_to_server(&PayloadId(WIRE.into()),PayloadBytes(vec![2]));}
  if !playing{return}
  if host.mod_key_mapping(ID,"choose").ok().is_some_and(|m|m.consume_click(host).unwrap_or(false)){
   let _=host.mod_screen_open(ID);return
  }
  if cast&&!was{let _=host.send_payload_to_server(&PayloadId(WIRE.into()),PayloadBytes(vec![1]));}
  for (id,byte) in [("alternate",3),("clear",4)]{
   if host.mod_key_mapping(ID,id).ok().is_some_and(|m|m.consume_click(host).unwrap_or(false)){
    if byte==4{MENU_ACTIVE.store(255,Ordering::Release);}
    let _=host.send_payload_to_server(&PayloadId(WIRE.into()),PayloadBytes(vec![byte]));
   }
  }
 }
 fn screen_render_post(&self,host:&dyn ClientHost,e:&mut ScreenRenderPostEvent<'_>){
  if host.mod_screen()!=Some(ID){return}
  let size=e.draw.size();*MENU_SIZE.get_or_init(||Mutex::new(size)).lock().unwrap_or_else(|p|p.into_inner())=size;
  let (c,r)=layout(size);let tau=std::f32::consts::TAU;
  let hover=wheel_pick(size,e.screen.mouse);
  e.draw.fill([0.,0.,size[0],size[1]],0xb0101018);
  for i in 0..9{
   let a=i as f32*tau/9.-std::f32::consts::FRAC_PI_2;
   let start=a-tau/18.+0.025;let end=a+tau/18.-0.025;
   let selected=MENU_ACTIVE.load(Ordering::Acquire)==i as u8;
   let color=if hover==Some(16+i as u8){COLORS[i]}else if selected{0xff5e536b}else{0xff292936};
   for n in 0..6{let u=start+(end-start)*n as f32/6.;let v=start+(end-start)*(n+1) as f32/6.;let point=|rr:f32,t:f32|[c[0]+rr*t.cos(),c[1]+rr*t.sin()];
    e.draw.quad([point(r*0.39,u),point(r,u),point(r,v),point(r*0.39,v)],color);
   }
   let label=LABELS[i];let p=[c[0]+a.cos()*r*0.71-(label.chars().count() as f32*2.6),c[1]+a.sin()*r*0.71-4.];
   e.draw.text(label,p,0xfff2f0e9,true);
  }
  e.draw.fill([c[0]-23.,c[1]-10.,46.,20.],0xff423747);
  e.draw.text("Nenhum",[c[0]-18.,c[1]-4.],0xffffffff,true);
  e.draw.text("MÃO DE DEUS",[c[0]-34.,(c[1]-r-20.).max(4.)],0xffe4c58a,true);
  e.draw.text("O: usar / segurar   P: alternativo   I: desligar",[8.,size[1]-14.],0xffbcb8c4,true);
 }
 fn screen_mouse_button_pressed_pre(&self,host:&dyn ClientHost,e:&mut ScreenMouseButtonPressedPreEvent<'_>)->Flow{
  if host.mod_screen()!=Some(ID){return Flow::Continue}
  let Some(size)=MENU_SIZE.get().map(|m|*m.lock().unwrap_or_else(|p|p.into_inner()))else{return Flow::Cancel};
  if let Some(value)=wheel_pick(size,e.screen.mouse){MENU_SELECTION.store(value,Ordering::Release);}
  Flow::Cancel
 }
 fn server_tick(&self,host:&mut dyn ServerHost,e:&mut ServerTickEvent<'_>){if matches!(e.phase,ServerTickPhase::ServerTickEnd{..}){tick(host);}}
 fn player_logged_out(&self,host:&mut dyn ServerHost,e:&mut PlayerLoggedOutEvent<'_>){clear(host,e.player);}
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn charge_window(){assert!(!charge_ready(19,20));assert!(charge_ready(20,20));assert!(charge_ready(220,20));assert!(!charge_ready(221,20));assert!(!charge_ready(30,0));}
 #[test]fn grip_damping(){assert_eq!(grip_motion([0.;3],[1.,0.,0.]),[-0.3,0.,0.]);assert!(norm2(grip_motion([50.,0.,0.],[0.;3]))<=1.45f64.powi(2)+1e-9);}
 #[test]fn decode_exact(){assert_eq!(decode(&[1]),Some(Action::Cast));assert_eq!(decode(&[24]),Some(Action::Select(Power::Reality)));assert_eq!(decode(&[1,0]),None);assert_eq!(decode(&[255]),None);}
 #[test]fn cycle_all(){let mut p=Power::Matter;for _ in 0..9{p=p.next();}assert_eq!(p,Power::Matter);}
 #[test]fn ray_clips(){assert!((ray_box([0.;3],[0.,0.,1.],[[-1.,-1.,5.],[1.,1.,6.]],10.).unwrap()-4.88).abs()<1e-6);assert!(ray_box([0.;3],[0.,0.,1.],[[4.,-1.,5.],[5.,1.,6.]],10.).is_none());}
 #[test]fn clamp_motion(){assert!((norm2(limit([3.,4.,0.],2.))-4.).abs()<1e-6);}
 #[test]fn wheel_center_and_first_sector(){assert_eq!(wheel_pick([400.,300.],[200.,150.]),Some(4));assert_eq!(wheel_pick([400.,300.],[200.,85.]),Some(16));assert_eq!(wheel_pick([400.,300.],[0.,0.]),None); }
}
