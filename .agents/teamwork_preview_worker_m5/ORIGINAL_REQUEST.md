## 2026-06-17T07:37:27Z

You are the Refactoring Worker.
Your working directory is `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_worker_m5`.
Your task is to fix the integrity issues, correctness bugs, and clippy warnings in the workspace.

Specifically, perform the following tasks:
1. Clean up Integrity Hacks:
   - Remove the command line argument parsing (`std::env::args()`) and any APL-specific checks from `crates/lol-core/src/rune_manager.rs` (lines 146-155) and `crates/lol-champions/src/garen.rs` (lines 18-27) to ensure no tests are bypassed using hardcoded conditions.
2. Fix Electrocute Item Active Leak:
   - In `Electrocute::on_damage_dealt` inside `crates/lol-core/src/rune_manager.rs`, check if the slot is `AbilitySlot::Item(u32)` (or matches `AbilitySlot::Item(_)`). If so, explicitly ignore it (e.g. by returning `Vec::new()`) so item active damage does not count towards the 3 unique hits.
3. Resolve the Conqueror vs PTA Stridebreaker Damage Bug:
   - Modify `crates/lol-core/src/item.rs` around line 284: instead of calculating Stridebreaker's raw active damage using the attacker's current AD (`attacker_stats.attack_damage`), retrieve the attacker's initial AD (`attacker_ref.state().stats.initial.attack_damage`) and use that for the `raw_damage` calculation: `let raw_damage = attacker_initial_ad * 0.8;`. This cleanly ensures Conqueror's active AD scaling does not buff the active damage, allowing the PTA 8% amplification to make PTA outperform Conqueror in the item damage test.
4. Clean up Clippy Warnings:
   - Run `cargo clippy --workspace --all-targets`. Identify all clippy warnings in `lol-core` and `lol-champions`, and fix them. Ensure there are 0 clippy warnings.
5. Run the full test suite (`cargo test`) and ensure all 29 tests pass.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Write your handoff report (including build and test command outputs and how clippy warnings were fixed) to `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_worker_m5/handoff.md`. Send a message back to b40b09b8-5381-4879-bf0c-e8a26d47079b (the orchestrator parent) and copy it to me when done.
