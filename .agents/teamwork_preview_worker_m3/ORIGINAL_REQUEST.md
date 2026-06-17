## 2026-06-17T07:20:52Z

You are the Press the Attack Worker.
Your working directory is `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_worker_m3`.
Your identity: type `teamwork_preview_worker`, milestone `m3`.
Your task is to implement the Press the Attack rune and update event handling for debuff application.

Please edit the following files:

1. `crates/lol-core/src/rune_manager.rs`:
   - In the `RuneEvent` enum, add the new variant:
     ```rust
     ApplyDebuff {
         name: String,
         duration: f64,
         damage_reduction_percent: f64,
     }
     ```
   - Implement the `PressTheAttack` struct:
     - Fields:
       - `is_melee: bool`
       - `stacks: u32`
       - `last_attack_time: f64` (initialize to `-999.0`)
       - `last_trigger_time: f64` (initialize to `-999.0`)
       - `was_exposed: bool`
       - `base_ad: f64` (initialize to `0.0`)
     - Implement `RuneEffect` for `PressTheAttack`:
       - `name` returns `"Press the Attack"`.
       - `get_bonus_stats` caches `self.base_ad = base_stats.attack_damage;` and returns `StatBlock::new()`.
       - `on_damage_dealt`:
         - If `self.was_exposed`:
           - If `time.as_f64() - self.last_trigger_time >= 6.0` (exposure duration expired), set `self.was_exposed = false`, `self.stacks = 0`.
           - Otherwise, return `Vec::new()`.
         - If `self.stacks > 0 && time.as_f64() - self.last_attack_time > 4.0` (stacks expired), set `self.stacks = 0`.
         - If `slot == crate::types::AbilitySlot::AutoAttack && !is_ability`:
           - Set `self.last_attack_time = time.as_f64()`.
           - Increment `self.stacks += 1`.
           - If `self.stacks >= 3`:
             - Set `self.last_trigger_time = time.as_f64()`.
             - Set `self.was_exposed = true`.
             - Reset `self.stacks = 0`.
             - Base damage: `40.0 + (180.0 - 40.0) / 17.0 * (level.saturating_sub(1) as f64)`.
             - Bonus AD: `(attacker_stats.attack_damage - self.base_ad).max(0.0)`.
             - AP: `attacker_stats.ability_power`.
             - Damage Type (Adaptive): physical if `bonus_ad > ap`, magic otherwise.
             - Return a `Vec` containing:
               1. `RuneEvent::StacksChanged { name: "Press the Attack".to_string(), stacks: 3 }`
               2. `RuneEvent::ApplyDebuff { name: "Press the Attack Exposure".to_string(), duration: 6.0, damage_reduction_percent: -0.08 }`
               3. `RuneEvent::DamageDealt { amount: base_damage, damage_type, slot: crate::types::AbilitySlot::PressTheAttack }`
         - Return `Vec::new()` by default.
       - `on_tick`:
         - If `self.was_exposed && time.as_f64() - self.last_trigger_time >= 6.0`:
           - Set `self.was_exposed = false`.
           - Set `self.stacks = 0`.
           - Return `vec![RuneEvent::StacksChanged { name: "Press the Attack".to_string(), stacks: 0 }]`.
         - Otherwise, return `Vec::new()`.

2. `crates/lol-core/src/event.rs`:
   - In `trigger_on_damage_dealt`, update the `RuneEvent::StacksChanged` match branch (around line 117):
     - In the `duration` calculation check, add checking for `"Press the Attack"` (e.g. `else if name.contains("Press the Attack") { 6.0 }`).
   - In `trigger_on_damage_dealt`, handle the new `RuneEvent::ApplyDebuff` variant:
     - Find the target champion ID by looking for a key in `self.champions` that is not equal to `actor`.
     - Define a local debuff struct implementing `crate::buff::StatusEffect`:
       - `id` returns `crate::types::EffectId(self.name.clone())`
       - `name` returns `&self.name`
       - `duration` returns `self.duration`
       - `refresh_behavior` returns `crate::buff::RefreshBehavior::RefreshDuration`
       - `max_stacks` returns `1`
       - `stat_modifiers` returns a `StatBlock` with `damage_reduction_percent` set to `self.damage_reduction_percent`.
     - Apply this debuff using `self.apply_buff(&target_id, Box::new(DebuffStructInstance))`.

3. `crates/lol-champions/src/garen.rs`:
   - In `GarenModule::create_instance`, register `PressTheAttack` in the keystone checker:
     ```rust
     } else if keystone_name == "Press the Attack" {
         state.rune_manager.add_effect(Box::new(lol_core::rune_manager::PressTheAttack::new(true)));
     ```

4. `crates/lol-champions/src/darius.rs`:
   - In `DariusModule::create_instance`, register `PressTheAttack` in the keystone checker:
     ```rust
     } else if keystone_name == "Press the Attack" {
         state.rune_manager.add_effect(Box::new(lol_core::rune_manager::PressTheAttack::new(true)));
     ```

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

After applying changes:
1. Verify that the project builds successfully by running `cargo build`.
2. Run `cargo test -p lol-core` to verify all lol-core unit tests pass.
3. Write your handoff report to `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_worker_m3/handoff.md`. Include cargo build and test command outputs.
4. Send a completion message to the Implementation Track Orchestrator (conversation ID: c3132716-5247-4b0c-b685-fa8da033089a).
