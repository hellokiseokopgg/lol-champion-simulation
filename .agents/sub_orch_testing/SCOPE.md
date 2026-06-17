# Scope: E2E Testing Track

## Architecture
- Opaque-box, requirement-driven testing.
- Uses cargo commands or rust integration tests (under `tests/` directory) to execute the simulation and inspect output report/breakdown.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | Test Infrastructure Design | Create `TEST_INFRA.md` defining feature inventory, test runner, layout, and scenario list. | None | DONE |
| 2 | Tier 1 Tests (Feature Coverage) | Create happy-path tests for Electrocute and PTA (minimum 5 tests per feature). | M1 | DONE |
| 3 | Tier 2 Tests (Boundary & Corner Cases) | Create edge cases, empty/invalid arguments, and cooldown limits tests. | M1 | DONE |
| 4 | Tier 3 Tests (Cross-Feature Combinations) | Create pairwise combination tests (runes + items, runes + abilities). | M1 | DONE |
| 5 | Tier 4 Tests (Real-World Workloads) | Create realistic combat match scenarios using Garen and Darius. | M1 | DONE |
| 6 | Publish E2E Test Suite | Finalize runner command, execute checks, verify structure, and write `TEST_READY.md`. | M2, M3, M4, M5 | DONE |

## Interface Contracts
- Tests must be runnable using `cargo test` or a CLI script.
- Tests must check that "Electrocute" and "Press the Attack" are correctly triggered, burst damage is applied, cooldowns are tracked, and PTA exposure amplifies subsequent damage.
