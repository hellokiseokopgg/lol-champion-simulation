## 2026-06-17T07:05:08Z
Create a file at `/Users/kskim/Projects/lol-champion-simulation/TEST_INFRA.md` with the following content:

```markdown
# E2E Test Infra: lol-champion-simulation

## Test Philosophy
- Opaque-box, requirement-driven. No dependency on implementation design.
- Methodology: Category-Partition + BVA + Pairwise + Workload Testing.

## Feature Inventory
| # | Feature | Source (requirement) | Tier 1 | Tier 2 | Tier 3 |
|---|---------|---------------------|:------:|:------:|:------:|
| 1 | Electrocute Trigger & Cooldown | ORIGINAL_REQUEST & PROJECT.md | 5      | 5      | ✓      |
| 2 | Electrocute Adaptive Damage | ORIGINAL_REQUEST & PROJECT.md | 5      | 5      | ✓      |
| 3 | PTA Trigger & Cooldown | ORIGINAL_REQUEST & PROJECT.md | 5      | 5      | ✓      |
| 4 | PTA Damage & Amplification | ORIGINAL_REQUEST & PROJECT.md | 5      | 5      | ✓      |

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

## Real-World Application Scenarios (Tier 4)
| # | Scenario | Features Exercised | Complexity |
|---|----------|--------------------|------------|
| 1 | Garen Electrocute vs Dummy | Electrocute triggers, cooldown tracks, custom APL combo | Medium |
| 2 | Darius PTA vs Dummy | PTA triggers, stacks accumulate on basic attacks, amplification increases Q/W damage | Medium |
| 3 | Garen PTA vs Darius Electrocute | 1v1 battle, both runes trigger, PTA amplifies Garen's damage, Electrocute inflicts burst | High |
| 4 | Darius Electrocute vs Garen PTA | 1v1 battle, Darius procs Electrocute with AA-W-Q combo, Garen procs PTA with spin/autos | High |
| 5 | Garen Electrocute vs Darius (Item Build) | High level, items equipped, Electrocute adaptive damage scales, MR reduces damage | High |

## Coverage Thresholds
- Tier 1: ≥5 per feature (Total 20 test cases)
- Tier 2: ≥5 per feature (Total 20 test cases)
- Tier 3: pairwise coverage of major feature interactions (Total 4 test cases)
- Tier 4: ≥5 realistic application scenarios (Total 5 test cases)
```

## 2026-06-17T07:17:41Z
Please update `TEST_INFRA.md` and create `TEST_READY.md` at the project root (`/Users/kskim/Projects/lol-champion-simulation`):

1. Edit `/Users/kskim/Projects/lol-champion-simulation/TEST_INFRA.md` to update the `Feature Inventory` table to group the features into N = 2 major features to match our implemented tests:
- Feature 1: Electrocute Rune
- Feature 2: Press the Attack Rune
Update the table and thresholds accordingly.

2. Create `/Users/kskim/Projects/lol-champion-simulation/TEST_READY.md` with the following content:
```markdown
# E2E Test Suite Ready

## Test Runner
- Command: `cargo test`
- Expected: all tests pass with exit code 0 (currently, all 29 tests compile successfully and will pass fully once the runes implementation track completes).

## Coverage Summary
| Tier | Count | Description |
|------|------:|-------------|
| 1. Feature Coverage | 10 | 5 tests for Electrocute, 5 tests for PTA |
| 2. Boundary & Corner | 10 | 5 boundary tests for Electrocute, 5 for PTA |
| 3. Cross-Feature | 4 | Pairwise combinations of runes, items, and abilities |
| 4. Real-World Application | 5 | Realistic combat matchups (Garen/Darius/Jinx/Dummy) |
| **Total** | **29** | |

## Feature Checklist
| Feature | Tier 1 | Tier 2 | Tier 3 | Tier 4 |
|---------|:------:|:------:|:------:|:------:|
| Electrocute Rune | 5 | 5 | ✓ | ✓ |
| Press the Attack Rune | 5 | 5 | ✓ | ✓ |
```

3. Run `cargo test` to verify that all 29 tests compile successfully and list which test files were compiled and run.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Write a handoff report in your directory when complete.
