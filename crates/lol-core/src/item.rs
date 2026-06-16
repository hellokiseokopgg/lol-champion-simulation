use crate::stats::StatBlock;

/// Represents an effect provided by an item (e.g., passive or active).
pub trait ItemEffect {
    fn name(&self) -> &str;
    
    /// Triggered when the champion casts an ability.
    fn on_ability_cast(&self, _sim: &mut crate::event::SimContext, _champion: &mut dyn crate::champion::ChampionInstance) {}
    
    /// Triggered when the champion's basic attack hits.
    fn on_attack_hit(&self, _sim: &mut crate::event::SimContext, _champion: &mut dyn crate::champion::ChampionInstance, _target: &mut dyn crate::champion::ChampionInstance) {}
}

pub struct Spellblade {
    pub base_ad_multiplier: f64, // e.g. 2.0 for Trinity Force
}

struct SpellbladeBuff;
impl crate::buff::StatusEffect for SpellbladeBuff {
    fn id(&self) -> crate::types::EffectId { crate::types::EffectId("Spellblade".to_string()) }
    fn name(&self) -> &str { "Spellblade" }
    fn duration(&self) -> f64 { 10.0 }
    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior { crate::buff::RefreshBehavior::RefreshDuration }
    fn max_stacks(&self) -> u32 { 1 }
    fn stat_modifiers(&self, _stacks: u32) -> crate::stats::StatBlock { crate::stats::StatBlock::new() } // Real implementation would add damage on hit
}

impl ItemEffect for Spellblade {
    fn name(&self) -> &str {
        "Spellblade"
    }

    fn on_ability_cast(&self, _sim: &mut crate::event::SimContext, champion: &mut dyn crate::champion::ChampionInstance) {
        champion.state_mut().buffs.apply_effect(Box::new(SpellbladeBuff), _sim.current_time);
    }
}

/// Represents a single item in the simulation.
pub struct Item {
    /// The unique identifier of the item.
    pub id: String,
    /// The human-readable name of the item.
    pub name: String,
    /// The raw stats provided by the item.
    pub stats: StatBlock,
    /// The special effects/passives the item provides.
    pub effects: Vec<Box<dyn ItemEffect>>,
}

/// Represents a champion's full item build (up to 6 standard items).
#[derive(Default)]
pub struct ItemBuild {
    /// The items currently in the build.
    pub items: Vec<Item>,
}

impl ItemBuild {
    /// Creates a new, empty item build.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds an item to the build, capping at 6 items.
    pub fn add_item(&mut self, item: Item) {
        if self.items.len() < 6 {
            self.items.push(item);
        }
    }

    /// Computes the aggregate stats from all items in the build.
    pub fn aggregate_stats(&self) -> StatBlock {
        let mut total = StatBlock::new();
        for item in &self.items {
            total = total + item.stats.clone();
        }
        // Note: Special multiplicative item effects (like Rabadon's Deathcap)
        // would be applied after the sum in a more advanced implementation.
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_build_stats() {
        let mut build = ItemBuild::new();
        
        let item1 = Item {
            id: "1038".to_string(),
            name: "B.F. Sword".to_string(),
            stats: StatBlock {
                attack_damage: 40.0,
                ..Default::default()
            },
            effects: vec![],
        };
        
        let item2 = Item {
            id: "1036".to_string(),
            name: "Long Sword".to_string(),
            stats: StatBlock {
                attack_damage: 10.0,
                ..Default::default()
            },
            effects: vec![],
        };

        build.add_item(item1);
        build.add_item(item2);

        let total = build.aggregate_stats();
        assert_eq!(total.attack_damage, 50.0);
    }

    #[test]
    fn test_item_build_limit() {
        let mut build = ItemBuild::new();
        for i in 0..10 {
            build.add_item(Item {
                id: i.to_string(),
                name: "Test Item".to_string(),
                stats: StatBlock::new(),
                effects: vec![],
            });
        }
        assert_eq!(build.items.len(), 6);
    }
}
