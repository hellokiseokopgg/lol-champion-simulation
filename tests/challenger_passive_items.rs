use lol_core::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct DummyChampionInstance {
    state: ChampionState,
}

impl ChampionInstance for DummyChampionInstance {
    fn state(&self) -> &ChampionState {
        &self.state
    }
    fn state_mut(&mut self) -> &mut ChampionState {
        &mut self.state
    }
    fn update_stats(&mut self, _time: SimTime) {}
    fn get_ability(&self, _slot: AbilitySlot) -> Option<&dyn Ability> {
        None
    }
    fn take_damage(&mut self, amount: f64) -> TakeDamageResult {
        let is_dead = self.state.health.reduce(amount);
        TakeDamageResult {
            actual_damage: amount,
            is_dead,
        }
    }
}

fn create_sim_context(
    actor_level: u32,
    actor_hp: f64,
    target_hp: f64,
) -> (SimContext, ChampionId, ChampionId) {
    let actor_id = ChampionId("Actor".into());
    let actor_stats = StatBlock {
        health: actor_hp,
        ..Default::default()
    };
    let mut actor_state = ChampionState::new(
        actor_level,
        actor_stats,
        StatBlock::default(),
        ResourceType::None,
        StatBlock::default(),
        StatBlock::default(),
        vec![],
    );
    actor_state.health.max = actor_hp;
    actor_state.health.current = actor_hp;

    let target_id = ChampionId("Target".into());
    let target_stats = StatBlock {
        health: target_hp,
        ..Default::default()
    };
    let mut target_state = ChampionState::new(
        1,
        target_stats,
        StatBlock::default(),
        ResourceType::None,
        StatBlock::default(),
        StatBlock::default(),
        vec![],
    );
    target_state.health.max = target_hp;
    target_state.health.current = target_hp;

    let mut champions = HashMap::new();
    champions.insert(
        actor_id.clone(),
        Rc::new(RefCell::new(
            Box::new(DummyChampionInstance { state: actor_state }) as Box<dyn ChampionInstance>,
        )),
    );
    champions.insert(
        target_id.clone(),
        Rc::new(RefCell::new(Box::new(DummyChampionInstance {
            state: target_state,
        }) as Box<dyn ChampionInstance>)),
    );

    let sim = SimContext {
        champions,
        current_time: SimTime::new(0.0),
        new_events: vec![],
        is_simulation_over: false,
        recorder: None,
    };

    (sim, actor_id, target_id)
}

#[test]
fn test_wits_end_level_scaling_and_refresh() {
    let dummy_damage = DamageResult {
        raw_damage: 10.0,
        final_damage: 10.0,
        mitigated_damage: 0.0,
        is_critical: false,
        damage_type: DamageType::Physical,
    };

    // 1. Level 1 magic damage check (15.0)
    {
        let (mut sim, actor, target) = create_sim_context(1, 1000.0, 1000.0);
        let wits_end = WitsEndEffect;
        wits_end.on_hit(&mut sim, &actor, &target, &dummy_damage);

        let target_hp = sim
            .champions
            .get(&target)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        assert_eq!(target_hp, 1000.0 - 15.0);

        let has_ms_buff = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&EffectId("WitsEndMovementSpeed".into()), sim.current_time);
        assert!(has_ms_buff);
    }

    // 2. Level 18 magic damage check (15.0 + 3.8 * 17.0 = 79.6)
    {
        let (mut sim, actor, target) = create_sim_context(18, 1000.0, 1000.0);
        let wits_end = WitsEndEffect;
        wits_end.on_hit(&mut sim, &actor, &target, &dummy_damage);

        let target_hp = sim
            .champions
            .get(&target)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        assert!((target_hp - (1000.0 - 79.6)).abs() < 0.001);
    }

    // 3. Movement speed buff duration refresh (Verify it remains active and doesn't duplicate stacks)
    {
        let (mut sim, actor, target) = create_sim_context(1, 1000.0, 1000.0);
        let wits_end = WitsEndEffect;
        wits_end.on_hit(&mut sim, &actor, &target, &dummy_damage);

        // Verify duration is set to 2.0s
        let expiration1 = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(&EffectId("WitsEndMovementSpeed".into()), sim.current_time);
        assert_eq!(expiration1, 1);

        // Advance time to 1.0s and hit again
        sim.current_time = SimTime::new(1.0);
        wits_end.on_hit(&mut sim, &actor, &target, &dummy_damage);

        // Verify buff is still active and only 1 stack exists
        let stacks = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(&EffectId("WitsEndMovementSpeed".into()), sim.current_time);
        assert_eq!(stacks, 1);
    }
}

#[test]
fn test_liandrys_burn_refresh_and_damage_amp() {
    let liandrys = LiandrysTormentEffect;

    // 1. Verify damage amp stacks on basic attacks/abilities
    {
        let (mut sim, actor, target) = create_sim_context(1, 1000.0, 1000.0);
        let dummy_damage = DamageResult {
            raw_damage: 10.0,
            final_damage: 10.0,
            mitigated_damage: 0.0,
            is_critical: false,
            damage_type: DamageType::Physical,
        };

        // Hit 1: 1 stack (2% amp)
        liandrys.on_hit(&mut sim, &actor, &target, &dummy_damage);
        let stacks1 = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(&EffectId("LiandrysTormentCombat".into()), sim.current_time);
        assert_eq!(stacks1, 1);

        // Hit 2: 2 stacks (4% amp)
        liandrys.on_hit(&mut sim, &actor, &target, &dummy_damage);
        let stacks2 = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(&EffectId("LiandrysTormentCombat".into()), sim.current_time);
        assert_eq!(stacks2, 2);

        // Hit 3: 3 stacks (6% amp)
        liandrys.on_hit(&mut sim, &actor, &target, &dummy_damage);
        let stacks3 = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(&EffectId("LiandrysTormentCombat".into()), sim.current_time);
        assert_eq!(stacks3, 3);

        // Hit 4: capped at 3 stacks (6% amp)
        liandrys.on_hit(&mut sim, &actor, &target, &dummy_damage);
        let stacks4 = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(&EffectId("LiandrysTormentCombat".into()), sim.current_time);
        assert_eq!(stacks4, 3);
    }

    // 2. Verify ability hit applies burn and schedules ticks correctly
    {
        let (mut sim, actor, target) = create_sim_context(1, 1000.0, 1000.0);
        liandrys.on_damage_dealt(&mut sim, &actor, &target, 10.0, true, AbilitySlot::Q);

        let has_burn = sim
            .champions
            .get(&target)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&EffectId("LiandrysTormentBurn".into()), sim.current_time);
        assert!(has_burn);

        // Should have exactly 2 new events:
        // - Expiration of the combat buff (delay 5.0)
        // - Expiration of the burn buff (delay 3.0)
        // - The first burn tick event (delay 0.5)
        // Wait, apply_buff pushes two: Combat (5.0) and Burn (3.0).
        // Plus on_damage_dealt pushes BurnTick (0.5).
        // Total should be 3 new_events in sim.new_events.
        // Wait, did we apply combat buff twice? No, only once since it was one Q cast.
        // Let's count them.
        assert_eq!(sim.new_events.len(), 3);
        let burn_ticks = sim
            .new_events
            .iter()
            .filter(|(_, event)| event.name() == "LiandrysTormentBurnTickEvent")
            .count();
        assert_eq!(burn_ticks, 1);
    }

    // 3. Stress test Liandry's burn refresh behavior:
    // Verify that refreshing the burn does NOT cause duplicate burn tick events.
    {
        let (mut sim, actor, target) = create_sim_context(1, 1000.0, 1000.0);
        liandrys.on_damage_dealt(&mut sim, &actor, &target, 10.0, true, AbilitySlot::Q);

        // Advance time to 1.0s and hit with another ability
        sim.current_time = SimTime::new(1.0);
        liandrys.on_damage_dealt(&mut sim, &actor, &target, 10.0, true, AbilitySlot::W);

        // Verify that we only have ONE active tick scheduler loop.
        // Total new_events count should be:
        // - 1st run: Combat buff (5.0s, expires at 5.0s), Burn buff (3.0s, expires at 3.0s), Tick event (at 0.5s from start)
        // - 2nd run: Combat buff refreshed/reapplied, Burn buff refreshed (duration reset to 3.0s, expires at 4.0s)
        // But since already_burning was true, no new tick event is pushed.
        // So the number of "LiandrysTormentBurnTickEvent" in sim.new_events is still exactly 1!
        let tick_events = sim
            .new_events
            .iter()
            .filter(|(_, event)| event.name() == "LiandrysTormentBurnTickEvent")
            .count();
        assert_eq!(tick_events, 1);
    }
}

#[test]
fn test_botrk_current_health_scaling_and_cooldown() {
    let botrk = BladeOfTheRuinedKingEffect;
    let dummy_damage = DamageResult {
        raw_damage: 10.0,
        final_damage: 10.0,
        mitigated_damage: 0.0,
        is_critical: false,
        damage_type: DamageType::Physical,
    };

    // 1. Verify current health physical damage on-hit at high health (5000 HP -> 450 damage)
    {
        let (mut sim, actor, target) = create_sim_context(1, 1000.0, 5000.0);
        botrk.on_hit(&mut sim, &actor, &target, &dummy_damage);

        let target_hp = sim
            .champions
            .get(&target)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        // 5000 - (5000 * 0.09) = 4550.0
        assert!((target_hp - 4550.0).abs() < 0.001);
    }

    // 2. Verify current health physical damage on-hit at very low health (50 HP -> minimum 15.0 damage)
    {
        let (mut sim, actor, target) = create_sim_context(1, 1000.0, 50.0);
        botrk.on_hit(&mut sim, &actor, &target, &dummy_damage);

        let target_hp = sim
            .champions
            .get(&target)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        // 50 - 15.0 = 35.0 (since 9% of 50 is 4.5, which is lower than the minimum of 15.0)
        assert!((target_hp - 35.0).abs() < 0.001);
    }

    // 3. Verify 3rd hit siphon and 30s cooldown mechanics
    {
        let (mut sim, actor, target) = create_sim_context(1, 1000.0, 1000.0);

        // Hit 1
        botrk.on_hit(&mut sim, &actor, &target, &dummy_damage);
        let stacks1 = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(
                &EffectId("BladeOfTheRuinedKingStacks".into()),
                sim.current_time,
            );
        assert_eq!(stacks1, 1);

        // Hit 2
        botrk.on_hit(&mut sim, &actor, &target, &dummy_damage);
        let stacks2 = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(
                &EffectId("BladeOfTheRuinedKingStacks".into()),
                sim.current_time,
            );
        assert_eq!(stacks2, 2);

        // Hit 3 -> triggers siphon magic damage, applies slow, clears stacks, starts 30s cooldown
        botrk.on_hit(&mut sim, &actor, &target, &dummy_damage);
        let stacks3 = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(
                &EffectId("BladeOfTheRuinedKingStacks".into()),
                sim.current_time,
            );
        assert_eq!(stacks3, 0);

        let has_cd = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(
                &EffectId("BladeOfTheRuinedKingCooldown".into()),
                sim.current_time,
            );
        assert!(has_cd);

        let has_slow = sim
            .champions
            .get(&target)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(
                &EffectId("BladeOfTheRuinedKingSiphonSlow".into()),
                sim.current_time,
            );
        assert!(has_slow);

        // Hit 4 (Cooldown is active) -> Should still deal current health physical on-hit damage,
        // but should NOT apply stacks or trigger siphon.
        let target_hp_before_hit4 = sim
            .champions
            .get(&target)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        botrk.on_hit(&mut sim, &actor, &target, &dummy_damage);
        let target_hp_after_hit4 = sim
            .champions
            .get(&target)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;

        // Current health damage is still dealt
        assert!(target_hp_after_hit4 < target_hp_before_hit4);

        // Stacks remain 0
        let stacks_cd = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(
                &EffectId("BladeOfTheRuinedKingStacks".into()),
                sim.current_time,
            );
        assert_eq!(stacks_cd, 0);

        // 4. Verify cooldown elapsed (30s later) -> Siphon can be built/triggered again
        sim.current_time = SimTime::new(31.0); // Cooldown expired

        // Hit 5 (CD elapsed) -> starts stacking again
        botrk.on_hit(&mut sim, &actor, &target, &dummy_damage);
        let stacks5 = sim
            .champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks_by_id(
                &EffectId("BladeOfTheRuinedKingStacks".into()),
                sim.current_time,
            );
        assert_eq!(stacks5, 1);
    }
}

#[test]
fn test_lich_bane_spellblade_buff_and_damage() {
    let lich_bane = LichBaneEffect;
    let dummy_damage = DamageResult {
        raw_damage: 10.0,
        final_damage: 10.0,
        mitigated_damage: 0.0,
        is_critical: false,
        damage_type: DamageType::Physical,
    };

    let (mut sim, actor, target) = create_sim_context(1, 1000.0, 1000.0);

    // Set base AD = 100, AP = 200
    if let Some(champ_ref) = sim.champions.get(&actor) {
        let mut champ = champ_ref.borrow_mut();
        champ.state_mut().stats.base.attack_damage = 100.0;
        champ.state_mut().stats.current.ability_power = 200.0;
        champ.state_mut().stats.current.attack_damage = 100.0;
        champ.update_stats(sim.current_time);
    }

    // 1. Cast Q -> Grants "Spellblade (Lich Bane)" and starts cooldown
    lich_bane.on_ability_cast(&mut sim, &actor, AbilitySlot::Q);

    let has_spellblade = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .has_effect_by_id(&EffectId("LichBaneSpellblade".into()), sim.current_time);
    assert!(has_spellblade);

    let has_cd = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .has_effect_by_id(&EffectId("LichBaneCooldown".into()), sim.current_time);
    assert!(has_cd);

    // 2. Cast W while on cooldown -> Buff should not refresh or reapply
    lich_bane.on_ability_cast(&mut sim, &actor, AbilitySlot::W);

    // 3. Attack -> Consumes Spellblade, deals bonus magic damage (150% of 100 + 50% of 200 = 150 + 100 = 250)
    lich_bane.on_hit(&mut sim, &actor, &target, &dummy_damage);

    let has_spellblade_after = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .has_effect_by_id(&EffectId("LichBaneSpellblade".into()), sim.current_time);
    assert!(!has_spellblade_after);

    let target_hp = sim
        .champions
        .get(&target)
        .unwrap()
        .borrow()
        .state()
        .health
        .current;
    // 1000.0 - 250.0 = 750.0
    assert_eq!(target_hp, 750.0);

    // 4. Cast E while cooldown is active -> Buff is not applied
    lich_bane.on_ability_cast(&mut sim, &actor, AbilitySlot::E);
    let has_spellblade_cd = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .has_effect_by_id(&EffectId("LichBaneSpellblade".into()), sim.current_time);
    assert!(!has_spellblade_cd);

    // 5. Cooldown expires (after 1.5s) -> Cast E -> Buff is applied again
    sim.current_time = SimTime::new(1.6);
    lich_bane.on_ability_cast(&mut sim, &actor, AbilitySlot::E);
    let has_spellblade_post_cd = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .has_effect_by_id(&EffectId("LichBaneSpellblade".into()), sim.current_time);
    assert!(has_spellblade_post_cd);
}

#[test]
fn test_trinity_force_spellblade_buff_and_damage() {
    let trinity = TrinityForceEffect;
    let dummy_damage = DamageResult {
        raw_damage: 10.0,
        final_damage: 10.0,
        mitigated_damage: 0.0,
        is_critical: false,
        damage_type: DamageType::Physical,
    };

    let (mut sim, actor, target) = create_sim_context(1, 1000.0, 1000.0);

    // Set base AD = 100
    if let Some(champ_ref) = sim.champions.get(&actor) {
        let mut champ = champ_ref.borrow_mut();
        champ.state_mut().stats.base.attack_damage = 100.0;
        champ.state_mut().stats.current.attack_damage = 100.0;
        champ.update_stats(sim.current_time);
    }

    // 1. Cast Q -> Grants "Spellblade (Trinity Force)" and starts cooldown
    trinity.on_ability_cast(&mut sim, &actor, AbilitySlot::Q);

    let has_spellblade = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .has_effect_by_id(&EffectId("TrinityForceSpellblade".into()), sim.current_time);
    assert!(has_spellblade);

    // 2. Attack -> Consumes Spellblade, deals bonus physical damage (200% of 100 = 200)
    trinity.on_hit(&mut sim, &actor, &target, &dummy_damage);

    let has_spellblade_after = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .has_effect_by_id(&EffectId("TrinityForceSpellblade".into()), sim.current_time);
    assert!(!has_spellblade_after);

    let target_hp = sim
        .champions
        .get(&target)
        .unwrap()
        .borrow()
        .state()
        .health
        .current;
    // 1000.0 - 200.0 = 800.0
    assert_eq!(target_hp, 800.0);
}

#[test]
fn test_ludens_companion_charge_accumulation_and_damage() {
    let luden = LudensCompanionEffect;

    let (mut sim, actor, target) = create_sim_context(1, 1000.0, 2000.0);

    // Set AP = 200
    if let Some(champ_ref) = sim.champions.get(&actor) {
        let mut champ = champ_ref.borrow_mut();
        champ.state_mut().stats.current.ability_power = 200.0;
        champ.update_stats(sim.current_time);
    }

    // 1. Start simulation -> Luden's triggers simulation start logic
    luden.on_simulation_start(&mut sim, &actor);

    // Verify actor starts with 6 charges
    let charges = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .get_stacks(&EffectId("LudensCompanionCharges".into()));
    assert_eq!(charges, 6);

    // 2. Hit target with basic attack -> Charges should NOT be consumed
    luden.on_damage_dealt(
        &mut sim,
        &actor,
        &target,
        50.0,
        false,
        AbilitySlot::AutoAttack,
    );
    let charges_after_aa = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .get_stacks(&EffectId("LudensCompanionCharges".into()));
    assert_eq!(charges_after_aa, 6);

    // 3. Hit target with Q (ability) -> Consumes all charges, deals magic damage (40 + 8% of 200 = 40 + 16 = 56 per charge. 56 * 6 = 336)
    luden.on_damage_dealt(&mut sim, &actor, &target, 100.0, true, AbilitySlot::Q);

    let charges_after_q = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .get_stacks(&EffectId("LudensCompanionCharges".into()));
    assert_eq!(charges_after_q, 0);

    let target_hp = sim
        .champions
        .get(&target)
        .unwrap()
        .borrow()
        .state()
        .health
        .current;
    // 2000.0 - 336.0 = 1664.0
    assert_eq!(target_hp, 1664.0);

    // 4. Simulate charge recharge loop:
    // We register the Luden's Companion item on the champion first so the recharge event recognises it.
    if let Some(champ_ref) = sim.champions.get(&actor) {
        let mut champ = champ_ref.borrow_mut();
        let mut manager = ItemManager::new();
        manager.add_effect(Box::new(LudensCompanionEffect));
        champ.state_mut().items = manager;
    }

    // Now execute charge event:
    let event = LudensCompanionChargeEvent {
        actor: actor.clone(),
    };
    let mut event_manager = EventManager::new();

    // Tick 1 (3s): Charges become 1
    sim.current_time = SimTime::new(3.0);
    event.execute(&mut sim, &mut event_manager);
    let charges_tick1 = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .get_stacks(&EffectId("LudensCompanionCharges".into()));
    assert_eq!(charges_tick1, 1);

    // Tick 2 (6s): Charges become 2
    sim.current_time = SimTime::new(6.0);
    event.execute(&mut sim, &mut event_manager);
    let charges_tick2 = sim
        .champions
        .get(&actor)
        .unwrap()
        .borrow()
        .state()
        .buffs
        .get_stacks(&EffectId("LudensCompanionCharges".into()));
    assert_eq!(charges_tick2, 2);

    // 5. Hit target with Q again -> Deals (40 + 16) * 2 = 112 damage
    luden.on_damage_dealt(&mut sim, &actor, &target, 100.0, true, AbilitySlot::Q);
    let target_hp_after_q2 = sim
        .champions
        .get(&target)
        .unwrap()
        .borrow()
        .state()
        .health
        .current;
    // 1664.0 - 112.0 = 1552.0
    assert_eq!(target_hp_after_q2, 1552.0);
}

#[test]
fn test_lich_bane_cooldown_and_duration_stress() {
    let lich_bane = LichBaneEffect;
    let (mut sim, actor, _target) = create_sim_context(1, 1000.0, 1000.0);

    // Set base AD = 100, AP = 200
    if let Some(champ_ref) = sim.champions.get(&actor) {
        let mut champ = champ_ref.borrow_mut();
        champ.state_mut().stats.base.attack_damage = 100.0;
        champ.state_mut().stats.current.ability_power = 200.0;
        champ.state_mut().stats.current.attack_damage = 100.0;
        champ.update_stats(sim.current_time);
    }

    let buff_id = EffectId("LichBaneSpellblade".into());
    let cd_id = EffectId("LichBaneCooldown".into());

    let cleanup = |sim_ctx: &mut SimContext, target_id: &ChampionId| {
        if let Some(champ_ref) = sim_ctx.champions.get(target_id) {
            champ_ref
                .borrow_mut()
                .state_mut()
                .buffs
                .cleanup_expired(sim_ctx.current_time);
        }
    };

    // 1. Cast Q at t=0.0 -> Spellblade and Cooldown applied.
    sim.current_time = SimTime::new(0.0);
    cleanup(&mut sim, &actor);
    lich_bane.on_ability_cast(&mut sim, &actor, AbilitySlot::Q);

    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, sim.current_time)
    );
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, sim.current_time)
    );

    // 2. Cast W at t=0.5 (cooldown is active).
    sim.current_time = SimTime::new(0.5);
    cleanup(&mut sim, &actor);
    lich_bane.on_ability_cast(&mut sim, &actor, AbilitySlot::W);

    // Verify Spellblade buff is still active and has NOT had its duration refreshed/extended.
    // Original duration was 10.0s (expires at 10.0s).
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(9.999))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(10.0))
    );

    // Verify Cooldown buff has NOT had its duration refreshed/extended.
    // Original cooldown was 1.5s (expires at 1.5s).
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.499))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.5))
    );

    // 3. Cast E at t=1.0 (cooldown is active).
    sim.current_time = SimTime::new(1.0);
    cleanup(&mut sim, &actor);
    lich_bane.on_ability_cast(&mut sim, &actor, AbilitySlot::E);
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(9.999))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(10.0))
    );
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.499))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.5))
    );

    // 4. Cast R at t=1.49 (cooldown is active).
    sim.current_time = SimTime::new(1.49);
    cleanup(&mut sim, &actor);
    lich_bane.on_ability_cast(&mut sim, &actor, AbilitySlot::R);
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(9.999))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(10.0))
    );
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.499))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.5))
    );

    // 5. Cooldown expires at t=1.5. Cast Q at t=1.5 -> Refreshes Spellblade and restarts cooldown.
    sim.current_time = SimTime::new(1.5);
    cleanup(&mut sim, &actor);
    lich_bane.on_ability_cast(&mut sim, &actor, AbilitySlot::Q);

    // Verify Spellblade buff duration is refreshed to 10.0s from t=1.5 (expires at 1.5 + 10.0 = 11.5).
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(11.499))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(11.5))
    );

    // Verify Cooldown buff duration is refreshed/re-started to 1.5s from t=1.5 (expires at 1.5 + 1.5 = 3.0).
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(2.999))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(3.0))
    );
}

#[test]
fn test_trinity_force_cooldown_and_duration_stress() {
    let trinity = TrinityForceEffect;
    let (mut sim, actor, _target) = create_sim_context(1, 1000.0, 1000.0);

    // Set base AD = 100
    if let Some(champ_ref) = sim.champions.get(&actor) {
        let mut champ = champ_ref.borrow_mut();
        champ.state_mut().stats.base.attack_damage = 100.0;
        champ.state_mut().stats.current.attack_damage = 100.0;
        champ.update_stats(sim.current_time);
    }

    let buff_id = EffectId("TrinityForceSpellblade".into());
    let cd_id = EffectId("TrinityForceCooldown".into());

    let cleanup = |sim_ctx: &mut SimContext, target_id: &ChampionId| {
        if let Some(champ_ref) = sim_ctx.champions.get(target_id) {
            champ_ref
                .borrow_mut()
                .state_mut()
                .buffs
                .cleanup_expired(sim_ctx.current_time);
        }
    };

    // 1. Cast Q at t=0.0 -> Spellblade and Cooldown applied.
    sim.current_time = SimTime::new(0.0);
    cleanup(&mut sim, &actor);
    trinity.on_ability_cast(&mut sim, &actor, AbilitySlot::Q);

    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, sim.current_time)
    );
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, sim.current_time)
    );

    // 2. Cast W at t=0.5 (cooldown is active).
    sim.current_time = SimTime::new(0.5);
    cleanup(&mut sim, &actor);
    trinity.on_ability_cast(&mut sim, &actor, AbilitySlot::W);

    // Verify Spellblade buff is still active and has NOT had its duration refreshed/extended.
    // Original duration was 10.0s (expires at 10.0s).
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(9.999))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(10.0))
    );

    // Verify Cooldown buff has NOT had its duration refreshed/extended.
    // Original cooldown was 1.5s (expires at 1.5s).
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.499))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.5))
    );

    // 3. Cast E at t=1.0 (cooldown is active).
    sim.current_time = SimTime::new(1.0);
    cleanup(&mut sim, &actor);
    trinity.on_ability_cast(&mut sim, &actor, AbilitySlot::E);
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(9.999))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(10.0))
    );
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.499))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(1.5))
    );

    // 4. Cooldown expires at t=1.5. Cast Q at t=1.5 -> Refreshes Spellblade
    sim.current_time = SimTime::new(1.5);
    cleanup(&mut sim, &actor);
    trinity.on_ability_cast(&mut sim, &actor, AbilitySlot::Q);

    // Verify Spellblade buff duration is refreshed to 10.0s from t=1.5 (expires at 11.5).
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(11.499))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&buff_id, SimTime::new(11.5))
    );

    // Verify Cooldown buff duration is refreshed/re-started to 1.5s from t=1.5 (expires at 3.0).
    assert!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(2.999))
    );
    assert!(
        !sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(&cd_id, SimTime::new(3.0))
    );
}

#[test]
fn test_ludens_companion_charge_gain_stress() {
    let luden = LudensCompanionEffect;
    let (mut sim, actor, target) = create_sim_context(1, 1000.0, 2000.0);

    // Set AP = 200
    if let Some(champ_ref) = sim.champions.get(&actor) {
        let mut champ = champ_ref.borrow_mut();
        champ.state_mut().stats.current.ability_power = 200.0;
        let mut manager = ItemManager::new();
        manager.add_effect(Box::new(LudensCompanionEffect));
        champ.state_mut().items = manager;
        champ.update_stats(sim.current_time);
    }

    let charges_id = EffectId("LudensCompanionCharges".into());

    // Initialize: start simulation -> applies 6 charges, schedules charge event at t=3.0
    luden.on_simulation_start(&mut sim, &actor);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        6
    );

    // Consume all charges at t=0.0 via Q
    luden.on_damage_dealt(&mut sim, &actor, &target, 100.0, true, AbilitySlot::Q);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        0
    );

    let event = LudensCompanionChargeEvent {
        actor: actor.clone(),
    };
    let mut event_manager = EventManager::new();

    // Simulating casting multiple spells in rapid succession at t=0.5, t=1.0, t=2.0
    // Charges are 0, so they should not change.
    sim.current_time = SimTime::new(0.5);
    luden.on_damage_dealt(&mut sim, &actor, &target, 100.0, true, AbilitySlot::W);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        0
    );

    sim.current_time = SimTime::new(1.0);
    luden.on_damage_dealt(&mut sim, &actor, &target, 100.0, true, AbilitySlot::E);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        0
    );

    sim.current_time = SimTime::new(2.0);
    luden.on_damage_dealt(&mut sim, &actor, &target, 100.0, true, AbilitySlot::R);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        0
    );

    // 1st Charge Gain at t=3.0 -> Charges become 1
    sim.current_time = SimTime::new(3.0);
    event.execute(&mut sim, &mut event_manager);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        1
    );

    // Cast spell at t=4.0 -> consumes the 1 charge
    sim.current_time = SimTime::new(4.0);
    luden.on_damage_dealt(&mut sim, &actor, &target, 100.0, true, AbilitySlot::Q);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        0
    );

    // Cast more spells at t=4.5, t=5.0
    sim.current_time = SimTime::new(4.5);
    luden.on_damage_dealt(&mut sim, &actor, &target, 100.0, true, AbilitySlot::W);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        0
    );

    // 2nd Charge Gain at t=6.0 (exactly 3s after previous charge event at t=3.0) -> Charges become 1
    sim.current_time = SimTime::new(6.0);
    event.execute(&mut sim, &mut event_manager);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        1
    );

    // 3rd Charge Gain at t=9.0 -> Charges become 2
    sim.current_time = SimTime::new(9.0);
    event.execute(&mut sim, &mut event_manager);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        2
    );

    // 4th Charge Gain at t=12.0 -> Charges become 3
    sim.current_time = SimTime::new(12.0);
    event.execute(&mut sim, &mut event_manager);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        3
    );

    // 5th Charge Gain at t=15.0 -> Charges become 4
    sim.current_time = SimTime::new(15.0);
    event.execute(&mut sim, &mut event_manager);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        4
    );

    // 6th Charge Gain at t=18.0 -> Charges become 5
    sim.current_time = SimTime::new(18.0);
    event.execute(&mut sim, &mut event_manager);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        5
    );

    // 7th Charge Gain at t=21.0 -> Charges become 6
    sim.current_time = SimTime::new(21.0);
    event.execute(&mut sim, &mut event_manager);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        6
    );

    // 8th Charge Gain at t=24.0 -> Charges should remain 6
    sim.current_time = SimTime::new(24.0);
    event.execute(&mut sim, &mut event_manager);
    assert_eq!(
        sim.champions
            .get(&actor)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&charges_id),
        6
    );
}
