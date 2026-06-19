use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};
use lol_core::types::ResourceType;

/// Factory for creating Target Dummy instances.
pub struct DummyModule;

impl ChampionModule for DummyModule {
    fn id(&self) -> &str {
        "Dummy"
    }

    fn create_instance(&self, config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let base_stats = config.base_stats.clone();
        let rune_stats = config.rune_page.aggregate_stats();
        let item_stats = config.item_build.aggregate_stats();
        let item_effects = Vec::new(); // Dummy has no items really, or ignores them
        let state = ChampionState::new(
            config.level,
            base_stats,
            config.growth_stats.clone(),
            ResourceType::None,
            rune_stats,
            item_stats,
            item_effects,
        );

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

    fn update_stats(&mut self, time: lol_core::types::SimTime) {
        let level = self.state.level as f64;
        let growth_multiplier = (level - 1.0) * (0.7025 + 0.0175 * (level - 1.0));
        let mut new_base = self.state.base_stats.clone();
        new_base.health += self.state.growth_stats.health * growth_multiplier;
        new_base.armor += self.state.growth_stats.armor * growth_multiplier;
        new_base.magic_resist += self.state.growth_stats.magic_resist * growth_multiplier;

        self.state.stats.base = new_base;

        let bonus = self.state.rune_stats.clone() + self.state.item_stats.clone();
        self.state.stats.recalculate_initial(&bonus);

        self.state.current_time = time;
        let mut total_bonus = self.state.buffs.aggregate_stats();
        let level = self.state.level;
        let hp_ratio = if self.state.stats.current.health > 0.0 {
            self.state.health.current / self.state.stats.current.health
        } else {
            1.0
        };
        total_bonus = total_bonus
            + self.state.rune_manager.get_bonus_stats(
                time,
                &self.state.stats.base,
                level,
                hp_ratio,
            );
        self.state.stats.recalculate_current(&total_bonus);
    }

    fn get_ability(
        &self,
        _slot: lol_core::types::AbilitySlot,
    ) -> Option<&dyn lol_core::ability::Ability> {
        None
    }

    fn take_damage(&mut self, amount: f64) -> lol_core::types::TakeDamageResult {
        self.state.health.reduce(amount);
        lol_core::types::TakeDamageResult {
            actual_damage: amount,
            is_dead: false,
        }
    }
}
