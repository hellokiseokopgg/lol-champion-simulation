# Scope: Implementation Track

## Architecture
- Updates to `lol-core` for damage amplification, adaptive damage, and rune effect implementation.
- Updates to `lol-champions` for Garen and Darius registration.
- Updates to `data` JSON files for static rune stats.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | Core Engine Update | Update `types.rs`, `damage.rs`, and `event.rs` to support damage amplification and adaptive damage triggers. | None | DONE |
| 2 | Electrocute Implementation | Implement `Electrocute` in `rune_manager.rs` and update Garen and Darius modules. | M1 | DONE |
| 3 | Press the Attack Implementation | Implement `Press the Attack` in `rune_manager.rs` and update Garen and Darius modules. | M1 | DONE |
| 4 | Data Integration | Add runes to `data/runes.json` and ensure full config load compatibility. | M2, M3 | DONE |
| 5 | Phase 1 E2E Verification | Pass all Tier 1-4 tests published in `TEST_READY.md`. | M4 | DONE |
| 6 | Phase 2 Adversarial Hardening | Generate adversarial tests (Tier 5) to audit coverage gaps and ensure robustness. | M5 | DONE |

## Interface Contracts
- Code must build and clippy must show 0 warnings.
- All modifications must follow instructions in `AGENTS.md`.
