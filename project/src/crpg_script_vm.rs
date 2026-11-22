//! Claude RPG scripting ABI 1: deterministic, externally loadable bytecode.
//!
//! Programs run ONLY after a trusted server hook emits a completed activity;
//! they cannot read the filesystem, allocate game objects, mutate the world,
//! run arbitrary native code or grant general XP. No loops, recursion or jumps.
//! Bound by 32 instructions/program, 16 programs, a four-cell stack and the
//! same global 64-XP/event budget as profession rules.
use std::{collections::BTreeSet,sync::{Arc,RwLock}};
use super::crpg_progression;

pub const ABI:u16=1;
pub const MAX_PROGRAMS:usize=16;
pub const MAX_INSTRUCTIONS:usize=32;
pub const EVENT_BUDGET:u64=64;
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Op { Units, Const(u64), Add, Mul, Min(u64), Max(u64), Award(String) }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Program {pub id:String,pub trigger:String,pub code:Vec<Op>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum VmError {Size,Header,Identity,Api,Trigger,Instruction(usize),Stack,NoAward,Duplicate}
#[derive(Default)]
pub struct Runtime {pub programs:Vec<Program>,pub errors:Vec<String>}
fn allowed_event(s:&str)->bool {
    matches!(s,"craft"|"fish"|"trade"|"tame"|"brew"|"enchant"|"smith"|"cook"|"smelt"|"mine"|"woodcut"|"harvest"|"kill")
}
fn valid_id(s:&str)->bool {
    (3..=72).contains(&s.len()) && s.split_once(':').is_some_and(|(ns,path)|
        !ns.is_empty() && !path.is_empty() && s.bytes().all(|c|
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c,b':'|b'.'|b'_'|b'-')))
}
impl Program {
    pub fn parse(source:&str)->Result<Self,VmError>{
        if source.len()>4096 {return Err(VmError::Size);}
        let (mut id,mut trigger,mut abi)=(None,None,None);
        let mut code=Vec::new();
        for (line_num,line) in source.lines().enumerate() {
            let l=line.trim();
            if l.is_empty()||l.starts_with('#'){continue;}
            if let Some(v)=l.strip_prefix("id=") {if id.replace(v.to_string()).is_some(){return Err(VmError::Header);}continue;}
            if let Some(v)=l.strip_prefix("api=") {if abi.replace(v.parse::<u16>().map_err(|_|VmError::Api)?).is_some(){return Err(VmError::Header);}continue;}
            if let Some(v)=l.strip_prefix("on=") {if trigger.replace(v.to_string()).is_some(){return Err(VmError::Header);}continue;}
            if code.len()>=MAX_INSTRUCTIONS{return Err(VmError::Size);}
            let words:Vec<_>=l.split_ascii_whitespace().collect();
            let op=match words.as_slice(){
                ["units"]=>Op::Units,
                ["const",v]=>Op::Const(v.parse().map_err(|_|VmError::Instruction(line_num+1))?),
                ["add"]=>Op::Add,
                ["mul"]=>Op::Mul,
                ["min",v]=>Op::Min(v.parse().map_err(|_|VmError::Instruction(line_num+1))?),
                ["max",v]=>Op::Max(v.parse().map_err(|_|VmError::Instruction(line_num+1))?),
                ["award",career] if crpg_progression::PROFICIENCIES.iter().any(|(id,_)|*id==*career)=>Op::Award(career.to_string()),
                _=>return Err(VmError::Instruction(line_num+1))
            };
            code.push(op);
        }
        let id=id.ok_or(VmError::Header)?;
        if !valid_id(&id){return Err(VmError::Identity);}
        if abi!=Some(ABI){return Err(VmError::Api);}
        let trigger=trigger.ok_or(VmError::Header)?;
        if !allowed_event(&trigger){return Err(VmError::Trigger);}
        if code.is_empty(){return Err(VmError::NoAward);}
        // Static stack verification makes all runtime execution stack-safe.
        let (mut depth,mut awards)=(0isize,0);
        for op in &code {
            match op {
                Op::Units|Op::Const(_)=>depth+=1,
                Op::Add|Op::Mul=>{if depth<2{return Err(VmError::Stack);}depth-=1;},
                Op::Min(_)|Op::Max(_)=>if depth<1{return Err(VmError::Stack)},
                Op::Award(_)=>{if depth<1{return Err(VmError::Stack);}depth-=1;awards+=1;}
            }
            if depth>4 {return Err(VmError::Stack);}
        }
        if awards==0{return Err(VmError::NoAward);}
        Ok(Self{id,trigger,code})
    }
    /// Return only bounded profession XP; 1 VM instruction always costs O(1).
    pub fn run(&self,units:u32,mut budget:u64)->Vec<(String,u64)>{
        if units==0||budget==0{return Vec::new();}
        let mut stack:Vec<u64>=Vec::with_capacity(4);
        let mut out=Vec::new();
        for op in &self.code {
            match op {
                Op::Units=>stack.push(u64::from(units.min(64))),
                Op::Const(v)=>stack.push(*v),
                Op::Add=>{let a=stack.pop().unwrap_or(0);let b=stack.pop().unwrap_or(0);stack.push(a.saturating_add(b));},
                Op::Mul=>{let a=stack.pop().unwrap_or(0);let b=stack.pop().unwrap_or(0);stack.push(a.saturating_mul(b));},
                Op::Min(n)=>if let Some(v)=stack.last_mut(){*v=(*v).min(*n)},
                Op::Max(n)=>if let Some(v)=stack.last_mut(){*v=(*v).max(*n)},
                Op::Award(profession)=>{
                    let amount=stack.pop().unwrap_or(0).min(budget);
                    if amount>0 {out.push((profession.clone(),amount));budget-=amount;}
                }
            }
        }
        out
    }
}
impl Runtime {
    pub fn register(&mut self, program:Program)->Result<(),VmError>{
        if self.programs.len()>=MAX_PROGRAMS{return Err(VmError::Size);}
        if self.programs.iter().any(|p|p.id==program.id){return Err(VmError::Duplicate);}
        self.programs.push(program);Ok(())
    }
    pub fn error(&mut self,s:String){if self.errors.len()<32{self.errors.push(s)}}
    pub fn run(&self,event:&str,units:u32,budget:u64)->Vec<(String,u64)>{
        if !allowed_event(event) || units==0{return Vec::new();}
        let mut remaining=budget.min(EVENT_BUDGET);
        let mut out=Vec::new();
        for program in self.programs.iter().filter(|p|p.trigger==event){
            for (name,amount) in program.run(units,remaining){
                remaining-=amount;out.push((name,amount));
            }
            if remaining==0{break;}
        }
        out
    }
}
pub fn bundled()->Runtime{
    let mut runtime=Runtime::default();
    for (name,data) in [("demo",include_str!("../data/claude_rpg/plugins_vm/bonus.crpgx"))] {
        match Program::parse(data){
            Ok(program)=>if let Err(e)=runtime.register(program){runtime.error(format!("{name}: {e:?}"))},
            Err(e)=>runtime.error(format!("{name}: {e:?}")),
        }
    }
    runtime
}
pub struct ScriptReload {pub shared:Arc<RwLock<Runtime>>}
impl mod_api::ServerReloadListener for ScriptReload {
    fn prepare(&self,ctx:&mod_api::ServerReloadContext)->Result<Box<dyn mod_api::ReloadApply>,mod_api::ReloadResourceError>{
        let mut next=bundled();
        let mut ids=BTreeSet::new();
        for (i,id) in ctx.list().enumerate(){
            if i>=48{next.error("limite de arquivos do runtime".into());break;}
            let Some(raw)=ctx.read(id)else{continue;};
            let Ok(src)=std::str::from_utf8(raw)else{next.error(format!("{id}: UTF-8 invalido"));continue;};
            match Program::parse(src){
                Ok(p)=>{
                    if !ids.insert(p.id.clone()){next.error(format!("{id}: ID repetido"));continue;}
                    if let Err(e)=next.register(p){next.error(format!("{id}: {e:?}"));}
                },
                Err(e)=>next.error(format!("{id}: {e:?}")),
            }
        }
        let shared=Arc::clone(&self.shared);
        Ok(<dyn mod_api::ReloadApply>::new(move ||{
            *shared.write().unwrap_or_else(|e|e.into_inner())=next;
        }))
    }
}
#[cfg(test)] mod tests {
    use super::*;
    const SCRIPT:&str="id=test:bonus\napi=1\non=trade\nunits\nconst 3\nmul\nmin 24\naward trading\n";
    #[test]fn parses_and_executes(){let p=Program::parse(SCRIPT).unwrap();assert_eq!(p.run(2,64),vec![("trading".into(),6)]);}
    #[test]fn rejects_unsafe_and_bad_scripts(){
        assert!(matches!(Program::parse("id=x:y\napi=1\non=trade\nloop 100\naward trading"),Err(VmError::Instruction(_))));
        assert_eq!(Program::parse("id=x:y\napi=1\non=trade\nadd\naward trading"),Err(VmError::Stack));
        assert_eq!(Program::parse("id=x:y\napi=3\non=trade\nunits\naward trading"),Err(VmError::Api));
    }
    #[test]fn multi_program_budget_and_immutability(){
        let p=Program::parse(SCRIPT).unwrap();let mut r=Runtime::default();r.register(p).unwrap();
        assert_eq!(r.run("trade",200,7),vec![("trading".into(),7)]);
        assert!(r.run("broken_block",1,64).is_empty());
    }
}
