use crate::stats::StatBlock;
use crate::types::DamageType;

/// The stages of damage calculation pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageStage {
    /// Initial base damage + ratios
    Raw,
    /// Flat resistance reduction
    FlatReduction,
    /// Percentage resistance reduction
    PercentReduction,
    /// Percentage penetration
    PercentPenetration,
    /// Flat penetration
    FlatPenetration,
    /// Damage reduction from effective resistance
    Mitigation,
    /// Damage absorption from shields
    ShieldAbsorption,
    /// Actual damage dealt to HP
    Final,
}

/// Represents the details of a damage instance.
#[derive(Debug, Clone)]
pub struct DamageResult {
    /// The damage before any mitigation.
    pub raw_damage: f64,
    /// The damage after resistance mitigation.
    pub mitigated_damage: f64,
    /// The final damage applied after all modifiers and shields.
    pub final_damage: f64,
    /// The type of damage dealt.
    pub damage_type: DamageType,
    /// Whether the damage was a critical strike.
    pub is_critical: bool,
}

/// Computes the effective resistance given target's resistance, flat/percent reduction, and flat/percent penetration.
pub fn effective_resistance(
    target_resist: f64,
    flat_reduction: f64,
    percent_reduction: f64,
    percent_penetration: f64,
    flat_penetration: f64,
) -> f64 {
    // 1. Flat reduction
    let mut res = target_resist - flat_reduction;

    // 2. Percent reduction (only applies if > 0)
    if res > 0.0 {
        res *= 1.0 - percent_reduction;
    }

    let is_positive_before_pen = res > 0.0;

    // 3. Percent penetration (only applies if > 0)
    if res > 0.0 {
        res *= 1.0 - percent_penetration;
    }

    // 4. Flat penetration (only applies if > 0)
    if res > 0.0 {
        res -= flat_penetration;
    }

    // Penetration cannot reduce resistance below 0 if it was positive.
    if is_positive_before_pen && res < 0.0 {
        0.0
    } else {
        res
    }
}

/// Applies resistance damage mitigation formula.
pub fn apply_resistance(damage: f64, effective_res: f64) -> f64 {
    if effective_res >= 0.0 {
        damage * (100.0 / (100.0 + effective_res))
    } else {
        damage * (2.0 - (100.0 / (100.0 - effective_res)))
    }
}

/// Core pipeline for calculating damage from an attacker to a defender.
pub struct DamagePipeline;

impl DamagePipeline {
    /// Processes a single damage instance through the standard League of Legends pipeline.
    pub fn process(
        raw_damage: f64,
        damage_type: DamageType,
        is_critical: bool,
        attacker_stats: &StatBlock,
        defender_stats: &StatBlock,
    ) -> DamageResult {
        if damage_type == DamageType::True {
            return DamageResult {
                raw_damage,
                mitigated_damage: raw_damage,
                final_damage: raw_damage,
                damage_type,
                is_critical,
            };
        }

        let (defender_res, flat_pen, percent_pen) = match damage_type {
            DamageType::Physical => (
                defender_stats.armor,
                attacker_stats.armor_pen_flat,
                attacker_stats.armor_pen_percent,
            ),
            DamageType::Magic => (
                defender_stats.magic_resist,
                attacker_stats.magic_pen_flat,
                attacker_stats.magic_pen_percent,
            ),
            DamageType::True => unreachable!(),
        };

        // Assume reduction is already applied to defender_stats or 0 for basic simulation.
        let eff_res = effective_resistance(defender_res, 0.0, 0.0, percent_pen, flat_pen);
        let mitigated_damage = apply_resistance(raw_damage, eff_res);

        // Apply damage reduction
        let mut final_damage = mitigated_damage;
        if defender_stats.damage_reduction_percent != 0.0 {
            final_damage *= 1.0 - defender_stats.damage_reduction_percent;
        }

        // Shield absorption would happen here in a fuller implementation.

        DamageResult {
            raw_damage,
            mitigated_damage,
            final_damage,
            damage_type,
            is_critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_resistance() {
        // 100 Armor, 10 flat reduction, 20% reduction, 30% pen, 10 flat pen
        // 100 - 10 = 90
        // 90 * (1 - 0.2) = 72
        // 72 * (1 - 0.3) = 50.4
        // 50.4 - 10 = 40.4
        let eff_res = effective_resistance(100.0, 10.0, 0.2, 0.3, 10.0);
        assert!((eff_res - 40.4).abs() < 1e-6);
    }

    #[test]
    fn test_effective_resistance_pen_cap() {
        // Flat penetration should not drop below 0
        let eff_res = effective_resistance(100.0, 0.0, 0.0, 0.0, 150.0);
        assert_eq!(eff_res, 0.0);
    }

    #[test]
    fn test_apply_resistance() {
        // 100 armor = 50% damage
        assert_eq!(apply_resistance(100.0, 100.0), 50.0);

        // 0 armor = 100% damage
        assert_eq!(apply_resistance(100.0, 0.0), 100.0);

        // -100 armor = 150% damage
        assert_eq!(apply_resistance(100.0, -100.0), 150.0);
    }

    #[test]
    fn test_damage_pipeline() {
        let mut attacker = StatBlock::new();
        attacker.armor_pen_flat = 10.0;
        attacker.armor_pen_percent = 0.0;

        let mut defender = StatBlock::new();
        defender.armor = 110.0;

        let result =
            DamagePipeline::process(100.0, DamageType::Physical, false, &attacker, &defender);

        assert_eq!(result.raw_damage, 100.0);
        // Effective armor = 110 - 10 = 100 -> 50% damage
        assert_eq!(result.mitigated_damage, 50.0);
    }
}
