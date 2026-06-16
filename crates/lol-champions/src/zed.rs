use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};

pub struct ZedModule;

impl ChampionModule for ZedModule {
    fn id(&self) -> &str { "Zed" }

    fn create_instance(&self, mut config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let rune_stats = config.rune_page.aggregate_stats();
        let item_stats = config.item_build.aggregate_stats();
        let mut item_effects = Vec::new();
        for item in &mut config.item_build.items {
            item_effects.append(&mut item.effects);
        }
        Box::new(ZedInstance {
            state: ChampionState::new(config.level, config.base_stats.clone(), config.growth_stats.clone(), lol_core::types::ResourceType::Energy, rune_stats, item_stats, item_effects),
            _config: config,
        })
    }
}

pub struct ZedInstance {
    state: ChampionState,
    _config: ChampionConfig,
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
