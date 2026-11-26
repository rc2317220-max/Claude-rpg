//! Versioned compact server-to-client combat message, independently testable.
//! Only the trusted server is permitted to send "rustpg:hit".
use super::rustpg_damage_meter::{DamageFamily,MAX_REPORTED_DAMAGE};

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct DamageHit {pub victim:u32,pub damage:f32}
impl DamageHit {
    pub fn new(victim:u32,damage:f32)->Option<Self> {
        if victim==0 || !damage.is_finite() || damage<=0.0 || damage>MAX_REPORTED_DAMAGE {return None;}
        Some(Self{victim,damage})
    }
}
#[derive(Clone,Debug,PartialEq)]
pub struct DamageNotice {
    /// Dimension at the moment of the authoritative hit (no cross-world popups).
    pub dimension:String,
    pub hit:DamageHit,
    pub at:[f64;3],
    pub attacker:i32,       // 0 if not known, otherwise DamageSource.causing
    pub family:DamageFamily,
}
impl DamageNotice {
    pub fn new(dimension:&str,hit:DamageHit,at:[f64;3],attacker:i32,family:DamageFamily)->Option<Self> {
        // A packet is accepted only for a real, unambiguous dimension key.
        // This value is copied from ServerHost::dimension, never from the client.
        if dimension.len()<3 || dimension.len()>96 || dimension.matches(':').count()!=1
            || !dimension.bytes().all(|c|c.is_ascii_lowercase() || c.is_ascii_digit()
                || c==b'_' || c==b'.' || c==b'-' || c==b'/' || c==b':') {return None;}
        DamageHit::new(hit.victim,hit.damage)?;
        if attacker<0 || at.iter().any(|v|!v.is_finite() || v.abs()>30_000_000.0) {return None;}
        Some(Self{dimension:dimension.to_owned(),hit,at,attacker,family})
    }
    pub fn encode(&self)->Vec<u8>{
        format!("3|{}|{}|{}|{:.4}|{:.4}|{:.4}|{}|{}",
            self.dimension,self.hit.victim,if self.hit.damage<0.00001{format!("{:e}",self.hit.damage)}else{self.hit.damage.to_string()},self.at[0],self.at[1],self.at[2],
            self.attacker,self.family as u8).into_bytes()
    }
    pub fn decode(bytes:&[u8])->Option<Self>{
        if bytes.len()<20 || bytes.len()>224 || !bytes.is_ascii(){return None;}
        let raw=std::str::from_utf8(bytes).ok()?;
        let mut fields=raw.split('|');
        if fields.next()? != "3" {return None;}
        let dimension=fields.next()?;
        fn uint(s:&str,max_len:usize)->Option<u32>{
            if s.is_empty() || s.len()>max_len || !s.bytes().all(|c|c.is_ascii_digit()) {return None;}
            s.parse().ok()
        }
        fn float(s:&str)->Option<f64>{
            if s.is_empty() || s.len()>25 || !s.bytes().all(|c|c.is_ascii_digit() || c==b'.' || c==b'-') {return None;}
            s.parse().ok()
        }
        let victim=uint(fields.next()?,10)?;
        let raw_damage=fields.next()?;
        if raw_damage.len()>32 || raw_damage.is_empty() || !raw_damage.bytes().all(|c|
            c.is_ascii_digit()||matches!(c,b'.'|b'-'|b'+'|b'e'|b'E')){return None;}
        let damage=raw_damage.parse::<f64>().ok()?;
        if !damage.is_finite(){return None;}
        if damage>f64::from(MAX_REPORTED_DAMAGE) || damage<=0.0 {return None;}
        let damage=DamageHit::new(victim,damage as f32)?;
        let at=[float(fields.next()?)?,float(fields.next()?)?,float(fields.next()?)?];
        let attacker=uint(fields.next()?,10)?;
        let attacker=i32::try_from(attacker).ok()?;
        let family=DamageFamily::from_code(uint(fields.next()?,2)? as u8)?;
        if fields.next().is_some(){return None;}
        Self::new(dimension,damage,at,attacker,family)
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test]fn damage_event_round_trip(){
        let n=DamageNotice::new("minecraft:overworld",DamageHit::new(20,27.625).unwrap(),
             [200.125,72.5,-114.25],7,DamageFamily::Fire).unwrap();
        assert_eq!(DamageNotice::decode(&n.encode()),Some(n));
    }
    #[test] fn dimension_is_mandatory() {
        let h=DamageHit::new(1,2.0).unwrap();
        assert!(DamageNotice::new("minecraft:overworld",h,[0.0,64.0,0.0],1,DamageFamily::Physical).is_some());
        assert!(DamageNotice::new("",h,[0.0,64.0,0.0],1,DamageFamily::Physical).is_none());
    }
    #[test]fn damage_is_not_rounded_away_and_large_hits_are_reported(){
        for damage in [f32::MIN_POSITIVE,0.000001,0.0123456,100_000_000.0,MAX_REPORTED_DAMAGE]{
            let n=DamageNotice::new("minecraft:overworld",DamageHit::new(20,damage).unwrap(),
                [1.0,2.0,3.0],7,DamageFamily::Physical).unwrap();
            assert_eq!(DamageNotice::decode(&n.encode()),Some(n));
        }
        assert!(DamageNotice::new("minecraft:overworld",DamageHit{victim:0,damage:10.0},
            [1.0,2.0,3.0],7,DamageFamily::Physical).is_none());
    }
    #[test]fn strict_protocol_validation(){
        for raw in ["", "2|4|2|1|2|3|4|0", "3|bad|0|2|1|2|3|4|0",
            "3|minecraft:overworld|3|NaN|1|2|3|4|0",
            "3|minecraft:overworld|3|-4|1|2|3|4|0",
            "3|minecraft:overworld|3|4|1|2|3|-4|0",
            "3|minecraft:overworld|3|4|1|2|30000001|4|0",
            "3|minecraft:overworld|3|4|1|2|3|4|255",
            "3|minecraft:overworld|3|4|1|2|3|4|0|EXTRA",
            "3|minecraft:over world|3|4|1|2|3|4|0"]{
            assert!(DamageNotice::decode(raw.as_bytes()).is_none(),"{raw}");
        }
    }
}
