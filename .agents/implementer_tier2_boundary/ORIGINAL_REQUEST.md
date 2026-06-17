## 2026-06-17T07:12:05Z
Create the integration test file `tests/tier2_boundary.rs` to implement Tier 2 (Boundary & Corner Cases) E2E tests.

The file should contain:
```rust
mod common;

use common::{run_with_apl, parse_breakdown, parse_gantt_events, GanttEvent};

// =========================================================================
// Electrocute Boundary & Edge Tests
// =========================================================================

/// Test 1: Verify Electrocute triggers when hits are within 3s window.
#[test]
fn test_electrocute_boundary_window_trigger() {
    // Hits at 0.0s, 1.5s, 2.9s -> all within 3.0 seconds. Should trigger.
    let apl = "runes=electrocute\nactions+=/Q,if=time<0.2\nactions+=/E,if=time>=1.5&time<1.7\nactions+=/AA,if=time>=2.9&time<3.1\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);
    
    let events = parse_gantt_events(&stdout, "Garen");
    let triggered = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => buff_name.to_lowercase().contains("electrocute"),
        _ => false,
    });
    
    let breakdown = parse_breakdown(&stdout, "Garen");
    let has_damage = breakdown.keys().any(|k| k.to_lowercase().contains("electrocute") || k.to_lowercase().contains("passive"));
    
    assert!(triggered || has_damage, "Electrocute should trigger when hits are within 3 seconds");
}

/// Test 2: Verify Electrocute does not trigger when hits are spaced more than 3s apart.
#[test]
fn test_electrocute_boundary_window_no_trigger() {
    // Hits at 0.0s, 3.5s, 7.0s -> spacing > 3s. Should not trigger.
    let apl = "runes=electrocute\nactions+=/Q,if=time<0.2\nactions+=/E,if=time>=3.5&time<3.7\nactions+=/AA,if=time>=7.0&time<7.2\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);
    
    let events = parse_gantt_events(&stdout, "Garen");
    let triggered = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => buff_name.to_lowercase().contains("electrocute"),
        _ => false,
    });
    
    assert!(!triggered, "Electrocute should not trigger when hits are spaced more than 3 seconds apart");
}

/// Test 3: Verify Electrocute triggers only once per cooldown period even if rapid hits continue.
#[test]
fn test_electrocute_boundary_rapid_hits() {
    // Spam basic attacks and abilities. Should trigger once initially, then go on cooldown.
    let apl = "runes=electrocute\nactions+=/Q\nactions+=/E\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);
    
    let events = parse_gantt_events(&stdout, "Garen");
    let trigger_times: Vec<u64> = events.iter().filter_map(|e| match e {
        GanttEvent::BuffApply { buff_name, time_ms } if buff_name.to_lowercase().contains("electrocute") => Some(*time_ms),
        _ => None,
    }).collect();
    
    // Cooldown is 20s. In 60s max duration, it should trigger at most 3 times.
    assert!(trigger_times.len() <= 3, "Electrocute should not trigger more than 3 times in 60s (triggered {} times)", trigger_times.len());
}

/// Test 4: Verify Electrocute cooldown behavior (cannot re-trigger within 20s).
#[test]
fn test_electrocute_boundary_cooldown_limit() {
    // Trigger at 0s, try to trigger again at 15s. Should not trigger.
    let apl = "runes=electrocute\nactions+=/Q,if=time<0.2|time>=15.0&time<15.2\nactions+=/E,if=time>=1.0&time<1.2|time>=16.0&time<16.2\nactions+=/AA,if=time>=2.0&time<2.2|time>=17.0&time<17.2\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);
    
    let events = parse_gantt_events(&stdout, "Garen");
    let trigger_times: Vec<u64> = events.iter().filter_map(|e| match e {
        GanttEvent::BuffApply { buff_name, time_ms } if buff_name.to_lowercase().contains("electrocute") => Some(*time_ms),
        _ => None,
    }).collect();
    
    if trigger_times.len() > 1 {
        let diff = trigger_times[1] - trigger_times[0];
        assert!(diff >= 20000, "Second trigger should be at least 20s after the first, but was {} ms", diff);
    }
}

/// Test 5: Verify Electrocute does not trigger on dummy target when using other runes.
#[test]
fn test_electrocute_boundary_other_runes() {
    let apl = "runes=conqueror\nactions+=/Q\nactions+=/E\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("conqueror"), None, apl, &[]);
    
    let breakdown = parse_breakdown(&stdout, "Garen");
    let has_electrocute = breakdown.keys().any(|k| k.to_lowercase().contains("electrocute"));
    assert!(!has_electrocute, "Electrocute should not trigger when not equipped");
}

// =========================================================================
// Press the Attack Boundary & Edge Tests
// =========================================================================

/// Test 6: Verify PTA stacks decay if attacks are spaced too far apart.
#[test]
fn test_pta_boundary_decay_no_trigger() {
    // Stacks decay after 4 seconds. Space attacks by 5s. Should not trigger.
    let apl = "runes=press_the_attack\nactions+=/AA,if=time<0.2|time>=5.0&time<5.2|time>=10.0&time<10.2\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);
    
    let events = parse_gantt_events(&stdout, "Garen");
    let triggered = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => buff_name.to_lowercase().contains("press the attack") || buff_name.to_lowercase().contains("pta"),
        _ => false,
    });
    
    assert!(!triggered, "PTA should not trigger when attacks are spaced 5s apart (decay window is 4s)");
}

/// Test 7: Verify PTA triggers when attacks are within the decay window (3.5s spacing).
#[test]
fn test_pta_boundary_decay_trigger() {
    // Hits at 0.0s, 3.5s, 7.0s. All within the 4s decay window. Should trigger.
    let apl = "runes=press_the_attack\nactions+=/AA,if=time<0.2|time>=3.5&time<3.7|time>=7.0&time<7.2\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);
    
    let events = parse_gantt_events(&stdout, "Garen");
    let triggered = events.iter().any(|e| match e {
        GanttEvent::BuffApply { buff_name, .. } => buff_name.to_lowercase().contains("press the attack") || buff_name.to_lowercase().contains("pta"),
        _ => false,
    });
    
    let breakdown = parse_breakdown(&stdout, "Garen");
    let has_damage = breakdown.keys().any(|k| k.to_lowercase().contains("press the attack") || k.to_lowercase().contains("pta") || k.to_lowercase().contains("passive"));
    
    assert!(triggered || has_damage, "PTA should trigger when attacks are within the 4s decay window");
}

/// Test 8: Verify PTA exposure does not refresh or stack on the same target.
#[test]
fn test_pta_boundary_no_re_application() {
    let apl = "runes=press_the_attack\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);
    
    let events = parse_gantt_events(&stdout, "Garen");
    let app_times: Vec<u64> = events.iter().filter_map(|e| match e {
        GanttEvent::BuffApply { buff_name, time_ms } if buff_name.to_lowercase().contains("press the attack") || buff_name.to_lowercase().contains("pta") => Some(*time_ms),
        _ => None,
    }).collect();
    
    if app_times.len() > 1 {
        let diff = app_times[1] - app_times[0];
        assert!(diff >= 6000, "PTA exposure debuff should not be re-applied or refreshed before it expires (6s)");
    }
}

/// Test 9: Verify PTA cooldown prevents immediate re-activation (6s).
#[test]
fn test_pta_boundary_cooldown() {
    let apl = "runes=press_the_attack\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("press_the_attack"), None, apl, &[]);
    
    let events = parse_gantt_events(&stdout, "Garen");
    let trigger_times: Vec<u64> = events.iter().filter_map(|e| match e {
        GanttEvent::BuffApply { buff_name, time_ms } if buff_name.to_lowercase().contains("press the attack") || buff_name.to_lowercase().contains("pta") => Some(*time_ms),
        _ => None,
    }).collect();
    
    if trigger_times.len() > 1 {
        let diff = trigger_times[1] - trigger_times[0];
        assert!(diff >= 6000, "PTA activation spacing must respect the 6s cooldown");
    }
}

/// Test 10: Verify PTA is not active when using Conqueror.
#[test]
fn test_pta_boundary_other_runes() {
    let apl = "runes=conqueror\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("conqueror"), None, apl, &[]);
    
    let breakdown = parse_breakdown(&stdout, "Garen");
    let has_pta = breakdown.keys().any(|k| k.to_lowercase().contains("press the attack") || k.to_lowercase().contains("pta"));
    assert!(!has_pta, "PTA should not trigger when Conqueror is equipped");
}
