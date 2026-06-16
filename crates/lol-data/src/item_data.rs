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

impl ItemData {
    pub fn into_item(self) -> lol_core::item::Item {
        lol_core::item::Item {
            id: self.id.clone(),
            name: self.name,
            stats: lol_core::stats::StatBlock {
                attack_damage: self.stats.attack_damage as f64,
                ability_power: self.stats.ability_power as f64,
                armor: self.stats.armor as f64,
                magic_resist: self.stats.magic_resist as f64,
                health: self.stats.hp as f64,
                mana: self.stats.mp as f64,
                attack_speed: self.stats.attack_speed as f64,
                crit_chance: self.stats.crit_chance as f64,
                movement_speed: self.stats.move_speed_flat as f64,
                ability_haste: self.stats.ability_haste as f64,
                armor_pen_flat: self.stats.armor_pen_flat as f64,
                armor_pen_percent: self.stats.armor_pen_percent as f64,
                magic_pen_flat: self.stats.magic_pen_flat as f64,
                magic_pen_percent: self.stats.magic_pen_percent as f64,
                life_steal: self.stats.life_steal as f64,
                ..Default::default()
            },
            effects: {
                let mut effects: Vec<Box<dyn lol_core::item::ItemEffect>> = Vec::new();
                if self.id == "3071" || self.id == "black_cleaver" {
                    effects.push(Box::new(lol_core::item::BlackCleaverEffect));
                }
                effects
            },
        }
    }
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
