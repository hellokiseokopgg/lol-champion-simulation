use lol_core::damage::DamagePipeline;
use lol_core::rune_manager::{Electrocute, RuneEffect};
use lol_core::stats::StatBlock;
use lol_core::types::{AbilitySlot, DamageType, SimTime};

#[test]
fn test_electrocute_cooldown_by_level() {
    // Cooldown formula: 25.0 - (25.0 - 20.0) / 17.0 * (level - 1)

    // Level 1: Cooldown should be 25.0s
    {
        let mut rune = Electrocute::new();
        let base_stats = StatBlock::new();
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);

        // Trigger Electrocute at t=0.0
        let attacker_stats = StatBlock::new();
        let _ = rune.on_damage_dealt(
            SimTime::new(0.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(0.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            1,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(1.0),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert!(!events.is_empty(), "Electrocute should trigger at level 1");

        // Try triggering again at t=25.9s (should not trigger if cooldown is 25.0s and hits spaced)
        // Let's test cooldown boundary: 24.9s after first proc (which was at t=1.0s, so t=25.9s)
        // 25.9s - 1.0s = 24.9s (less than 25.0s cooldown)
        let _ = rune.on_damage_dealt(
            SimTime::new(25.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(25.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            1,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(25.9),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert!(
            events.is_empty(),
            "Electrocute should still be on cooldown at 24.9s"
        );

        // Trigger at t=26.1s (25.1s after first proc, cooldown elapsed)
        let _ = rune.on_damage_dealt(
            SimTime::new(26.1),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(26.6),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            1,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(27.1),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert!(
            !events.is_empty(),
            "Electrocute should trigger after 25s cooldown"
        );
    }

    // Level 18: Cooldown should be 20.0s
    {
        let mut rune = Electrocute::new();
        let base_stats = StatBlock::new();
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 18, 1.0);

        // Trigger at t=0.0
        let attacker_stats = StatBlock::new();
        let _ = rune.on_damage_dealt(
            SimTime::new(0.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            18,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(0.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            18,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(1.0),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            18,
        );
        assert!(!events.is_empty(), "Electrocute should trigger at level 18");

        // Try triggering at t=20.9s (19.9s elapsed)
        let _ = rune.on_damage_dealt(
            SimTime::new(20.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            18,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(20.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            18,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(20.9),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            18,
        );
        assert!(
            events.is_empty(),
            "Electrocute should still be on cooldown at 19.9s"
        );

        // Trigger at t=21.1s (20.1s elapsed)
        let _ = rune.on_damage_dealt(
            SimTime::new(21.1),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            18,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(21.6),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            18,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(22.1),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            18,
        );
        assert!(
            !events.is_empty(),
            "Electrocute should trigger after 20s cooldown"
        );
    }
}

#[test]
fn test_pta_damage_amplification_types() {
    let attacker = StatBlock::new();
    let defender_no_pta = StatBlock::new();
    let mut defender_with_pta = StatBlock::new();
    defender_with_pta.damage_reduction_percent = -0.08; // PTA 8% amplification

    // 1. Physical Damage: 100 raw, 0 armor -> should be 100 vs 108
    {
        let res_no_pta = DamagePipeline::process(
            100.0,
            DamageType::Physical,
            false,
            &attacker,
            &defender_no_pta,
        );
        let res_pta = DamagePipeline::process(
            100.0,
            DamageType::Physical,
            false,
            &attacker,
            &defender_with_pta,
        );
        assert_eq!(res_no_pta.final_damage, 100.0);
        assert_eq!(res_pta.final_damage, 108.0);
    }

    // 2. Magic Damage: 100 raw, 0 MR -> should be 100 vs 108
    {
        let res_no_pta =
            DamagePipeline::process(100.0, DamageType::Magic, false, &attacker, &defender_no_pta);
        let res_pta = DamagePipeline::process(
            100.0,
            DamageType::Magic,
            false,
            &attacker,
            &defender_with_pta,
        );
        assert_eq!(res_no_pta.final_damage, 100.0);
        assert_eq!(res_pta.final_damage, 108.0);
    }

    // 3. True Damage: 100 raw -> should be 100 vs 100 (True damage ignores amplification/reduction)
    {
        let res_no_pta =
            DamagePipeline::process(100.0, DamageType::True, false, &attacker, &defender_no_pta);
        let res_pta = DamagePipeline::process(
            100.0,
            DamageType::True,
            false,
            &attacker,
            &defender_with_pta,
        );
        assert_eq!(res_no_pta.final_damage, 100.0);
        assert_eq!(res_pta.final_damage, 100.0);
    }
}

#[test]
fn test_electrocute_same_slot_overwrite() {
    let mut rune = Electrocute::new();
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // Sequence: AA (t=0.0) -> Q (t=1.0) -> AA (t=2.0)
    // At t=0.0, AA hits
    let events = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());

    // At t=1.0, Q hits
    let events = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());

    // At t=2.0, AA hits again.
    // In our implementation, the first AA is removed, and the second AA is pushed.
    // Total distinct slots in recent_hits is 2 ([Q, AA]), so it does NOT trigger.
    let events = rune.on_damage_dealt(
        SimTime::new(2.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(
        events.is_empty(),
        "Electrocute should NOT trigger on AA -> Q -> AA in current implementation because same-slot overwrites"
    );
}

#[test]
fn test_electrocute_window_duration_boundary() {
    let mut rune = Electrocute::new();
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // Test window boundary: 3.15s (current implementation uses 3.15s limit)
    // Hit 1: Q at t=0.0
    let _ = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );
    // Hit 2: W at t=1.5
    let _ = rune.on_damage_dealt(
        SimTime::new(1.5),
        10.0,
        true,
        AbilitySlot::W,
        &attacker_stats,
        1,
    );
    // Hit 3: E at t=3.1
    // 3.1 - 0.0 = 3.1 <= 3.15 -> should trigger
    let events = rune.on_damage_dealt(
        SimTime::new(3.1),
        10.0,
        true,
        AbilitySlot::E,
        &attacker_stats,
        1,
    );
    assert!(
        !events.is_empty(),
        "Electrocute should trigger at 3.1s since 3.1 <= 3.15"
    );

    // Reset rune cooldown
    rune.last_proc_time = -999.0;

    // Hit 1: Q at t=0.0
    let _ = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );
    // Hit 2: W at t=1.5
    let _ = rune.on_damage_dealt(
        SimTime::new(1.5),
        10.0,
        true,
        AbilitySlot::W,
        &attacker_stats,
        1,
    );
    // Hit 3: E at t=3.20
    // 3.2 - 0.0 = 3.2 > 3.15 -> first hit should be cleaned up, should not trigger
    let events = rune.on_damage_dealt(
        SimTime::new(3.2),
        10.0,
        true,
        AbilitySlot::E,
        &attacker_stats,
        1,
    );
    assert!(
        events.is_empty(),
        "Electrocute should NOT trigger at 3.2s because window expired (3.2 > 3.15)"
    );
}

#[test]
fn test_pta_stack_decay_boundary() {
    let mut rune = lol_core::rune_manager::PressTheAttack::new(true);
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // Hit 1: AA at t=0.0
    let _ = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert_eq!(rune.stacks, 1);

    // Hit 2: AA at t=4.0. Time difference is exactly 4.0 -> should not decay (limit is > 4.0)
    let _ = rune.on_damage_dealt(
        SimTime::new(4.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert_eq!(rune.stacks, 2);

    // Reset stacks
    rune.stacks = 1;
    rune.last_attack_time = 0.0;

    // Hit 2: AA at t=4.01. Time difference is 4.01 > 4.0 -> should decay and reset to 1
    let _ = rune.on_damage_dealt(
        SimTime::new(4.01),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert_eq!(rune.stacks, 1);
}

#[test]
fn test_pta_exposure_reset_and_immediate_stacking() {
    let mut rune = lol_core::rune_manager::PressTheAttack::new(true);
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // Trigger PTA at t=0.0
    let _ = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    let _ = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    let events = rune.on_damage_dealt(
        SimTime::new(2.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(!events.is_empty());
    assert!(rune.was_exposed);
    assert_eq!(rune.last_trigger_time, 2.0);

    // AA at t=7.9 (5.9s since trigger) -> still exposed, should be ignored
    let _ = rune.on_damage_dealt(
        SimTime::new(7.9),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert_eq!(rune.stacks, 0);

    // AA at t=8.0 (exactly 6.0s since trigger) -> resets exposure and starts stacking (stacks = 1)
    let _ = rune.on_damage_dealt(
        SimTime::new(8.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(!rune.was_exposed);
    assert_eq!(rune.stacks, 1);
}

#[test]
fn test_electrocute_adaptive_damage_type() {
    let base_stats = StatBlock {
        attack_damage: 80.0,
        ..Default::default()
    };
    let mut attacker_stats = StatBlock {
        attack_damage: 80.0,
        ability_power: 0.0,
        ..Default::default()
    };

    // Case 1: bonus AD (50) > AP (40) -> Physical
    {
        let mut rune = Electrocute::new();
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
        attacker_stats.attack_damage = 130.0; // 50 bonus AD
        attacker_stats.ability_power = 40.0; // 40 AP
        let _ = rune.on_damage_dealt(
            SimTime::new(0.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(0.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            1,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(1.0),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert!(!events.is_empty());
        match &events[0] {
            lol_core::rune_manager::RuneEvent::DamageDealt { damage_type, .. } => {
                assert_eq!(*damage_type, DamageType::Physical);
            }
            _ => panic!("Expected DamageDealt"),
        }
    }

    // Case 2: bonus AD (40) < AP (50) -> Magic
    {
        let mut rune = Electrocute::new();
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
        attacker_stats.attack_damage = 120.0; // 40 bonus AD
        attacker_stats.ability_power = 50.0; // 50 AP
        let _ = rune.on_damage_dealt(
            SimTime::new(0.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(0.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            1,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(1.0),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert!(!events.is_empty());
        match &events[0] {
            lol_core::rune_manager::RuneEvent::DamageDealt { damage_type, .. } => {
                assert_eq!(*damage_type, DamageType::Magic);
            }
            _ => panic!("Expected DamageDealt"),
        }
    }

    // Case 3: bonus AD (50) == AP (50) -> Magic (fallback)
    {
        let mut rune = Electrocute::new();
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
        attacker_stats.attack_damage = 130.0; // 50 bonus AD
        attacker_stats.ability_power = 50.0; // 50 AP
        let _ = rune.on_damage_dealt(
            SimTime::new(0.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(0.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            1,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(1.0),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert!(!events.is_empty());
        match &events[0] {
            lol_core::rune_manager::RuneEvent::DamageDealt { damage_type, .. } => {
                assert_eq!(*damage_type, DamageType::Magic);
            }
            _ => panic!("Expected DamageDealt"),
        }
    }
}

#[test]
fn test_electrocute_item_ignored_sequence() {
    let mut rune = Electrocute::new();
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // Hit 1: AA at t=0.0
    let events = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());

    // Hit 2: Item active at t=0.5 -> ignored completely, does not count as hit
    let events = rune.on_damage_dealt(
        SimTime::new(0.5),
        10.0,
        true,
        AbilitySlot::Item(6631),
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());

    // Hit 3: Q at t=1.0
    let events = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());

    // Hit 4: E at t=1.5
    // Sequence of non-item slots: AA, Q, E -> 3 unique hits within 3s window. Should trigger.
    let events = rune.on_damage_dealt(
        SimTime::new(1.5),
        10.0,
        true,
        AbilitySlot::E,
        &attacker_stats,
        1,
    );
    assert_eq!(
        events.len(),
        1,
        "Electrocute should trigger since Item is ignored"
    );
}

#[test]
fn test_electrocute_overwrite_and_trigger() {
    let mut rune = Electrocute::new();
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // Hit 1: Q at t=0.0
    let _ = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );

    // Hit 2: W at t=0.5
    let _ = rune.on_damage_dealt(
        SimTime::new(0.5),
        10.0,
        true,
        AbilitySlot::W,
        &attacker_stats,
        1,
    );

    // Hit 3: Q at t=1.0 (overwrites Hit 1 Q, leaving W and Q in recent_hits)
    let _ = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );
    assert_eq!(rune.recent_hits.len(), 2);

    // Hit 4: E at t=1.5 (triggers Electrocute with W, Q, E)
    let events = rune.on_damage_dealt(
        SimTime::new(1.5),
        10.0,
        true,
        AbilitySlot::E,
        &attacker_stats,
        1,
    );
    assert_eq!(
        events.len(),
        1,
        "Electrocute should trigger on overwritten sequence"
    );
}

#[test]
fn test_pta_proc_damage_amplification() {
    let mut rune = lol_core::rune_manager::PressTheAttack::new(true);
    let base_stats = StatBlock {
        attack_damage: 80.0,
        ..Default::default()
    };
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);

    let attacker_stats = StatBlock {
        attack_damage: 80.0,
        ..Default::default()
    };
    let mut defender_stats = StatBlock {
        armor: 0.0,
        magic_resist: 0.0,
        ..Default::default()
    };

    // 1st attack
    let _ = rune.on_damage_dealt(
        SimTime::new(0.0),
        80.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    // 2nd attack
    let _ = rune.on_damage_dealt(
        SimTime::new(1.0),
        80.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    // 3rd attack triggers PTA
    let events = rune.on_damage_dealt(
        SimTime::new(2.0),
        80.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );

    assert_eq!(events.len(), 3);

    // Event 1 is ApplyDebuff
    let damage_reduction_percent = if let lol_core::rune_manager::RuneEvent::ApplyDebuff {
        damage_reduction_percent: dr,
        ..
    } = &events[1]
    {
        *dr
    } else {
        panic!("Expected ApplyDebuff at index 1");
    };

    // Apply the debuff to defender stats
    defender_stats.damage_reduction_percent = damage_reduction_percent;

    // Event 2 is DamageDealt
    if let lol_core::rune_manager::RuneEvent::DamageDealt {
        amount,
        damage_type,
        ..
    } = &events[2]
    {
        // Run it through the DamagePipeline
        let result = DamagePipeline::process(
            *amount,
            *damage_type,
            false,
            &attacker_stats,
            &defender_stats,
        );

        // base PTA damage at level 1: 40.0
        // Amplified by 8%: 40.0 * 1.08 = 43.2
        assert_eq!(
            result.final_damage, 43.2,
            "PTA proc damage should be amplified by its own exposure debuff in the current implementation"
        );
    } else {
        panic!("Expected DamageDealt at index 2");
    }
}

#[test]
fn test_pta_multiple_decays_and_triggers() {
    let mut rune = lol_core::rune_manager::PressTheAttack::new(true);
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // 1. Attack 1 at t=0.0 -> stacks = 1
    let events = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());
    assert_eq!(rune.stacks, 1);

    // 2. Attack 2 at t=1.0 -> stacks = 2
    let events = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());
    assert_eq!(rune.stacks, 2);

    // 3. Delay 4.5s (decay). Attack 3 at t=5.5 -> stacks reset to 0, then increment to 1
    let events = rune.on_damage_dealt(
        SimTime::new(5.5),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());
    assert_eq!(rune.stacks, 1);

    // 4. Attack 4 at t=6.5 -> stacks = 2
    let events = rune.on_damage_dealt(
        SimTime::new(6.5),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());
    assert_eq!(rune.stacks, 2);

    // 5. Attack 5 at t=7.5 -> triggers PTA
    let events = rune.on_damage_dealt(
        SimTime::new(7.5),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert_eq!(events.len(), 3);
    assert!(rune.was_exposed);
    assert_eq!(rune.stacks, 0);

    // 6. Attack 6 at t=12.5 (still exposed, trigger time + 5.0s) -> should not stack or do anything
    let events = rune.on_damage_dealt(
        SimTime::new(12.5),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());
    assert_eq!(rune.stacks, 0);

    // 7. Attack 7 at t=14.5 (trigger time + 7.0s, exposure expired) -> starts stacking again
    let events = rune.on_damage_dealt(
        SimTime::new(14.5),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());
    assert!(!rune.was_exposed);
    assert_eq!(rune.stacks, 1);
}

#[test]
fn test_electrocute_mid_cooldown_level_up() {
    let mut rune = Electrocute::new();
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // Trigger Electrocute at t=0.0 (level 1)
    let _ = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );
    let _ = rune.on_damage_dealt(
        SimTime::new(0.5),
        10.0,
        true,
        AbilitySlot::W,
        &attacker_stats,
        1,
    );
    let events = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(!events.is_empty());

    // Try triggering again at t=21.0 at level 1 (cooldown is 25s, so it should not trigger)
    let _ = rune.on_damage_dealt(
        SimTime::new(21.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );
    let _ = rune.on_damage_dealt(
        SimTime::new(21.5),
        10.0,
        true,
        AbilitySlot::W,
        &attacker_stats,
        1,
    );
    let events = rune.on_damage_dealt(
        SimTime::new(22.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert!(
        events.is_empty(),
        "Should not trigger because level 1 cooldown is 25s"
    );

    // Reset hit history but keep last_proc_time (simulating that the above hits didn't trigger it but were registered)
    rune.recent_hits.clear();

    // Try triggering again at t=21.0, but now we are level 18 (cooldown is 20s, so 21s > 20s, it should trigger!)
    let _ = rune.on_damage_dealt(
        SimTime::new(21.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        18,
    );
    let _ = rune.on_damage_dealt(
        SimTime::new(21.5),
        10.0,
        true,
        AbilitySlot::W,
        &attacker_stats,
        18,
    );
    let events = rune.on_damage_dealt(
        SimTime::new(22.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        18,
    );
    assert!(
        !events.is_empty(),
        "Should trigger because level 18 cooldown is 20s"
    );
}

#[test]
fn test_electrocute_simultaneous_hits() {
    let mut rune = Electrocute::new();
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // Hit 1: Q at t=1.0
    let events = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());

    // Hit 2: W at t=1.0 (exact same simulation time)
    let events = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        true,
        AbilitySlot::W,
        &attacker_stats,
        1,
    );
    assert!(events.is_empty());

    // Hit 3: E at t=1.0 (exact same simulation time)
    let events = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        true,
        AbilitySlot::E,
        &attacker_stats,
        1,
    );
    assert_eq!(
        events.len(),
        1,
        "Electrocute should trigger on simultaneous distinct hits"
    );
}

#[test]
fn test_electrocute_negative_and_zero_cooldown_adversarial() {
    // Level 0: saturating sub maps 0 -> 0. Cooldown should be 25.0s
    {
        let mut rune = Electrocute::new();
        let base_stats = StatBlock::new();
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 0, 1.0);
        let attacker_stats = StatBlock::new();

        let _ = rune.on_damage_dealt(
            SimTime::new(0.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            0,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(0.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            0,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(1.0),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            0,
        );
        assert!(!events.is_empty(), "Electrocute triggers at level 0");

        // Try triggering again at t=25.9s (cooldown is 25s, so 25.9 - 1.0 = 24.9 < 25s -> should not trigger)
        let _ = rune.on_damage_dealt(
            SimTime::new(25.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            0,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(25.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            0,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(25.9),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            0,
        );
        assert!(events.is_empty(), "Level 0 cooldown boundary respected");
    }

    // Level 100: Cooldown is 25.0 - 5.0 / 17.0 * 99 = -4.117s.
    // In lol-core implementation, a negative cooldown means the time-since-last-proc (e.g. 0.02s)
    // is greater than cooldown (-4.117s), allowing spamming without cooldown.
    {
        let mut rune = Electrocute::new();
        let base_stats = StatBlock::new();
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 100, 1.0);
        let attacker_stats = StatBlock::new();

        // First trigger at t = 1.0
        let _ = rune.on_damage_dealt(
            SimTime::new(0.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            100,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(0.5),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            100,
        );
        let events = rune.on_damage_dealt(
            SimTime::new(1.0),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            100,
        );
        assert!(!events.is_empty(), "Electrocute triggers at level 100");

        // Second trigger at t = 1.02s (only 0.02s elapsed!)
        // Cooldown check is `time - last_proc_time < cooldown` -> `0.02 < -4.117` is false!
        // So the cooldown check is bypassed and it can trigger again immediately!
        let _ = rune.on_damage_dealt(
            SimTime::new(1.00),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            100,
        );
        let _ = rune.on_damage_dealt(
            SimTime::new(1.01),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            100,
        );
        let events2 = rune.on_damage_dealt(
            SimTime::new(1.02),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            100,
        );
        assert!(
            !events2.is_empty(),
            "Electrocute triggers repeatedly when level is high enough to make cooldown negative"
        );
    }
}

#[test]
fn test_pta_ability_no_reset_and_decay() {
    let mut rune = lol_core::rune_manager::PressTheAttack::new(true);
    let base_stats = StatBlock::new();
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // t = 0.0: AA -> stacks = 1, last_attack_time = 0.0
    let _ = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert_eq!(rune.stacks, 1);

    // t = 3.0: Q (ability) -> should NOT increment stacks, and should NOT update last_attack_time
    let _ = rune.on_damage_dealt(
        SimTime::new(3.0),
        10.0,
        true,
        AbilitySlot::Q,
        &attacker_stats,
        1,
    );
    assert_eq!(rune.stacks, 1);
    assert_eq!(rune.last_attack_time, 0.0);

    // t = 4.1: AA -> since 4.1 - 0.0 > 4.0, the stacks should decay to 0 first, then increment to 1
    let _ = rune.on_damage_dealt(
        SimTime::new(4.1),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    assert_eq!(rune.stacks, 1);
}

#[test]
fn test_pta_zero_stat_magic_fallback() {
    let mut rune = lol_core::rune_manager::PressTheAttack::new(true);
    let base_stats = StatBlock {
        attack_damage: 80.0,
        ..Default::default()
    };
    let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock {
        attack_damage: 80.0, // bonus_ad = 80 - 80 = 0.0
        ability_power: 0.0,
        ..Default::default()
    };

    // Stacks: 1, 2, 3 -> Trigger PTA
    let _ = rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    let _ = rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    let events = rune.on_damage_dealt(
        SimTime::new(2.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );

    assert_eq!(events.len(), 3);
    match &events[2] {
        lol_core::rune_manager::RuneEvent::DamageDealt { damage_type, .. } => {
            assert_eq!(
                *damage_type,
                DamageType::Magic,
                "PTA should deal Magic damage when bonus AD and AP are both 0.0"
            );
        }
        _ => panic!("Expected DamageDealt event"),
    }
}

#[test]
fn test_pta_melee_vs_ranged_duration_and_amplification() {
    // Create Melee PTA
    let mut melee_rune = lol_core::rune_manager::PressTheAttack::new(true);
    // Create Ranged PTA
    let mut ranged_rune = lol_core::rune_manager::PressTheAttack::new(false);

    let base_stats = StatBlock::new();
    let _ = melee_rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let _ = ranged_rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1, 1.0);
    let attacker_stats = StatBlock::new();

    // Trigger Melee
    let _ = melee_rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    let _ = melee_rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    let melee_events = melee_rune.on_damage_dealt(
        SimTime::new(2.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );

    // Trigger Ranged
    let _ = ranged_rune.on_damage_dealt(
        SimTime::new(0.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    let _ = ranged_rune.on_damage_dealt(
        SimTime::new(1.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );
    let ranged_events = ranged_rune.on_damage_dealt(
        SimTime::new(2.0),
        10.0,
        false,
        AbilitySlot::AutoAttack,
        &attacker_stats,
        1,
    );

    assert_eq!(melee_events.len(), 3);
    assert_eq!(ranged_events.len(), 3);

    // Compare Melee vs Ranged Debuff
    let (m_dur, m_amp) = match &melee_events[1] {
        lol_core::rune_manager::RuneEvent::ApplyDebuff {
            duration,
            damage_reduction_percent,
            ..
        } => (*duration, *damage_reduction_percent),
        _ => panic!("Expected ApplyDebuff"),
    };
    let (r_dur, r_amp) = match &ranged_events[1] {
        lol_core::rune_manager::RuneEvent::ApplyDebuff {
            duration,
            damage_reduction_percent,
            ..
        } => (*duration, *damage_reduction_percent),
        _ => panic!("Expected ApplyDebuff"),
    };

    assert_eq!(
        m_dur, r_dur,
        "PTA exposure duration should be identical for melee and ranged"
    );
    assert_eq!(
        m_amp, r_amp,
        "PTA exposure amplification should be identical for melee and ranged"
    );
}
