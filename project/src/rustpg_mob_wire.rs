//! Authoritative, bounded mob-health snapshots for client GUI bars.
//! The 0.9 SDK deliberately exposes living() only for the LOCAL client player.
//! External mobs' HP is therefore replicated by our mod from the server.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct MobSample {pub id:u32,pub hp:f32,pub max_hp:f32,pub armor:f32,pub damage:f32,pub level:u16}
impl MobSample {
    pub fn valid(&self)->bool {
        self.id!=0 && self.hp.is_finite() && self.hp>0.0 &&
        self.max_hp.is_finite() && self.max_hp>0.0 && self.max_hp<=1_000_000_000.0 &&
        self.hp<=self.max_hp && self.armor.is_finite() &&
        self.armor>=0.0 && self.armor<=10_000.0 && self.damage.is_finite() && self.damage>=-1.0 && self.damage<=1_000_000_000.0 && self.level>=1
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
        Some(Self{dimension:dimension.into(),mobs})
    }
    pub fn encode(&self)->Vec<u8> {
        let mut v=Vec::with_capacity(3+self.dimension.len()+22*self.mobs.len());
        v.extend_from_slice(&[2,self.dimension.len() as u8]);
        v.extend_from_slice(self.dimension.as_bytes());
        v.push(self.mobs.len() as u8);
        for m in &self.mobs {
            v.extend_from_slice(&m.id.to_le_bytes());
            v.extend_from_slice(&m.hp.to_le_bytes());
            v.extend_from_slice(&m.max_hp.to_le_bytes());
            v.extend_from_slice(&m.armor.to_le_bytes());
            v.extend_from_slice(&m.damage.to_le_bytes());
            v.extend_from_slice(&m.level.to_le_bytes());
        }
        v
    }
    pub fn decode(b:&[u8])->Option<Self> {
        if b.len()<3 || b.len()>3+96+22*Self::CAP || b[0]!=2{return None;}
        let len=b[1] as usize;
        if len>96 || b.len()<3+len{return None;}
        let dimension=std::str::from_utf8(&b[2..2+len]).ok()?;
        let n=b[2+len] as usize;
        if n>Self::CAP || b.len()!=3+len+n*22{return None;}
        let mut mobs=Vec::with_capacity(n);
        for chunk in b[3+len..].chunks_exact(22){
            let u32at=|i:usize|u32::from_le_bytes(chunk[i..i+4].try_into().unwrap_or([0;4]));
            let f32at=|i:usize|f32::from_bits(u32at(i));
            let level=u16::from_le_bytes(chunk[20..22].try_into().unwrap_or([0;2]));
            mobs.push(MobSample{id:u32at(0),hp:f32at(4),max_hp:f32at(8),armor:f32at(12),damage:f32at(16),level});
        }
        Self::new(dimension,mobs)
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn snapshot_roundtrip(){
        let m=MobSample{id:13,hp:17.0,max_hp:20.0,armor:4.5,damage:7.0,level:3};
        let packet=MobPacket::new("minecraft:overworld",vec![m]).unwrap();
        assert_eq!(MobPacket::decode(&packet.encode()),Some(packet));
    }
    #[test]fn malformed_packets_rejected(){
        let p=MobPacket::new("minecraft:overworld",vec![MobSample{id:4,hp:2.0,max_hp:10.0,armor:0.0,damage:-1.0,level:1}]).unwrap();
        let mut bytes=p.encode();
        let corrupt_at=bytes.len()-14;
        bytes[corrupt_at..corrupt_at+4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(MobPacket::decode(&bytes).is_none());
        assert!(MobPacket::decode(&p.encode()[..p.encode().len()-1]).is_none());
        assert!(MobPacket::new("minecraft:overworld",vec![MobSample{id:0,..p.mobs[0]}]).is_none());
    }
}
