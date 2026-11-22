//! Claude RPG combat telemetry, inspired by the interaction patterns of
//! xypp's Damage Number 2.0.5 (MIT).  This is a clean Rust implementation;
//! no Java bytecode or Java assets are bundled or executed.
//! A record is created only from the server's final-health-damage event.
use std::collections::VecDeque;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum DamageFamily {
    Physical=0, Fire=1, Magic=2, Projectile=3, Explosion=4,
    Poison=5, Fall=6, Environment=7, Other=8, Critical=9, Wither=10,
}
impl DamageFamily {
    pub fn from_code(code:u8)->Option<Self> {
        Some(match code {
            0=>Self::Physical, 1=>Self::Fire, 2=>Self::Magic,
            3=>Self::Projectile, 4=>Self::Explosion, 5=>Self::Poison,
            6=>Self::Fall, 7=>Self::Environment, 8=>Self::Other,
            9=>Self::Critical, 10=>Self::Wither,
            _=>return None,
        })
    }
    pub fn classify(source:&str)->Self {
        let s=source.strip_prefix("minecraft:").unwrap_or(source);
        if s.contains("wither") {Self::Wither}
        else if s.contains("fire") || s=="in_fire" || s=="on_fire" || s=="lava" {Self::Fire}
        else if s.contains("magic") || s=="dragon_breath" {Self::Magic}
        else if s.contains("arrow") || s.contains("projectile") || s=="trident" || s=="thrown" {Self::Projectile}
        else if s.contains("explosion") || s=="bad_respawn_point" {Self::Explosion}
        else if s.contains("poison") || s=="sweet_berry_bush" || s=="cactus" {Self::Poison}
        else if s.contains("fall") || s=="fly_into_wall" {Self::Fall}
        else if matches!(s,"drown"|"freeze"|"starve"|"in_wall"|"outside_border"|"generic_kill"|"cramming"|"hot_floor") {Self::Environment}
        else if matches!(s,"generic"|"player_attack"|"mob_attack"|"mob_attack_no_aggro"|"sting"|"thorns") {Self::Physical}
        else {Self::Other}
    }
    pub fn label(self)->&'static str {
        match self {
            Self::Physical=>"FÍSICO", Self::Fire=>"FOGO", Self::Magic=>"MAGIA",
            Self::Projectile=>"PROJÉTIL", Self::Explosion=>"EXPLOSÃO",
            Self::Poison=>"VENENO", Self::Fall=>"QUEDA",
            Self::Environment=>"AMBIENTE", Self::Other=>"OUTROS",
            Self::Critical=>"CRÍTICO", Self::Wither=>"WITHER",
        }
    }
    pub fn color(self)->u32 {
        match self {
            Self::Physical=>0xFFF8F6F0, Self::Fire=>0xFFFF8238,
            Self::Magic=>0xFFC08CFF, Self::Projectile=>0xFF6ED6FF,
            Self::Explosion=>0xFFFFD45D, Self::Poison=>0xFF7FEA74,
            Self::Fall=>0xFFE4E7EE, Self::Environment=>0xFF9DD5E5,
            Self::Other=>0xFFCDD8E6, Self::Critical=>0xFFFFDE51,
            Self::Wither=>0xFF9763C2,
        }
    }
}

pub const COMBO_GAP_TICKS:u32=80;   // 4 seconds since last successful hit
pub const SUMMARY_TICKS:u32=36;     // keep summary visible after combo ends
pub const HISTORY_CAP:usize=6;
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct DamageEntry {pub amount:f32,pub family:DamageFamily,pub tick:u32}

#[derive(Default,Clone,Debug)]
pub struct DamageMeter {
    total:f64,
    combo:u32,
    last_hit:Option<u32>,
    recent:VecDeque<DamageEntry>,
}
impl DamageMeter {
    pub fn record(&mut self,now:u32,amount:f32,family:DamageFamily)->bool {
        // Fail-closed: no NaN, negative, 0 or extreme damage enters counters.
        if !amount.is_finite() || amount<=0.0 || amount>1_000_000.0{return false;}
        if self.last_hit.is_none_or(|last|now.wrapping_sub(last)>COMBO_GAP_TICKS) {
            self.total=0.0;
            self.combo=0;
            self.recent.clear();
        }
        self.total=(self.total+amount as f64).min(1_000_000_000.0);
        self.combo=self.combo.saturating_add(1);
        self.last_hit=Some(now);
        self.recent.push_back(DamageEntry{amount,family,tick:now});
        while self.recent.len()>HISTORY_CAP {self.recent.pop_front();}
        true
    }
    pub fn clear(&mut self){*self=Self::default();}
    pub fn active(&self,now:u32)->bool {
        self.last_hit.is_some_and(|last|now.wrapping_sub(last)<=COMBO_GAP_TICKS+SUMMARY_TICKS)
    }
    pub fn ongoing(&self,now:u32)->bool {
        self.last_hit.is_some_and(|last|now.wrapping_sub(last)<=COMBO_GAP_TICKS)
    }
    pub fn total(&self)->f64{self.total}
    pub fn combo(&self)->u32{self.combo}
    pub fn recent(&self)->impl DoubleEndedIterator<Item=&DamageEntry>{self.recent.iter()}
    pub fn age(&self,now:u32)->Option<u32>{self.last_hit.map(|t|now.wrapping_sub(t))}
}

#[cfg(test)] mod tests {
    use super::*;
    #[test]fn classifies_actual_sources(){
        assert_eq!(DamageFamily::classify("minecraft:player_attack"),DamageFamily::Physical);
        assert_eq!(DamageFamily::classify("minecraft:on_fire"),DamageFamily::Fire);
        assert_eq!(DamageFamily::classify("minecraft:magic"),DamageFamily::Magic);
        assert_eq!(DamageFamily::classify("minecraft:arrow"),DamageFamily::Projectile);
        assert_eq!(DamageFamily::classify("minecraft:wither"),DamageFamily::Wither);
        assert_eq!(DamageFamily::classify("minecraft:wither_skull"),DamageFamily::Wither);
    }
    #[test]fn combo_expiration_and_summary(){
        let mut m=DamageMeter::default();
        assert!(m.record(0,4.0,DamageFamily::Physical));
        assert!(m.record(40,3.5,DamageFamily::Fire));
        assert_eq!(m.combo(),2);assert!((m.total()-7.5).abs()<1e-6);
        assert!(m.active(150));assert!(!m.ongoing(150));
        assert!(!m.active(157));
        assert!(m.record(170,2.0,DamageFamily::Magic));
        assert_eq!(m.combo(),1); assert_eq!(m.total(),2.0);
    }
    #[test]fn rejects_overflow_and_bounds_history(){
        let mut m=DamageMeter::default();
        for bad in [0.0,-5.0,f32::INFINITY,f32::NAN,1_000_001.0] {
            assert!(!m.record(1,bad,DamageFamily::Other));
        }
        for i in 0..64 {assert!(m.record(i,1.0,DamageFamily::Physical));}
        assert_eq!(m.combo(),64); assert_eq!(m.recent().count(),HISTORY_CAP);
    }
    #[test]fn tick_wrap_is_safe(){
        let mut m=DamageMeter::default();
        m.record(u32::MAX-2,10.0,DamageFamily::Physical);
        assert!(m.active(4));
        assert_eq!(m.age(4),Some(7));
    }
}
