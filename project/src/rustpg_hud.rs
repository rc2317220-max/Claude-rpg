//! Player and combat HUD drawing shared by Horizon and the visual lab.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudModel {
    pub vanilla_hp: (f64, f64),
    pub vitality: (f64, f64),
    pub stamina: (f64, f64),
    pub absorption: f64,
    pub xp: (u64, u64),
    pub level: u32,
    pub rank: String,
    pub exhausted: bool,
    pub critical: bool,
    pub target_visible: bool,
}
impl HudModel {
    pub fn hp_fraction(&self) -> f32 {
        if self.vanilla_hp.1 <= 0.0 {
            0.0
        } else {
            (self.vanilla_hp.0 / self.vanilla_hp.1).clamp(0.0, 1.0) as f32
        }
    }
}

use super::rustpg_bar_vfx::BarState;
use super::rustpg_boss_ui::BossPalette;
use super::rustpg_damage_meter::DamageFamily;
use super::rustpg_hud_geometry::{HudBounds, XP_BAR_HEIGHT, hp_stamina_height, vital_row_step, VITAL_BAR_WIDTH};
use super::rustpg_ui_layout::Rect;
use super::rustpg_ui_system::{Layer, alpha, fit, lighten, text_width};
use super::rustpg_ui_theme::{AETHER, AMBER, GOLD, HP, MOON, STAMINA, WHITE, XP};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect { Rect { x, y, w, h } }

pub struct Meter {
    pub fraction: f32,
    pub animation: BarState,
    pub value: String,
}

#[derive(Clone,Copy)]
enum MeterKind { Health, Stamina, Experience }

fn atlas_meter(layer: &mut Layer, area: Rect, kind: MeterKind, fraction: f32, fx: BarState) {
    let (fill, frame, accent) = match kind {
        MeterKind::Health => ("rustpg:gui/hud/health/hud_health_fill_trim","rustpg:gui/hud/health/hud_health_frame_trim", HP),
        MeterKind::Stamina => ("rustpg:gui/hud/stamina/hud_stamina_fill_trim","rustpg:gui/hud/stamina/hud_stamina_frame_trim", STAMINA),
        MeterKind::Experience => ("rustpg:gui/hud/experience/hud_experience_fill_trim","rustpg:gui/hud/experience/hud_experience_frame_trim", XP),
    };
    // The trimmed frame is 506×52px; its original fill occupies (19,14)–(487,38).
    let inner = rect(area.x + area.w*19.0/506.0, area.y + area.h*14.0/52.0,
                     area.w*468.0/506.0, area.h*24.0/52.0);
    let amount = if fx.initialized { fx.display } else { fraction }.clamp(0.0,1.0);
    layer.rect(inner, 0xE3152528);
    if fx.initialized && fx.ghost > amount {
        layer.rect(rect(inner.x+inner.w*amount, inner.y, inner.w*(fx.ghost-amount), inner.h), 0xD1B9C5BD);
    }
    if amount > 0.0 {
        layer.sprite(fill, [0.0,0.0,468.0*amount,24.0], Rect{w:inner.w*amount,..inner});
        if fx.initialized && fx.display > fx.ghost {
            layer.rect(rect(inner.x+inner.w*fx.ghost, inner.y, inner.w*(amount-fx.ghost),inner.h), 0x6657E9B3);
        }
        if fx.flash > 0.02 {
            layer.rect(rect(inner.x+inner.w*amount-1.0,inner.y,1.0,inner.h), alpha(accent,(fx.flash*180.0) as u8));
        }
    }
    layer.sprite(frame, [0.0,0.0,506.0,52.0], area);
}

pub struct PlayerVitals<'a> {
    pub level: u32,
    pub rank: &'a str,
    pub points: u64,
    pub hp: Meter,
    pub stamina: Meter,
    pub food: f32,
    pub armor: f64,
    pub air: Option<u32>,
    pub recovery: Option<f32>,
    pub relic: Option<(&'static str, &'static str)>,
}

pub fn player_width(screen: f32, bounds: HudBounds) -> f32 {
    (screen * 0.53 - 8.0).min(bounds.w).min(196.0*bounds.scale).max(116.0)
}

pub fn player(layer: &mut Layer, screen: f32, bounds: HudBounds, vitals: &PlayerVitals) {
    let x = bounds.x;
    let y = bounds.y;
    let s = bounds.scale;
    let width = player_width(screen, bounds);
    let right = x+width;
    let emblem = 7.0;
    layer.diamond_outline(x+emblem,y+emblem,emblem,1.0,GOLD,0xB229302B);
    let rank = vitals.rank.chars().next().map(String::from).unwrap_or_default();
    layer.text(&rank,x+emblem-text_width(&rank)*0.5,y+2.5,GOLD);
    let header = format!("NV {} · {} PT", vitals.level,vitals.points);
    let header_room=width-19.0-if vitals.relic.is_some(){27.0}else{0.0};
    layer.text(fit(&header,header_room),x+19.0,y+2.0,WHITE);
    if let Some((icon, status)) = vitals.relic {
        layer.sprite(icon,[0.0,0.0,128.0,128.0],rect(right-24.0,y+1.0,11.0,11.0));
        layer.text(status,right-text_width(status),y+2.0,AETHER);
    }
    let value_room = text_width(&vitals.hp.value).max(text_width(&vitals.stamina.value))
        .min(width*0.42).max(29.0);
    let bar_x=x+18.0;
    let bar_w=(width-18.0-value_room-5.0).min(VITAL_BAR_WIDTH*s);
    let value_right=bar_x+bar_w+5.0+value_room;
    let row_step=vital_row_step(s);
    for (i,(label,meter,kind)) in [("HP",&vitals.hp,MeterKind::Health),("SP",&vitals.stamina,MeterKind::Stamina)].into_iter().enumerate() {
        let row_y = y+17.0+row_step*i as f32;
        let bar_h=hp_stamina_height(s);
        let tint=match kind {
            MeterKind::Health if meter.fraction<=0.20 => HP,
            MeterKind::Stamina if vitals.recovery.is_some() => AMBER,
            _ => MOON,
        };
        layer.text(label,x,row_y,tint);
        atlas_meter(layer,rect(bar_x,row_y+(9.0-bar_h)*0.5,bar_w,bar_h),kind,meter.fraction,meter.animation);
        let value=fit(&meter.value,value_room);
        layer.text(&value,value_right-text_width(&value),row_y,WHITE);
    }
    let survival_y = y+17.0+2.0*row_step;
    layer.text("FOME",x,survival_y,0xFFF0CF8F);
    let food_w=(width*0.20).clamp(19.0,40.0);
    let food=rect(x+26.0,survival_y+3.0,food_w,3.0);
    layer.rect(food,0xCC372D23);
    layer.rect(Rect{w:food_w*vitals.food.clamp(0.0,1.0),..food},GOLD);
    let armor_x=food.x+food.w+7.0;
    layer.sprite("rustpg:gui/icons/attributes/icon_defense",[0.0,0.0,128.0,128.0],rect(armor_x,survival_y-1.0,11.0,11.0));
    layer.text(format!("{:.0}",vitals.armor.max(0.0)),armor_x+13.0,survival_y,MOON);
    if let Some(air)=vitals.air {
        let label=format!("{air}%");
        let air_x=right-text_width(&label);
        layer.sprite("rustpg:gui/icons/elements/icon_air",[0.0,0.0,128.0,128.0],rect(air_x-11.0,survival_y-1.0,10.0,10.0));
        layer.text(label,air_x,survival_y,AETHER);
    }
    if let Some(progress)=vitals.recovery {
        let recovery_y=survival_y+12.0;
        layer.text("EXAUSTO",x,recovery_y,AMBER);
        let track=rect(x+45.0,recovery_y+4.0,(width-48.0).min(66.0),2.0);
        layer.rect(track,0xCC372D23);
        layer.rect(Rect{w:track.w*progress.clamp(0.0,1.0),..track},AMBER);
    }
}

pub fn experience(layer:&mut Layer,w:f32,h:f32,scale:f32,level:u32,fraction:f32,fx:BarState) {
    let width=(w-18.0).min(216.0);
    let height=XP_BAR_HEIGHT*scale;
    let area=rect((w-width)*0.5,(h-24.0-height).max(8.0),width,height);
    atlas_meter(layer,area,MeterKind::Experience,fraction,fx);
    if h>=185.0 {
        let label=format!("NV {level} · {:.0}%",fraction*100.0);
        layer.text(&label,(w-text_width(&label))*0.5,area.y-10.0,GOLD);
    }
}

#[derive(Clone,Copy)]
pub struct CombatLayout {
    pub area: Rect,
    pub preview: Rect,
    pub column: Rect,
}
impl CombatLayout {
    pub fn new(w:f32, bounds:HudBounds, boss:bool) -> Self {
        let left = bounds.x + player_width(w,bounds) + 8.0;
        let width = if boss {(w*0.46).clamp(190.0,330.0)} else {(w*0.27).clamp(148.0,192.0)};
        let width=width.min(w-left-6.0);
        let x=if boss {((w-width)*0.5).max(left)} else {w-width-6.0};
        let y=7.0;
        let preview_w=if width<138.0 {16.0} else if boss {25.0} else {21.0};
        let area=rect(x,y,width,46.0);
        let preview=rect(x+2.0,y+5.0,preview_w,if boss{30.0}else{25.0});
        let column=rect(x+preview_w+7.0,y,width-preview_w-8.0,area.h);
        Self {area,preview,column}
    }
    pub fn telemetry(self) -> (f32,f32,f32) {
        (self.area.x,self.area.y+self.area.h+5.0,self.area.w)
    }
}

pub fn target(layer:&mut Layer,layout:CombatLayout,name:&str,metadata:&str,
              health:Option<(f32,String)>,palette:BossPalette) {
    let CombatLayout{column,preview,..}=layout;
    let c=column;
    let accent=if palette.boss {GOLD} else {AETHER};
    // Open corner marks locate the live entity without covering the world.
    for (x,y) in [(preview.x,preview.y),(preview.x+preview.w-5.0,preview.y+preview.h-1.0)] {
        layer.rect(rect(x,y,5.0,1.0),alpha(accent,0xC0));
    }
    layer.rect(rect(preview.x,preview.y,1.0,4.0),alpha(accent,0xC0));
    layer.rect(rect(preview.x+preview.w-1.0,preview.y+preview.h-4.0,1.0,4.0),alpha(accent,0xC0));
    layer.text(fit(name,c.w),c.x,c.y+2.0,WHITE);
    // Drop a complete secondary field before shortening a word. A clipped
    // "ARM…" with no value wastes the last column on a compact target HUD.
    let mut summary=metadata;
    while text_width(summary)>c.w {
        let Some((shorter,_))=summary.rsplit_once(" · ") else {break;};
        summary=shorter;
    }
    layer.text(fit(summary,c.w),c.x,c.y+13.0,if palette.boss{GOLD}else{MOON});
    let Some((fraction,hp))=health else {
        layer.text(fit("Aguardando HP",c.w),c.x,c.y+26.0,AETHER);
        return;
    };
    let track=rect(c.x,c.y+26.0,c.w,if palette.boss{5.0}else{4.0});
    layer.rect(track,0xD210211F);
    let filled=Rect{w:c.w*fraction.clamp(0.0,1.0),..track};
    layer.rect(filled,palette.fill);
    layer.rect(Rect{h:1.0,..filled},alpha(lighten(palette.fill,0.45),0xDC));
    for i in 1..palette.segments {
        layer.rect(rect(c.x+c.w*f32::from(i)/f32::from(palette.segments),track.y,1.0,track.h),0xDA0A1517);
    }
    if palette.boss {
        layer.rect(rect(c.x,track.y-1.0,c.w,1.0),alpha(palette.border,(45.0+palette.pulse*64.0) as u8));
    }
    let percent=format!("{:.0}%",fraction*100.0);
    layer.text(fit(&hp,c.w-text_width(&percent)-4.0),c.x,c.y+34.0,MOON);
    layer.text(&percent,c.x+c.w-text_width(&percent),c.y+34.0,WHITE);
}

pub fn damage_meter(layer:&mut Layer,arena:(f32,f32,f32),h:f32,value:f64,combo:u32,
                    recent:impl IntoIterator<Item=(DamageFamily,f64)>) {
    let (x,y,w)=arena;
    layer.rect(rect(x,y-3.0,w,1.0),alpha(GOLD,0x76));
    let combo_label=format!("{combo}x");
    let total=if w<105.0{format!("{value:.0} DMG")}else{format!("{value:.1} DMG")};
    layer.text(fit(&total,w-text_width(&combo_label)-4.0),x,y,GOLD);
    layer.text(&combo_label,x+w-text_width(&combo_label),y,GOLD);
    for (idx,(family,amount)) in recent.into_iter().take(2).enumerate() {
        let row_y=y+12.0*(1+idx) as f32;
        if row_y+10.0>h-25.0 { break; }
        let value=format!("{amount:.1}");
        let right=x+w-text_width(&value);
        layer.text(fit(family.label(),right-x-4.0),x,row_y,alpha(family.color(),0xDF));
        layer.text(value,right,row_y,family.color());
    }
}


/// Draw at the point projected by the host. The fixture can provide the same
/// screen coordinate without needing a live actor or camera.
pub fn damage_popup(layer: &mut Layer, point: [f32; 2], index: usize, age: u32, amount: f64, family: DamageFamily) {
    let x = point[0] + (index as f32 % 3.0 - 1.0) * 7.0;
    let y = point[1] - age as f32 * 0.75;
    let opacity = (30_u32.saturating_sub(age).min(20) * 255 / 20) as u8;
    if opacity == 0 { return; }
    let color = alpha(family.color(), opacity);
    let critical = family == DamageFamily::Critical;
    let label = if critical { format!("★ {amount:.1}") } else { format!("{amount:.1}") };
    layer.text(&label, x, y, color);

    let sparks = if critical { 6 } else { 2 };
    let radius = (2.0 + age as f32 * 0.28).min(12.0) * super::rustpg_timeline::spark(age);
    for j in 0..sparks {
        let angle = j as f32 * std::f32::consts::TAU / sparks as f32 + index as f32 * 0.77;
        let size = if j % 2 == 0 { 2.0 } else { 1.0 };
        layer.rect(rect(x + angle.cos()*radius, y + angle.sin()*radius*0.64, size, size), color);
    }
}
