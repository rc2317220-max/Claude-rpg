//! Item-local enchantment progression used by the enchanting skill tree.
//!
//! Vanilla enchantments remain in their normal component (and are therefore
//! understood by every vanilla/Horizon system). Levels that continue past the
//! engine reader's 255 ceiling are retained independently in minecraft:custom_data.
//! This makes the progression persistent, transferable with the item and
//! independent from the player profile that originally forged it.
use std::collections::BTreeMap;
use mod_api::{Compound,ItemStack,Tag};

const VIRTUAL:&str="claude_rpg:enchant_levels";
const MASTERY_SPENT:&str="claude_rpg:enchant_mastery_spent";

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

/// Persistent level accumulated by Claude RPG. When absent, callers should use
/// the ordinary enchantment component.
pub fn virtual_level(stack:&ItemStack,enchantment:&str)->Option<u64>{
    custom_map(stack,VIRTUAL).get(enchantment).copied()
}
pub fn virtual_levels(stack:&ItemStack)->BTreeMap<String,u64>{custom_map(stack,VIRTUAL)}

/// Make Horizon's ordinary enchantment reader see as much of the virtual level
/// as it can represent. Values above 255 remain in custom_data for Claude RPG's
/// own consumers instead of being truncated in the save.
pub fn expose_to_reader(event:&mut mod_api::GetEnchantmentLevelEvent<'_>){
    for (id,level) in virtual_levels(event.stack) {
        let Ok(id)=mod_api::Identifier::parse(&id) else{continue;};
        if event.target.is_some_and(|target|target!=&id){continue;}
        event.enchantments.upgrade(id,level.min(mod_api::ItemEnchantments::MAX_LEVEL as u64) as u32);
    }
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

    let mut changed=false;
    for (id,_) in level_one {
        let physical_left=left_normal.get(&id).copied().unwrap_or(0)
            .max(left_stored.get(&id).copied().unwrap_or(0));
        let base=virtuals.get(&id).copied().unwrap_or(physical_left);
        let vanilla=actual.get(&id).copied().unwrap_or(0);
        let next=if transcendence {
            Some(base.saturating_add(1))
        } else if vanilla<=base {
            let used=spent.get(&id).copied().unwrap_or(0);
            (used<mastery).then(||{
                spent.insert(id.clone(),used.saturating_add(1));
                base.saturating_add(1)
            })
        } else {
            None
        };
        let Some(next)=next else{continue;};
        virtuals.insert(id.clone(),next);
        actual.entry(id).and_modify(|v|*v=(*v).max(next.min(255)))
            .or_insert(next.min(255));
        changed=true;
    }
    changed && write_maps(output,component,&actual,&virtuals,&spent)
}

/// Extra levels above Horizon's ordinary 255-level reader. Consumers may map
/// this onto their own bounded gameplay formula without losing persisted rank.
pub fn overflow_level(stack:&ItemStack,enchantment:&str)->u64{
    virtual_level(stack,enchantment).unwrap_or(0).saturating_sub(255)
}

#[cfg(test)]
mod tests{
    use super::*;
    fn with_levels(stack:&mut ItemStack,component:&str,entries:&[(&str,u64)]){
        let mut patch=Compound::new();
        let mut levels=Compound::new();
        for (id,n) in entries {levels.insert((*id).into(),tag_number(*n));}
        patch.insert(component.into(),Tag::Compound(levels));
        stack.set_components(patch).unwrap();
    }
    #[test]fn transcendence_keeps_counting_past_the_native_reader_limit(){
        let mut left=ItemStack::of(mod_api::ItemKey(1));
        with_levels(&mut left,"minecraft:enchantments",&[("minecraft:sharpness",255)]);
        let mut book=ItemStack::of(mod_api::ItemKey(2));
        with_levels(&mut book,"minecraft:stored_enchantments",&[("minecraft:sharpness",1)]);
        let mut out=left.clone();
        assert!(apply_level_one_book(&left,&book,&mut out,false,0,true));
        assert_eq!(virtual_level(&out,"minecraft:sharpness"),Some(256));
        assert_eq!(component_map(&out,"minecraft:enchantments")["minecraft:sharpness"],255);
        left=out.clone();
        assert!(apply_level_one_book(&left,&book,&mut out,false,0,true));
        assert_eq!(virtual_level(&out,"minecraft:sharpness"),Some(257));
    }
    #[test]fn mastery_is_consumed_only_after_vanilla_stops_raising(){
        let mut left=ItemStack::of(mod_api::ItemKey(1));
        with_levels(&mut left,"minecraft:enchantments",&[("minecraft:sharpness",5)]);
        let mut book=ItemStack::of(mod_api::ItemKey(2));
        with_levels(&mut book,"minecraft:stored_enchantments",&[("minecraft:sharpness",1)]);
        let mut out=left.clone();
        assert!(apply_level_one_book(&left,&book,&mut out,false,2,false));
        assert_eq!(virtual_level(&out,"minecraft:sharpness"),Some(6));
        left=out.clone();
        assert!(apply_level_one_book(&left,&book,&mut out,false,2,false));
        assert_eq!(virtual_level(&out,"minecraft:sharpness"),Some(7));
        left=out.clone();
        assert!(!apply_level_one_book(&left,&book,&mut out,false,2,false));
    }
}
