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
        "minecraft:elder_guardian"|"rustpg:ashen_sentinel"|"rustpg:crystal_wyrm"|
        "rustpg:eclipse_ogre"|"rustpg:rune_colossus"|"rustpg:storm_gryphon") || {
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
    let fill=match kind {
        "rustpg:crystal_wyrm"=>match phase {1=>0xFF7DEBFF,2=>0xFF35CDEB,_=>0xFFB8F7FF},
        "rustpg:rune_colossus"=>match phase {1=>0xFF74EDCA,2=>0xFF53C7AA,_=>0xFFD1FFFA},
        "rustpg:storm_gryphon"=>match phase {1=>0xFF70C4FF,2=>0xFF478EF0,_=>0xFFC1E9FF},
        "rustpg:eclipse_ogre"=>match phase {1=>0xFFFFA24F,2=>0xFFFF6A3D,_=>0xFFFF394F},
        _=>match phase {1=>0xFFCD688C,2=>0xFFFFB665,_=>0xFFFF5B67},
    };
    let pulse=super::rustpg_timeline::boss_pulse(tick);
    BossPalette{boss:true,segments:5,phase,fill,border:0xFFEFAF92,pulse}
}
/// A real engine-rendered portrait and the HP text must NEVER overlap.
/// Dimensions are GUI pixels; the game's DrawContext owns the actual mob model.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct CombatCard {
    pub x:f32, pub y:f32, pub w:f32, pub h:f32,
    pub portrait:[f32;4],
    pub text_x:f32, pub text_w:f32,
}
pub fn card_layout(screen_width:f32,boss:bool)->Option<CombatCard> {
    if !screen_width.is_finite()||screen_width<160.0{return None;}
    let w=if boss {(screen_width*0.70).clamp(204.0,414.0).min(screen_width-12.0)}
        else {(screen_width*0.28).clamp(164.0,212.0).min(screen_width-12.0)};
    let h=if boss{82.0}else{54.0};
    let x=if boss{(screen_width-w)*0.5}else{screen_width-w-7.0};
    let y=7.0;
    let portrait_w=if boss {(w*0.23).clamp(57.0,79.0)}
        else{39.0_f32.min(w*0.26)};
    let portrait=[x+5.0,y+4.0,x+5.0+portrait_w,y+h-5.0];
    let text_x=portrait[2]+7.0;
    let text_w=(x+w-text_x-7.0).max(22.0);
    Some(CombatCard{x,y,w,h,portrait,text_x,text_w})
}
/// Separate compact damage panel below the real target, with a visible creature
/// and no opaque full-screen box. Returns [x,y,w,h] in GUI pixels.
pub fn damage_card(screen_width:f32,target_bottom:f32,scale:f32)->Option<[f32;4]> {
    if !screen_width.is_finite()||screen_width<160.0{return None;}
    let s=if scale.is_finite(){scale.clamp(0.75,1.5)}else{1.0};
    let w=(164.0*s).min((screen_width-12.0).max(0.0));
    let h=46.0*s;
    Some([screen_width-w-7.0,target_bottom+6.0,w,h])
}

#[cfg(test)] mod tests {
    use super::*;
    #[test]fn boss_card_reserves_a_real_model_viewport(){
        for width in [160.0,260.0,320.0,600.0,1920.0] {
            let boss=card_layout(width,true).unwrap();
            let ordinary=card_layout(width,false).unwrap();
            for card in [boss,ordinary] {
                assert!(card.x>=0.0 && card.x+card.w<=width);
                assert!(card.portrait[0]>=card.x && card.portrait[2]<card.text_x);
                assert!(card.text_x+card.text_w<=card.x+card.w);
                assert!(card.portrait[1]>=card.y && card.portrait[3]<=card.y+card.h);
            }
            assert!(boss.h>ordinary.h && boss.portrait[3]-boss.portrait[1]>ordinary.portrait[3]-ordinary.portrait[1]);
        }
    }
    #[test]fn damage_panel_sits_below_target_and_fits_phone(){
        for width in [260.0,320.0,480.0,960.0] {
            let target=card_layout(width,true).unwrap();
            for scale in [0.75,1.0,1.5] {
                let [x,y,w,h]=damage_card(width,target.y+target.h,scale).unwrap();
                assert!(x>=0.0 && x+w<=width && y>=target.y+target.h && h>0.0);
            }
        }
    }
    #[test]fn phases_follow_real_hp_no_fake_resource(){
        let a=style("minecraft:wither","Boss",0.9,0);
        let b=style("minecraft:wither","Boss",0.3,11);
        assert!(a.boss&&b.boss);
        assert_eq!(a.phase,1);assert_eq!(b.phase,3);
        assert_eq!(style("minecraft:cow","Normal",0.01,1).phase,1);
        assert_eq!(style("minecraft:cow","Normal",0.01,1).segments,1);
    }
}
