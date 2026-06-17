## 2026-06-17T07:55:23Z
You are the Bug Fix Worker.
Your working directory is `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_worker_m5_final`.
Your task is to fix the remaining correctness, convention, and clippy issues identified by the reviewers.

Specifically, implement the following fixes:

1. Darius Swapped Slots Fix:
   - In `crates/lol-champions/src/darius.rs` under `DariusAutoAttack::execute` (around line 692), modify the `ctx.trigger_on_damage_dealt` call to pass `if has_w_buff { lol_core::types::AbilitySlot::W } else { lol_core::types::AbilitySlot::AutoAttack }` instead of `lol_core::types::AbilitySlot::Passive`.
   - In `crates/lol-champions/src/darius.rs` under `HemorrhageTickEvent::execute` (around line 236), modify the `ctx.trigger_on_damage_dealt` call to pass `lol_core::types::AbilitySlot::Passive` instead of `lol_core::types::AbilitySlot::AutoAttack`, and set `is_ability` to `true`.

2. Darius Noxian Guillotine (R) Rune Trigger:
   - In `crates/lol-champions/src/darius.rs` under `DariusR::execute`, add a call to `ctx.trigger_on_damage_dealt(actor, final_damage, true, AbilitySlot::R);` right after the damage is recorded/applied (around line 565 or 566).

3. Base AD Caching Bug Fix:
   - In the `update_stats` methods of all champion modules (`crates/lol-champions/src/ahri.rs`, `crates/lol-champions/src/darius.rs`, `crates/lol-champions/src/dummy.rs`, `crates/lol-champions/src/garen.rs`, `crates/lol-champions/src/jinx.rs`, `crates/lol-champions/src/zed.rs`), replace the `rune_manager.get_bonus_stats` call:
     Change `&self.state.base_stats` to `&self.state.stats.base` so that the runes receive and cache the current level's base AD instead of the Level 1 base AD. (Note: in `darius.rs`, the level variable is `level_u32` instead of `level`).

4. Safe Stridebreaker Active Option Handling:
   - In `crates/lol-core/src/item.rs` under `StridebreakerActive::execute` (lines 276-277), replace the `.unwrap()` calls with safe matching (e.g. `let attacker_cell = match ctx.champions.get(actor) { Some(c) => c, None => return };`) to prevent panics and follow the unwrap prohibition convention.

5. Missing Doc Comments:
   - In `crates/lol-core/src/rune_manager.rs`, add doc comments (`///`) to the public structs `Electrocute` and `PressTheAttack` to comply with the project's documentation conventions.

6. Clean up Clippy Warnings in the workspace:
   - Run `cargo clippy --workspace --all-targets` and resolve the warnings. Specifically, in `crates/lol-apl/src/expression.rs`, replace manual string slicing with `strip_prefix` (e.g. `if let Some(val_str) = input.strip_prefix("health.pct<") { ... }`). In `crates/lol-report/src/collector.rs` at line 93, use `.or_default()` instead of `.or_insert_with(Vec::new)`. Ensure the entire workspace has exactly 0 clippy warnings.

7. Verify all tests pass (`cargo test --workspace`).

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Write your handoff report to `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_worker_m5_final/handoff.md`. Send a completion message when done.
