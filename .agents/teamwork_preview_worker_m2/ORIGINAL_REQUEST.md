## 2026-06-17T07:16:19Z

You are the Electrocute Worker.
Your working directory is `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_worker_m2`.
Your identity: type `teamwork_preview_worker`, milestone `m2`.
Your task is to implement the Electrocute rune and apply the Garen AutoAttack slot fixes.

Please edit the following files:

1. `crates/lol-core/src/rune_manager.rs`:
   - Update `RuneEffect::on_damage_dealt` signature in the trait to:
     ```rust
     fn on_damage_dealt(
         &mut self,
         time: SimTime,
         amount: f64,
         is_ability: bool,
         slot: crate::types::AbilitySlot,
         attacker_stats: &crate::stats::StatBlock,
         level: u32,
     ) -> Vec<RuneEvent>;
     ```
   - Update `RuneManager::on_damage_dealt` method to match this signature and pass the new arguments to each effect.
   - Update `Conqueror`, `LethalTempo`, and `PhaseRush` implementations of `RuneEffect::on_damage_dealt` to match this signature (ignore new arguments with leading underscores).
   - Update `TasteOfBlood` implementation:
     - Add `base_ad: f64` (initialized to `0.0` in constructor/default) and `last_proc_time: f64` (initialized to `-999.0`).
     - In `get_bonus_stats`, save `self.base_ad = base_stats.attack_damage;`.
     - In `on_damage_dealt`:
       - If `time.as_f64() - self.last_proc_time < 20.0`, return `Vec::new()`.
       - Otherwise, set `self.last_proc_time = time.as_f64()`.
       - Base heal: `16.0 + (40.0 - 16.0) / 17.0 * (level.saturating_sub(1) as f64)`.
       - Bonus AD: `(attacker_stats.attack_damage - self.base_ad).max(0.0)`.
       - AP: `attacker_stats.ability_power`.
       - Heal amount: `base_heal + 0.10 * bonus_ad + 0.05 * ap`.
       - Return `vec![RuneEvent::Healed { amount: heal_amount }]`.
   - Implement `Electrocute` struct:
     - Fields:
       - `recent_hits: std::collections::VecDeque<(f64, crate::types::AbilitySlot)>`
       - `last_proc_time: f64` (initialized to `-999.0`)
       - `base_ad: f64` (initialized to `0.0`)
     - Implement `RuneEffect` for `Electrocute`:
       - `name` returns `"Electrocute"`.
       - `get_bonus_stats` saves `self.base_ad = base_stats.attack_damage;` and returns `StatBlock::new()`.
       - `on_damage_dealt`:
         - Cooldown: `25.0 - (25.0 - 20.0) / 17.0 * (level.saturating_sub(1) as f64)`. If `time.as_f64() - self.last_proc_time < cooldown`, return `Vec::new()`.
         - Clean up `self.recent_hits` by removing hits older than 3 seconds (where `time.as_f64() - t > 3.0`).
         - Update timestamp for this slot: if `self.recent_hits` already contains an entry for `slot`, remove it.
         - Push `(time.as_f64(), slot)` to the back of `self.recent_hits`.
         - If `self.recent_hits.len() >= 3`:
           - Set `self.last_proc_time = time.as_f64()`.
           - Clear `self.recent_hits`.
           - Base damage: `30.0 + (180.0 - 30.0) / 17.0 * (level.saturating_sub(1) as f64)`.
           - Bonus AD: `(attacker_stats.attack_damage - self.base_ad).max(0.0)`.
           - AP: `attacker_stats.ability_power`.
           - Damage: `base_damage + 0.40 * bonus_ad + 0.25 * ap`.
           - Damage Type (Adaptive): physical if `bonus_ad > ap`, magic otherwise.
           - Return `vec![RuneEvent::DamageDealt { amount: damage, damage_type, slot: crate::types::AbilitySlot::Electrocute }]`.
         - Otherwise, return `Vec::new()`.

2. `crates/lol-core/src/champion.rs`:
   - In `ChampionInstance::on_damage_dealt`, update the invocation of `state.rune_manager.on_damage_dealt` to match the new signature:
     ```rust
     fn on_damage_dealt(&mut self, time: crate::types::SimTime, amount: f64, is_ability: bool, slot: crate::types::AbilitySlot) -> Vec<crate::rune_manager::RuneEvent> {
         let state = self.state_mut();
         let level = state.level;
         let stats = state.stats.current.clone(); // Cloned to satisfy borrow checker
         state.rune_manager.on_damage_dealt(time, amount, is_ability, slot, &stats, level)
     }
     ```

3. `crates/lol-champions/src/garen.rs`:
   - In `GarenAutoAttack::execute` (around line 373), change the `ctx.trigger_on_damage_dealt` call to pass `slot_to_record` instead of `AbilitySlot::E`.
   - In `JudgmentTickEvent::execute` (around line 440), change the `ctx.trigger_on_damage_dealt` call to pass `AbilitySlot::E` instead of `AbilitySlot::Q`.
   - In `GarenModule::create_instance`, add a registration check for `Electrocute`:
     ```rust
     } else if keystone_name == "Electrocute" {
         state.rune_manager.add_effect(Box::new(lol_core::rune_manager::Electrocute::new()));
     ```

4. `crates/lol-champions/src/darius.rs`:
   - In `DariusModule::create_instance`, add a registration check for `Electrocute`:
     ```rust
     } else if keystone_name == "Electrocute" {
         state.rune_manager.add_effect(Box::new(lol_core::rune_manager::Electrocute::new()));
     ```
