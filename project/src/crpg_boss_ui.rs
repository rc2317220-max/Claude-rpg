//! Independent boss-bar styling: informed by the uploaded Enhanced Boss Bars
//! resource pack and Remnant Bosses' phased bars; no external textures copied.
//! The client receives HP through the existing Starling authoritative target.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct BossPalette {
    pub boss:bool,
    pub segments:u8,
    pub phase:u8,
    pub fill:u32,
    pub border:u32,
    pub pulse:f32,
}
fn is_boss(kind:&str,tier:&str)->bool {
    matches!(kind,"minecraft:ender_dragon"|"minecraft:wither"|"minecraft:warden"|
        "minecraft:elder_guardian") || {
        let lower=tier.to_lowercase();
        lower.contains("chefe") || lower.contains("lendário") || lower.contains("boss")
    }
}
pub fn style(kind:&str,tier:&str,hp_ratio:f32,tick:u32)->BossPalette {
    let boss=is_boss(kind,tier);
    let hp=if hp_ratio.is_finite(){hp_ratio.clamp(0.0,1.0)}else{0.0};
    if !boss {return BossPalette{boss:false,segments:1,phase:1,
        fill:0xFF65D88D,border:0xFF8AA8B0,pulse:0.0};}
    let phase=if hp>0.67{1}else if hp>0.33{2}else{3};
    let fill=match phase {1=>0xFFCD688C,2=>0xFFFFB665,_=>0xFFFF5B67};
    let pulse=super::crpg_timeline::boss_pulse(tick);
    BossPalette{boss:true,segments:5,phase,fill,border:0xFFEFAF92,pulse}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test]fn phases_follow_real_hp_no_fake_resource(){
        let a=style("minecraft:wither","Boss",0.9,0);
        let b=style("minecraft:wither","Boss",0.3,11);
        assert!(a.boss&&b.boss);
        assert_eq!(a.phase,1);assert_eq!(b.phase,3);
        assert_eq!(style("minecraft:cow","Normal",0.01,1).phase,1);
        assert_eq!(style("minecraft:cow","Normal",0.01,1).segments,1);
    }
}
