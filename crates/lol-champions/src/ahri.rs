use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};

pub struct AhriModule;

impl ChampionModule for AhriModule {
    fn id(&self) -> &str { "Ahri" }

    fn create_instance(&self, config: ChampionConfig) -> Box<dyn ChampionInstance> {
        Box::new(AhriInstance {
            state: ChampionState::new(config.base_stats.clone(), lol_core::types::ResourceType::Mana),
        })
    }
}

pub struct AhriInstance {
    state: ChampionState,
}

impl ChampionInstance for AhriInstance {
    fn state(&self) -> &ChampionState { &self.state }
    fn state_mut(&mut self) -> &mut ChampionState { &mut self.state }
    fn update_stats(&mut self) {
        let buffs_stats = self.state.buffs.aggregate_stats();
        self.state.stats.recalculate_current(&buffs_stats);
    }
    
    fn get_ability(&self, _slot: lol_core::types::AbilitySlot) -> Option<&dyn lol_core::ability::Ability> {
        None
    }
}
