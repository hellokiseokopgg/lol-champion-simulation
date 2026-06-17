# Project: LoL Champion Simulation Rune Extension

## Architecture
- **lol-core**: Event manager, status effects, damage pipeline, and rune manager. We will add `Electrocute` and `Press the Attack` runes to the rune manager, update `DamagePipeline` to support damage amplification and adaptive damage, and add timeline events.
- **lol-data**: Game data loader for runes. We will add `Electrocute` and `Press the Attack` to `data/runes.json`.
- **lol-champions**: Champion-specific modules. We will register `Electrocute` and `Press the Attack` in Garen and Darius modules when equipped.
- **lol-apl**: APL parser and executor. Handles execution of action lists (no changes needed, but must be compatible).
- **lol-report**: Reporting and formatting. Formats timeline events and breakdown charts (no changes needed, but must format new slots/buffs correctly).

## Code Layout
- `crates/lol-core/src/rune_manager.rs`: Add implementations for `Electrocute` and `Press the Attack` runes.
- `crates/lol-core/src/damage.rs`: Support negative `damage_reduction_percent` (damage amplification) or explicit amplification.
- `crates/lol-core/src/types.rs`: Add `Electrocute` and `PressTheAttack` to `AbilitySlot` enum.
- `crates/lol-core/src/event.rs`: Add `RuneEvent::DamageDealt` handling and target-finding logic to `trigger_on_damage_dealt`.
- `crates/lol-champions/src/garen.rs`: Support equipping the new runes.
- `crates/lol-champions/src/darius.rs`: Support equipping the new runes.
- `data/runes.json`: Add definition for the new runes.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | E2E Testing Track | Design E2E test infra and E2E test cases (Tiers 1-4). Create `TEST_INFRA.md` and publish `TEST_READY.md`. | None | DONE (Conv: 5a8606c0-fe7e-4ac9-931e-d96a29c011da) |
| 2 | Core Engine Update | Update `types.rs`, `damage.rs`, and `event.rs` to support rune damage and amplification. | M1 | DONE (Conv: c3132716-5247-4b0c-b685-fa8da033089a) |
| 3 | Electrocute Implementation | Implement `Electrocute` in `rune_manager.rs` and update champions to register it. | M2 | DONE (Conv: c3132716-5247-4b0c-b685-fa8da033089a) |
| 4 | Press the Attack Implementation | Implement `Press the Attack` in `rune_manager.rs` and update champions to register it. | M2 | DONE (Conv: c3132716-5247-4b0c-b685-fa8da033089a) |
| 5 | Data and Registry updates | Add runes to `data/runes.json` and ensure full integration. | M3, M4 | DONE (Conv: c3132716-5247-4b0c-b685-fa8da033089a) |
| 6 | Verification & Hardening | Pass E2E tests, audit integrity, and perform Phase 2 adversarial hardening. | M5 | DONE (Conv: 4ec36e23-d757-4a2c-9b93-63787a5ab694) |

## Interface Contracts
### `trigger_on_damage_dealt` behavior
- When actor deals damage, the `SimContext` notifies the rune manager.
- If a rune triggers, it returns a `RuneEvent::DamageDealt` (or similar) to `SimContext`.
- `SimContext` processes the damage against the target (the other champion in the 1v1 sim) using `DamagePipeline`.
- The damage event is recorded under `AbilitySlot::Electrocute` or `AbilitySlot::PressTheAttack`.

### PTA Exposure Buff
- debuff name: `"Press the Attack Exposure"`
- stat modifier: `damage_reduction_percent: -0.08` (8% amplification)
