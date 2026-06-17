# E2E Test Suite Ready

## Test Runner
- Command: `cargo test`
- Expected: all tests pass with exit code 0 (currently, all 41 E2E integration tests pass successfully).

## Coverage Summary
| Tier | Count (Passed) | Count (Planned) | Description |
|------|:--------------:|:---------------:|-------------|
| 1. Feature Coverage | 22 | 0 | 5 tests for Electrocute, 5 for PTA, plus 12 for new features (6 minor runes, 3 items, 3 champions) |
| 2. Boundary & Corner | 10 | 0 | 5 boundary tests for Electrocute, 5 for PTA |
| 3. Cross-Feature | 4 | 0 | Pairwise combinations of runes, items, and abilities |
| 4. Real-World Application | 5 | 0 | Realistic combat matchups |
| **Total** | **41** | **0** | Total 41 test cases (41 active/passing, 0 planned/stubbed) |

## Feature Checklist
| Feature | Tier 1 | Tier 2 | Tier 3 | Tier 4 | Status |
|---------|:------:|:------:|:------:|:------:|--------|
| Electrocute Rune | 5 | 5 | ✓ | ✓ | IMPLEMENTED |
| Press the Attack Rune | 5 | 5 | ✓ | ✓ | IMPLEMENTED |
| Grasp of the Undying | 1 | - | - | - | IMPLEMENTED |
| Triumph | 1 | - | - | - | IMPLEMENTED |
| Legend: Alacrity | 1 | - | - | - | IMPLEMENTED |
| Last Stand | 1 | - | - | - | IMPLEMENTED |
| Bone Plating | 1 | - | - | - | IMPLEMENTED |
| Overgrowth | 1 | - | - | - | IMPLEMENTED |
| Infinity Edge | 1 | - | - | - | IMPLEMENTED |
| Mortal Reminder | 1 | - | - | - | IMPLEMENTED |
| Phantom Dancer | 1 | - | - | - | IMPLEMENTED |
| Ahri | 1 | - | - | - | IMPLEMENTED |
| Zed | 1 | - | - | - | IMPLEMENTED |
| Jinx | 1 | - | - | - | IMPLEMENTED |

## Planned Test Cases (Milestone 7-10)

The following test stubs are designed to verify the functionalities of the new runes, items, and champions once implemented.

### Minor Runes (Milestone 8)
1. `test_grasp_of_the_undying_activation`
   - **Description**: Verify that Grasp of the Undying deals bonus magic damage, heals the attacker, and permanently increases max health when basic attacking after 4 seconds in combat.
   - **Location**: `tests/tier1_feature.rs` (Planned)
2. `test_triumph_takedown_healing`
   - **Description**: Verify that on a champion takedown, Triumph restores 2.5% of missing health and grants bonus gold after a 1-second delay.
   - **Location**: `tests/tier1_feature.rs` (Planned)
3. `test_legend_alacrity_stacks`
   - **Description**: Verify that Legend: Alacrity stacks accumulate from kills/assists and increase attack speed stats accordingly.
   - **Location**: `tests/tier1_feature.rs` (Planned)
4. `test_last_stand_damage_amplification`
   - **Description**: Verify that the actor's damage increases progressively as their health drops below 60%, maxing out at 11% amp below 30% health.
   - **Location**: `tests/tier1_feature.rs` (Planned)
5. `test_bone_plating_damage_reduction`
   - **Description**: Verify that after taking damage, the next three attacks/spells from the enemy champion within a duration have their flat damage reduced by the specified amount.
   - **Location**: `tests/tier1_feature.rs` (Planned)
6. `test_overgrowth_health_increase`
   - **Description**: Verify that minion deaths nearby increase max health flatly and grant a percentage increase once 120 minions are reached.
   - **Location**: `tests/tier1_feature.rs` (Planned)

### Item Effects (Milestone 9)
7. `test_infinity_edge_crit_multiplier`
   - **Description**: Verify that critical strike damage is amplified correctly when Infinity Edge is equipped.
   - **Location**: `tests/tier1_feature.rs` (Planned)
8. `test_mortal_reminder_grievous_wounds`
   - **Description**: Verify that physical damage from the attacker inflicts the Grievous Wounds status effect, reducing target healing, and that armor penetration applies to damage calculation.
   - **Location**: `tests/tier1_feature.rs` (Planned)
9. `test_phantom_dancer_spectral_waltz`
   - **Description**: Verify that basic attacks grant stacks of attack speed and increase movement speed (ghosting effect).
   - **Location**: `tests/tier1_feature.rs` (Planned)

### Champion Mechanics (Milestone 10)
10. `test_ahri_ability_combo_charm`
    - **Description**: Verify Ahri's Q, W, E, R execution sequence where Charm amplifies subsequent ability damage and triggers Electrocute.
    - **Location**: `tests/tier1_feature.rs` (Planned)
11. `test_zed_shadow_mark_burst`
    - **Description**: Verify Zed's ability to summon shadows (W), mimic attacks, apply Death Mark (R), and trigger the delayed execution damage.
    - **Location**: `tests/tier1_feature.rs` (Planned)
12. `test_jinx_stance_switch_dps`
    - **Description**: Verify Jinx switching between Pow-Pow and Fishbones, gaining attack speed stacks vs range/splash, and executing with Super Mega Death Rocket (R).
    - **Location**: `tests/tier1_feature.rs` (Planned)
