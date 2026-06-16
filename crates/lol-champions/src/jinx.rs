use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};

pub struct JinxModule;

impl ChampionModule for JinxModule {
    fn id(&self) -> &str { "Jinx" }

    fn create_instance(&self, mut config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let bonus_stats = config.aggregate_bonus_stats();
        let mut item_effects = Vec::new();
        for item in &mut config.item_build.items {
            item_effects.append(&mut item.effects);
        }
        Box::new(JinxInstance {
            state: ChampionState::new(config.base_stats.clone(), lol_core::types::ResourceType::Mana, bonus_stats, item_effects),
        })
    }
}

pub struct JinxInstance {
    state: ChampionState,
}

impl ChampionInstance for JinxInstance {
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
