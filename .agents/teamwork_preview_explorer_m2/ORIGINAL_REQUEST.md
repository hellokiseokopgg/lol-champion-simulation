## 2026-06-17T07:13:28Z

You are the Electrocute Explorer.
Your working directory is `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_explorer_m2`.
Your identity: type `teamwork_preview_explorer`, milestone `m2`.
Your task is to analyze the codebase and design the Electrocute rune implementation and the Garen AutoAttack bug fix.

Specifically:
1. Examine `crates/lol-core/src/rune_manager.rs`:
   - Design the `Electrocute` struct and its implementation of `RuneEffect`.
   - Since Electrocute requires the attacker's stats and level in `on_damage_dealt` to calculate adaptive damage scaling and level-based cooldown/damage, evaluate changing the `on_damage_dealt` signature in the `RuneEffect` trait to:
     `fn on_damage_dealt(&mut self, time: SimTime, amount: f64, is_ability: bool, slot: crate::types::AbilitySlot, attacker_stats: &StatBlock, level: u32) -> Vec<RuneEvent>;`
   - If you do this, check how `Conqueror`, `LethalTempo`, `TasteOfBlood`, and `PhaseRush` need to be updated.
2. Examine `crates/lol-core/src/champion.rs`:
   - Design how `ChampionInstance::on_damage_dealt` should invoke the updated `RuneManager::on_damage_dealt` signature, passing the champion's current stats and level.
3. Examine `crates/lol-champions/src/garen.rs`:
   - Find the Garen AutoAttack trigger slot bug (which mistakenly passes `AbilitySlot::E` to `trigger_on_damage_dealt` instead of `AbilitySlot::AutoAttack` or `slot_to_record`) and design the fix.
   - Design how Garen's `create_instance` should register `Electrocute` when equipped.
4. Examine `crates/lol-champions/src/darius.rs`:
   - Design how Darius's `create_instance` should register `Electrocute` when equipped.

Write your proposed changes and findings to `/Users/kskim/Projects/lol-champion-simulation/.agents/teamwork_preview_explorer_m2/analysis.md`. When complete, send a message to the Implementation Track Orchestrator (conversation ID: c3132716-5247-4b0c-b685-fa8da033089a) with a summary.
