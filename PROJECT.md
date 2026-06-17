# Project: LoL Champion Simulation Rune Extension

## Architecture
- **lol-core**: Event manager, status effects, damage pipeline, and rune manager. We will add `Electrocute` and `Press the Attack` runes to the rune manager, update `DamagePipeline` to support damage amplification and adaptive damage, and add timeline events. We will also add support for 6 minor runes (Grasp of the Undying, Triumph, Legend: Alacrity, Last Stand, Bone Plating, and Overgrowth), and item effects for Infinity Edge, Mortal Reminder, and Phantom Dancer.
- **lol-data**: Game data loader for runes. We will add `Electrocute` and `Press the Attack` to `data/runes.json`. We will map items to their dynamic effects in `item_data.rs`.
- **lol-champions**: Champion-specific modules. We will register `Electrocute` and `Press the Attack` in Garen and Darius modules when equipped. We will implement new champion modules for Ahri, Zed, and Jinx in `ahri.rs`, `zed.rs`, and `jinx.rs`.
- **lol-apl**: APL parser and executor. Handles execution of action lists (no changes needed, but must be compatible).
- **lol-report**: Reporting and formatting. Formats timeline events and breakdown charts (no changes needed, but must format new slots/buffs correctly).

## Code Layout
- `crates/lol-core/src/rune_manager.rs`: Add implementations for `Electrocute` and `Press the Attack` runes, and new minor runes (Grasp of the Undying, Triumph, Legend: Alacrity, Last Stand, Bone Plating, and Overgrowth).
- `crates/lol-core/src/damage.rs`: Support negative `damage_reduction_percent` (damage amplification) or explicit amplification.
- `crates/lol-core/src/types.rs`: Add `Electrocute` and `PressTheAttack` to `AbilitySlot` enum.
- `crates/lol-core/src/event.rs`: Add `RuneEvent::DamageDealt` handling and target-finding logic to `trigger_on_damage_dealt`.
- `crates/lol-champions/src/garen.rs`: Support equipping the new runes.
- `crates/lol-champions/src/darius.rs`: Support equipping the new runes.
- `data/runes.json`: Add definition for the new runes.
- `crates/lol-core/src/item.rs`: Add item effects for Infinity Edge, Mortal Reminder, and Phantom Dancer.
- `crates/lol-data/src/item_data.rs`: Map items to their dynamic effects.
- `crates/lol-champions/src/ahri.rs`: Implement Ahri's abilities, passive, and essence theft logic.
- `crates/lol-champions/src/zed.rs`: Implement Zed's replication, energy system, passive, and shadow slash/death mark.
- `crates/lol-champions/src/jinx.rs`: Implement Jinx's Pow-Pow/Fishbones stances, W, E, and R abilities.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | E2E Testing Track | Design E2E test infra and E2E test cases (Tiers 1-4). Create `TEST_INFRA.md` and publish `TEST_READY.md`. | None | DONE (Conv: 5a8606c0-fe7e-4ac9-931e-d96a29c011da) |
| 2 | Core Engine Update | Update `types.rs`, `damage.rs`, and `event.rs` to support rune damage and amplification. | M1 | DONE (Conv: c3132716-5247-4b0c-b685-fa8da033089a) |
| 3 | Electrocute Implementation | Implement `Electrocute` in `rune_manager.rs` and update champions to register it. | M2 | DONE (Conv: c3132716-5247-4b0c-b685-fa8da033089a) |
| 4 | Press the Attack Implementation | Implement `Press the Attack` in `rune_manager.rs` and update champions to register it. | M2 | DONE (Conv: c3132716-5247-4b0c-b685-fa8da033089a) |
| 5 | Data and Registry updates | Add runes to `data/runes.json` and ensure full integration. | M3, M4 | DONE (Conv: c3132716-5247-4b0c-b685-fa8da033089a) |
| 6 | Verification & Hardening | Pass E2E tests, audit integrity, and perform Phase 2 adversarial hardening. | M5 | DONE (Conv: 4ec36e23-d757-4a2c-9b93-63787a5ab694) |
| 7 | E2E Test Suite Expansion (E2E Testing Track) | Scope: Add Tier 1-4 integration test cases for 6 minor runes, 3 champions (Ahri, Zed, Jinx), and 3 items. Update `TEST_INFRA.md` and publish `TEST_READY.md`. | M6 | DONE |
| 8 | Minor Runes Implementation | Scope: Implement Grasp of the Undying, Triumph, Legend: Alacrity, Last Stand, Bone Plating, and Overgrowth. | M7 | DONE |
| 9 | Item Effects Implementation | Scope: Implement passive effects for Infinity Edge, Mortal Reminder, and Phantom Dancer. | M7 | DONE |
| 10 | Complete Champion Mechanics | Scope: Implement abilities, passive, and status effects for Ahri, Zed, and Jinx. | M7 | DONE |
| 11 | Final Integration Track & E2E Pass | Scope: Run all E2E tests and ensure 100% pass with 0 regressions. | M8, M9, M10 | DONE |
| 12 | Adversarial Hardening (Tier 5) | Scope: Edge-case verification and Forensic Audit. | M11 | DONE |

## Interface Contracts
### `trigger_on_damage_dealt` behavior
- When actor deals damage, the `SimContext` notifies the rune manager.
- If a rune triggers, it returns a `RuneEvent::DamageDealt` (or similar) to `SimContext`.
- `SimContext` processes the damage against the target (the other champion in the 1v1 sim) using `DamagePipeline`.
- The damage event is recorded under `AbilitySlot::Electrocute` or `AbilitySlot::PressTheAttack`.

### PTA Exposure Buff
- debuff name: `"Press the Attack Exposure"`
- stat modifier: `damage_reduction_percent: -0.08` (8% amplification)
