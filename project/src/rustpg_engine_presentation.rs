// Presentation rules shared by the HUD and the RPG screen.
// No Horizon types live here: these functions are deterministic and testable.

use super::rustpg_profile::Profile;

/// Hunter rank derived from RPG progress, never from vanilla health.
/// The score intentionally reuses Profile::power(), which includes level,
/// combat proficiencies, invested combat nodes, boss kills and dimensions.
pub fn hunter_rank(power: f64) -> &'static str {
    let p = if power.is_finite() {
        power.max(0.0)
    } else {
        0.0
    };
    match p {
        p if p < 10.0 => "E",
        p if p < 25.0 => "D",
        p if p < 50.0 => "C",
        p if p < 100.0 => "B",
        p if p < 180.0 => "A",
        p if p < 300.0 => "S",
        _ => "Monarca",
    }
}

pub fn profile_rank(profile: &Profile) -> &'static str {
    hunter_rank(profile.power())
}

/// Rank bands, in the same order the HUD ladder draws them.
pub const RANK_BANDS: [(&str, f64); 7] = [
    ("E", 0.0),
    ("D", 10.0),
    ("C", 25.0),
    ("B", 50.0),
    ("A", 100.0),
    ("S", 180.0),
    ("Monarca", 300.0),
];

/// The next rank and how far the hunter already walked inside the current band.
/// Returns `None` for the next rank when the profile already sits at the top.
pub fn rank_progress(power: f64) -> (Option<&'static str>, f32) {
    let p = if power.is_finite() {
        power.max(0.0)
    } else {
        0.0
    };
    for pair in RANK_BANDS.windows(2) {
        let (_, low) = pair[0];
        let (next, high) = pair[1];
        if p < high {
            let t = ((p - low) / (high - low)).clamp(0.0, 1.0) as f32;
            return (Some(next), t);
        }
    }
    (None, 1.0)
}

pub fn ratio(current: f64, maximum: f64) -> f32 {
    if !current.is_finite() || !maximum.is_finite() || maximum <= 0.0 {
        return 0.0;
    }
    (current / maximum).clamp(0.0, 1.0) as f32
}

pub fn stamina_label(profile: &Profile) -> String {
    format!(
        "VIGOR {:.0}/{:.0}",
        profile.stamina.current.max(0.0),
        profile.stamina.max.max(0.0)
    )
}

pub fn vitality_label(profile: &Profile) -> String {
    format!(
        "VITALIDADE {:.0}/{:.0}",
        profile.vitality.current.max(0.0),
        profile.vitality.max.max(1.0)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_uses_power_boundaries() {
        assert_eq!(hunter_rank(0.0), "E");
        assert_eq!(hunter_rank(9.99), "E");
        assert_eq!(hunter_rank(10.0), "D");
        assert_eq!(hunter_rank(25.0), "C");
        assert_eq!(hunter_rank(50.0), "B");
        assert_eq!(hunter_rank(100.0), "A");
        assert_eq!(hunter_rank(180.0), "S");
        assert_eq!(hunter_rank(300.0), "Monarca");
    }

    #[test]
    fn rank_does_not_read_vanilla_health() {
        let mut p = Profile::new();
        p.vitality.max = 10_000.0;
        assert_eq!(profile_rank(&p), "E");
        p.level = 100;
        assert_ne!(profile_rank(&p), "E");
    }

    #[test]
    fn ratios_and_labels_are_safe() {
        assert_eq!(ratio(5.0, 10.0), 0.5);
        assert_eq!(ratio(5.0, 0.0), 0.0);
        let p = Profile::new();
        assert!(stamina_label(&p).contains('/'));
        assert!(vitality_label(&p).contains('/'));
    }

    #[test]
    fn rank_progress_walks_inside_the_band() {
        assert_eq!(rank_progress(0.0), (Some("D"), 0.0));
        assert_eq!(rank_progress(5.0), (Some("D"), 0.5));
        let (next, t) = rank_progress(30.0);
        assert_eq!(next, Some("B"));
        assert!((t - 0.2).abs() < 0.001, "{t}");
        assert_eq!(rank_progress(400.0), (None, 1.0));
        // NaN and negatives never escape the first band.
        assert_eq!(rank_progress(f64::NAN).0, Some("D"));
        assert_eq!(rank_progress(-10.0).1, 0.0);
    }
}
