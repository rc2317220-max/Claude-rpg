//! Authoritative combo timing inspired by Better Combat's `ComboState` and
//! Epic Fight's staged attacks, implemented independently for Horizon Rust.
//! No animation/geometry assumptions: only actual server-confirmed melee hits.
use std::collections::BTreeMap;
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Entry {pub target:u32,pub last_tick:u64,pub streak:u8}
#[derive(Default)] pub struct Tracker {streaks:BTreeMap<(String,u32),Entry>}
impl Tracker {
    /// Returns a bounded multiplier and a combo index; 1st hit is vanilla.
    /// The optional skill is independent per weapon. No damage from misses.
    pub fn landed(&mut self,dimension:&str,attacker:u32,victim:u32,now:u64,
        extra_per_stack:f64)->(u8,f32){
        if attacker==victim{return (0,1.0)}
        let key=(dimension.to_owned(),attacker);
        let combo=self.streaks.entry(key).or_insert(Entry{target:victim,last_tick:0,streak:0});
        // Avoid duplicated damage callbacks in the same tick.
        if combo.last_tick==now && combo.streak>0 {return (0,1.0)}
        let valid=combo.target==victim && now>=combo.last_tick && now-combo.last_tick<=30;
        combo.streak=if valid {combo.streak.saturating_add(1).min(5)} else {1};
        combo.target=victim;combo.last_tick=now;
        let stacks=combo.streak.saturating_sub(1) as f64;
        let skill=if extra_per_stack.is_finite(){extra_per_stack.clamp(0.0,0.05)}else{0.0};
        let multiplier=(1.0+stacks*skill).min(1.20) as f32;
        (combo.streak,multiplier)
    }
    pub fn purge(&mut self,now:u64){self.streaks.retain(|_,e|now.saturating_sub(e.last_tick)<60);}
    pub fn clear_entity(&mut self,dim:&str,id:u32){self.streaks.remove(&(dim.to_owned(),id));}
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn verified_hits_progress_reset_and_cap(){
        let mut c=Tracker::default();
        assert_eq!(c.landed("overworld",1,9,1,0.0),(1,1.0));
        assert_eq!(c.landed("overworld",1,9,2,0.0).0,2);
        assert_eq!(c.landed("overworld",1,9,2,0.0),(0,1.0));
        assert_eq!(c.landed("overworld",1,10,3,0.0).0,1);
        assert_eq!(c.landed("overworld",1,10,40,0.0).0,1);
        for t in 41..51{let _=c.landed("overworld",1,10,t,100.0);}
        assert!(c.landed("overworld",1,10,51,100.0).1<=1.25);
        c.purge(200);assert!(c.streaks.is_empty());
    }
}
