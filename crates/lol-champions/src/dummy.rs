use lol_core::champion::{ChampionModule, ChampionInstance, ChampionConfig, ChampionState};
use lol_core::types::ResourceType;

/// Factory for creating Target Dummy instances.
pub struct DummyModule;

impl ChampionModule for DummyModule {
    fn id(&self) -> &str {
        "Dummy"
    }

    fn create_instance(&self, config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let base_stats = config.base_stats.clone();
        let bonus_stats = config.aggregate_bonus_stats();
        let item_effects = Vec::new(); // Dummy has no items really, or ignores them
        let state = ChampionState::new(base_stats, ResourceType::None, bonus_stats, item_effects);
        
        Box::new(DummyInstance {
            state,
            _config: config,
        })
    }
}

pub struct DummyInstance {
    pub state: ChampionState,
    pub _config: ChampionConfig,
}

impl ChampionInstance for DummyInstance {
    fn state(&self) -> &ChampionState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ChampionState {
        &mut self.state
    }

    fn update_stats(&mut self) {
        let buffs_stats = self.state.buffs.aggregate_stats();
        self.state.stats.recalculate_current(&buffs_stats);
    }
    
    fn get_ability(&self, _slot: lol_core::types::AbilitySlot) -> Option<&dyn lol_core::ability::Ability> {
        None
    }
    
    fn take_damage(&mut self, amount: f64) -> bool {
        self.state.health.reduce(amount);
        // Target Dummy never dies, so it always returns false, 
        // or we can say if health reaches 0, it doesn't matter.
        false 
    }
}
