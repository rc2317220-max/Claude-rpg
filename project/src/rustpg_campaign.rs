//! Campaign inspired by the *structure* of FTB Quests: a directed objective
//! graph with independent branches, unlock dependencies and atomic rewards.
//! Original Rust implementation. Only authoritative game events may call record.
//! No player's GUI can directly submit objective progress.
use std::fmt;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Action {
    Logs, CraftTable, CraftWoodPick, Stone, CraftStonePick, Ore, SmeltIron,
    KillHostile, KillElite, KillBoss, Fish, Harvest, Tame, Trade, Enchant,
    Craft, KillPassive, KillLegendary,
}
#[derive(Clone,Copy,Debug)]
pub struct Mission {
    pub id:&'static str,
    pub title:&'static str,
    pub action:Action,
    pub amount:u32,
    /// All prerequisite objectives must have been completed before this event.
    pub needs:u32,
    pub xp:u64,
    pub points:u64,
}
const fn bit(i:u32)->u32 {1<<i}
pub const MISSIONS:[Mission;18]=[
    Mission{id:"forest", title:"Primeiros recursos",action:Action::Logs,amount:16,needs:0,xp:100,points:1},
    Mission{id:"bench",title:"Criar bancada",action:Action::CraftTable,amount:1,needs:bit(0),xp:120,points:1},
    Mission{id:"wood_pick",title:"Primeira picareta",action:Action::CraftWoodPick,amount:1,needs:bit(1),xp:140,points:1},
    Mission{id:"stone",title:"Era da pedra",action:Action::Stone,amount:32,needs:bit(2),xp:175,points:2},
    Mission{id:"stone_pick",title:"Ferramenta melhor",action:Action::CraftStonePick,amount:1,needs:bit(3),xp:180,points:1},
    Mission{id:"ore",title:"Primeiro minério",action:Action::Ore,amount:24,needs:bit(4),xp:250,points:2},
    Mission{id:"iron",title:"Fundição",action:Action::SmeltIron,amount:8,needs:bit(5),xp:280,points:2},
    Mission{id:"hostiles",title:"Sobrevivência",action:Action::KillHostile,amount:10,needs:bit(6),xp:360,points:3},
    Mission{id:"elites",title:"Caça de elites",action:Action::KillElite,amount:3,needs:bit(7),xp:500,points:3},
    Mission{id:"boss",title:"A primeira lenda",action:Action::KillBoss,amount:1,needs:bit(8),xp:800,points:5},
    Mission{id:"fish",title:"Pescador",action:Action::Fish,amount:8,needs:bit(1),xp:180,points:1},
    Mission{id:"farm",title:"Colheita abundante",action:Action::Harvest,amount:24,needs:bit(1),xp:220,points:1},
    Mission{id:"animals",title:"Domar companheiros",action:Action::Tame,amount:2,needs:bit(10)|bit(11),xp:300,points:2},
    Mission{id:"trade",title:"Comércio",action:Action::Trade,amount:6,needs:bit(11),xp:240,points:2},
    Mission{id:"enchant",title:"Encantador",action:Action::Enchant,amount:3,needs:bit(6),xp:300,points:2},
    Mission{id:"artisan",title:"Artesão",action:Action::Craft,amount:24,needs:bit(1),xp:200,points:1},
    Mission{id:"hunt",title:"Caçador",action:Action::KillPassive,amount:12,needs:bit(0),xp:190,points:1},
    Mission{id:"legend",title:"Caçador de lendas",action:Action::KillLegendary,amount:1,needs:bit(9)|bit(12)|bit(14),xp:1600,points:10},
];
const ALL_BITS:u32=(1<<MISSIONS.len())-1;

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Campaign {
    pub completed:u32,
    pub progress:[u32; MISSIONS.len()],
}
impl Default for Campaign {fn default()->Self {Self {completed:0, progress:[0;MISSIONS.len()]}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Completion {pub index:usize,pub xp:u64,pub points:u64}
impl Campaign {
    pub fn is_complete(&self,index:usize)->bool {index<MISSIONS.len() && self.completed & bit(index as u32)!=0}
    pub fn active(&self,index:usize)->bool {index<MISSIONS.len() && !self.is_complete(index)
        && (self.completed & MISSIONS[index].needs)==MISSIONS[index].needs}
    /// Progression is calculated using prerequisites that existed BEFORE
    /// the event. An event cannot complete a dependency AND its child.
    pub fn record(&mut self,action:Action,quantity:u32)->Vec<Completion>{
        if quantity==0{return Vec::new()}
        let count=quantity.min(1024);
        let before=self.completed;
        let mut awards=Vec::new();
        for (i,m) in MISSIONS.iter().enumerate(){
            let mask=bit(i as u32);
            if m.action!=action || before&mask!=0 || before&m.needs!=m.needs{continue;}
            self.progress[i]=self.progress[i].saturating_add(count).min(m.amount);
            if self.progress[i]==m.amount{
                self.completed|=mask;
                awards.push(Completion{index:i,xp:m.xp,points:m.points});
            }
        }
        awards
    }
    pub fn encode(&self)->String {
        format!("1;{:x};{}",self.completed&ALL_BITS,self.progress.iter().map(|v|v.to_string())
            .collect::<Vec<_>>().join(","))
    }
    pub fn decode(s:&str)->Option<Self>{
        let mut fields=s.split(';');
        if fields.next()? !="1"{return None}
        let completed=u32::from_str_radix(fields.next()?,16).ok()? & ALL_BITS;
        let nums=fields.next()?;
        if fields.next().is_some(){return None}
        let data=nums.split(',').map(str::parse::<u32>).collect::<Result<Vec<_>,_>>().ok()?;
        if data.len()!=MISSIONS.len(){return None}
        let mut progress=[0;MISSIONS.len()];
        for (i,n) in data.into_iter().enumerate(){progress[i]=n.min(MISSIONS[i].amount);}
        for (i,goal) in MISSIONS.iter().enumerate(){
            if completed & bit(i as u32)!=0 {progress[i]=goal.amount;}
        }
        Some(Self{completed,progress})
    }
    pub fn completed_count(&self)->u32 {self.completed.count_ones()}
    pub fn active_count(&self)->usize {(0..MISSIONS.len()).filter(|i|self.active(*i)).count()}
    pub fn details(&self)->impl Iterator<Item=(usize,&'static Mission,u32)> + '_ {
        MISSIONS.iter().enumerate().filter(|(i,_)|self.active(*i))
            .map(|(i,m)|(i,m,self.progress[i]))
    }
}
impl fmt::Display for Completion {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        let m=&MISSIONS[self.index];write!(f,"Missão '{}' concluída! +{} XP e +{} pontos",m.title,self.xp,self.points)
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn no_child_credit_from_same_event_and_rewards_once() {
        let mut c=Campaign::default();
        assert!(c.record(Action::CraftTable,1).is_empty());
        assert_eq!(c.record(Action::Logs,16).len(),1);
        assert!(c.record(Action::Logs,400).is_empty());
        assert_eq!(c.record(Action::CraftTable,1).len(),1);
        assert!(c.active(2)&&c.active(10)&&c.active(11)&&c.active(15));
        assert!(!c.active(13));
    }
    #[test] fn no_progress_duplication_after_reload() {
        let mut c=Campaign::default();c.record(Action::Logs,16);
        let saved=c.encode();let mut d=Campaign::decode(&saved).unwrap();
        assert_eq!(c,d);assert!(d.record(Action::Logs,16).is_empty());
        assert!(d.record(Action::CraftTable,1).len()==1);
    }
    #[test] fn corrupt_data_cannot_grant_all_missions() {
        assert!(Campaign::decode("1;ffff;0").is_none());
        assert!(Campaign::decode("2;0;0").is_none());
    }
}
