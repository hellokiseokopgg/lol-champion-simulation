use serde::{Deserialize, Serialize};

/// Represents a collection of standard combat stats in the simulation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StatBlock {
    /// Maximum health points
    pub health: f64,
    /// Health regeneration per 5 seconds
    pub health_regen: f64,
    /// Maximum mana points (or energy/rage depending on champion)
    pub mana: f64,
    /// Mana regeneration per 5 seconds
    pub mana_regen: f64,
    /// Attack Damage (AD)
    pub attack_damage: f64,
    /// Ability Power (AP)
    pub ability_power: f64,
    /// Armor, reduces incoming physical damage
    pub armor: f64,
    /// Magic Resistance (MR), reduces incoming magic damage
    pub magic_resist: f64,
    /// Attack Speed, attacks per second
    pub attack_speed: f64,
    /// Attack speed ratio, used as base for bonus attack speed calculations
    pub attack_speed_ratio: Option<f64>,
    /// Movement speed in units per second
    pub movement_speed: f64,
    /// Critical strike chance (0.0 to 1.0)
    pub crit_chance: f64,
    /// Critical strike damage multiplier (e.g., 1.75 for 175%)
    pub crit_damage: f64,
    /// Flat armor penetration (Lethality calculates into this)
    pub armor_pen_flat: f64,
    /// Percentage armor penetration (0.0 to 1.0)
    pub armor_pen_percent: f64,
    /// Flat magic penetration
    pub magic_pen_flat: f64,
    /// Percentage magic penetration (0.0 to 1.0)
    pub magic_pen_percent: f64,
    /// Ability Haste, reduces cooldowns
    pub ability_haste: f64,
    /// Life Steal percentage (0.0 to 1.0)
    pub life_steal: f64,
    /// Omnivamp percentage (0.0 to 1.0)
    pub omnivamp: f64,
    /// Percentage armor reduction (e.g., 0.25 for 25% shred) applied to the champion
    pub armor_reduction_percent: f64,
    /// Percentage damage reduction (e.g., 0.3 for 30% reduction)
    pub damage_reduction_percent: f64,
}

impl StatBlock {
    /// Creates a new `StatBlock` initialized with zeros.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates a stat's value at a given level based on the League of Legends growth formula.
    ///
    /// # Arguments
    ///
    /// * `base` - The base value of the stat at level 1.
    /// * `growth` - The per-level growth value of the stat.
    /// * `level` - The current level of the champion (typically 1 to 18).
    pub fn stat_at_level(base: f64, growth: f64, level: u32) -> f64 {
        if level <= 1 {
            return base;
        }
        let n = (level - 1) as f64;
        base + growth * n * (0.7025 + 0.0175 * n)
    }
}

impl std::ops::Add for StatBlock {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            health: self.health + rhs.health,
            health_regen: self.health_regen + rhs.health_regen,
            mana: self.mana + rhs.mana,
            mana_regen: self.mana_regen + rhs.mana_regen,
            attack_damage: self.attack_damage + rhs.attack_damage,
            ability_power: self.ability_power + rhs.ability_power,
            armor: self.armor + rhs.armor,
            magic_resist: self.magic_resist + rhs.magic_resist,
            attack_speed: self.attack_speed + rhs.attack_speed,
            attack_speed_ratio: self.attack_speed_ratio.or(rhs.attack_speed_ratio),
            movement_speed: self.movement_speed + rhs.movement_speed,
            crit_chance: self.crit_chance + rhs.crit_chance,
            crit_damage: self.crit_damage + rhs.crit_damage,
            armor_pen_flat: self.armor_pen_flat + rhs.armor_pen_flat,
            armor_pen_percent: self.armor_pen_percent + rhs.armor_pen_percent,
            magic_pen_flat: self.magic_pen_flat + rhs.magic_pen_flat,
            magic_pen_percent: self.magic_pen_percent + rhs.magic_pen_percent,
            ability_haste: self.ability_haste + rhs.ability_haste,
            life_steal: self.life_steal + rhs.life_steal,
            omnivamp: self.omnivamp + rhs.omnivamp,
            armor_reduction_percent: self.armor_reduction_percent + rhs.armor_reduction_percent,
            damage_reduction_percent: 1.0 - ((1.0 - self.damage_reduction_percent) * (1.0 - rhs.damage_reduction_percent)),
        }
    }
}

/// The Three-Layer Stats architecture pattern.
/// Represents the transition of stats from their raw state to fully buffed state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ThreeLayerStats {
    /// Base stats at the champion's current level without any items, runes, or buffs.
    pub base: StatBlock,
    /// Stats including base values plus static bonuses from items and runes.
    pub initial: StatBlock,
    /// Fully calculated stats including all active, temporary buffs and effects.
    pub current: StatBlock,
}

impl ThreeLayerStats {
    /// Initialize a new `ThreeLayerStats` starting from a provided base `StatBlock`.
    pub fn new(base: StatBlock) -> Self {
        Self {
            initial: base.clone(),
            current: base.clone(),
            base,
        }
    }

    /// Re-calculates `initial` stats by applying static bonuses (items/runes) to `base` stats.
    pub fn recalculate_initial(&mut self, items_and_runes: &StatBlock) {
        self.initial = self.base.clone() + items_and_runes.clone();
        // Special logic like Rabadon's or multiplicative stats would go here in a full engine
    }

    /// Re-calculates `current` stats by applying temporary buffs to the `initial` stats.
    pub fn recalculate_current(&mut self, buffs: &StatBlock) {
        self.current = self.initial.clone() + buffs.clone();
        
        // Apply % armor reduction
        if self.current.armor_reduction_percent > 0.0 {
            self.current.armor = self.current.armor * (1.0 - self.current.armor_reduction_percent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_at_level_formula() {
        let base = 600.0;
        let growth = 100.0;

        // Level 1 should be exactly base
        assert_eq!(StatBlock::stat_at_level(base, growth, 1), 600.0);

        // Level 2 should be base + growth * 1 * (0.7025 + 0.0175 * 1) = 600 + 100 * 0.72 = 672.0
        assert_eq!(StatBlock::stat_at_level(base, growth, 2), 672.0);

        // Level 18 should be exactly base + growth * 17
        // Because 17 * (0.7025 + 0.0175 * 17) = 17 * (0.7025 + 0.2975) = 17 * 1.0 = 17.0
        assert_eq!(StatBlock::stat_at_level(base, growth, 18), 600.0 + 100.0 * 17.0);
    }

    #[test]
    fn test_stat_block_addition() {
        let a = StatBlock {
            attack_damage: 50.0,
            health: 500.0,
            ..Default::default()
        };
        let b = StatBlock {
            attack_damage: 20.0,
            armor: 30.0,
            ..Default::default()
        };

        let c = a + b;
        assert_eq!(c.attack_damage, 70.0);
        assert_eq!(c.health, 500.0);
        assert_eq!(c.armor, 30.0);
    }

    #[test]
    fn test_three_layer_stats() {
        let base = StatBlock {
            attack_damage: 60.0,
            ..Default::default()
        };
        let mut stats = ThreeLayerStats::new(base);

        assert_eq!(stats.current.attack_damage, 60.0);

        let items = StatBlock {
            attack_damage: 40.0,
            ..Default::default()
        };
        stats.recalculate_initial(&items);
        assert_eq!(stats.initial.attack_damage, 100.0);

        let buffs = StatBlock {
            attack_damage: 25.0,
            ..Default::default()
        };
        stats.recalculate_current(&buffs);
        assert_eq!(stats.current.attack_damage, 125.0);

        // Base should remain unchanged
        assert_eq!(stats.base.attack_damage, 60.0);
    }
}
