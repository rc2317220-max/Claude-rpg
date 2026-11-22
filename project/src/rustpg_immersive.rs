//! Milestones and streaks driven only by verified server gameplay actions.
//! Design reference: Immersive Level Up's advancement/title feedback and
//! Project MMO's profession event stream. Repeated client packets cannot count.
use std::collections::BTreeMap;
#[derive(Clone,Debug,PartialEq,Eq,Default)]
pub struct Chain {pub current:u32,pub best:u32,pub last_tick:u64,pub claimed:u8}
#[derive(Clone,Debug,PartialEq,Eq,Default)]
pub struct Masteries {pub tracks:BTreeMap<String,Chain>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Award {pub event:String,pub milestone:u32,pub xp:u64}
const TARGETS:[(u32,u64);4]=[(5,10),(10,20),(20,40),(50,100)];
pub const EVENTS:[&str;13]=["mine","woodcut","harvest","kill","craft","fish","trade",
    "brew","enchant","smith","cook","smelt","tame"];
impl Masteries {
    pub fn record(&mut self,event:&str,server_tick:u64)->Vec<Award>{
        if server_tick==0 || !EVENTS.contains(&event){return Vec::new();}
        let state=self.tracks.entry(event.to_owned()).or_default();
        // One credit per server tick per activity; no double credit when hooks
        // see both a completed action and its associated inventory transaction.
        if state.last_tick==server_tick {return Vec::new();}
        let consecutive=state.last_tick>0 && server_tick>state.last_tick &&
            server_tick-state.last_tick<=200;
        state.current=if consecutive {state.current.saturating_add(1).min(50)}else{1};
        state.last_tick=server_tick;
        state.best=state.best.max(state.current);
        let mut out=Vec::new();
        for (i,(target,amount)) in TARGETS.iter().enumerate(){
            let mask=1u8<<i;
            if state.current>=*target && state.claimed&mask==0 {
                state.claimed|=mask;
                out.push(Award{event:event.to_owned(),milestone:*target,xp:*amount});
            }
        }
        out
    }
    pub fn encode(&self)->Vec<(String,String)> {
        self.tracks.iter().filter(|(k,_)|EVENTS.contains(&k.as_str())).map(|(id,chain)|{
            (id.clone(),format!("{};{};{};{}",chain.current.min(50),chain.best.min(50),
                chain.last_tick,chain.claimed&0x0f))
        }).collect()
    }
    pub fn decode_line(&mut self,event:&str,value:&str)->bool {
        if !EVENTS.contains(&event) {return false;}
        let fields:Vec<_>=value.split(';').collect();
        if fields.len()!=4 {return false;}
        let (Ok(current),Ok(best),Ok(last_tick),Ok(claimed))=(
            fields[0].parse::<u32>(),fields[1].parse::<u32>(),
            fields[2].parse::<u64>(),fields[3].parse::<u8>()) else{return false;};
        if current>50 || best>50 || claimed>15 || best<current {return false;}
        self.tracks.insert(event.to_owned(),Chain{current,best,last_tick,claimed});
        true
    }
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn events_award_once_even_after_reload(){
        let mut s=Masteries::default();let mut grants=Vec::new();
        for now in 1..=50 {
            grants.extend(s.record("mine",now));
            assert!(s.record("mine",now).is_empty());
        }
        assert_eq!(grants.iter().map(|a|a.xp).sum::<u64>(),170);
        let saved=s.encode();let mut restored=Masteries::default();
        for (event,line) in saved {assert!(restored.decode_line(&event,&line));}
        assert!(restored.record("mine",51).is_empty());
        assert_eq!(restored.tracks["mine"].best,50);
    }
    #[test]fn idle_and_invalid_do_not_multiply_rewards(){
        let mut s=Masteries::default();
        for now in 1..=4 {assert!(s.record("craft",now).is_empty());}
        assert!(s.record("craft",1000).is_empty());
        assert_eq!(s.tracks["craft"].current,1);
        assert!(s.record("garbage",1001).is_empty());
        assert!(!s.decode_line("mine","-1;100;999;1"));
    }
}
