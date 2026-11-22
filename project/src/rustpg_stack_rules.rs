//! Global pre-progression stacking baseline, applied to real item defaults.
//! This does NOT change a specific player's inventory cap; that requires an
//! engine-level player-aware stack merge/pickup hook.

/// Divide ordinary 64 and 16 stacks by four, keeping special 1-stack items.
pub fn initial_capacity(native:u64)->u64 {
    match native { 64=>16, 16=>4, other=>other }
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn vanilla_stack_classes(){
        assert_eq!(initial_capacity(64),16);
        assert_eq!(initial_capacity(16),4);
        assert_eq!(initial_capacity(1),1);
        assert_eq!(initial_capacity(99),99);
    }
}
