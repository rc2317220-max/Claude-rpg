//! Independent gameplay implementation; not the Java mod or its assets.
use std::{collections::HashMap,sync::{Arc,Mutex,OnceLock}};
use mod_api::{self, Hooks, Registrar, RegistrationContext, ClientHost, ServerHost,
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
struct Session{power:Option<Power>,next:u64,grip:Option<EntityRef>,slow_until:u64,heat_until:u64,marker:Option<Marker>,celestial_charge:u64,time_until:u64,last_heat:u64}
impl Default for Session{fn default()->Self{Self{power:None,next:0,grip:None,slow_until:0,heat_until:0,marker:None,celestial_charge:0,time_until:0,last_heat:0}}}
#[derive(Default)]
struct State{tick:u64,players:HashMap<u128,Session>,paused:HashMap<EntityRef,u128>}
static STATE:OnceLock<Mutex<State>>=OnceLock::new();
fn state()->&'static Mutex<State>{STATE.get_or_init(||Mutex::new(State::default()))}
fn lock()->std::sync::MutexGuard<'static,State>{state().lock().unwrap_or_else(|p|p.into_inner())}
#[derive(Default)]
struct Divine;
fn message(host:&mut dyn ServerHost,pl:PlayerRef,s:impl Into<String>){let _=pl.action_bar(host,Text::literal(s.into()));}
fn authorized(host:&dyn ServerHost,pl:PlayerRef)->bool{pl.is_creative(host).unwrap_or(false)&&!pl.is_spectator(host).unwrap_or(true)}
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
 for e in ids{let _=e.mob().set_ai_paused(host,false);}
}
fn clear(host:&mut dyn ServerHost,owner:u128){
 lock().players.remove(&owner);release_pause(host,owner);
}
fn apply(host:&mut dyn ServerHost,p:PlayerRef,action:Action){
 let uuid=p.uuid();
 if action==Action::Clear {clear(host,uuid);message(host,p,"Poder desativado");return}
 if !authorized(host,p){clear(host,uuid);message(host,p,"Protótipo: disponível apenas no criativo");return}
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
  if power==Power::Grip{
   if let Some(e)=s.grip.take(){if e.level(host)==Some(level)&&valid_target(host,e){let at=e.position(host).unwrap_or(eye);let _=e.set_velocity(host,add(scale(dir,2.4),[0.,0.28,0.]));trail(host,level,0,eye,at);}}
  }
  if power==Power::Heat{s.heat_until=0;}
  lock().players.insert(uuid,s);return
 }
 if now<s.next{return}
 s.next=now+8;
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
   if !lock().players.values().any(|v|v.grip==Some(e)){s.grip=Some(e);trail(host,level,0,eye,hit);}
  }
 }
 Power::Heat=>{
  s.heat_until=if s.heat_until>now{0}else{now+100};s.last_heat=0;
 }
 Power::Lightning=>{
  if let Some((e,hit))=target(host,p,MAX_RANGE){
   let _=level.strike_lightning(host,hit,true);let _=hurt(host,e,p,18.,"minecraft:lightning_bolt");fx(host,level,3,hit,[0.25;3],28);
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
  // Original prototype effect: charge a celestial-style projectile burst; no fake sky substitution.
  if s.celestial_charge==0 {s.celestial_charge=now+20;fx(host,level,4,add(eye,scale(dir,3.)),[0.45;3],32);message(host,p,"Carga celestial: use novamente após 1 s");}
  else if now>=s.celestial_charge{
   if let Some((e,hit))=target(host,p,MAX_RANGE){trail(host,level,4,eye,hit);let _=hurt(host,e,p,35.,"minecraft:magic");fx(host,level,4,hit,[1.;3],40);}
   s.celestial_charge=0;s.next=now+40;
  }
 }
 Power::Reality=>{message(host,p,"Realidade: reescrita reversível ainda não implementada");}
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
    let desired=add(eye,add(scale(dir,4.2),[0.,-0.4,0.]));let delta=sub(desired,q);
    if level.line_of_sight(host,eye,desired).unwrap_or(false){let _=e.set_velocity(host,limit(scale(delta,0.34),1.3));if now%3==0{trail(host,level,0,eye,q)}}
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
     if e.mob().ai_paused(host)==Some(false)&&e.mob().set_ai_paused(host,true).is_ok(){lock().paused.insert(e,uuid);}
     if let Some(v)=e.velocity(host){let _=e.set_velocity(host,scale(v,0.20));}
    }
   }
   // Release distant mobs, dimension changes, and stale handles promptly.
   let release=lock().paused.iter().filter(|(e,o)|**o==uuid&&(!e.position(host).is_some_and(|q|norm2(sub(q,eye))<=16.*16.)||e.level(host)!=Some(level))).map(|(e,_)|*e).collect::<Vec<_>>();
   for e in release{let _=e.mob().set_ai_paused(host,false);lock().paused.remove(&e);}
  }else{release_pause(host,uuid);}
  if s.time_until>now&&now%5==0{if let Some(t)=level.clock_time(host){let _=level.set_clock_time(host,t.saturating_add(100));}}
  if s.marker.as_ref().is_some_and(|m|m.expires<now||m.level!=level){s.marker=None;}
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
  "help"=>{let _=p.tell(host,Text::literal("Divine Powers (experimental, criativo): U seleciona, O usa, P alternativo, I desliga. /divine_powers <poder>"));},
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
   r.register_key_mapping(KeyRegistration::new(id,key).label(format!("key.divine_powers.{id}")).category("key.categories.divine_powers").context(KeyConflictContext::InGame));
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
  if !host.playing()||host.local_player().is_none(){return}
  for (id,byte) in [("choose",0),("cast",1),("alternate",3),("clear",4)]{
   let click=host.mod_key_mapping(ID,id).ok().is_some_and(|mapping|mapping.consume_click(host).unwrap_or(false));
   if click{let _=host.send_payload_to_server(&PayloadId(WIRE.into()),PayloadBytes(vec![byte]));}
  }
 }
 fn server_tick(&self,host:&mut dyn ServerHost,e:&mut ServerTickEvent<'_>){if matches!(e.phase,ServerTickPhase::ServerTickEnd{..}){tick(host);}}
 fn player_logged_out(&self,host:&mut dyn ServerHost,e:&mut PlayerLoggedOutEvent<'_>){clear(host,e.player);}
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn decode_exact(){assert_eq!(decode(&[1]),Some(Action::Cast));assert_eq!(decode(&[24]),Some(Action::Select(Power::Reality)));assert_eq!(decode(&[1,0]),None);assert_eq!(decode(&[255]),None);}
 #[test]fn cycle_all(){let mut p=Power::Matter;for _ in 0..9{p=p.next();}assert_eq!(p,Power::Matter);}
 #[test]fn ray_clips(){assert!((ray_box([0.;3],[0.,0.,1.],[[-1.,-1.,5.],[1.,1.,6.]],10.).unwrap()-4.88).abs()<1e-6);assert!(ray_box([0.;3],[0.,0.,1.],[[4.,-1.,5.],[5.,1.,6.]],10.).is_none());}
 #[test]fn clamp_motion(){assert!((norm2(limit([3.,4.,0.],2.))-4.).abs()<1e-6);}
}
