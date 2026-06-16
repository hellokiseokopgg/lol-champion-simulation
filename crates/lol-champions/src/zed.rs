use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};

pub struct ZedModule;

impl ChampionModule for ZedModule {
    fn id(&self) -> &str { "Zed" }

    fn create_instance(&self, mut config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let bonus_stats = config.aggregate_bonus_stats();
        let mut item_effects = Vec::new();
        for item in &mut config.item_build.items {
            item_effects.append(&mut item.effects);
        }
        Box::new(ZedInstance {
            state: ChampionState::new(config.base_stats.clone(), lol_core::types::ResourceType::Energy, bonus_stats, item_effects),
        })
    }
}

pub struct ZedInstance {
    state: ChampionState,
}

impl ChampionInstance for ZedInstance {
    fn state(&self) -> &ChampionState { &self.state }
    fn state_mut(&mut self) -> &mut ChampionState { &mut self.state }
    fn update_stats(&mut self) {
        let buffs_stats = self.state.buffs.aggregate_stats();
        self.state.stats.recalculate_current(&buffs_stats);
    }
    
    fn get_ability(&self, _slot: lol_core::types::AbilitySlot) -> Option<&dyn lol_core::ability::Ability> {
        None
    }
    
    fn take_damage(&mut self, amount: f64) -> bool {
        self.state.health.reduce(amount)
    }
}
