//! RustPG native profession plug-in packs. The rules are loaded from
//! text included in the .hmod at compilation, not executable untrusted code.
//! All events are emitted by trusted server callbacks; UI cannot award XP.
use std::{collections::BTreeSet,sync::{Arc,RwLock}};
use super::rustpg_progression;

pub const PACK_ABI: u16 = 1;
pub const MAX_RULES: usize = 96;
pub const MAX_EVENT_XP: u64 = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub event: String,
    pub profession: String,
    pub xp_per_unit: u64,
    pub max_units: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pack {
    pub id: String,
    pub rules: Vec<Rule>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackError {
    Header, Identity, Abi, DuplicatePack, InvalidRule(usize),
    UnknownProfession(usize), UnknownEvent(usize), DuplicateRule(usize), Limit,
}
#[derive(Default)]
pub struct Registry {
    packs: Vec<Pack>,
    keys: BTreeSet<(String,String)>,
    errors: Vec<String>,
}

fn allowed_event(event: &str) -> bool {
    super::rustpg_immersive::EVENTS.contains(&event)
}
pub(crate) fn valid_id(id:&str)->bool{
    (3..=72).contains(&id.len()) && id.split_once(':').is_some_and(|(ns,path)|
        !ns.is_empty()&&!path.is_empty()&&!path.contains(':')) &&
        id.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||matches!(c,b':'|b'.'|b'_'|b'-'))
}
impl std::fmt::Display for PackError {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        match self{
            Self::Header=>write!(f,"cabeçalho ausente ou repetido"),
            Self::Identity=>write!(f,"identificador inválido"),
            Self::Abi=>write!(f,"versão de interface incompatível"),
            Self::DuplicatePack=>write!(f,"pacote duplicado"),
            Self::InvalidRule(line)=>write!(f,"regra inválida (linha {line})"),
            Self::UnknownProfession(line)=>write!(f,"profissão desconhecida (linha {line})"),
            Self::UnknownEvent(line)=>write!(f,"evento desconhecido (linha {line})"),
            Self::DuplicateRule(line) if *line>0=>write!(f,"regra duplicada (linha {line})"),
            Self::DuplicateRule(_)=>write!(f,"regra duplicada entre pacotes"),
            Self::Limit=>write!(f,"limite de regras atingido"),
        }
    }
}
impl Pack {
    pub fn parse(source:&str)->Result<Self,PackError> {
        if source.len()>16_384 {return Err(PackError::Limit);}
        let mut id=None;
        let mut abi=None;
        let mut rules=Vec::new();
        let mut seen=BTreeSet::new();
        for (idx,line) in source.lines().enumerate() {
            let line=line.trim();
            if line.is_empty() || line.starts_with('#') {continue;}
            if let Some(key)=line.strip_prefix("id=") {
                if id.replace(key.to_string()).is_some() {return Err(PackError::Header);}
                continue;
            }
            if let Some(value)=line.strip_prefix("api=") {
                if abi.replace(value.parse::<u16>().map_err(|_|PackError::Abi)?)
                    .is_some() {return Err(PackError::Header);}
                continue;
            }
            let columns:Vec<_>=line.split('|').collect();
            if columns.len()!=4 {return Err(PackError::InvalidRule(idx+1));}
            let event=columns[0].trim();
            let profession=columns[1].trim();
            if !allowed_event(event) {return Err(PackError::UnknownEvent(idx+1));}
            if !rustpg_progression::PROFICIENCIES.iter().any(|(key,_)|*key==profession) {
                return Err(PackError::UnknownProfession(idx+1));
            }
            let xp=columns[2].trim().parse::<u64>().map_err(|_|PackError::InvalidRule(idx+1))?;
            let max=columns[3].trim().parse::<u32>().map_err(|_|PackError::InvalidRule(idx+1))?;
            if xp==0 || xp>32 || max==0 || max>64 || rules.len()>=MAX_RULES {
                return Err(PackError::InvalidRule(idx+1));
            }
            if !seen.insert((event.to_string(),profession.to_string())) {
                return Err(PackError::DuplicateRule(idx+1));
            }
            rules.push(Rule {event:event.to_string(),profession:profession.to_string(),
                xp_per_unit:xp,max_units:max});
        }
        let id=id.ok_or(PackError::Header)?;
        if !valid_id(&id){return Err(PackError::Identity);}
        if abi!=Some(PACK_ABI) {return Err(PackError::Abi);}
        Ok(Pack{id,rules})
    }
}
impl Registry {
    pub fn new()->Self {Self::default()}
    pub fn register(&mut self,pack:Pack)->Result<(),PackError> {
        if self.packs.iter().any(|p|p.id==pack.id) {return Err(PackError::DuplicatePack);}
        if self.keys.len()+pack.rules.len()>MAX_RULES {return Err(PackError::Limit);}
        if !valid_id(&pack.id){return Err(PackError::Identity);}
        let mut new_keys=BTreeSet::new();
        for (index,rule) in pack.rules.iter().enumerate(){
            let line=index+1;
            if !allowed_event(&rule.event){return Err(PackError::UnknownEvent(line));}
            if !rustpg_progression::PROFICIENCIES.iter().any(|(key,_)|*key==rule.profession){return Err(PackError::UnknownProfession(line));}
            if !(1..=32).contains(&rule.xp_per_unit)||!(1..=64).contains(&rule.max_units){return Err(PackError::InvalidRule(line));}
            let key=(rule.event.clone(),rule.profession.clone());
            if self.keys.contains(&key){return Err(PackError::DuplicateRule(0));}
            if !new_keys.insert(key){return Err(PackError::DuplicateRule(line));}
        }
        self.keys.extend(new_keys);
        self.packs.push(pack);
        Ok(())
    }
    pub fn pack_count(&self)->usize {self.packs.len()}
    pub fn rule_count(&self)->usize {self.keys.len()}
    pub fn errors(&self)->&[String] {&self.errors}
    pub fn record_error(&mut self,error:String) {
        if self.errors.len()<32 {self.errors.push(error);}
    }
    /// Returns only profession XP; scripts never grant generic XP, items or state.
    /// Even if many packs target one event, the sum cannot exceed 64 XP.
    pub fn award(&self,event:&str,units:u32)->Vec<(String,u64)> {
        if !allowed_event(event) || units==0 {return Vec::new();}
        let mut budget=MAX_EVENT_XP;
        let mut awards=Vec::new();
        for p in &self.packs {
            for r in &p.rules {
                if r.event!=event || budget==0 {continue;}
                let amount=r.xp_per_unit.saturating_mul(u64::from(units.min(r.max_units))).min(budget);
                if amount>0 {awards.push((r.profession.clone(),amount));budget-=amount;}
            }
        }
        awards
    }
}
pub fn bundled()->Registry {
    let packs=[
        ("default",include_str!("../data/rustpg/plugins/professions.rustpg")),
        ("extra",include_str!("../data/rustpg/plugins/extra.rustpg")),
        ("advanced",include_str!("../data/rustpg/plugins/activities_advanced.rustpg")),
    ];
    let mut reg=Registry::new();
    for (label,contents) in packs {
        match Pack::parse(contents) {
            Ok(pack)=>if let Err(error)=reg.register(pack) {
                reg.errors.push(format!("{label}: {error}"));
            },
            Err(error)=>reg.errors.push(format!("{label}: {error}")),
        }
    }
    reg
}

/// Reload selected-world profession content from datapacks, replacing the
/// previous generation only after preparation succeeds. No filesystem access,
/// no executable plug-ins and no per-tick parsing. Every world and /reload
/// receives a completely fresh registry (old packs cannot linger).
pub struct ProfessionPackReload {
    pub shared: Arc<RwLock<Registry>>,
}
impl mod_api::ServerReloadListener for ProfessionPackReload {
    fn prepare(&self,context:&mod_api::ServerReloadContext)
        ->Result<Box<dyn mod_api::ReloadApply>,mod_api::ReloadResourceError> {
        let mut next=bundled();
        for (index,id) in context.list().enumerate() {
            if index>=64 {
                next.record_error("limite de 64 pacotes de profissões atingido".into());
                break;
            }
            let Some(bytes)=context.read(id) else{continue;};
            let source=match std::str::from_utf8(bytes) {
                Ok(src)=>src,
                Err(_)=>{
                    next.record_error(format!("{id}: dados UTF-8 inválidos"));
                    continue;
                }
            };
            match Pack::parse(source) {
                Ok(pack)=>if let Err(error)=next.register(pack) {
                    next.record_error(format!("{id}: {error}"));
                },
                Err(error)=>next.record_error(format!("{id}: {error}")),
            }
        }
        let shared=self.shared.clone();
        Ok(<dyn mod_api::ReloadApply>::new(move || {
            *shared.write().unwrap_or_else(|e|e.into_inner())=next;
        }))
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn accepts_core_and_allows_real_events() {
        let reg=bundled();
        assert_eq!(reg.pack_count(),3);
        assert_eq!(reg.award("trade",1),vec![("trading".into(),12)]);
        assert_eq!(reg.award("mine",1),vec![("mining".into(),3)]);
        assert_eq!(reg.award("woodcut",1),vec![("woodcutting".into(),3)]);
        assert_eq!(reg.award("smelt",300),vec![("smithing".into(),24)]);
        assert!(reg.award("free_xp",999).is_empty());
        assert!(reg.award("trade",0).is_empty());
    }
    #[test] fn refuses_bad_schema_or_duplicate_rule() {
        assert_eq!(Pack::parse("id=example:test\napi=3\ntrade|trading|4|1").unwrap_err(),PackError::Abi);
        assert!(matches!(Pack::parse("id=example:test\napi=1\nunknown|trading|4|1"),Err(PackError::UnknownEvent(3))));
        assert!(matches!(Pack::parse("id=example:test\napi=1\ntrade|not_a_prof|4|1"),Err(PackError::UnknownProfession(3))));
        let mut reg=Registry::new();
        let a=Pack::parse("id=example:a\napi=1\ntrade|trading|4|1").unwrap();
        let b=Pack::parse("id=example:b\napi=1\ntrade|trading|6|1").unwrap();
        assert!(reg.register(a).is_ok());
        assert_eq!(reg.register(b),Err(PackError::DuplicateRule(0)));
    }
    #[test] fn rejects_bad_ids_and_direct_invalid_registration(){
        for id in [":test","abc:","abc:def:ghi"]{
            assert_eq!(Pack::parse(&format!("id={id}\napi=1")),Err(PackError::Identity));
        }
        let mut r=Registry::new();
        let mut pack=Pack::parse("id=example:direct\napi=1\ntrade|trading|4|1").unwrap();
        pack.rules.push(pack.rules[0].clone());
        assert!(matches!(r.register(pack),Err(PackError::DuplicateRule(_))));
        assert_eq!(r.rule_count(),0);
        let mut pack=Pack::parse("id=example:direct\napi=1\ntrade|trading|4|1").unwrap();
        pack.rules[0].xp_per_unit=999;
        assert_eq!(r.register(pack),Err(PackError::InvalidRule(1)));
        assert_eq!(r.pack_count(),0);
    }
    #[test] fn never_exceeds_event_cap() {
        let mut reg=Registry::new();
        let p=Pack::parse("id=x:y\napi=1\ntrade|trading|32|64\ntrade|alchemy|32|64").unwrap();
        reg.register(p).unwrap();
        assert_eq!(reg.award("trade",100).iter().map(|(_,v)|v).sum::<u64>(),MAX_EVENT_XP);
    }
}
