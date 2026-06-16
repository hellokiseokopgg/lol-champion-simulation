use serde::{Deserialize, Serialize};

/// Represents all static data associated with an item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ItemData {
    /// The unique identifier of the item.
    pub id: String,
    /// The display name of the item.
    pub name: String,
    /// Gold cost of the item.
    pub cost: u32,
    /// Stats provided by the item.
    #[serde(default)]
    pub stats: ItemStats,
}

/// Statistics provided by an item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ItemStats {
    #[serde(default)]
    pub attack_damage: f32,
    #[serde(default)]
    pub ability_power: f32,
    #[serde(default)]
    pub armor: f32,
    #[serde(default)]
    pub magic_resist: f32,
    #[serde(default)]
    pub hp: f32,
    #[serde(default)]
    pub mp: f32,
    /// Attack speed percentage (e.g., 0.20 for 20%)
    #[serde(default)]
    pub attack_speed: f32,
    /// Critical strike chance (e.g., 0.20 for 20%)
    #[serde(default)]
    pub crit_chance: f32,
    #[serde(default)]
    pub move_speed_flat: f32,
    /// Movement speed percentage (e.g., 0.05 for 5%)
    #[serde(default)]
    pub move_speed_percent: f32,
    #[serde(default)]
    pub armor_pen_flat: f32, // Lethality
    /// Armor penetration percentage (e.g., 0.30 for 30%)
    #[serde(default)]
    pub armor_pen_percent: f32,
    #[serde(default)]
    pub magic_pen_flat: f32,
    /// Magic penetration percentage (e.g., 0.40 for 40%)
    #[serde(default)]
    pub magic_pen_percent: f32,
    #[serde(default)]
    pub life_steal: f32,
    #[serde(default)]
    pub ability_haste: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_item() {
        let json = r#"{
            "id": "long_sword",
            "name": "Long Sword",
            "cost": 350,
            "stats": {
                "attack_damage": 10.0
            }
        }"#;

        let item: ItemData = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, "long_sword");
        assert_eq!(item.name, "Long Sword");
        assert_eq!(item.cost, 350);
        assert_eq!(item.stats.attack_damage, 10.0);
        assert_eq!(item.stats.ability_power, 0.0);
    }
}
