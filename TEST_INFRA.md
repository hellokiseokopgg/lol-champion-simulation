# E2E Test Infra: lol-champion-simulation

## Test Philosophy
- Opaque-box, requirement-driven. No dependency on implementation design.
- Methodology: Category-Partition + BVA + Pairwise + Workload Testing.

## Feature Inventory
| # | Feature | Source (requirement) | Tier 1 | Tier 2 | Tier 3 |
|---|---------|---------------------|:------:|:------:|:------:|
| 1 | Electrocute Rune | ORIGINAL_REQUEST & PROJECT.md | 5      | 5      | ✓      |
| 2 | Press the Attack Rune | ORIGINAL_REQUEST & PROJECT.md | 5      | 5      | ✓      |
| 3 | Grasp of the Undying | Milestone 8 & PROJECT.md | 2      | 2      | ✓      |
| 4 | Triumph | Milestone 8 & PROJECT.md | 2      | 2      | ✓      |
| 5 | Legend: Alacrity | Milestone 8 & PROJECT.md | 2      | 2      | ✓      |
| 6 | Last Stand | Milestone 8 & PROJECT.md | 2      | 2      | ✓      |
| 7 | Bone Plating | Milestone 8 & PROJECT.md | 2      | 2      | ✓      |
| 8 | Overgrowth | Milestone 8 & PROJECT.md | 2      | 2      | ✓      |
| 9 | Infinity Edge | Milestone 9 & PROJECT.md | 2      | 2      | ✓      |
| 10| Mortal Reminder | Milestone 9 & PROJECT.md | 2      | 2      | ✓      |
| 11| Phantom Dancer | Milestone 9 & PROJECT.md | 2      | 2      | ✓      |
| 12| Ahri | Milestone 10 & PROJECT.md | 3      | 3      | ✓      |
| 13| Zed | Milestone 10 & PROJECT.md | 3      | 3      | ✓      |
| 14| Jinx | Milestone 10 & PROJECT.md | 3      | 3      | ✓      |

## Test Architecture
- Test runner: `cargo test` runs the integration tests under the `tests/` directory.
- Invocation: Integration tests spawn the `lol-champion-simulation` binary using `std::process::Command`, executing `cargo run -- simulate` with specified flags and APL scripts, and capture stdout for assertions.
- Test case format: Rust code with descriptive test names. Custom APL scripts are written to temp files or passed via arguments.
- Directory layout:
  - `tests/common/mod.rs`: Test helpers to invoke the binary, write temp APL files, parse CLI output, and check conditions.
  - `tests/tier1_feature.rs`: Tier 1 (Feature Coverage) test cases.
  - `tests/tier2_boundary.rs`: Tier 2 (Boundary & Corner cases) test cases.
  - `tests/tier3_combo.rs`: Tier 3 (Cross-feature combinations) test cases.
  - `tests/tier4_realworld.rs`: Tier 4 (Real-world application scenarios) test cases.

## Test Scenarios Detail (Tier 1 - Tier 3)

### Tier 1: Feature Coverage
- **Minor Runes**:
  - Grasp of the Undying: Verify that basic attacks on a 4s combat timer trigger bonus magic damage, heal the actor, and grant permanent max health.
  - Triumph: Verify that takedowns trigger a delayed heal of 2.5% max health and bonus gold.
  - Legend: Alacrity: Verify that champion takedowns or minion points grant stacks that increase attack speed.
  - Last Stand: Verify damage amplification scale from 5% to 11% based on actor health between 60% and 30%.
  - Bone Plating: Verify that after taking damage from an enemy champion, the next 3 spells or attacks deal flat reduced damage.
  - Overgrowth: Verify that being near dying minions increases max health, with an additional % bonus at 120 minions.
- **Items**:
  - Infinity Edge: Verify critical strike damage is increased (e.g., from 175% to 215%).
  - Mortal Reminder: Verify physical attacks apply Grievous Wounds (reducing healing) and provide armor penetration.
  - Phantom Dancer: Verify basic attacks grant attack speed stacks and ghosting (no collision, movement speed).
- **Champions**:
  - Ahri: Verify Q (Orb of Deception, true damage return), W (Fox-Fire targeting), E (Charm, damage amp & pull), R (Spirit Rush charges), and Passive (Essence Theft healing).
  - Zed: Verify Passive (Contempt for the Weak extra magic damage on low health), Q (Razor Shuriken damage reduction per target), W (Living Shadow mimicking), E (Shadow Slash slow), R (Death Mark shadow placement and delayed pop).
  - Jinx: Verify Q (Switcheroo! Pow-Pow attack speed stacking, Fishbones range/mana drain/splash), W (Zap! slow/reveal), E (Flame Chompers root), R (Super Mega Death Rocket execution damage based on missing health).

### Tier 2: Boundary & Corner Cases
- **Minor Runes**:
  - Grasp of the Undying: Combat timer ticking exactly at 3.9s vs 4.0s; Grasp proc consuming stack instantly.
  - Triumph: Takedown at the exact frame of actor death; healing value cap checks.
  - Legend: Alacrity: Behavior when reaching the max cap of 10 stacks; non-integer speed calculations.
  - Last Stand: Damage amplification scaling at exactly 60%, 45%, and 30% health boundary marks.
  - Bone Plating: Buff triggering on multi-hit spells (does one multi-hit count as 1 stack or multiple?).
  - Overgrowth: Exact transition at 120 minion deaths.
- **Items**:
  - Infinity Edge: Scaling behavior when critical strike rate is 0% vs 100%.
  - Mortal Reminder: Duration refresh of Grievous Wounds when reapplied at 0.1s remaining.
  - Phantom Dancer: Stacks decaying sequentially vs all at once when out of combat.
- **Champions**:
  - Ahri: Charm hitting target at max cast range boundary; Q returning damage while Ahri is dead.
  - Zed: Death Mark popping after Zed is dead or when the target invulnerable/shielded.
  - Jinx: Switching Q stance rapidly within a single attack animation; R damage scaling at minimum range vs maximum range.

### Tier 3: Cross-Feature Combinations
- **Grasp & Overgrowth**: Scaling max health synergy where Grasp and Overgrowth amplify each other's bonuses.
- **Last Stand & Triumph**: Low-health damage boost from Last Stand followed immediately by Triumph healing on takedown, causing Last Stand amp to decrease.
- **Infinity Edge & Phantom Dancer**: Critical strike scaling with rapid attack speed stacks.
- **Zed Replication & Mortal Reminder**: Zed shadow clones applying Grievous Wounds and armor penetration on multiple targets.

## Real-World Application Scenarios (Tier 4)
| # | Scenario | Features Exercised | Complexity |
|---|----------|--------------------|------------|
| 1 | Garen Electrocute vs Dummy | Electrocute triggers, cooldown tracks, custom APL combo | Medium |
| 2 | Darius PTA vs Dummy | PTA triggers, stacks accumulate on basic attacks, amplification increases Q/W damage | Medium |
| 3 | Garen PTA vs Darius Electrocute | 1v1 battle, both runes trigger, PTA amplifies Garen's damage, Electrocute inflicts burst | High |
| 4 | Darius Electrocute vs Garen PTA | 1v1 battle, Darius procs Electrocute with AA-W-Q combo, Garen procs PTA with spin/autos | High |
| 5 | Garen Electrocute vs Darius (Item Build) | High level, items equipped, Electrocute adaptive damage scales, MR reduces damage | High |
| 6 | Ahri (Electrocute/Triumph) vs Jinx (PTA/PD) | Mid-game trade where Ahri attempts a full burst and Jinx kites back using Phantom Dancer's passive | High |
| 7 | Zed (Last Stand/IE) vs Garen (Grasp/Bone Plating) | High-stakes execution trade where Zed attempts to proc Death Mark, Garen absorbs burst with Bone Plating | High |
| 8 | Jinx (Mortal Reminder/Alacrity) vs Darius (Overgrowth/Triumph) | Late-game DPS check. Jinx applies Grievous Wounds to cut Darius's Q healing | High |
| 9 | Ahri vs Zed 1v1 Mirror-like Trade | Burst vs evasion, testing energy management, cooldown tracking, and skillshot hits | High |

## Coverage Thresholds
- Tier 1: ≥2-5 per feature (Total 35+ test cases)
- Tier 2: ≥2-5 per feature (Total 35+ test cases)
- Tier 3: pairwise coverage of major feature interactions (Total 8 test cases)
- Tier 4: ≥9 realistic application scenarios (Total 9 test cases)
