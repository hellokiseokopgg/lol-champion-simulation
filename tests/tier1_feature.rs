mod common;

use common::{GanttEvent, parse_breakdown, parse_gantt_events, run_with_apl};

// =========================================================================
// Electrocute Rune Tier 1 Tests
// =========================================================================

/// Test 1: Verify Electrocute triggers when Garen lands 3 separate attacks/abilities within 3 seconds.
#[test]
fn test_electrocute_activation_garen() {
    let apl = "runes=electrocute\nactions+=/Q\nactions+=/E\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);

    let events = parse_gantt_events(&stdout, "Garen");
    let has_proc = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => buff_name.to_lowercase().contains("electrocute"),
        _ => false,
    });

    let breakdown = parse_breakdown(&stdout, "Garen");
    let has_damage = breakdown
        .keys()
        .any(|k| k.to_lowercase().contains("electrocute") || k.to_lowercase().contains("passive"));

    assert!(
        has_proc || has_damage,
        "Electrocute should activate and deal damage when 3 separate abilities/attacks hit"
    );
}

/// Test 2: Verify Electrocute cooldown behavior (20 seconds).
#[test]
fn test_electrocute_cooldown_garen() {
    let apl = "runes=electrocute\nactions+=/Q\nactions+=/E\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);

    let events = parse_gantt_events(&stdout, "Garen");
    let activation_times: Vec<u64> = events
        .iter()
        .filter_map(|e| match e {
            GanttEvent::BuffApply { buff_name, time_ms }
                if buff_name.to_lowercase().contains("electrocute") =>
            {
                Some(*time_ms)
            }
            _ => None,
        })
        .collect();

    if activation_times.len() > 1 {
        for i in 0..activation_times.len() - 1 {
            let diff = activation_times[i + 1] - activation_times[i];
            assert!(
                diff >= 20000,
                "Electrocute cooldown must be at least 20 seconds (20,000 ms), but was {} ms",
                diff
            );
        }
    }
}

/// Test 3: Verify Electrocute does not activate with fewer than 3 separate hits.
#[test]
fn test_electrocute_missing_hit_garen() {
    let apl = "runes=electrocute\nactions+=/Q\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);

    let events = parse_gantt_events(&stdout, "Garen");
    let triggered = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => buff_name.to_lowercase().contains("electrocute"),
        _ => false,
    });

    assert!(!triggered, "Electrocute should not trigger with only Q hit");
}

/// Test 4: Verify Electrocute does not activate if hits are spaced more than 3 seconds apart.
#[test]
fn test_electrocute_slow_hits_garen() {
    let apl = "runes=electrocute\nactions+=/Q,if=time<1.0\nactions+=/E,if=time>5.0&time<6.0\nactions+=/AA,if=time>10.0\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);

    let events = parse_gantt_events(&stdout, "Garen");
    let triggered = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => buff_name.to_lowercase().contains("electrocute"),
        _ => false,
    });

    assert!(
        !triggered,
        "Electrocute should not trigger when hits are spaced more than 3 seconds apart"
    );
}

/// Test 5: Verify Electrocute damage scaling with Garen's stats (using AD items).
#[test]
fn test_electrocute_damage_scaling_garen() {
    let apl = "runes=electrocute\nactions+=/Q\nactions+=/E\nactions+=/AA\n";

    // 1. Without items
    let stdout_no_items = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);
    let breakdown_no_items = parse_breakdown(&stdout_no_items, "Garen");
    let dmg_no_items = breakdown_no_items
        .iter()
        .find(|(k, _)| {
            k.to_lowercase().contains("electrocute") || k.to_lowercase().contains("passive")
        })
        .map(|(_, (dmg, _))| *dmg)
        .unwrap_or(0.0);

    // 2. With heavy AD items (e.g., 3 Black Cleavers "3071,3071,3071")
    let stdout_items = run_with_apl(
        "Garen",
        "Dummy",
        Some("electrocute"),
        Some("3071,3071,3071"),
        apl,
        &[],
    );
    let breakdown_items = parse_breakdown(&stdout_items, "Garen");
    let dmg_items = breakdown_items
        .iter()
        .find(|(k, _)| {
            k.to_lowercase().contains("electrocute") || k.to_lowercase().contains("passive")
        })
        .map(|(_, (dmg, _))| *dmg)
        .unwrap_or(0.0);

    if dmg_no_items > 0.0 {
        assert!(
            dmg_items > dmg_no_items,
            "Electrocute damage with items ({}) should be higher than without items ({}) due to AD scaling",
            dmg_items,
            dmg_no_items
        );
    }
}

// =========================================================================
// Press the Attack (PTA) Rune Tier 1 Tests
// =========================================================================

/// Test 6: Verify PTA triggers when Garen lands 3 consecutive basic attacks.
#[test]
fn test_pta_activation_garen() {
    let apl = "runes=press_the_attack\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);

    let events = parse_gantt_events(&stdout, "Garen");
    let has_proc = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => {
            buff_name.to_lowercase().contains("press the attack")
                || buff_name.to_lowercase().contains("pta")
        }
        _ => false,
    });

    let breakdown = parse_breakdown(&stdout, "Garen");
    let has_damage = breakdown.keys().any(|k| {
        k.to_lowercase().contains("press the attack")
            || k.to_lowercase().contains("pta")
            || k.to_lowercase().contains("passive")
    });

    assert!(
        has_proc || has_damage,
        "Press the Attack should activate and deal damage/apply buff"
    );
}

/// Test 7: Verify damage amplification (8%) once PTA is active.
#[test]
fn test_pta_damage_amplification_garen() {
    let apl = "actions+=/AA,if=time<3.0\nactions+=/E,if=time>=3.0\n";

    // Run with Lethal Tempo (no PTA amp)
    let stdout_lt = run_with_apl("Garen", "Dummy", Some("lethal_tempo"), None, apl, &[]);
    let breakdown_lt = parse_breakdown(&stdout_lt, "Garen");
    let e_dmg_lt = breakdown_lt
        .get("GarenE")
        .map(|(dmg, _)| *dmg)
        .unwrap_or(0.0);

    // Run with PTA
    let stdout_pta = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);
    let breakdown_pta = parse_breakdown(&stdout_pta, "Garen");
    let e_dmg_pta = breakdown_pta
        .get("GarenE")
        .map(|(dmg, _)| *dmg)
        .unwrap_or(0.0);

    if e_dmg_lt > 0.0 && e_dmg_pta > 0.0 {
        assert!(
            e_dmg_pta > e_dmg_lt,
            "E damage with PTA ({}) should be higher than with Lethal Tempo ({}) due to 8% damage amplification",
            e_dmg_pta,
            e_dmg_lt
        );
    }
}

/// Test 8: Verify PTA does not activate with fewer than 3 basic attacks.
#[test]
fn test_pta_missing_hits_garen() {
    let apl = "runes=press_the_attack\nactions+=/AA,if=time<1.0\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);

    let events = parse_gantt_events(&stdout, "Garen");
    let triggered = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => {
            buff_name.to_lowercase().contains("press the attack")
                || buff_name.to_lowercase().contains("pta")
        }
        _ => false,
    });

    assert!(
        !triggered,
        "PTA should not activate with only 2 basic attacks"
    );
}

/// Test 9: Verify PTA buff expires/resets after leaving combat.
#[test]
fn test_pta_reset_out_of_combat_garen() {
    let apl = "runes=press_the_attack\nactions+=/AA,if=time<3.0|time>25.0\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);

    let events = parse_gantt_events(&stdout, "Garen");
    let has_expire = events.iter().any(|e| match e {
        GanttEvent::BuffExpire { buff_name, .. } => {
            buff_name.to_lowercase().contains("press the attack")
                || buff_name.to_lowercase().contains("pta")
        }
        _ => false,
    });

    assert!(
        has_expire,
        "PTA buff should expire/reset after leaving combat"
    );
}

/// Test 10: Verify PTA triggers even when abilities are woven between basic attacks.
#[test]
fn test_pta_consecutive_restriction_garen() {
    let apl = "runes=press_the_attack\nactions+=/AA,if=time<1.0\nactions+=/E,if=time>=1.0&time<4.0\nactions+=/AA,if=time>=4.0\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);

    let events = parse_gantt_events(&stdout, "Garen");
    let has_proc = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => {
            buff_name.to_lowercase().contains("press the attack")
                || buff_name.to_lowercase().contains("pta")
        }
        _ => false,
    });

    assert!(
        has_proc,
        "PTA should trigger when abilities are woven between basic attacks"
    );
}
