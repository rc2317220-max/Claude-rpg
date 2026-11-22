// Safe arithmetic for very large progressions: every value stays finite and no
// integer wraps. Ranks are `u64`, enchantment levels `u32`, stat values `f64`.

/// A finite f64: NaN becomes 0, infinities become +-1e300 (far above any gameplay value).
pub fn finite(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else if v.is_infinite() {
        if v > 0.0 {
            1e300
        } else {
            -1e300
        }
    } else {
        v.clamp(-1e300, 1e300)
    }
}

/// a / b, 0 when b is 0 or the result isn't finite.
pub fn safe_div(a: f64, b: f64) -> f64 {
    if b == 0.0 || !b.is_finite() {
        return 0.0;
    }
    finite(a / b)
}

/// A float for the game (f32), clamped to f32's finite range.
pub fn to_f32(v: f64) -> f32 {
    let v = finite(v);
    v.clamp(f32::MIN as f64, f32::MAX as f64) as f32
}

/// Diminishing returns: approaches `cap` and never reaches it. `k` is the value
/// at which half the cap is reached. Used for chances and damage reduction.
pub fn diminishing(value: f64, k: f64, cap: f64) -> f64 {
    let value = finite(value).max(0.0);
    if k <= 0.0 {
        return 0.0;
    }
    finite(cap * value / (value + k))
}

/// Deterministic 64-bit hash (SplitMix64), for rolls that must not change on reload.
pub fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// A uniform roll in [0, 1) from a hash.
pub fn unit(h: u64) -> f64 {
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// A small process-wide RNG for hooks that have no game random (xorshift64*).
pub struct Rng(pub u64);
impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0.max(1);
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn chance(&mut self, p: f64) -> bool {
        let p = finite(p);
        if p <= 0.0 {
            return false;
        }
        p >= 1.0 || unit(self.next_u64()) < p
    }
    /// Chance above 100% gives guaranteed extra: 2.35 → 2 extra + 35% for a third.
    pub fn overflow_count(&mut self, p: f64) -> u64 {
        let p = finite(p).max(0.0);
        let whole = p.floor();
        let mut n = if whole >= u32::MAX as f64 {
            u32::MAX as u64
        } else {
            whole as u64
        };
        if self.chance(p - whole) {
            n = n.saturating_add(1);
        }
        n
    }
}
