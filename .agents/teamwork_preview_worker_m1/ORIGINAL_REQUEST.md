## 2026-06-17T07:10:31Z

You are the Core Engine Worker.
Your working directory is `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_worker_m1`.
Your identity: type `teamwork_preview_worker`, milestone `m1`.
Your task is to implement the Core Engine changes to support rune damage and amplification.

Please edit the following files:
1. `crates/lol-core/src/types.rs`:
   - Add `Electrocute` and `PressTheAttack` to the `AbilitySlot` enum.
2. `crates/lol-core/src/damage.rs`:
   - In `DamagePipeline::process` (around line 127), change the condition `if defender_stats.damage_reduction_percent > 0.0` to `if defender_stats.damage_reduction_percent != 0.0` to support negative values (damage amplification).
3. `crates/lol-core/src/rune_manager.rs`:
   - Update `RuneEvent` enum to add the following variant:
     ```rust
     DamageDealt {
         amount: f64,
         damage_type: crate::types::DamageType,
         slot: crate::types::AbilitySlot,
     }
     ```
4. `crates/lol-core/src/event.rs`:
   - In `trigger_on_damage_dealt`, handle the new `RuneEvent::DamageDealt` variant:
     - Find the target champion ID by looking for a key in `self.champions` that is not equal to `actor`.
     - Retrieve `attacker_stats` from the actor and `defender_stats` from the target champion.
     - Process the damage through `crate::damage::DamagePipeline::process`.
     - Apply final damage to the target champion using `take_damage`.
     - Record the damage event using `record_damage` if `self.recorder` is present.
     - If the target is dead, push a `DeathEvent` to `self.new_events`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

After applying the changes:
1. Verify that the project builds successfully by running `cargo build`.
2. Run unit tests using `cargo test --workspace` to ensure no existing tests are broken.
3. Write your handoff report to `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_worker_m1/handoff.md`. Include the cargo build and test command outputs.
4. Send a completion message to the Implementation Track Orchestrator (conversation ID: c3132716-5247-4b0c-b685-fa8da033089a).
