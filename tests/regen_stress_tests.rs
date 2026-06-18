use lol_core::{
    ChampionId, ChampionInstance, ChampionState, EventManager, GameSimulation, ResourceType,
    SimConfig, SimContext, SimEvent, SimTime, StatBlock,
};
use std::cell::RefCell;
use std::rc::Rc;

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
            is_dead: self.state.health.current <= 0.0,
        }
    }
}

/// Verify edge cases with zero or negative regeneration values.
/// This includes the specific finding that negative resource regeneration can drive
/// resource (e.g., Mana) negative because the restore function does not clamp it to >= 0.0,
/// and that negative health regeneration can also make health negative, but stops
/// once health is <= 0.0 because the champion is then considered "dead" and skipped in RegenTickEvent.
#[test]
fn test_stress_zero_and_negative_regeneration() {
    // 1. Zero regeneration values
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

        let champ = Rc::new(RefCell::new(
            Box::new(MockChampion { state }) as Box<dyn ChampionInstance>
        ));
        let config = SimConfig { max_duration: 5.0 };
        let mut sim = GameSimulation::new(config);
        sim.add_actor(ChampionId("ZeroRegen".to_string()), champ.clone());
        sim.run(None);

        // Health and resource should remain unchanged (50.0)
        assert_eq!(champ.borrow().state().health.current, 50.0);
        assert_eq!(champ.borrow().state().resource.current, 50.0);
    }

    // 2. Negative regeneration values leading to negative values / health clamp behavior
    {
        let mut base_stats = StatBlock::new();
        base_stats.health = 10.0;
        base_stats.health_regen = -20.0; // -4.0 per second -> -2.0 per 0.5s tick
        base_stats.mana = 10.0;
        base_stats.mana_regen = -40.0; // -8.0 per second -> -4.0 per 0.5s tick

        let mut state = ChampionState::new(
            1,
            base_stats,
            StatBlock::new(),
            ResourceType::Mana,
            StatBlock::new(),
            StatBlock::new(),
            Vec::new(),
        );
        state.health.current = 1.0;
        state.resource.current = 5.0;

        let champ = Rc::new(RefCell::new(
            Box::new(MockChampion { state }) as Box<dyn ChampionInstance>
        ));
        let config = SimConfig { max_duration: 2.0 }; // 4 ticks at 0.5, 1.0, 1.5, 2.0
        let mut sim = GameSimulation::new(config);
        sim.add_actor(ChampionId("NegClamp".to_string()), champ.clone());
        sim.run(None);

        // Let's trace ticks:
        // Tick 1 (0.5s):
        //   health: 1.0 - 2.0 = -1.0 (Resource::restore allows negative values)
        //   resource: 5.0 - 4.0 = 1.0
        // Tick 2 (1.0s):
        //   health: -1.0 <= 0.0 -> RegenTickEvent skips this champion! (so health remains -1.0, resource remains 1.0)
        // Tick 3 (1.5s):
        //   health <= 0.0 -> skip!
        // Tick 4 (2.0s):
        //   health <= 0.0 -> skip!
        // Therefore, at the end:
        assert_eq!(champ.borrow().state().health.current, -1.0);
        assert_eq!(champ.borrow().state().resource.current, 1.0);
    }

    // 3. Negative regeneration of resource alone (health is positive, so it keeps ticking)
    {
        let mut base_stats = StatBlock::new();
        base_stats.health = 100.0;
        base_stats.health_regen = 0.0;
        base_stats.mana = 100.0;
        base_stats.mana_regen = -50.0; // -10.0 per second -> -5.0 per 0.5s tick

        let mut state = ChampionState::new(
            1,
            base_stats,
            StatBlock::new(),
            ResourceType::Mana,
            StatBlock::new(),
            StatBlock::new(),
            Vec::new(),
        );
        state.health.current = 100.0;
        state.resource.current = 10.0;

        let champ = Rc::new(RefCell::new(
            Box::new(MockChampion { state }) as Box<dyn ChampionInstance>
        ));
        let config = SimConfig { max_duration: 2.0 }; // 4 ticks at 0.5, 1.0, 1.5, 2.0
        let mut sim = GameSimulation::new(config);
        sim.add_actor(ChampionId("NegMana".to_string()), champ.clone());
        sim.run(None);

        // Ticks: 4 ticks, total mana change: -5.0 * 4 = -20.0
        // Start mana: 10.0. End mana: 10.0 - 20.0 = -10.0
        // Since health remains 100.0, the champion is alive and ticks keep running.
        // And resource goes negative.
        assert_eq!(champ.borrow().state().resource.current, -10.0);
    }
}

struct ModifyStatsEvent {
    target: ChampionId,
    new_max_hp: f64,
    new_max_mana: f64,
}

impl SimEvent for ModifyStatsEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut EventManager) {
        if let Some(champ_ref) = ctx.champions.get(&self.target) {
            let mut champ = champ_ref.borrow_mut();
            champ.state_mut().stats.current.health = self.new_max_hp;
            champ.state_mut().stats.current.mana = self.new_max_mana;
        }
    }
    fn name(&self) -> &str {
        "ModifyStatsEvent"
    }
}

struct CheckStatsEvent {
    target: ChampionId,
    expected_hp: f64,
    expected_mana: f64,
}

impl SimEvent for CheckStatsEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut EventManager) {
        if let Some(champ_ref) = ctx.champions.get(&self.target) {
            let champ = champ_ref.borrow();
            assert_eq!(champ.state().health.current, self.expected_hp);
            assert_eq!(champ.state().resource.current, self.expected_mana);
        }
    }
    fn name(&self) -> &str {
        "CheckStatsEvent"
    }
}

/// Verify that resource regeneration caps at max values and adapts
/// dynamically when max values are increased or decreased mid-simulation.
#[test]
fn test_stress_resource_regeneration_capping_and_dynamic_max() {
    let mut base_stats = StatBlock::new();
    base_stats.health = 100.0;
    base_stats.health_regen = 10.0; // 2.0 per second -> 1.0 per tick
    base_stats.mana = 100.0;
    base_stats.mana_regen = 10.0; // 2.0 per second -> 1.0 per tick

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

    let champ = Rc::new(RefCell::new(
        Box::new(MockChampion { state }) as Box<dyn ChampionInstance>
    ));
    let champ_id = ChampionId("CapChamp".to_string());

    let config = SimConfig { max_duration: 3.0 };
    let mut sim = GameSimulation::new(config);
    sim.add_actor(champ_id.clone(), champ.clone());

    // Schedule stats modification at 1.1s (increase max hp/mana to 120.0)
    sim.event_manager_mut().schedule(
        SimTime::new(1.1),
        Box::new(ModifyStatsEvent {
            target: champ_id.clone(),
            new_max_hp: 120.0,
            new_max_mana: 120.0,
        }),
    );

    // Schedule check at 1.8s:
    // Ticks at 0.5s and 1.0s capped at 100.0
    // ModifyStatsEvent at 1.1s changes max to 120.0
    // Tick at 1.5s regenerates 1.0 -> 101.0
    sim.event_manager_mut().schedule(
        SimTime::new(1.8),
        Box::new(CheckStatsEvent {
            target: champ_id.clone(),
            expected_hp: 101.0,
            expected_mana: 101.0,
        }),
    );

    // Schedule stats modification at 2.1s (decrease max hp/mana to 80.0)
    sim.event_manager_mut().schedule(
        SimTime::new(2.1),
        Box::new(ModifyStatsEvent {
            target: champ_id.clone(),
            new_max_hp: 80.0,
            new_max_mana: 80.0,
        }),
    );

    // Run simulation
    sim.run(None);

    // At 3.0s (end of simulation), health and resource should be clamped at 80.0
    // At 2.5s tick: max synced to 80.0, health and resource clamped to 80.0, restore adds 1.0 but clamps again to 80.0.
    // At 3.0s tick: health and resource remain 80.0.
    assert_eq!(champ.borrow().state().health.current, 80.0);
    assert_eq!(champ.borrow().state().resource.current, 80.0);
}

/// Verify regeneration behavior for champions with ResourceType::None or zero stats.
#[test]
fn test_stress_resource_type_none_and_zero_stats() {
    // 1. ResourceType::None with positive health_regen: health should regenerate but resource should not
    {
        let mut base_stats = StatBlock::new();
        base_stats.health = 100.0;
        base_stats.health_regen = 10.0; // 2.0 per second -> 1.0 per tick
        base_stats.mana = 100.0;
        base_stats.mana_regen = 10.0;

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

        let champ = Rc::new(RefCell::new(
            Box::new(MockChampion { state }) as Box<dyn ChampionInstance>
        ));
        let config = SimConfig { max_duration: 2.0 };
        let mut sim = GameSimulation::new(config);
        sim.add_actor(ChampionId("NoneRes".to_string()), champ.clone());
        sim.run(None);

        // Health: 50.0 + 2.0 * 2.0 = 54.0
        assert_eq!(champ.borrow().state().health.current, 54.0);
        // Resource: Should remain 50.0 (no regeneration for ResourceType::None)
        assert_eq!(champ.borrow().state().resource.current, 50.0);
    }

    // 2. Zero regeneration stats with ResourceType::Energy: health shouldn't regenerate but energy should (flat 10/s)
    {
        let mut base_stats = StatBlock::new();
        base_stats.health = 100.0;
        base_stats.mana = 200.0; // Max energy capacity (stored in mana field)

        let mut state = ChampionState::new(
            1,
            base_stats,
            StatBlock::new(),
            ResourceType::Energy,
            StatBlock::new(),
            StatBlock::new(),
            Vec::new(),
        );
        state.health.current = 50.0;
        state.resource.current = 50.0;

        let champ = Rc::new(RefCell::new(
            Box::new(MockChampion { state }) as Box<dyn ChampionInstance>
        ));
        let config = SimConfig { max_duration: 2.0 };
        let mut sim = GameSimulation::new(config);
        sim.add_actor(ChampionId("EnergyZero".to_string()), champ.clone());
        sim.run(None);

        // Health: Should remain 50.0 (health_regen = 0.0)
        assert_eq!(champ.borrow().state().health.current, 50.0);
        // Energy: Should regenerate 10.0/s * 2.0 = 20.0 -> 50.0 + 20.0 = 70.0
        assert_eq!(champ.borrow().state().resource.current, 70.0);
    }
}

struct DealDamageEvent {
    target: ChampionId,
    damage: f64,
}

impl SimEvent for DealDamageEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut EventManager) {
        if let Some(champ_ref) = ctx.champions.get(&self.target) {
            let mut champ = champ_ref.borrow_mut();
            let _ = champ.take_damage(self.damage);
        }
    }
    fn name(&self) -> &str {
        "DealDamageEvent"
    }
}

/// Verify that champions dying mid-simulation stop regenerating both health and resources immediately.
#[test]
fn test_stress_regeneration_stops_on_death() {
    let mut base_stats = StatBlock::new();
    base_stats.health = 100.0;
    base_stats.health_regen = 10.0; // 2.0 HP per second -> 1.0 per tick
    base_stats.mana = 100.0;
    base_stats.mana_regen = 10.0; // 2.0 Mana per second -> 1.0 per tick

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

    let champ = Rc::new(RefCell::new(
        Box::new(MockChampion { state }) as Box<dyn ChampionInstance>
    ));
    let champ_id = ChampionId("DyingChamp".to_string());

    let config = SimConfig { max_duration: 4.0 };
    let mut sim = GameSimulation::new(config);
    sim.add_actor(champ_id.clone(), champ.clone());

    // Schedule lethal damage at 1.8s
    sim.event_manager_mut().schedule(
        SimTime::new(1.8),
        Box::new(DealDamageEvent {
            target: champ_id.clone(),
            damage: 100.0,
        }),
    );

    sim.run(None);

    // At 0.5s tick: health=51.0, mana=51.0
    // At 1.0s tick: health=52.0, mana=52.0
    // At 1.5s tick: health=53.0, mana=53.0
    // At 1.8s: DealDamageEvent deals 100.0 damage. health becomes 0.0 (clamped).
    // At 2.0s tick, 2.5s tick, 3.0s tick, 3.5s tick, 4.0s tick:
    //   health <= 0.0 check triggers, regeneration skips.
    // So final health should be 0.0 and final resource should be 53.0.
    assert_eq!(champ.borrow().state().health.current, 0.0);
    assert_eq!(champ.borrow().state().resource.current, 53.0);
}
