use crate::champion::ChampionInstance;
use crate::event::{EventManager, SimContext};
use crate::types::SimTime;

/// Configuration parameters for a simulation run.
pub struct SimConfig {
    /// The maximum duration of the simulation in seconds.
    pub max_duration: f64,
}

/// The core game simulation engine.
pub struct GameSimulation {
    event_manager: EventManager,
    config: SimConfig,

    // In a full implementation, this would hold multiple actors.
    // For a 1v1 sim, we would have attacker and defender.
    // Keeping it generic for now.
    // Keeping it generic for now.
    pub actors: std::collections::HashMap<
        crate::types::ChampionId,
        std::rc::Rc<std::cell::RefCell<Box<dyn ChampionInstance>>>,
    >,
}

impl GameSimulation {
    /// Creates a new simulation with the given configuration.
    pub fn new(config: SimConfig) -> Self {
        Self {
            event_manager: EventManager::new(),
            config,
            actors: std::collections::HashMap::new(),
        }
    }

    /// Adds a champion instance to the simulation.
    pub fn add_actor(
        &mut self,
        id: crate::types::ChampionId,
        actor: std::rc::Rc<std::cell::RefCell<Box<dyn ChampionInstance>>>,
    ) {
        self.actors.insert(id, actor);
    }

    /// Access the event manager to queue initial events (e.g. APL execution).
    pub fn event_manager_mut(&mut self) -> &mut EventManager {
        &mut self.event_manager
    }

    /// Runs the simulation until the max duration is reached or no events remain.
    pub fn run(
        &mut self,
        recorder: Option<std::rc::Rc<std::cell::RefCell<dyn crate::event::EventRecorder>>>,
    ) {
        let mut ctx = SimContext {
            current_time: SimTime::new(0.0),
            recorder,
            new_events: Vec::new(),
            champions: self.actors.clone(),
            is_simulation_over: false,
        };

        // Record initial health for all champions
        if let Some(rec) = &ctx.recorder {
            for (id, actor_ref) in &self.actors {
                let actor = actor_ref.borrow();
                let current_hp = actor.state().health.current;
                let max_hp = actor.state().stats.current.health;
                rec.borrow_mut().record_resource_update(
                    crate::types::SimTime::new(0.0),
                    id.clone(),
                    "HP".to_string(),
                    current_hp,
                    max_hp,
                );
            }
        }

        let max_time = SimTime::new(self.config.max_duration);

        // Schedule the periodic resource regeneration tick event at 0.5s
        self.event_manager.schedule(
            SimTime::new(0.5),
            Box::new(crate::event::RegenTickEvent),
        );

        // Delegate to the EventManager's run loop
        self.event_manager.run(&mut ctx, max_time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::champion::ChampionState;
    use crate::stats::StatBlock;
    use crate::types::{ResourceType, ChampionId};
    use std::rc::Rc;
    use std::cell::RefCell;

    struct MockChampion {
        state: ChampionState,
    }

    impl ChampionInstance for MockChampion {
        fn state(&self) -> &ChampionState {
            &self.state
        }
        fn state_mut(&mut self) -> &mut ChampionState {
            &mut self.state
        }
        fn update_stats(&mut self, _time: SimTime) {}
        fn get_ability(&self, _slot: crate::types::AbilitySlot) -> Option<&dyn crate::ability::Ability> {
            None
        }
        fn take_damage(&mut self, amount: f64) -> crate::types::TakeDamageResult {
            self.state.health.reduce(amount);
            crate::types::TakeDamageResult {
                actual_damage: amount,
                is_dead: self.state.health.current <= 0.0,
            }
        }
    }

    #[test]
    fn test_simulation_run() {
        let config = SimConfig { max_duration: 10.0 };
        let mut sim = GameSimulation::new(config);

        // Sim starts empty, should just advance time directly to max
        sim.run(None);

        assert_eq!(sim.event_manager_mut().current_time().as_f64(), 10.0);
    }

    #[test]
    fn test_automated_resource_regeneration() {
        let mut base_stats = StatBlock::new();
        base_stats.health = 100.0;
        base_stats.health_regen = 10.0; // 10 per 5s -> 2 per second
        base_stats.mana = 100.0;
        base_stats.mana_regen = 5.0; // 5 per 5s -> 1 per second

        let mut state = ChampionState::new(
            1,
            base_stats,
            StatBlock::new(),
            ResourceType::Mana,
            StatBlock::new(),
            StatBlock::new(),
            Vec::new(),
        );
        // Deplete resources to 50%
        state.health.current = 50.0;
        state.resource.current = 50.0;

        let champ = Rc::new(RefCell::new(Box::new(MockChampion { state }) as Box<dyn ChampionInstance>));

        let config = SimConfig { max_duration: 5.0 };
        let mut sim = GameSimulation::new(config);
        sim.add_actor(ChampionId("TestChamp".to_string()), champ.clone());

        sim.run(None);

        // Check health: 50.0 + (10.0 / 5.0) * 5.0 = 50.0 + 10.0 = 60.0
        assert_eq!(champ.borrow().state().health.current, 60.0);
        // Check mana: 50.0 + (5.0 / 5.0) * 5.0 = 50.0 + 5.0 = 55.0
        assert_eq!(champ.borrow().state().resource.current, 55.0);
    }

    #[test]
    fn test_automated_energy_regeneration() {
        let mut base_stats = StatBlock::new();
        base_stats.health = 100.0;
        base_stats.health_regen = 0.0;
        base_stats.mana = 200.0;
        base_stats.mana_regen = 0.0;

        let mut state = ChampionState::new(
            1,
            base_stats,
            StatBlock::new(),
            ResourceType::Energy,
            StatBlock::new(),
            StatBlock::new(),
            Vec::new(),
        );
        state.health.current = 100.0;
        state.resource.current = 100.0;

        let champ = Rc::new(RefCell::new(Box::new(MockChampion { state }) as Box<dyn ChampionInstance>));

        let config = SimConfig { max_duration: 5.0 };
        let mut sim = GameSimulation::new(config);
        sim.add_actor(ChampionId("ZedMock".to_string()), champ.clone());

        sim.run(None);

        // Energy: 10.0/s * 5.0s = 50.0 -> 100.0 + 50.0 = 150.0
        assert_eq!(champ.borrow().state().resource.current, 150.0);
    }

    #[test]
    fn test_regen_zero_and_negative() {
        // 1. Zero regeneration: health/resource should not change
        {
            let mut base_stats = StatBlock::new();
            base_stats.health = 100.0;
            base_stats.health_regen = 0.0;
            base_stats.mana = 100.0;
            base_stats.mana_regen = 0.0;

            let mut state = ChampionState::new(
                1,
                base_stats,
                StatBlock::new(),
                ResourceType::Mana,
                StatBlock::new(),
                StatBlock::new(),
                Vec::new(),
            );
            state.health.current = 50.0;
            state.resource.current = 50.0;

            let champ = Rc::new(RefCell::new(Box::new(MockChampion { state }) as Box<dyn ChampionInstance>));
            let config = SimConfig { max_duration: 5.0 };
            let mut sim = GameSimulation::new(config);
            sim.add_actor(ChampionId("ZeroRegen".to_string()), champ.clone());

            sim.run(None);

            assert_eq!(champ.borrow().state().health.current, 50.0);
            assert_eq!(champ.borrow().state().resource.current, 50.0);
        }

        // 2. Negative regeneration: should reduce health/resource since restore adds negative amount
        {
            let mut base_stats = StatBlock::new();
            base_stats.health = 100.0;
            base_stats.health_regen = -10.0; // -2.0 HP per second
            base_stats.mana = 100.0;
            base_stats.mana_regen = -5.0; // -1.0 Mana per second

            let mut state = ChampionState::new(
                1,
                base_stats,
                StatBlock::new(),
                ResourceType::Mana,
                StatBlock::new(),
                StatBlock::new(),
                Vec::new(),
            );
            state.health.current = 50.0;
            state.resource.current = 50.0;

            let champ = Rc::new(RefCell::new(Box::new(MockChampion { state }) as Box<dyn ChampionInstance>));
            let config = SimConfig { max_duration: 5.0 };
            let mut sim = GameSimulation::new(config);
            sim.add_actor(ChampionId("NegativeRegen".to_string()), champ.clone());

            sim.run(None);

            // Health: 50.0 + (-10.0 / 5.0) * 5.0 = 50.0 - 10.0 = 40.0
            assert_eq!(champ.borrow().state().health.current, 40.0);
            // Mana: 50.0 + (-5.0 / 5.0) * 5.0 = 50.0 - 5.0 = 45.0
            assert_eq!(champ.borrow().state().resource.current, 45.0);
        }
    }

    #[test]
    fn test_regen_capping_at_max() {
        let mut base_stats = StatBlock::new();
        base_stats.health = 100.0;
        base_stats.health_regen = 10.0; // 2.0 HP per second
        base_stats.mana = 100.0;
        base_stats.mana_regen = 10.0; // 2.0 Mana per second

        let mut state = ChampionState::new(
            1,
            base_stats,
            StatBlock::new(),
            ResourceType::Mana,
            StatBlock::new(),
            StatBlock::new(),
            Vec::new(),
        );
        state.health.current = 99.0;
        state.resource.current = 99.0;

        let champ = Rc::new(RefCell::new(Box::new(MockChampion { state }) as Box<dyn ChampionInstance>));
        let config = SimConfig { max_duration: 5.0 };
        let mut sim = GameSimulation::new(config);
        sim.add_actor(ChampionId("CapRegen".to_string()), champ.clone());

        sim.run(None);

        // Health and Mana should be capped at 100.0 (max)
        assert_eq!(champ.borrow().state().health.current, 100.0);
        assert_eq!(champ.borrow().state().resource.current, 100.0);
    }

    #[test]
    fn test_regen_no_resource_type_or_no_stats() {
        // 1. ResourceType::None: should regenerate health but not resource
        {
            let mut base_stats = StatBlock::new();
            base_stats.health = 100.0;
            base_stats.health_regen = 10.0; // 2.0 HP per second
            base_stats.mana = 100.0;
            base_stats.mana_regen = 10.0; // 2.0 Mana per second

            let mut state = ChampionState::new(
                1,
                base_stats,
                StatBlock::new(),
                ResourceType::None,
                StatBlock::new(),
                StatBlock::new(),
                Vec::new(),
            );
            state.health.current = 50.0;
            state.resource.current = 50.0;

            let champ = Rc::new(RefCell::new(Box::new(MockChampion { state }) as Box<dyn ChampionInstance>));
            let config = SimConfig { max_duration: 5.0 };
            let mut sim = GameSimulation::new(config);
            sim.add_actor(ChampionId("NoResourceType".to_string()), champ.clone());

            sim.run(None);

            // Health: 50.0 + (10.0 / 5.0) * 5.0 = 60.0
            assert_eq!(champ.borrow().state().health.current, 60.0);
            // Resource (None) should NOT regenerate
            assert_eq!(champ.borrow().state().resource.current, 50.0);
        }

        // 2. No stats (all zero except starting health so they don't die)
        {
            let mut base_stats = StatBlock::new();
            base_stats.health = 100.0;
            base_stats.health_regen = 0.0;
            base_stats.mana = 0.0;
            base_stats.mana_regen = 0.0;

            let mut state = ChampionState::new(
                1,
                base_stats,
                StatBlock::new(),
                ResourceType::None,
                StatBlock::new(),
                StatBlock::new(),
                Vec::new(),
            );
            state.health.current = 50.0;
            state.resource.current = 0.0;

            let champ = Rc::new(RefCell::new(Box::new(MockChampion { state }) as Box<dyn ChampionInstance>));
            let config = SimConfig { max_duration: 5.0 };
            let mut sim = GameSimulation::new(config);
            sim.add_actor(ChampionId("NoStats".to_string()), champ.clone());

            sim.run(None);

            assert_eq!(champ.borrow().state().health.current, 50.0);
            assert_eq!(champ.borrow().state().resource.current, 0.0);
        }
    }

    #[test]
    fn test_regen_dead_champion() {
        let mut base_stats = StatBlock::new();
        base_stats.health = 100.0;
        base_stats.health_regen = 10.0; // 2.0 HP per second
        base_stats.mana = 100.0;
        base_stats.mana_regen = 10.0; // 2.0 Mana per second

        let mut state = ChampionState::new(
            1,
            base_stats,
            StatBlock::new(),
            ResourceType::Mana,
            StatBlock::new(),
            StatBlock::new(),
            Vec::new(),
        );
        state.health.current = 0.0; // Dead!
        state.resource.current = 50.0;

        let champ = Rc::new(RefCell::new(Box::new(MockChampion { state }) as Box<dyn ChampionInstance>));
        let config = SimConfig { max_duration: 5.0 };
        let mut sim = GameSimulation::new(config);
        sim.add_actor(ChampionId("DeadChamp".to_string()), champ.clone());

        sim.run(None);

        // Dead champions should not regenerate health or mana
        assert_eq!(champ.borrow().state().health.current, 0.0);
        assert_eq!(champ.borrow().state().resource.current, 50.0);
    }
}
