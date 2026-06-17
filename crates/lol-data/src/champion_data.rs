use serde::{Deserialize, Serialize};

/// Represents all static data associated with a single champion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChampionData {
    /// The unique identifier of the champion (e.g., "garen").
    pub id: String,
    /// The display name of the champion.
    pub name: String,
    /// Base stats at level 1.
    pub base_stats: BaseStats,
    /// Stat growth per level.
    pub growth_stats: GrowthStats,
    /// Abilities data.
    #[serde(default)]
    pub skills: Vec<SkillData>,
}

/// Base stats for a champion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseStats {
    pub hp: f32,
    pub mp: f32,
    pub hp_regen: f32,
    pub mp_regen: f32,
    pub armor: f32,
    pub magic_resist: f32,
    pub attack_damage: f32,
    pub attack_speed: f32,
    pub attack_range: f32,
    pub move_speed: f32,
    pub attack_delay_offset: Option<f64>,
    pub attack_speed_ratio: Option<f64>,
    pub windup_percent: Option<f64>,
    pub windup_modifier: Option<f64>,
}

/// Stat growth per level for a champion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrowthStats {
    pub hp: f32,
    pub mp: f32,
    pub hp_regen: f32,
    pub mp_regen: f32,
    pub armor: f32,
    pub magic_resist: f32,
    pub attack_damage: f32,
    pub attack_speed: f32,
}

/// Data for a specific skill (Q, W, E, R, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillData {
    /// Identifier for the skill (e.g., "garen_q").
    pub id: String,
    /// Display name of the skill.
    pub name: String,
    /// Base damage values per skill level.
    #[serde(default)]
    pub base_damage: Vec<f32>,
    /// Cooldown values per skill level.
    #[serde(default)]
    pub cooldowns: Vec<f32>,
    /// AD ratio (can be a flat value or array per level).
    #[serde(default)]
    pub ad_ratio: Vec<f32>,
    /// AP ratio.
    #[serde(default)]
    pub ap_ratio: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_champion() {
        let json = r#"{
            "id": "garen",
            "name": "Garen",
            "base_stats": {
                "hp": 690,
                "mp": 0,
                "hp_regen": 8,
                "mp_regen": 0,
                "armor": 38,
                "magic_resist": 32,
                "attack_damage": 69,
                "attack_speed": 0.625,
                "attack_range": 175,
                "move_speed": 340
            },
            "growth_stats": {
                "hp": 98,
                "mp": 0,
                "hp_regen": 0.5,
                "mp_regen": 0,
                "armor": 4.2,
                "magic_resist": 2.05,
                "attack_damage": 4.5,
                "attack_speed": 0.0362
            }
        }"#;

        let champ: ChampionData = serde_json::from_str(json).unwrap();
        assert_eq!(champ.id, "garen");
        assert_eq!(champ.base_stats.hp, 690.0);
        assert_eq!(champ.growth_stats.armor, 4.2);
    }
}
