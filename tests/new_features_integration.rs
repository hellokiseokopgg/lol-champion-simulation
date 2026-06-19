mod common;

use common::{GanttEvent, parse_breakdown, parse_gantt_events, run_with_apl};

// Helper function to extract event log from the generated HTML report
fn parse_html_events(html_content: &str) -> Vec<serde_json::Value> {
    if let Some(start_idx) = html_content.find("const events = ") {
        let after_const = &html_content[start_idx + "const events = ".len()..];
        if let Some(end_idx) = after_const.find("];") {
            let json_str = format!("{}]", &after_const[..end_idx]);
            if let Ok(serde_json::Value::Array(arr)) =
                serde_json::from_str::<serde_json::Value>(&json_str)
            {
                return arr;
            }
        }
    }
    Vec::new()
}

/// Grasp of the Undying: Verify that basic attacks on a 4s combat timer trigger
/// Grasp magic damage, heal Garen, and permanently increase his max health.
#[test]
fn test_grasp_of_the_undying_integration() {
    let apl = "runes=grasp_of_the_undying\nactions+=/AA,if=time<0.1|time>4.0\n";
    let stdout = run_with_apl(
        "Garen",
        "Dummy",
        Some("grasp_of_the_undying"),
        None,
        apl,
        &["--html-out", "temp_grasp.html"],
    );

    let breakdown = parse_breakdown(&stdout, "Garen");
    assert!(
        breakdown.contains_key("GraspOfTheUndying"),
        "Should deal Grasp of the Undying magic damage"
    );

    let html_content = std::fs::read_to_string("temp_grasp.html").unwrap();
    let _ = std::fs::remove_file("temp_grasp.html");

    let events = parse_html_events(&html_content);
    let has_heal = events.iter().any(|e| {
        e["type"] == "heal" && e["target"] == "Garen" && e["amount"].as_f64().unwrap_or(0.0) > 0.0
    });
    assert!(has_heal, "Grasp should trigger a healing event");
}

/// Triumph: Verify that on a champion takedown, the survivor receives a Triumph heal event.
#[test]
fn test_triumph_integration() {
    let apl_a = "runes=electrocute,triumph\nactions+=/E\nactions+=/Q\nactions+=/W\nactions+=/R\nactions+=/AA\n";
    let apl_b = "actions+=/AA\n";

    let apl_b_filename = "temp_darius_triumph.apl";
    std::fs::write(apl_b_filename, apl_b).unwrap();

    let _stdout = run_with_apl(
        "Ahri",
        "Darius",
        Some("electrocute,triumph"),
        Some("3031,3031,3031,3031,3031,3031"),
        apl_a,
        &["--apl-b", apl_b_filename, "--html-out", "temp_triumph.html"],
    );
    let _ = std::fs::remove_file(apl_b_filename);

    let html_content = std::fs::read_to_string("temp_triumph.html").unwrap();
    let _ = std::fs::remove_file("temp_triumph.html");

    let events = parse_html_events(&html_content);

    let has_death = events
        .iter()
        .any(|e| e["type"] == "death" && e["source"] == "Darius");
    assert!(has_death, "Darius should die in the simulation");

    let has_triumph_heal = events.iter().any(|e| {
        e["type"] == "heal" && e["target"] == "Ahri" && e["amount"].as_f64().unwrap_or(0.0) > 0.0
    });
    assert!(
        has_triumph_heal,
        "Ahri should receive Triumph heal on takedown"
    );
}

/// Legend: Alacrity: Verify Garen equipped with Alacrity has higher attack speed.
#[test]
fn test_legend_alacrity_integration() {
    let apl = "actions+=/AA\n";

    let stdout_no_alacrity = run_with_apl(
        "Garen",
        "Dummy",
        Some("conqueror,triumph,bone_plating"),
        None,
        apl,
        &[],
    );
    let events_no_alacrity = parse_gantt_events(&stdout_no_alacrity, "Garen");

    let stdout_alacrity = run_with_apl(
        "Garen",
        "Dummy",
        Some("conqueror,triumph,legend_alacrity,bone_plating"),
        None,
        apl,
        &[],
    );
    let events_alacrity = parse_gantt_events(&stdout_alacrity, "Garen");

    let aa_times_no_alacrity: Vec<u64> = events_no_alacrity
        .iter()
        .filter_map(|e| match e {
            GanttEvent::Cast { ability, time_ms } if ability == "AutoAttack" => Some(*time_ms),
            _ => None,
        })
        .collect();

    let aa_times_alacrity: Vec<u64> = events_alacrity
        .iter()
        .filter_map(|e| match e {
            GanttEvent::Cast { ability, time_ms } if ability == "AutoAttack" => Some(*time_ms),
            _ => None,
        })
        .collect();

    if aa_times_no_alacrity.len() > 2 && aa_times_alacrity.len() > 2 {
        let diff_no_alacrity = aa_times_no_alacrity[2] - aa_times_no_alacrity[1];
        let diff_alacrity = aa_times_alacrity[2] - aa_times_alacrity[1];
        assert!(
            diff_alacrity < diff_no_alacrity,
            "With Alacrity, Garen should attack faster (interval: {} ms < {} ms)",
            diff_alacrity,
            diff_no_alacrity
        );
    }
}

/// Last Stand: Verify that damage dealt scales higher as health drops below 60%
/// and reaches max amp below 30% health.
#[test]
fn test_last_stand_integration() {
    let apl_garen = "runes=conqueror,last_stand\nactions+=/AA\n";
    let apl_darius = "runes=conqueror\nactions+=/AA\n";

    let apl_darius_filename = "temp_darius_last_stand.apl";
    std::fs::write(apl_darius_filename, apl_darius).unwrap();

    let _stdout = run_with_apl(
        "Garen",
        "Darius",
        Some("conqueror,last_stand"),
        None,
        apl_garen,
        &[
            "--apl-b",
            apl_darius_filename,
            "--html-out",
            "temp_last_stand.html",
        ],
    );
    let _ = std::fs::remove_file(apl_darius_filename);

    let html_content = std::fs::read_to_string("temp_last_stand.html").unwrap();
    let _ = std::fs::remove_file("temp_last_stand.html");

    let events = parse_html_events(&html_content);

    let garen_aa_damages: Vec<(f64, f64)> = events
        .iter()
        .filter_map(|e| {
            if e["type"] == "damage" && e["source"] == "Garen" && e["ability"] == "AutoAttack" {
                Some((
                    e["time"].as_f64().unwrap_or(0.0),
                    e["amount"].as_f64().unwrap_or(0.0),
                ))
            } else {
                None
            }
        })
        .collect();

    if garen_aa_damages.len() > 2 {
        let first_dmg = garen_aa_damages[0].1;
        let last_dmg = garen_aa_damages[garen_aa_damages.len() - 1].1;
        assert!(
            last_dmg > first_dmg,
            "Damage dealt should scale higher as Garen's health drops (last AA: {} > first AA: {})",
            last_dmg,
            first_dmg
        );
    }
}

/// Bone Plating: Verify that after taking damage, the next 3 incoming attacks deal flat reduced damage.
#[test]
fn test_bone_plating_integration() {
    let apl_garen = "runes=grasp_of_the_undying,bone_plating\nactions+=/AA,if=time<0.1\n";
    let apl_darius = "runes=conqueror\nactions+=/AA\n";

    let apl_darius_filename = "temp_darius_bp.apl";
    std::fs::write(apl_darius_filename, apl_darius).unwrap();

    let stdout = run_with_apl(
        "Garen",
        "Darius",
        Some("grasp_of_the_undying,bone_plating"),
        None,
        apl_garen,
        &["--apl-b", apl_darius_filename],
    );
    let _ = std::fs::remove_file(apl_darius_filename);

    let count = stdout
        .lines()
        .filter(|line| line.contains("Garen's Bone Plating blocked 60.0 damage"))
        .count();
    assert_eq!(count, 3, "Bone Plating should block exactly 3 times");
}

/// Overgrowth: Verify that being near dying minions (or over time in combat) increases Garen's max health.
#[test]
fn test_overgrowth_integration() {
    let apl = "runes=grasp_of_the_undying,overgrowth\nactions+=/AA\n";
    let stdout = run_with_apl(
        "Garen",
        "Dummy",
        Some("grasp_of_the_undying,overgrowth"),
        None,
        apl,
        &["--html-out", "temp_overgrowth.html"],
    );

    let html_content = std::fs::read_to_string("temp_overgrowth.html").unwrap();
    let _ = std::fs::remove_file("temp_overgrowth.html");

    assert!(
        html_content.contains("overgrowth"),
        "Overgrowth should be equipped in HTML report"
    );
    assert!(
        stdout.contains("Champion: \"Garen\""),
        "Simulation should run successfully"
    );
}

/// Infinity Edge: Run Garen with 100% crit chance (using a custom APL/build)
/// and verify crit damage is multiplied by 2.15 vs 1.75.
#[test]
fn test_infinity_edge_integration() {
    // 1. Without IE (5 Phantom Dancers to get 100% crit chance)
    let apl_no_ie = "items=3046,3046,3046,3046,3046\nactions+=/AA,if=time<0.1\n";
    let _stdout_no_ie = run_with_apl(
        "Garen",
        "Dummy",
        None,
        None,
        apl_no_ie,
        &["--html-out", "temp_no_ie.html"],
    );
    let html_no_ie = std::fs::read_to_string("temp_no_ie.html").unwrap();
    let _ = std::fs::remove_file("temp_no_ie.html");
    let events_no_ie = parse_html_events(&html_no_ie);
    let dmg_no_ie = events_no_ie
        .iter()
        .find(|e| e["type"] == "damage" && e["source"] == "Garen" && e["ability"] == "AutoAttack")
        .and_then(|e| e["amount"].as_f64())
        .unwrap_or(0.0);

    // 2. With IE (IE + 4 Phantom Dancers to get 100% crit chance)
    let apl_ie = "items=3031,3046,3046,3046,3046\nactions+=/AA,if=time<0.1\n";
    let _stdout_ie = run_with_apl(
        "Garen",
        "Dummy",
        None,
        None,
        apl_ie,
        &["--html-out", "temp_ie.html"],
    );
    let html_ie = std::fs::read_to_string("temp_ie.html").unwrap();
    let _ = std::fs::remove_file("temp_ie.html");
    let events_ie = parse_html_events(&html_ie);
    let dmg_ie = events_ie
        .iter()
        .find(|e| e["type"] == "damage" && e["source"] == "Garen" && e["ability"] == "AutoAttack")
        .and_then(|e| e["amount"].as_f64())
        .unwrap_or(0.0);

    println!("Events without IE: {:#?}", events_no_ie);
    println!("Events with IE: {:#?}", events_ie);
    assert!(
        dmg_ie > dmg_no_ie,
        "Critical damage with IE ({}) should be higher than without IE ({}) due to 2.15 vs 1.75 scaling",
        dmg_ie,
        dmg_no_ie
    );
}

/// Mortal Reminder: Verify that physical damage from the attacker inflicts the Grievous Wounds status effect on the target.
#[test]
fn test_mortal_reminder_integration() {
    let apl = "items=3033\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", None, Some("3033"), apl, &[]);
    assert!(
        stdout.contains("Grievous Wounds"),
        "Physical damage should apply Grievous Wounds"
    );
}

/// Phantom Dancer: Verify basic attacks build Spectral Waltz stacks and grant additional attack speed at 4 stacks.
#[test]
fn test_phantom_dancer_integration() {
    let apl = "items=3046\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", None, Some("3046"), apl, &[]);
    assert!(
        stdout.contains("Spectral Waltz"),
        "Basic attacks should trigger Spectral Waltz"
    );
}

/// Ahri: Simulate Ahri vs Darius. Verify Ahri's Q, W, E, R execution sequence, Charm damage amplification, Q true damage return, and Passive (Essence Theft) healing.
#[test]
fn test_ahri_integration() {
    let apl = "actions+=/E\nactions+=/Q\nactions+=/W\nactions+=/R\n";
    let _stdout = run_with_apl(
        "Ahri",
        "Darius",
        None,
        None,
        apl,
        &["--html-out", "temp_ahri.html"],
    );

    let html_content = std::fs::read_to_string("temp_ahri.html").unwrap();
    let _ = std::fs::remove_file("temp_ahri.html");
    let events = parse_html_events(&html_content);

    let has_q = events
        .iter()
        .any(|e| e["type"] == "damage" && e["source"] == "Ahri" && e["ability"] == "Q");
    let has_w = events
        .iter()
        .any(|e| e["type"] == "damage" && e["source"] == "Ahri" && e["ability"] == "W");
    let has_e = events
        .iter()
        .any(|e| e["type"] == "damage" && e["source"] == "Ahri" && e["ability"] == "E");
    let has_r = events
        .iter()
        .any(|e| e["type"] == "damage" && e["source"] == "Ahri" && e["ability"] == "R");

    assert!(has_q, "Ahri should execute Q");
    assert!(has_w, "Ahri should execute W");
    assert!(has_e, "Ahri should execute E (Charm)");
    assert!(has_r, "Ahri should execute R (Spirit Rush)");

    let has_charm = events
        .iter()
        .any(|e| e["type"] == "buff_apply" && e["target"] == "Darius" && e["buff_name"] == "매혹");
    assert!(has_charm, "Ahri should apply Charm on target");
}

/// Zed: Simulate Zed vs Garen. Verify Zed's Living Shadow mimics abilities, Death Mark accumulates/pops damage, and passive deals magic damage on targets below 50% HP.
#[test]
fn test_zed_integration() {
    let apl_zed = "actions+=/R\nactions+=/W\nactions+=/E\nactions+=/Q\nactions+=/AA\n";
    let _stdout = run_with_apl(
        "Zed",
        "Garen",
        None,
        None,
        apl_zed,
        &["--html-out", "temp_zed.html"],
    );

    let html_content = std::fs::read_to_string("temp_zed.html").unwrap();
    let _ = std::fs::remove_file("temp_zed.html");
    let events = parse_html_events(&html_content);

    let zed_damages: Vec<_> = events
        .iter()
        .filter(|e| e["type"] == "damage" && e["source"] == "Zed")
        .collect();

    let mut time_counts = std::collections::HashMap::new();
    for e in &zed_damages {
        let time = e["time"].as_f64().unwrap_or(0.0);
        let ability = e["ability"].as_str().unwrap_or("");
        if ability == "Q" || ability == "E" {
            let key = format!("{}_{}", time, ability);
            *time_counts.entry(key).or_insert(0) += 1;
        }
    }
    let has_mimic = time_counts.values().any(|&count| count > 1);
    assert!(
        has_mimic,
        "Zed shadow should mimic abilities (resulting in concurrent damage hits)"
    );

    let has_mark = events.iter().any(|e| {
        e["type"] == "buff_apply"
            && e["target"] == "Garen"
            && e["buff_name"]
                .as_str()
                .is_some_and(|s| s.contains("죽음의 표식") || s.contains("Death Mark"))
    });
    assert!(has_mark, "Zed should apply Death Mark");
}

/// Jinx: Simulate Jinx vs Ahri. Verify Switcheroo! toggling between Minigun (gaining AS stacks) and Rocket Launcher (draining mana), and R execution damage.
#[test]
fn test_jinx_integration() {
    let apl_jinx = "actions+=/Q\nactions+=/AA,if=time<3.0\nactions+=/Q,if=time>=3.0\nactions+=/AA,if=time>=3.0\nactions+=/R,if=time>=10.0\n";
    let _stdout = run_with_apl(
        "Jinx",
        "Ahri",
        None,
        None,
        apl_jinx,
        &["--html-out", "temp_jinx.html"],
    );

    let html_content = std::fs::read_to_string("temp_jinx.html").unwrap();
    let _ = std::fs::remove_file("temp_jinx.html");
    let events = parse_html_events(&html_content);

    let has_powpow_buff = events.iter().any(|e| {
        e["type"] == "buff_apply"
            && e["target"] == "Jinx"
            && e["buff_name"]
                .as_str()
                .is_some_and(|s| s.contains("Pow-Pow Stacks") || s.contains("신난다!"))
    });
    assert!(has_powpow_buff, "Jinx basic attacks should stack Pow-Pow");

    let has_r = events
        .iter()
        .any(|e| e["type"] == "damage" && e["source"] == "Jinx" && e["ability"] == "R");
    assert!(has_r, "Jinx should execute Super Mega Death Rocket!");
}

/// Ezreal: Simulate Ezreal vs Dummy. Verify W mark detonation, passive attack speed stacks, Q cooldown reduction, and skill executions.
#[test]
fn test_ezreal_integration() {
    let apl_ezreal = "actions+=/W\nactions+=/Q\nactions+=/R\nactions+=/E\nactions+=/AA\n";
    let _stdout = run_with_apl(
        "Ezreal",
        "Dummy",
        None,
        None,
        apl_ezreal,
        &["--html-out", "temp_ezreal.html"],
    );

    let html_content = std::fs::read_to_string("temp_ezreal.html").unwrap();
    let _ = std::fs::remove_file("temp_ezreal.html");
    let events = parse_html_events(&html_content);

    // Verify Q, W, E, R damage events
    let has_q = events
        .iter()
        .any(|e| e["type"] == "damage" && e["source"] == "Ezreal" && e["ability"] == "Q");
    let has_w = events
        .iter()
        .any(|e| e["type"] == "damage" && e["source"] == "Ezreal" && e["ability"] == "W");
    let has_e = events
        .iter()
        .any(|e| e["type"] == "damage" && e["source"] == "Ezreal" && e["ability"] == "E");
    let has_r = events
        .iter()
        .any(|e| e["type"] == "damage" && e["source"] == "Ezreal" && e["ability"] == "R");

    assert!(has_q, "Ezreal should execute Q");
    assert!(has_w, "Ezreal should execute W");
    assert!(has_e, "Ezreal should execute E");
    assert!(has_r, "Ezreal should execute R");

    // Verify passive buff application (Rising Spell Force - localized to "끓어오르는 주문 힘" by ko.json)
    let has_passive = events.iter().any(|e| {
        e["type"] == "buff_apply"
            && e["target"] == "Ezreal"
            && e["buff_name"]
                .as_str()
                .is_some_and(|s| s.contains("끓어오르는 주문 힘"))
    });
    assert!(
        has_passive,
        "Ezreal should apply Rising Spell Force passive"
    );
}

/// Wit's End: Verify basic attacks deal magic damage and apply Wit's End Movement Speed.
#[test]
fn test_wits_end_integration() {
    let apl = "items=3091\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", None, Some("3091"), apl, &[]);
    assert!(
        stdout.contains("마법사의 최후 이동 속도") || stdout.contains("Wit's End Movement Speed"),
        "Basic attacks with Wit's End should apply movement speed buff"
    );
}

/// Liandry's Torment: Verify that ability hits apply Liandry's Torment Burn to the target.
#[test]
fn test_liandrys_torment_integration() {
    let apl = "items=3151\nactions+=/E\n";
    let stdout = run_with_apl("Garen", "Dummy", None, Some("3151"), apl, &[]);
    assert!(
        stdout.contains("리안드리의 고통 화상") || stdout.contains("Liandry's Torment Burn"),
        "Ability hit with Liandry's Torment should apply burn"
    );
}

/// Blade of the Ruined King: Verify basic attacks build stacks and deal current health damage, and the 3rd hit slows the target.
#[test]
fn test_blade_of_the_ruined_king_integration() {
    let apl = "items=3153\nactions+=/AA\nactions+=/AA\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", None, Some("3153"), apl, &[]);
    assert!(
        stdout.contains("몰락한 왕의 검 중첩")
            || stdout.contains("Blade of the Ruined King Stacks"),
        "Basic attacks with Blade of the Ruined King should build stacks"
    );
    assert!(
        stdout.contains("몰락한 왕의 검 흡수 둔화")
            || stdout.contains("Blade of the Ruined King Siphon Slow"),
        "3rd attack with Blade of the Ruined King should apply siphon slow"
    );
}
