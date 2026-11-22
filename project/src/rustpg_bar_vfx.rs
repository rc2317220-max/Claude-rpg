//! Local-only bar animation: authoritative values are never modified.
//! The leading fill eases toward the new value; its afterimage trails behind.
#[derive(Clone,Copy,Debug,Default,PartialEq)]
pub struct BarState {
    pub display:f32,
    pub ghost:f32,
    pub flash:f32,
    pub direction:i8,
    pub initialized:bool,
}
impl BarState {
    pub fn step(&mut self,target:f32){
        let target=if target.is_finite(){target.clamp(0.0,1.0)}else{0.0};
        if !self.initialized{
            *self=Self{display:target,ghost:target,flash:0.0,direction:0,initialized:true};
            return;
        }
        let difference=target-self.display;
        if difference.abs()>0.006 {
            if target>self.ghost+0.012 || target<self.display-0.012 {
                self.direction=if difference>0.0{1}else{-1};
                self.flash=1.0;
            }
        }
        // Fast primary, delayed second layer; no fabricated HP values.
        // Foreground is always exactly the latest authoritative ratio.
        self.display=target;
        self.ghost+=(target-self.ghost)*0.085;
        if (self.display-target).abs()<0.002{self.display=target;}
        if (self.ghost-target).abs()<0.002{self.ghost=target;}
        self.flash=(self.flash-0.09).max(0.0);
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn damage_and_regen_ease_without_overshooting(){
        let mut b=BarState::default();b.step(1.0);
        b.step(0.4);assert_eq!(b.display,0.4);assert!(b.ghost>b.display);
        for _ in 0..100{b.step(0.4)}
        assert!((b.display-0.4).abs()<0.003);
        b.step(0.8);assert_eq!(b.display,0.8);
    }
    #[test]fn lethal_hit_goes_exactly_to_zero(){
        let mut b=BarState::default();b.step(1.0);b.step(0.0);
        assert_eq!(b.display,0.0);
        assert!(b.ghost>0.0);
        for _ in 0..100 {b.step(0.0);}
        assert_eq!(b.ghost,0.0);
    }
    #[test]fn resets_first_sample_without_fake_damage(){
        let mut b=BarState::default();b.step(0.25);
        assert_eq!(b.display,0.25);assert_eq!(b.flash,0.0);
    }
}
