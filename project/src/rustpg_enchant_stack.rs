//! Item-local enchantment progression used by the enchanting skill tree.
//!
//! Vanilla enchantments remain in their normal component (and are therefore
//! understood by every vanilla/Horizon system). Levels that continue past the
//! engine reader's 255 ceiling are retained independently in minecraft:custom_data.
//! This makes the progression persistent, transferable with the item and
//! independent from the player profile that originally forged it.
use std::collections::BTreeMap;
use mod_api::{Compound,ItemStack,Tag};

const VIRTUAL:&str="rustpg:enchant_levels";
const MASTERY_SPENT:&str="rustpg:enchant_mastery_spent";

fn number(tag:&Tag)->Option<u64>{
    match tag {
        Tag::Byte(v)=>u64::try_from(*v).ok(),
        Tag::Short(v)=>u64::try_from(*v).ok(),
        Tag::Int(v)=>u64::try_from(*v).ok(),
        Tag::Long(v)=>u64::try_from(*v).ok(),
        _=>None,
    }
}
fn tag_number(value:u64)->Tag{
    i32::try_from(value).map_or_else(|_|Tag::Long(value.min(i64::MAX as u64) as i64),Tag::Int)
}
fn component_map(stack:&ItemStack,id:&str)->BTreeMap<String,u64>{
    let Some(patch)=stack.components().map(|c|c.as_nbt()) else{return BTreeMap::new();};
    let Some(Tag::Compound(levels))=patch.get(&id.into()) else{return BTreeMap::new();};
    levels.iter().filter_map(|(id,level)|
        number(level).filter(|v|*v>0).map(|v|(id.to_string_lossy(),v)))
        .collect()
}
fn custom_map(stack:&ItemStack,id:&str)->BTreeMap<String,u64>{
    let Some(patch)=stack.components().map(|c|c.as_nbt()) else{return BTreeMap::new();};
    let Some(Tag::Compound(custom))=patch.get(&"minecraft:custom_data".into()) else{
        return BTreeMap::new();
    };
    let Some(Tag::Compound(levels))=custom.get(&id.into()) else{return BTreeMap::new();};
    levels.iter().filter_map(|(id,level)|
        number(level).map(|v|(id.to_string_lossy(),v))).collect()
}
fn set_compound_map(root:&mut Compound,id:&str,values:&BTreeMap<String,u64>){
    let levels=values.iter().filter(|(_,v)|**v>0)
        .map(|(id,v)|(id.as_str().into(),tag_number(*v))).collect::<Compound>();
    root.insert(id.into(),Tag::Compound(levels));
}
fn write_maps(stack:&mut ItemStack,component:&str,
    actual:&BTreeMap<String,u64>,virtuals:&BTreeMap<String,u64>,
    spent:&BTreeMap<String,u64>)->bool {
    let mut patch=stack.components().map(|c|c.as_nbt().clone()).unwrap_or_default();
    set_compound_map(&mut patch,component,actual);
    let mut custom=match patch.get(&"minecraft:custom_data".into()) {
        Some(Tag::Compound(c))=>c.clone(),
        _=>Compound::new(),
    };
    set_compound_map(&mut custom,VIRTUAL,virtuals);
    set_compound_map(&mut custom,MASTERY_SPENT,spent);
    patch.insert("minecraft:custom_data".into(),Tag::Compound(custom));
    stack.set_components(patch).is_ok()
}

/// Persistent level accumulated by RustPG. When absent, callers should use
/// the ordinary enchantment component.
pub fn virtual_level(stack:&ItemStack,enchantment:&str)->Option<u64>{
    custom_map(stack,VIRTUAL).get(enchantment).copied()
}
pub fn virtual_levels(stack:&ItemStack)->BTreeMap<String,u64>{custom_map(stack,VIRTUAL)}

/// Make Horizon's ordinary enchantment reader see as much of the virtual level
/// as it can represent. Values above 255 remain in custom_data for RustPG's
/// own consumers instead of being truncated in the save.
pub fn expose_to_reader(event:&mut mod_api::GetEnchantmentLevelEvent<'_>){
    for (id,level) in virtual_levels(event.stack) {
        let Ok(id)=mod_api::Identifier::parse(&id) else{continue;};
        if event.target.is_some_and(|target|target!=&id){continue;}
        event.enchantments.upgrade(id,level.min(mod_api::ItemEnchantments::MAX_LEVEL as u64) as u32);
    }
}

fn advance_level_one(
    level_one:&[(String,u64)],
    left_normal:&BTreeMap<String,u64>,
    left_stored:&BTreeMap<String,u64>,
    actual:&mut BTreeMap<String,u64>,
    virtuals:&mut BTreeMap<String,u64>,
    spent:&mut BTreeMap<String,u64>,
    mastery:u64,
    transcendence:bool,
)->bool {
    let mut changed=false;
    for (id,_) in level_one {
        let physical_left=left_normal.get(id).copied().unwrap_or(0)
            .max(left_stored.get(id).copied().unwrap_or(0));
        let base=virtuals.get(id).copied().unwrap_or(physical_left);
        // The anvil may accept only part of a multi-enchantment book.
        // A missing enchantment in its actual output was not accepted.
        let Some(vanilla)=actual.get(id).copied().filter(|v|*v>0) else{continue;};
        let next=if transcendence {
            Some(base.saturating_add(1))
        } else if vanilla<=base {
            let used=spent.get(id).copied().unwrap_or(0);
            (used<mastery).then(||{
                spent.insert(id.clone(),used.saturating_add(1));
                base.saturating_add(1)
            })
        } else {
            None
        };
        let Some(next)=next else{continue;};
        virtuals.insert(id.clone(),next);
        actual.entry(id.clone()).and_modify(|v|*v=(*v).max(next.min(255)))
            .or_insert(next.min(255));
        changed=true;
    }
    changed
}

/// Apply one successful level-I enchanted-book combination.
///
/// mastery is a finite number of cap bypasses after vanilla stops raising a
/// particular enchantment. transcendence removes that finite bypass limit.
/// The caller still requires vanilla to have produced an output, so this does
/// not make incompatible items/enchantments compatible and does not invent
/// recipes.
pub fn apply_level_one_book(left:&ItemStack,right:&ItemStack,output:&mut ItemStack,
    output_is_book:bool,mastery:u64,transcendence:bool)->bool {
    let right_levels=component_map(right,"minecraft:stored_enchantments");
    let level_one=right_levels.into_iter().filter(|(_,level)|*level==1).collect::<Vec<_>>();
    if level_one.is_empty() || (!transcendence && mastery==0){return false;}

    let component=if output_is_book{"minecraft:stored_enchantments"}else{"minecraft:enchantments"};
    let left_normal=component_map(left,"minecraft:enchantments");
    let left_stored=component_map(left,"minecraft:stored_enchantments");
    let mut actual=component_map(output,component);
    let mut virtuals=custom_map(left,VIRTUAL);
    for (id,value) in custom_map(output,VIRTUAL) {
        virtuals.entry(id).and_modify(|v|*v=(*v).max(value)).or_insert(value);
    }
    let mut spent=custom_map(left,MASTERY_SPENT);
    for (id,value) in custom_map(output,MASTERY_SPENT) {
        spent.entry(id).and_modify(|v|*v=(*v).max(value)).or_insert(value);
    }

    let changed=advance_level_one(
        &level_one,&left_normal,&left_stored,&mut actual,&mut virtuals,&mut spent,
        mastery,transcendence);
    changed && write_maps(output,component,&actual,&virtuals,&spent)
}

/// Extra levels above Horizon's ordinary 255-level reader. Consumers may map
/// this onto their own bounded gameplay formula without losing persisted rank.
pub fn overflow_level(stack:&ItemStack,enchantment:&str)->u64{
    virtual_level(stack,enchantment).unwrap_or(0).saturating_sub(255)
}

/// Starling's vanilla reader sees at most 255 levels. The complete item
/// rank is still stored in custom_data, and every additional Sharpness rank
/// contributes 0.5 final HP damage on a *server-confirmed melee hit*.
////
//// The stored rank can grow indefinitely; the effective damage is clamped
//// to avoid infinite floats and integer overflows in the engine.
pub fn sharpness_overflow_damage(base:f32,overflow:u64)->f32 {
    if !base.is_finite() || base<=0.0 || overflow==0 {return base;}
    let extra=overflow.min(4096) as f64 * 0.5;
    (f64::from(base)+extra).min(1.0e9) as f32
}

#[cfg(test)]
mod tests{
    use super::*;
    fn map(entries:&[(&str,u64)])->BTreeMap<String,u64>{
        entries.iter().map(|(k,v)|((*k).to_owned(),*v)).collect()
    }
    #[test]fn sharpness_beyond_255_affects_melee_without_breaking_native_limits(){
        assert_eq!(sharpness_overflow_damage(10.0,0),10.0);
        assert_eq!(sharpness_overflow_damage(10.0,1),10.5);
        assert_eq!(sharpness_overflow_damage(10.0,1000),510.0);
        assert_eq!(sharpness_overflow_damage(10.0,u64::MAX),2058.0);
        assert_eq!(sharpness_overflow_damage(f32::NAN,1).is_nan(),true);
        assert_eq!(sharpness_overflow_damage(0.0,100),0.0);
    }
    #[test]fn transcendence_keeps_counting_past_the_native_reader_limit(){
        let level_one=vec![("minecraft:sharpness".to_owned(),1)];
        let normal=map(&[("minecraft:sharpness",255)]);
        let stored=BTreeMap::new();
        let mut actual=normal.clone();
        let mut virtuals=BTreeMap::new();
        let mut spent=BTreeMap::new();
        assert!(advance_level_one(&level_one,&normal,&stored,&mut actual,&mut virtuals,
            &mut spent,0,true));
        assert_eq!(virtuals["minecraft:sharpness"],256);
        assert_eq!(actual["minecraft:sharpness"],255);
        let normal=actual.clone();
        assert!(advance_level_one(&level_one,&normal,&stored,&mut actual,&mut virtuals,
            &mut spent,0,true));
        assert_eq!(virtuals["minecraft:sharpness"],257);
        assert_eq!(actual["minecraft:sharpness"],255);
    }
    #[test]fn partially_accepted_book_cannot_insert_rejected_enchantment(){
        let one=vec![("minecraft:sharpness".to_owned(),1),("minecraft:smite".to_owned(),1)];
        let normal=map(&[("minecraft:sharpness",5)]);
        let stored=BTreeMap::new();
        for transcendence in [false,true] {
            let mut actual=normal.clone();
            let mut virtuals=BTreeMap::new();
            let mut spent=BTreeMap::new();
            assert!(advance_level_one(&one,&normal,&stored,&mut actual,&mut virtuals,
                &mut spent,2,transcendence));
            assert_eq!(virtuals["minecraft:sharpness"],6);
            assert!(!actual.contains_key("minecraft:smite"));
            assert!(!virtuals.contains_key("minecraft:smite"));
            assert!(!spent.contains_key("minecraft:smite"));
        }
    }
    #[test]fn mastery_is_consumed_only_after_vanilla_stops_raising(){
        let level_one=vec![("minecraft:sharpness".to_owned(),1)];
        let normal=map(&[("minecraft:sharpness",5)]);
        let stored=BTreeMap::new();
        let mut actual=normal.clone();
        let mut virtuals=BTreeMap::new();
        let mut spent=BTreeMap::new();
        assert!(advance_level_one(&level_one,&normal,&stored,&mut actual,&mut virtuals,
            &mut spent,2,false));
        assert_eq!(virtuals["minecraft:sharpness"],6);
        assert_eq!(spent["minecraft:sharpness"],1);
        let normal=actual.clone();
        assert!(advance_level_one(&level_one,&normal,&stored,&mut actual,&mut virtuals,
            &mut spent,2,false));
        assert_eq!(virtuals["minecraft:sharpness"],7);
        assert_eq!(spent["minecraft:sharpness"],2);
        let normal=actual.clone();
        assert!(!advance_level_one(&level_one,&normal,&stored,&mut actual,&mut virtuals,
            &mut spent,2,false));
    }
}
