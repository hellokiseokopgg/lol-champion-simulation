use crate::stats::StatBlock;

/// Represents a primary or secondary rune path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunePath {
    Precision,
    Domination,
    Sorcery,
    Resolve,
    Inspiration,
}

/// Represents an effect provided by a selected rune or stat shard.
pub trait RuneEffect {
    /// The name of the rune.
    fn name(&self) -> &str;
    
    /// The static stats provided by the rune.
    fn stats(&self) -> StatBlock {
        StatBlock::new()
    }
    
    // Dynamic effects (like Conqueror stacking) would require event hooks.
    /// Triggered when the champion deals damage to an enemy champion.
    fn on_damage_dealt(&self, _sim: &mut crate::event::SimContext, _champion: &mut dyn crate::champion::ChampionInstance, _damage: &crate::damage::DamageResult) {}
}

struct ConquerorBuff;
impl crate::buff::StatusEffect for ConquerorBuff {
    fn id(&self) -> crate::types::EffectId { crate::types::EffectId("Conqueror".to_string()) }
    fn name(&self) -> &str { "Conqueror" }
    fn duration(&self) -> f64 { 5.0 } // 5 seconds duration
    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior { crate::buff::RefreshBehavior::AddStack }
    fn max_stacks(&self) -> u32 { 12 }
    fn stat_modifiers(&self, stacks: u32) -> crate::stats::StatBlock {
        let mut stats = crate::stats::StatBlock::new();
        // Provides 1.2 - 2.7 Bonus AD per stack (simplified to 2.0)
        stats.attack_damage = 2.0 * (stacks as f64);
        // Note: AP is also provided but we'll stick to AD for Garen
        stats
    }
}

pub struct ConquerorRune;

impl RuneEffect for ConquerorRune {
    fn name(&self) -> &str { "Conqueror" }

    fn on_damage_dealt(&self, _sim: &mut crate::event::SimContext, champion: &mut dyn crate::champion::ChampionInstance, _damage: &crate::damage::DamageResult) {
        // Gain 2 stacks for melee (mocked here, we assume melee for now)
        champion.state_mut().buffs.apply_effect(Box::new(ConquerorBuff), _sim.current_time);
        champion.state_mut().buffs.apply_effect(Box::new(ConquerorBuff), _sim.current_time);
    }
}

/// Represents a champion's full rune page.
pub struct RunePage {
    pub primary_path: RunePath,
    pub secondary_path: RunePath,
    pub keystone: Box<dyn RuneEffect>,
    pub primary_runes: Vec<Box<dyn RuneEffect>>,
    pub secondary_runes: Vec<Box<dyn RuneEffect>>,
    pub stat_shards: Vec<Box<dyn RuneEffect>>,
}

struct EmptyRune;
impl RuneEffect for EmptyRune {
    fn name(&self) -> &str { "Empty" }
}

impl Default for RunePage {
    fn default() -> Self {
        Self {
            primary_path: RunePath::Precision,
            secondary_path: RunePath::Domination,
            keystone: Box::new(EmptyRune),
            primary_runes: vec![],
            secondary_runes: vec![],
            stat_shards: vec![],
        }
    }
}

impl RunePage {
    /// Aggregates all static stats provided by the runes and stat shards.
    pub fn aggregate_stats(&self) -> StatBlock {
        let mut total = StatBlock::new();
        total = total + self.keystone.stats();
        for r in &self.primary_runes {
            total = total + r.stats();
        }
        for r in &self.secondary_runes {
            total = total + r.stats();
        }
        for s in &self.stat_shards {
            total = total + s.stats();
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRune {
        name: String,
        ad: f64,
    }

    impl RuneEffect for TestRune {
        fn name(&self) -> &str { &self.name }
        fn stats(&self) -> StatBlock {
            StatBlock {
                attack_damage: self.ad,
                ..Default::default()
            }
        }
    }

    #[test]
    fn test_rune_page_stats() {
        let page = RunePage {
            primary_path: RunePath::Precision,
            secondary_path: RunePath::Domination,
            keystone: Box::new(TestRune { name: "Conqueror".to_string(), ad: 0.0 }),
            primary_runes: vec![Box::new(TestRune { name: "Triumph".to_string(), ad: 0.0 })],
            secondary_runes: vec![Box::new(TestRune { name: "Eyeball".to_string(), ad: 18.0 })],
            stat_shards: vec![
                Box::new(TestRune { name: "Adaptive".to_string(), ad: 5.4 }),
                Box::new(TestRune { name: "Adaptive".to_string(), ad: 5.4 }),
            ],
        };

        let total = page.aggregate_stats();
        // 18.0 + 5.4 + 5.4 = 28.8
        assert!((total.attack_damage - 28.8).abs() < 1e-6);
    }
}
