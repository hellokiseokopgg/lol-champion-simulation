# E2E Test Infra: lol-champion-simulation

## Test Philosophy
- Opaque-box, requirement-driven. No dependency on implementation design.
- Methodology: Category-Partition + BVA + Pairwise + Workload Testing.

## Feature Inventory
| # | Feature | Source (requirement) | Tier 1 | Tier 2 | Tier 3 |
|---|---------|---------------------|:------:|:------:|:------:|
| 1 | Electrocute Rune | ORIGINAL_REQUEST & PROJECT.md | 5      | 5      | ✓      |
| 2 | Press the Attack Rune | ORIGINAL_REQUEST & PROJECT.md | 5      | 5      | ✓      |

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
- Tier 1: ≥5 per feature (Total 10 test cases)
- Tier 2: ≥5 per feature (Total 10 test cases)
- Tier 3: pairwise coverage of major feature interactions (Total 4 test cases)
- Tier 4: ≥5 realistic application scenarios (Total 5 test cases)
