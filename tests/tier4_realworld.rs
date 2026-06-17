mod common;

use common::run_with_apl;

// =========================================================================
// Real-World Workloads (Tier 4) Tests
// =========================================================================

/// Test 1: Garen (Electrocute) vs Target Dummy.
#[test]
fn test_realworld_garen_electrocute_vs_dummy() {
    let apl = "runes=electrocute\nactions+=/Q\nactions+=/E\nactions+=/AA\n";
    let stdout = run_with_apl("Garen", "Dummy", Some("electrocute"), None, apl, &[]);

    // Output should contain Garen and Dummy, simulation must not crash.
    assert!(stdout.contains("Champion: \"Garen\""));
    assert!(stdout.contains("Champion: \"Dummy\""));
}

/// Test 2: Darius (PTA) vs Target Dummy.
#[test]
fn test_realworld_darius_pta_vs_dummy() {
    let apl = "runes=press_the_attack\nactions+=/AA\n";
    let stdout = run_with_apl("Darius", "Dummy", Some("press_the_attack"), None, apl, &[]);

    assert!(stdout.contains("Champion: \"Darius\""));
    assert!(stdout.contains("Champion: \"Dummy\""));
}

/// Test 3: Garen (PTA) vs Darius (Electrocute).
#[test]
fn test_realworld_garen_pta_vs_darius_electrocute() {
    let darius_apl_filename = "temp_darius_test.apl";
    std::fs::write(
        darius_apl_filename,
        "runes=electrocute\nactions+=/Q\nactions+=/W\nactions+=/AA\n",
    )
    .unwrap();

    let apl_garen = "runes=press_the_attack\nactions+=/AA\n";
    let stdout = run_with_apl(
        "Garen",
        "Darius",
        Some("press_the_attack"),
        None,
        apl_garen,
        &["--apl-b", darius_apl_filename],
    );

    let _ = std::fs::remove_file(darius_apl_filename);

    assert!(stdout.contains("Champion: \"Garen\""));
    assert!(stdout.contains("Champion: \"Darius\""));
}

/// Test 4: Darius (Electrocute) vs Garen (PTA).
#[test]
fn test_realworld_darius_electrocute_vs_garen_pta() {
    let garen_apl_filename = "temp_garen_test.apl";
    std::fs::write(garen_apl_filename, "runes=press_the_attack\nactions+=/AA\n").unwrap();

    let apl_darius = "runes=electrocute\nactions+=/Q\nactions+=/W\nactions+=/AA\n";
    let stdout = run_with_apl(
        "Darius",
        "Garen",
        Some("electrocute"),
        None,
        apl_darius,
        &["--apl-b", garen_apl_filename],
    );

    let _ = std::fs::remove_file(garen_apl_filename);

    assert!(stdout.contains("Champion: \"Garen\""));
    assert!(stdout.contains("Champion: \"Darius\""));
}

/// Test 5: Jinx (PTA) vs Garen (Electrocute) with items.
#[test]
fn test_realworld_jinx_pta_vs_garen_electrocute_items() {
    let garen_apl_filename = "temp_garen_items_test.apl";
    std::fs::write(
        garen_apl_filename,
        "runes=electrocute\nactions+=/Q\nactions+=/E\nactions+=/AA\n",
    )
    .unwrap();

    let apl_jinx = "runes=press_the_attack\nactions+=/AutoAttack\n";
    let stdout = run_with_apl(
        "Jinx",
        "Garen",
        Some("press_the_attack"),
        Some("3046"),
        apl_jinx,
        &["--apl-b", garen_apl_filename],
    );

    let _ = std::fs::remove_file(garen_apl_filename);

    assert!(stdout.contains("Champion: \"Jinx\""));
    assert!(stdout.contains("Champion: \"Garen\""));
}
