# Plan — Implementation of Electrocute and Press the Attack Runes

## Goal
Implement the Electrocute and Press the Attack (PTA) runes by updating the core engine (types, damage pipeline, events), champion modules (Garen, Darius), and data files. Verify correctness through E2E tests and adversarial hardening.

## Roadmap

### Milestone 1: Core Engine Update
- **Objective**: Update core types and logic to support damage amplification and adaptive damage triggers.
- **Tasks**:
  1. Add `AbilitySlot::Electrocute` and `AbilitySlot::PressTheAttack` to `crates/lol-core/src/types.rs`.
  2. Update `crates/lol-core/src/damage.rs` to support damage amplification (negative `damage_reduction_percent`).
  3. Update `crates/lol-core/src/event.rs`'s `trigger_on_damage_dealt` to perform target-finding (finding the non-actor champion in the 1v1 sim) and process `RuneEvent::DamageDealt` to deal rune damage to the target.
- **Verification**: Ensure the engine compile/builds.

### Milestone 2: Electrocute Implementation
- **Objective**: Implement Electrocute keystone rune logic.
- **Tasks**:
  1. Define `Electrocute` struct implementing `RuneEffect` in `crates/lol-core/src/rune_manager.rs`.
  2. Implement hit tracking for Electrocute: triggers when 3 unique attacks/abilities hit the target within 3 seconds. Deals 30-180 (based on level) + 0.1 bonus AD + 0.05 AP adaptive damage. Cooldown: 25-20 seconds (based on level).
  3. Register Electrocute in Garen and Darius module initializers.
- **Verification**: Run unit/integration tests for Garen/Darius equipping Electrocute.

### Milestone 3: Press the Attack Implementation
- **Objective**: Implement Press the Attack keystone rune logic.
- **Tasks**:
  1. Define `PressTheAttack` struct implementing `RuneEffect` in `crates/lol-core/src/rune_manager.rs`.
  2. Implement attack tracking: triggers on 3 consecutive basic attacks against the same target. Deals 40-180 (based on level) adaptive damage, and exposes the target (applying the "Press the Attack Exposure" debuff, increasing damage taken by 8% for 6 seconds).
  3. Register Press the Attack in Garen and Darius module initializers.
- **Verification**: Run unit/integration tests for Garen/Darius equipping PTA.

### Milestone 4: Data Integration
- **Objective**: Update static database JSON.
- **Tasks**:
  1. Add `electrocute` and `press_the_attack` entries to `data/runes.json`.
- **Verification**: Ensure simulation config parses the new runes without crashing.

### Milestone 5: Phase 1 E2E Verification
- **Objective**: Validate the full implementation against the published E2E test suite.
- **Tasks**:
  1. Monitor `TEST_READY.md`.
  2. Run the E2E test suite and address any bugs or failing tests.
- **Verification**: All Tier 1-4 tests pass.

### Milestone 6: Phase 2 Adversarial Hardening
- **Objective**: Identify and close code coverage gaps.
- **Tasks**:
  1. Use challenger subagents to write and execute adversarial tests.
  2. Fix bugs revealed by hardening.
- **Verification**: Challenger confirms no remaining coverage gaps, and Forensic Auditor certifies integrity.
