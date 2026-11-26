//! Authoritative, bounded mob-health snapshots for client GUI bars.
//! The 0.9 SDK deliberately exposes living() only for the LOCAL client player.
//! External mobs' HP is therefore replicated by our mod from the server.
use crate::{rustpg_scaling::Tier,rustpg_evil_att};
use std::collections::BTreeSet;

/// v3 carries effective attack, authoritative classification and mutation.
/// 0 is unknown tier / no mutation; tier 1 is explicitly Normal.
const VERSION:u8=3;
const SAMPLE_BYTES:usize=24;
pub const MAX_SAMPLE_AGE:u32=20;
const MUTATION_IDS:[&str;9]=["venomous","flame","frost","storm","armored","vampiric","berserk","regenerative","abyssal"];
pub fn fresh(now:u32,received:Option<u32>)->bool{
    received.is_some_and(|then|now.wrapping_sub(then)<=MAX_SAMPLE_AGE)
}
pub fn tier_code(tier:Tier)->u8{
    match tier{Tier::Normal=>1,Tier::Fortified=>2,Tier::Elite=>3,Tier::Champion=>4,
        Tier::MiniBoss=>5,Tier::Boss=>6,Tier::Legendary=>7}
}
pub fn mutation_code(id:Option<&str>)->u8{
    id.and_then(|id|MUTATION_IDS.iter().position(|x|*x==id)).map_or(0,|i|(i+1) as u8)
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct MobSample {pub id:u32,pub hp:f32,pub max_hp:f32,pub armor:f32,pub attack:f32,pub level:u16,pub tier:u8,pub mutation:u8}
impl MobSample {
    pub fn tier_name(&self)->&'static str{
        self.tier.checked_sub(1).and_then(|i|Tier::ALL.get(i as usize))
            .map_or("Sem classificação",|tier|tier.name())
    }
    pub fn mutation_names(&self)->Vec<String>{
        self.mutation.checked_sub(1).and_then(|i|MUTATION_IDS.get(i as usize))
            .and_then(|id|rustpg_evil_att::get(id)).map(|m|vec![m.name.to_owned()]).unwrap_or_default()
    }
    pub fn valid(&self)->bool {
        self.id!=0 && self.hp.is_finite() && self.hp>0.0 &&
        self.max_hp.is_finite() && self.max_hp>0.0 && self.max_hp<=1_000_000_000.0 &&
        self.hp<=self.max_hp && self.armor.is_finite() &&
        self.armor>=0.0 && self.armor<=10_000.0 && self.level>=1 &&
        self.attack.is_finite() && (0.0..=10_000.0).contains(&self.attack) &&
        self.tier<=7 && usize::from(self.mutation)<=MUTATION_IDS.len() &&
        (self.tier!=0 || self.mutation==0)
    }
}
#[derive(Clone,Debug,PartialEq)]
pub struct MobPacket {pub dimension:String,pub mobs:Vec<MobSample>}
impl MobPacket {
    pub const CAP:usize=24;
    pub fn new(dimension:&str,mobs:Vec<MobSample>)->Option<Self> {
        if dimension.len()<3 || dimension.len()>96 || dimension.matches(':').count()!=1 ||
            !dimension.bytes().all(|c|c.is_ascii_lowercase() || c.is_ascii_digit() ||
                matches!(c,b'_'|b'.'|b'-'|b'/'|b':')) ||
            mobs.len()>Self::CAP || !mobs.iter().all(MobSample::valid) {return None;}
        let mut ids=BTreeSet::new();
        if !mobs.iter().all(|m|ids.insert(m.id)){return None;}
        Some(Self{dimension:dimension.into(),mobs})
    }
    pub fn encode(&self)->Vec<u8> {
        let mut v=Vec::with_capacity(3+self.dimension.len()+SAMPLE_BYTES*self.mobs.len());
        v.extend_from_slice(&[VERSION,self.dimension.len() as u8]);
        v.extend_from_slice(self.dimension.as_bytes());
        v.push(self.mobs.len() as u8);
        for m in &self.mobs {
            v.extend_from_slice(&m.id.to_le_bytes());
            v.extend_from_slice(&m.hp.to_le_bytes());
            v.extend_from_slice(&m.max_hp.to_le_bytes());
            v.extend_from_slice(&m.armor.to_le_bytes());
            v.extend_from_slice(&m.attack.to_le_bytes());
            v.extend_from_slice(&m.level.to_le_bytes());
            v.extend_from_slice(&[m.tier,m.mutation]);
        }
        v
    }
    pub fn decode(b:&[u8])->Option<Self> {
        if b.len()<3 || b.len()>3+96+SAMPLE_BYTES*Self::CAP || b[0]!=VERSION{return None;}
        let len=b[1] as usize;
        if len>96 || b.len()<3+len{return None;}
        let dimension=std::str::from_utf8(&b[2..2+len]).ok()?;
        let n=b[2+len] as usize;
        if n>Self::CAP || b.len()!=3+len+n*SAMPLE_BYTES{return None;}
        let mut mobs=Vec::with_capacity(n);
        for chunk in b[3+len..].chunks_exact(SAMPLE_BYTES){
            let u32at=|i:usize|u32::from_le_bytes(chunk[i..i+4].try_into().unwrap_or([0;4]));
            let f32at=|i:usize|f32::from_bits(u32at(i));
            let level=u16::from_le_bytes(chunk[20..22].try_into().unwrap_or([0;2]));
            mobs.push(MobSample{id:u32at(0),hp:f32at(4),max_hp:f32at(8),armor:f32at(12),attack:f32at(16),level,tier:chunk[22],mutation:chunk[23]});
        }
        Self::new(dimension,mobs)
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn snapshot_roundtrip(){
        let m=MobSample{id:13,hp:17.0,max_hp:20.0,armor:4.5,attack:7.5,level:3,tier:tier_code(Tier::Champion),mutation:mutation_code(Some("vampiric"))};
        let packet=MobPacket::new("minecraft:overworld",vec![m]).unwrap();
        assert_eq!(MobPacket::decode(&packet.encode()),Some(packet));
    }
    #[test]fn malformed_packets_rejected(){
        let p=MobPacket::new("minecraft:overworld",vec![MobSample{id:4,hp:2.0,max_hp:10.0,armor:0.0,attack:0.0,level:1,tier:1,mutation:0}]).unwrap();
        let mut bytes=p.encode();
        let corrupt_at=bytes.len()-SAMPLE_BYTES+4;
        bytes[corrupt_at..corrupt_at+4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(MobPacket::decode(&bytes).is_none());
        assert!(MobPacket::decode(&p.encode()[..p.encode().len()-1]).is_none());
        assert!(MobPacket::new("minecraft:overworld",vec![MobSample{id:0,..p.mobs[0]}]).is_none());
    }
}

#[cfg(test)]mod metadata_tests{
    use super::*;
    #[test]fn explicit_tiers_and_mutations_roundtrip(){
        for tier in Tier::ALL{
            for id in MUTATION_IDS{
                let m=MobSample{id:13,hp:17.0,max_hp:20.0,armor:4.5,attack:7.5,level:3,
                    tier:tier_code(tier),mutation:mutation_code(Some(id))};
                let p=MobPacket::new("minecraft:overworld",vec![m]).unwrap();
                let restored=MobPacket::decode(&p.encode()).unwrap().mobs[0];
                assert_eq!(restored,m);
                assert_eq!(restored.tier_name(),tier.name());
                assert_eq!(restored.mutation_names(),vec![rustpg_evil_att::get(id).unwrap().name.to_owned()]);
            }
        }
    }
    #[test]fn rejects_unknown_codes_duplicate_ids_and_wrong_version(){
        let m=MobSample{id:13,hp:17.0,max_hp:20.0,armor:4.5,attack:7.5,level:3,tier:1,mutation:0};
        let p=MobPacket::new("minecraft:overworld",vec![m]).unwrap();
        let mut b=p.encode();b[0]=2;assert!(MobPacket::decode(&b).is_none());
        let mut b=p.encode();let n=b.len();b[n-1]=255;assert!(MobPacket::decode(&b).is_none());
        let mut b=p.encode();let n=b.len();b[n-2]=8;assert!(MobPacket::decode(&b).is_none());
        assert!(MobPacket::new("minecraft:overworld",vec![m,m]).is_none());
    }
    #[test]fn stale_packets_expire_across_tick_wrap(){
        assert!(!fresh(10,None));assert!(fresh(120,Some(100)));assert!(!fresh(121,Some(100)));
        assert!(fresh(3,Some(u32::MAX-3)));assert!(!fresh(30,Some(u32::MAX-3)));
    }
}
