// Target HUD data and selection primitives.
// The Horizon adapter supplies rendered entity snapshots; no fake last-attacked
// target is used.

#[derive(Clone, Debug, PartialEq)]
pub struct TargetSnapshot {
    pub entity: u32,
    pub kind: String,
    pub name: String,
    pub level: u32,
    pub rank: String,
    pub current_hp: f64,
    pub max_hp: f64,
    pub armor: f64,
    pub tier: String,
    pub mutations: Vec<String>,
    pub badges: Vec<String>,
}

impl TargetSnapshot {
    /// Normalizes entity data before it reaches the HUD. Runtime data may be
    /// incomplete while an entity is joining or leaving the rendered frame.
    pub fn sanitized(mut self) -> Self {
        self.name = if self.name.trim().is_empty() {
            "ENTIDADE".into()
        } else {
            self.name.trim().to_string()
        };
        self.kind = self.kind.trim().to_string();
        self.level = self.level.max(1);
        self.max_hp = if self.max_hp.is_finite() && self.max_hp > 0.0 {
            self.max_hp
        } else {
            1.0
        };
        self.armor = if self.armor.is_finite() {self.armor.clamp(0.0,1024.0)}else{0.0};
        self.current_hp = if self.current_hp.is_finite() {
            self.current_hp.clamp(0.0, self.max_hp)
        } else {
            0.0
        };
        self.rank = if self.rank.trim().is_empty() {
            "NORMAL".into()
        } else {
            self.rank.trim().to_string()
        };
        self.tier = if self.tier.trim().is_empty() {
            self.rank.clone()
        } else {
            self.tier.trim().to_string()
        };
        self.mutations.retain(|x| !x.trim().is_empty());
        self.badges.retain(|x| !x.trim().is_empty());
        self
    }

    pub fn fraction(&self) -> f32 {
        if self.max_hp <= 0.0 {
            0.0
        } else {
            (self.current_hp / self.max_hp).clamp(0.0, 1.0) as f32
        }
    }
}

/// Selects the closest rendered entity to the crosshair. The renderer is the
/// authoritative source available to this revision: unlike combat history, it
/// only considers entities visible in the current frame.
pub fn select_crosshair_entity(
    visible: &[(u32, [f32; 2])],
    viewport: [f32; 2],
    radius_fraction: f32,
) -> Option<u32> {
    let center = [viewport[0] * 0.5, viewport[1] * 0.5];
    let radius = viewport[0].min(viewport[1]).max(0.0) * radius_fraction.max(0.0);
    let radius_sq = radius * radius;
    visible
        .iter()
        .filter_map(|(id, p)| {
            if *id == 0 || !p[0].is_finite() || !p[1].is_finite() {
                return None;
            }
            let dx = p[0] - center[0];
            let dy = p[1] - center[1];
            let distance = dx * dx + dy * dy;
            (distance <= radius_sq).then_some((*id, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Slab intersection for a future entity-ray path. Kept independent of the
/// Horizon adapter so it can be used if the pinned revision exposes entity
/// bounds directly.
pub fn ray_aabb(
    origin: [f64; 3],
    direction: [f64; 3],
    min: [f64; 3],
    max: [f64; 3],
    reach: f64,
) -> Option<f64> {
    if !reach.is_finite() || reach < 0.0 {
        return None;
    }
    let mut near: f64 = 0.0;
    let mut far: f64 = reach;
    for i in 0..3 {
        if !origin[i].is_finite()
            || !direction[i].is_finite()
            || !min[i].is_finite()
            || !max[i].is_finite()
        {
            return None;
        }
        if direction[i].abs() < 1e-9 {
            if origin[i] < min[i] || origin[i] > max[i] {
                return None;
            }
            continue;
        }
        let mut a = (min[i] - origin[i]) / direction[i];
        let mut b = (max[i] - origin[i]) / direction[i];
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        near = near.max(a);
        far = far.min(b);
        if near > far {
            return None;
        }
    }
    (near <= reach && far >= 0.0).then_some(near.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_hit_distance() {
        assert_eq!(
            ray_aabb(
                [0.0, 0.5, -5.0],
                [0.0, 0.0, 1.0],
                [-1.0, 0.0, 0.0],
                [1.0, 2.0, 1.0],
                100.0
            ),
            Some(5.0)
        );
    }

    #[test]
    fn misses_parallel_box() {
        assert!(ray_aabb(
            [3.0, 0.5, -5.0],
            [0.0, 0.0, 1.0],
            [-1.0, 0.0, 0.0],
            [1.0, 2.0, 1.0],
            100.0
        )
        .is_none());
    }

    #[test]
    fn rejects_non_finite_ray_inputs() {
        assert!(ray_aabb(
            [0.0, 0.0, 0.0],
            [f64::NAN, 0.0, 1.0],
            [-1.0, -1.0, -1.0],
            [1.0, 1.0, 1.0],
            10.0
        )
        .is_none());
    }

    #[test]
    fn crosshair_picks_nearest_visible_entity() {
        let visible = vec![
            (11, [180.0, 120.0]),
            (22, [191.0, 120.0]),
            (33, [400.0, 120.0]),
        ];
        assert_eq!(
            select_crosshair_entity(&visible, [380.0, 240.0], 0.22),
            Some(22)
        );
    }

    #[test]
    fn crosshair_rejects_invalid_points_and_zero_ids() {
        let visible = vec![(0, [190.0, 120.0]), (7, [f32::NAN, 120.0])];
        assert_eq!(
            select_crosshair_entity(&visible, [380.0, 240.0], 0.22),
            None
        );
    }

    #[test]
    fn snapshot_sanitizes_bad_runtime_values() {
        let target = TargetSnapshot {
            entity: 4,
            kind: "  minecraft:zombie ".into(),
            name: "  ".into(),
            level: 0,
            rank: "".into(),
            current_hp: f64::NAN,
            max_hp: -1.0,
            armor: f64::NAN,
            tier: "".into(),
            mutations: vec!["".into(), "Glacial".into()],
            badges: vec!["".into(), "Épico".into()],
        }
        .sanitized();
        assert_eq!(target.name, "ENTIDADE");
        assert_eq!(target.level, 1);
        assert_eq!(target.current_hp, 0.0);
        assert_eq!(target.max_hp, 1.0);
        assert_eq!(target.armor, 0.0);
        assert_eq!(target.mutations, vec!["Glacial"]);
        assert_eq!(target.badges, vec!["Épico"]);
    }
}
