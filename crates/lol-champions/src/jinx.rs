use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};

pub struct JinxModule;

impl ChampionModule for JinxModule {
    fn id(&self) -> &str { "Jinx" }

    fn create_instance(&self, mut config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let rune_stats = config.rune_page.aggregate_stats();
        let item_stats = config.item_build.aggregate_stats();
        let mut item_effects = Vec::new();
        for item in &mut config.item_build.items {
            item_effects.append(&mut item.effects);
        }
        Box::new(JinxInstance {
            state: ChampionState::new(config.level, config.base_stats.clone(), config.growth_stats.clone(), lol_core::types::ResourceType::Mana, rune_stats, item_stats, item_effects),
            _config: config,
        })
    }
}

pub struct JinxInstance {
    state: ChampionState,
    _config: ChampionConfig,
}

impl ChampionInstance for JinxInstance {
    fn state(&self) -> &ChampionState { &self.state }
    fn state_mut(&mut self) -> &mut ChampionState { &mut self.state }
    fn update_stats(&mut self, time: lol_core::types::SimTime) {
        let mut total_bonus = self.state.buffs.aggregate_stats();
        let level = self.state.level;
        total_bonus = total_bonus + self.state.rune_manager.get_bonus_stats(time, &self.state.base_stats, level);
        self.state.stats.recalculate_current(&total_bonus);
    }
    
    fn get_ability(&self, _slot: lol_core::types::AbilitySlot) -> Option<&dyn lol_core::ability::Ability> {
        None
    }
    
    fn take_damage(&mut self, amount: f64) -> lol_core::types::TakeDamageResult {
        let is_dead = self.state.health.reduce(amount);
        lol_core::types::TakeDamageResult { actual_damage: amount, is_dead }
    }
}
