//! Server-checked melee reach and basic collision primitives.
//! These primitives are usable by future swept/area attacks; no extra
//! damage is applied unless Starling has confirmed an actual attack.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Aabb {pub min:[f64;3],pub max:[f64;3]}
impl Aabb {
    pub fn valid(self)->bool {
        (0..3).all(|i|self.min[i].is_finite()&&self.max[i].is_finite()&&self.min[i]<=self.max[i])
    }
    pub fn intersects(self,other:Self)->bool{
        self.valid()&&other.valid()&&(0..3).all(|i|self.min[i]<=other.max[i]&&self.max[i]>=other.min[i])
    }
    pub fn closest(self,point:[f64;3])->Option<[f64;3]>{
        (self.valid() && point.iter().all(|x|x.is_finite()))
            .then(||std::array::from_fn(|i|point[i].clamp(self.min[i],self.max[i])))
    }
}
pub fn within_melee_reach(attacker:[f64;3],victim:[f64;3],reach:f64)->bool {
    if attacker.iter().chain(victim.iter()).any(|x|!x.is_finite()) || !reach.is_finite(){return false}
    let r=reach.clamp(0.0,7.0);
    let [dx,dy,dz]=[attacker[0]-victim[0],attacker[1]-victim[1],attacker[2]-victim[2]];
    dx*dx+dz*dz<=(r+1.0).powi(2)&&dy.abs()<=3.5
}
pub fn intersects_sweep(a:Aabb,b:Aabb,from:[f64;3],to:[f64;3])->bool {
    if !a.valid()||!b.valid()||from.iter().chain(to.iter()).any(|x|!x.is_finite()){return false}
    // Conservative swept broad phase. The narrow phase is left to the engine.
    let swept=Aabb{min:std::array::from_fn(|i|a.min[i]+from[i].min(to[i])),
                   max:std::array::from_fn(|i|a.max[i]+from[i].max(to[i]))};
    swept.intersects(b)
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn rejects_far_and_invalid_melee(){
        assert!(within_melee_reach([0.0;3],[2.5,0.0,1.0],4.0));
        assert!(!within_melee_reach([0.0;3],[99.0,0.0,0.0],4.0));
        assert!(!within_melee_reach([f64::NAN,0.0,0.0],[0.0;3],4.0));
    }
    #[test]fn swept_broad_phase_handles_motion(){
        let a=Aabb{min:[0.0;3],max:[1.0;3]};
        let b=Aabb{min:[2.0,0.0,0.0],max:[3.0,1.0,1.0]};
        assert!(intersects_sweep(a,b,[0.0;3],[2.0,0.0,0.0]));
        assert!(!intersects_sweep(a,b,[0.0;3],[0.1,0.0,0.0]));
    }
}
