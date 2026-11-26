//! Proveniência persistente dos minérios recolocados (sem dependência do motor).
//! O mundo mantém uma cópia por dimensão em AttachmentType<PlacedOres>.
use std::collections::HashSet;

const LIMIT: usize = 50_000;
const MAGIC: &[u8;4] = b"CRP5";
const LEGACY_MAGIC: &[u8;4] = b"CRP4";
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct PlacedOres { blocks:HashSet<i64>, overflowed:bool, uncertain_chunks:HashSet<i64> }
impl PlacedOres {
    pub fn new()->Self {Self::default()}
    pub fn mark(&mut self,pos:[i32;3]) {
        let key=pack(pos);
        if self.blocks.contains(&key) || self.uncertain_chunks.contains(&chunk_key(pos)){return;}
        if self.blocks.len()>=LIMIT {
            // Only the affected chunk becomes uncertain, not the whole dimension.
            self.uncertain_chunks.insert(chunk_key(pos));return;
        }
        self.blocks.insert(key);
    }
    /// Clears a marker when mining, including a previously replaced ore.
    /// Legacy global overflow is retained because discarded positions cannot be reconstructed.
    pub fn take_placed(&mut self,pos:[i32;3])->bool {
        let was=self.blocks.remove(&pack(pos));
        was || self.overflowed || self.uncertain_chunks.contains(&chunk_key(pos))
    }
    pub fn is_placed(&self,pos:[i32;3])->bool {self.overflowed || self.blocks.contains(&pack(pos)) || self.uncertain_chunks.contains(&chunk_key(pos))}
    pub fn encode(&self)->Vec<u8> {
        let mut out=Vec::with_capacity(13+(self.blocks.len()+self.uncertain_chunks.len())*8);
        out.extend_from_slice(MAGIC);out.push(u8::from(self.overflowed));
        out.extend_from_slice(&(self.blocks.len() as u32).to_le_bytes());
        let mut entries:Vec<_>=self.blocks.iter().copied().collect();
        entries.sort_unstable();
        for n in entries {out.extend_from_slice(&n.to_le_bytes());}
        out.extend_from_slice(&(self.uncertain_chunks.len() as u32).to_le_bytes());
        let mut chunks:Vec<_>=self.uncertain_chunks.iter().copied().collect();chunks.sort_unstable();
        for n in chunks {out.extend_from_slice(&n.to_le_bytes());}
        out
    }
    pub fn decode(data:&[u8])->Option<Self>{
        if data.len()<9 || data[4]>1 {return None;}
        let legacy=&data[..4]==LEGACY_MAGIC;
        if !legacy && &data[..4]!=MAGIC{return None;}
        let count=u32::from_le_bytes(data[5..9].try_into().ok()?) as usize;
        if count>LIMIT{return None;}
        let end=9+count*8;
        if data.len()<end{return None;}
        let mut blocks=HashSet::with_capacity(count);
        for bytes in data[9..end].chunks_exact(8) {
            if !blocks.insert(i64::from_le_bytes(bytes.try_into().ok()?)){return None;}
        }
        let mut uncertain_chunks=HashSet::new();
        if legacy {
            if data.len()!=end{return None;}
        }else{
            let n=u32::from_le_bytes(data.get(end..end+4)?.try_into().ok()?) as usize;
            if n.checked_mul(8)?.checked_add(end+4)?!=data.len(){return None;}
            for bytes in data[end+4..].chunks_exact(8){
                if !uncertain_chunks.insert(i64::from_le_bytes(bytes.try_into().ok()?)){return None;}
            }
        }
        Some(Self{blocks,overflowed:data[4]!=0,uncertain_chunks})
    }
}
fn chunk_key(p:[i32;3])->i64 {((i64::from(p[0].div_euclid(16)))<<32) | i64::from(p[2].div_euclid(16) as u32)}
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
    #[test]fn overflow_is_local_and_legacy_saves_remain_conservative(){
        let mut a=PlacedOres::new();
        for x in 0..LIMIT as i32 {a.mark([x,64,0]);}
        a.mark([-1,64,-1]);
        assert!(a.is_placed([-2,65,-2]));
        assert!(!a.is_placed([-20,64,-20]));
        assert!(a.take_placed([0,64,0]));
        a.mark([-20,64,-20]);
        assert!(a.is_placed([-20,64,-20]));
        assert_eq!(PlacedOres::decode(&a.encode()),Some(a));
        let mut old=b"CRP4".to_vec();old.push(1);old.extend_from_slice(&0u32.to_le_bytes());
        let old=PlacedOres::decode(&old).unwrap();
        assert!(old.is_placed([200,64,200]));
        assert_eq!(PlacedOres::decode(&old.encode()),Some(old));
    }
    #[test]fn refuses_corrupt_snapshots() {
        let mut a=PlacedOres::new().encode();a.push(0);
        assert!(PlacedOres::decode(&a).is_none());
    }
}
