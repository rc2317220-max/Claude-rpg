//! Proveniência persistente dos minérios recolocados (sem dependência do motor).
//! O mundo mantém uma cópia por dimensão em AttachmentType<PlacedOres>.
use std::collections::{HashMap,HashSet};

const LIMIT: usize = 50_000;
const MAGIC: &[u8;4] = b"CRP4";
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct PlacedOres { blocks:HashSet<i64>, overflowed:bool }
impl PlacedOres {
    pub fn new()->Self {Self::default()}
    pub fn mark(&mut self,pos:[i32;3]) {
        let key=pack(pos);
        if self.blocks.contains(&key){return;}
        if self.blocks.len()>=LIMIT {self.overflowed=true;return;}
        self.blocks.insert(key);
    }
    /// Clears a marker when mining, including a previously replaced ore.
    /// When at capacity, deny all ore XP rather than allow a reusable XP farm.
    pub fn take_placed(&mut self,pos:[i32;3])->bool {
        let was=self.blocks.remove(&pack(pos));
        was || self.overflowed
    }
    pub fn is_placed(&self,pos:[i32;3])->bool {self.overflowed || self.blocks.contains(&pack(pos))}
    pub fn encode(&self)->Vec<u8> {
        let mut out=Vec::with_capacity(9+self.blocks.len()*8);
        out.extend_from_slice(MAGIC);out.push(u8::from(self.overflowed));
        out.extend_from_slice(&(self.blocks.len() as u32).to_le_bytes());
        let mut entries:Vec<_>=self.blocks.iter().copied().collect();
        entries.sort_unstable();
        for n in entries {out.extend_from_slice(&n.to_le_bytes());}
        out
    }
    pub fn decode(data:&[u8])->Option<Self>{
        if data.len()<9 || &data[..4]!=MAGIC || data[4]>1 {return None;}
        let count=u32::from_le_bytes(data[5..9].try_into().ok()?) as usize;
        if count>LIMIT || data.len()!=9+count*8 {return None;}
        let mut blocks=HashSet::with_capacity(count);
        for bytes in data[9..].chunks_exact(8) {
            blocks.insert(i64::from_le_bytes(bytes.try_into().ok()?));
        }
        Some(Self{blocks,overflowed:data[4]!=0})
    }
}
/// Minecraft's BlockPos 26/12/26 packing, preserving negative X/Y/Z.
pub fn pack(p:[i32;3])->i64 {
    (((p[0] as u64 & 0x3ff_ffff)<<38) |
     ((p[2] as u64 & 0x3ff_ffff)<<12) |
      (p[1] as u64 & 0xfff)) as i64
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn snapshots_are_deterministic(){
        let mut a=PlacedOres::new();
        a.mark([5,64,-3]);a.mark([-100,-59,20]);
        assert_eq!(PlacedOres::decode(&a.encode()),Some(a.clone()));
        assert!(a.take_placed([5,64,-3]));
        assert!(!a.take_placed([5,64,-3]));
        assert_eq!(PlacedOres::decode(&a.encode()),Some(a));
    }
    #[test]fn refuses_corrupt_snapshots() {
        let mut a=PlacedOres::new().encode();a.push(0);
        assert!(PlacedOres::decode(&a).is_none());
    }
}

/// A successful server-side break authorizes at most one matching loot callback.
/// Missing, stale or mismatched records leave the vanilla drop unchanged.
pub struct BreakLedger<T>{
    permits:HashMap<(String,u128,[i32;3]),(T,u64)>,
}
impl<T> Default for BreakLedger<T>{fn default()->Self{Self{permits:HashMap::new()}}}
impl<T:PartialEq> BreakLedger<T>{
    pub fn issue(&mut self,dim:&str,player:u128,pos:[i32;3],state:T,tick:u64){
        self.expire(tick);
        if self.permits.len()<4096{self.permits.insert((dim.to_owned(),player,pos),(state,tick));}
    }
    pub fn revoke(&mut self,dim:&str,player:u128,pos:[i32;3]){self.permits.remove(&(dim.to_owned(),player,pos));}
    pub fn consume(&mut self,dim:&str,player:u128,pos:[i32;3],state:T,tick:u64)->bool{
        self.permits.remove(&(dim.to_owned(),player,pos)).is_some_and(|(expected,at)|state==expected && at==tick)
    }
    pub fn expire(&mut self,tick:u64){self.permits.retain(|_,(_,at)|*at==tick);}
}
#[cfg(test)]mod permits_tests{use super::*;#[test]fn one_break_one_drop_same_player_dimension_and_state(){
    let mut l=BreakLedger::default();let pos=[2,64,5];l.issue("overworld",1,pos,42,10);
    assert!(!l.consume("nether",1,pos,42,10));assert!(!l.consume("overworld",2,pos,42,10));
    assert!(l.consume("overworld",1,pos,42,10));assert!(!l.consume("overworld",1,pos,42,10));
    l.issue("overworld",1,pos,42,10);assert!(!l.consume("overworld",1,pos,43,10));
    l.issue("overworld",1,pos,42,10);assert!(!l.consume("overworld",1,pos,42,11));
    l.issue("overworld",1,pos,42,11);l.revoke("overworld",1,pos);assert!(!l.consume("overworld",1,pos,42,11));
}}
