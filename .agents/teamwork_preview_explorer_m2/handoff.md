# Handoff Report — Electrocute & Garen AutoAttack Slot Fixes

## 1. Observation

- **Observation 1**: `RuneEffect`'s current `on_damage_dealt` signature in `crates/lol-core/src/rune_manager.rs:26`:
  ```rust
  fn on_damage_dealt(&mut self, time: SimTime, amount: f64, is_ability: bool, slot: crate::types::AbilitySlot) -> Vec<RuneEvent>;
  ```
- **Observation 2**: Existing dynamic runes in `crates/lol-core/src/rune_manager.rs` include `Conqueror` (line 96), `LethalTempo` (line 182), `TasteOfBlood` (line 249), and `PhaseRush` (line 286).
- **Observation 3**: `ChampionInstance::on_damage_dealt` in `crates/lol-core/src/champion.rs:108`:
  ```rust
  fn on_damage_dealt(&mut self, time: crate::types::SimTime, amount: f64, is_ability: bool, slot: crate::types::AbilitySlot) -> Vec<crate::rune_manager::RuneEvent> {
      let state = self.state_mut();
      state.rune_manager.on_damage_dealt(time, amount, is_ability, slot)
  }
  ```
- **Observation 4**: Garen's basic attack execution in `crates/lol-champions/src/garen.rs:373`:
  ```rust
  ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, is_ability, lol_core::types::AbilitySlot::E);
  ```
- **Observation 5**: Garen's spin execution in `crates/lol-champions/src/garen.rs:440`:
  ```rust
  ctx.trigger_on_damage_dealt(&self.attacker, damage_result.final_damage, true, lol_core::types::AbilitySlot::Q);
  ```
- **Observation 6**: Darius's basic attack execution in `crates/lol-champions/src/darius.rs:691`:
  ```rust
  ctx.trigger_on_damage_dealt(
      actor,
      damage_result.final_damage,
      is_ability,
      lol_core::types::AbilitySlot::Passive,
  );
  ```
- **Observation 7**: `cargo test --workspace` fails due to un-implemented `Electrocute` and `Press the Attack` tests:
  ```
  failures:
      test_electrocute_activation_garen
      test_pta_activation_garen
      test_pta_consecutive_restriction_garen
      test_pta_reset_out_of_combat_garen
  ```

---

## 2. Logic Chain

1. **Changing `RuneEffect::on_damage_dealt` signature (from Observation 1)**:
   - To calculate Electrocute's damage and determine its adaptive damage type, we need `attacker_stats` and `level` (as described in the prompt).
   - Changing the signature to `fn on_damage_dealt(..., attacker_stats: &StatBlock, level: u32)` satisfies this requirement.
   - All four dynamic runes in `rune_manager.rs` (from Observation 2) must match this trait signature.
2. **Obtaining `bonus_ad` inside the rune**:
   - `StatBlock` holds total stats and does not split base and bonus AD.
   - Runes receive `base_stats` inside `get_bonus_stats`. By storing `base_stats.attack_damage` in a struct field, runes can dynamically compute `bonus_ad = (current_ad - base_ad).max(0.0)` during `on_damage_dealt`.
3. **Updating `ChampionInstance::on_damage_dealt` (from Observation 3)**:
   - To query `RuneManager` with the new signature, we must pull `state.stats.current` and `state.level`. Cloning `current` avoids compiler-borrow conflicts while querying `state.rune_manager`.
4. **Fixing Garen AutoAttack Slot Bug (from Observation 4 & 5)**:
   - Observation 4 shows Garen's basic attacks trigger damage dealt using `AbilitySlot::E` (incorrect). Replacing this with `slot_to_record` corrects it.
   - Observation 5 shows Garen's E ticks trigger damage dealt using `AbilitySlot::Q` (incorrect). Replacing this with `AbilitySlot::E` corrects it.

---

## 3. Caveats

- **Press the Attack**: This investigation is focused solely on `Electrocute` and Garen's bugs, so `Press the Attack` has not been designed or investigated in detail here (although it is mentioned in the test failures).
- **Melee/Ranged Adaptations**: While `Electrocute` in standard League of Legends does not differ between melee and ranged, if future requirements dictate differences, it can easily adapt using existing framework checks.

---

## 4. Conclusion

- **Rune Signature**: Modifying `RuneEffect::on_damage_dealt` signature is feasible and robust.
- **Electrocute design**: The designed structure implements `RuneEffect`, caches `base_ad`, verifies 3 unique hits within 3s using slot uniqueness and timestamp check for auto-attacks, and fires adaptive damage using the `RuneEvent::DamageDealt` variant.
- **Bug Fix**: Garen's basic attacks should use `slot_to_record` on damage triggers, and Garen's E ticks should use `AbilitySlot::E`.
- **Rune Page Registration**: Both Garen and Darius modules should add `keystone_name == "Electrocute"` checks in their respective `create_instance` methods to add `Electrocute` to their `RuneManager`.

---

## 5. Verification Method

1. **Compilation**: Run `cargo check` to verify no compiler errors.
2. **Tier 1 Feature Tests**: Run `cargo test --test tier1_feature` to verify that `test_electrocute_activation_garen`, `test_electrocute_cooldown_garen`, `test_electrocute_missing_hit_garen`, `test_electrocute_slow_hits_garen`, and `test_electrocute_damage_scaling_garen` all pass after implementing the changes.
