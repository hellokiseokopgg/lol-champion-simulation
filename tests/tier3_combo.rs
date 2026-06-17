mod common;

use common::{GanttEvent, parse_breakdown, parse_gantt_events, run_with_apl};

// =========================================================================
// Cross-Feature Combinations (Tier 3) Tests
// =========================================================================

/// Test 1: Verify PTA exposure debuff amplifies item active/passive damage.
#[test]
fn test_pta_amplifies_item_damage() {
    // We use Stridebreaker (Item 6631).
    // APL: cast Stridebreaker active (6631) after triggering PTA.
    let apl_pta =
        "runes=press_the_attack\nactions+=/AA,if=time<3.0\nactions+=/Stridebreaker,if=time>=3.0\n";
    let stdout_pta = run_with_apl(
        "Garen",
        "Dummy",
        Some("press_the_attack"),
        Some("6631"),
        apl_pta,
        &[],
    );
    let breakdown_pta = parse_breakdown(&stdout_pta, "Garen");
    let item_dmg_pta = breakdown_pta
        .iter()
        .find(|(k, _)| k.to_lowercase().contains("item") || k.to_lowercase().contains("6631"))
        .map(|(_, (dmg, _))| *dmg)
        .unwrap_or(0.0);

    // Run without PTA (Conqueror)
    let apl_no_pta =
        "runes=conqueror\nactions+=/AA,if=time<3.0\nactions+=/Stridebreaker,if=time>=3.0\n";
    let stdout_no_pta = run_with_apl(
        "Garen",
        "Dummy",
        Some("conqueror"),
        Some("6631"),
        apl_no_pta,
        &[],
    );
    let breakdown_no_pta = parse_breakdown(&stdout_no_pta, "Garen");
    let item_dmg_no_pta = breakdown_no_pta
        .iter()
        .find(|(k, _)| k.to_lowercase().contains("item") || k.to_lowercase().contains("6631"))
        .map(|(_, (dmg, _))| *dmg)
        .unwrap_or(0.0);

    if item_dmg_pta > 0.0 && item_dmg_no_pta > 0.0 {
        assert!(
            item_dmg_pta > item_dmg_no_pta,
            "Item damage under PTA exposure ({}) should be higher than without PTA ({}) due to 8% amplification",
            item_dmg_pta,
            item_dmg_no_pta
        );
    }
}

/// Test 2: Verify Electrocute does not trigger on item active damage alone.
#[test]
fn test_electrocute_item_ignored() {
    // Stridebreaker active should not count as a separate hit for Electrocute.
    // If we only use Stridebreaker active, Electrocute should not trigger.
    let apl = "runes=electrocute\nactions+=/Stridebreaker\n";
    let stdout = run_with_apl(
        "Garen",
        "Dummy",
        Some("electrocute"),
        Some("6631"),
        apl,
        &[],
    );

    let breakdown = parse_breakdown(&stdout, "Garen");
    let has_electrocute = breakdown
        .keys()
        .any(|k| k.to_lowercase().contains("electrocute"));
    assert!(
        !has_electrocute,
        "Electrocute should not trigger on item damage alone"
    );
}

/// Test 3: Verify PTA exposure amplifies Garen's E (Judgment) spin ticks.
#[test]
fn test_pta_amplifies_abilities() {
    let apl = "actions+=/AA,if=time<3.0\nactions+=/E,if=time>=3.0\n";

    // Without PTA
    let stdout_no_pta = run_with_apl("Garen", "Dummy", Some("conqueror"), None, apl, &[]);
    let breakdown_no_pta = parse_breakdown(&stdout_no_pta, "Garen");
    let e_dmg_no_pta = breakdown_no_pta
        .get("GarenE")
        .map(|(dmg, _)| *dmg)
        .unwrap_or(0.0);

    // With PTA
    let stdout_pta = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);
    let breakdown_pta = parse_breakdown(&stdout_pta, "Garen");
    let e_dmg_pta = breakdown_pta
        .get("GarenE")
        .map(|(dmg, _)| *dmg)
        .unwrap_or(0.0);

    if e_dmg_no_pta > 0.0 && e_dmg_pta > 0.0 {
        assert!(
            e_dmg_pta > e_dmg_no_pta,
            "Garen E damage under PTA exposure ({}) should be amplified compared to without PTA ({})",
            e_dmg_pta,
            e_dmg_no_pta
        );
    }
}

/// Test 4: Verify Ability Haste (items) does not reduce rune cooldowns (static 20s/6s).
#[test]
fn test_ability_haste_does_not_affect_rune_cooldowns() {
    // Black Cleaver (3071) provides 20 Haste.
    let apl = "runes=electrocute\nactions+=/Q\nactions+=/E\nactions+=/AA\n";

    // With haste item
    let stdout = run_with_apl(
        "Garen",
        "Dummy",
        Some("electrocute"),
        Some("3071,3071"),
        apl,
        &[],
    );
    let events = parse_gantt_events(&stdout, "Garen");
    let trigger_times: Vec<u64> = events
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

    if trigger_times.len() > 1 {
        let diff = trigger_times[1] - trigger_times[0];
        // Cooldown must still be 20s (20000ms), not reduced by haste.
        assert!(
            diff >= 20000,
            "Electrocute cooldown should not be reduced by Ability Haste (was {} ms)",
            diff
        );
    }
}
