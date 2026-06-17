# Handoff Report — Core Engine Design Analysis for Milestone 1

## 1. Observation
We observed the following files and definitions in the workspace:

- **`crates/lol-core/src/types.rs`**:
  `AbilitySlot` is defined on lines 26–41 as:
  ```rust
  pub enum AbilitySlot {
      Q,
      W,
      E,
      R,
      Passive,
      AutoAttack,
      Item(u32),
  }
  ```
  No exhaustive pattern matching on `AbilitySlot` was found in the workspace.

- **`crates/lol-core/src/damage.rs`**:
  Damage reduction is processed on lines 125–129 as:
  ```rust
          // Apply damage reduction
          let mut final_damage = mitigated_damage;
          if defender_stats.damage_reduction_percent > 0.0 {
              final_damage *= (1.0 - defender_stats.damage_reduction_percent);
          }
  ```

- **`crates/lol-core/src/rune_manager.rs`**:
  `RuneEvent` is defined on lines 5–8 as:
  ```rust
  pub enum RuneEvent {
      StacksChanged { name: String, stacks: u32 },
      Healed { amount: f64 },
  }
  ```
  And `RuneEffect::on_damage_dealt` (line 21) returns `Vec<RuneEvent>`.

- **`crates/lol-core/src/event.rs`**:
  `trigger_on_damage_dealt` is defined on lines 93–134 and processes stack changes and healing events, but does not currently handle any damage-dealing rune events.

- **`crates/lol-report/src/report_template.html`**:
  In `getIconData` (lines 836–849), equipped runes are matched with:
  ```javascript
          if (typeof runesData !== 'undefined' && runesData[champ]) {
              for (const rune of runesData[champ]) {
                  if (name.includes(rune.name)) {
                      return { type: 'rune', id: rune.id };
                  }
                  // ...
  ```

---

## 2. Logic Chain
1. **Adding Ability Slots**: Since `AbilitySlot` does not have any exhaustive match statements, adding `AbilitySlot::Electrocute` and `AbilitySlot::PressTheAttack` is completely safe and won't trigger compilation errors elsewhere in the workspace.
2. **Supporting Damage Amplification**: Changing `defender_stats.damage_reduction_percent > 0.0` to `!= 0.0` in `damage.rs` allows negative percentages. E.g., if a target has `-0.08` (8% amplification from Press the Attack), the multiplier `(1.0 - (-0.08))` becomes `1.08`, amplifying the final damage.
3. **Defining RuneEvent::DamageDealt**: Adding a `RuneEvent::DamageDealt { amount, damage_type, slot }` variant to the `RuneEvent` enum allows any rune to emit damage requests directly during its lifecycle (`on_damage_dealt` or `on_tick`) without requiring trait method signature alterations.
4. **Target-Finding and Processing**: In a 1v1 simulation, the opponent of `actor` is simply the other key in `self.champions` (retrieved via `self.champions.keys().find(|&id| id != actor)`). In `trigger_on_damage_dealt`, this `target_id` is used to:
   - Retrieve defender stats and attacker stats.
   - Run `DamagePipeline::process` to compute mitigated damage.
   - Record the final damage using `recorder.record_damage`.
   - Apply health reduction using `take_damage`.
   - Fire a `DeathEvent` if the target's health is depleted.
5. **Interactive Timeline Icon Resolution**: Since `PressTheAttack` (enum debug representation) contains no spaces and does not match the rune name `"Press the Attack"`, we must add an explicit check `rune.name === "Press the Attack" && name.includes("PressTheAttack")` in `report_template.html` to guarantee the Press the Attack icon loads properly.

---

## 3. Caveats
- We assume that rune damage itself does not trigger recursive on-damage-dealt events, which is standard in League of Legends (avoiding infinite recursion). Calling `take_damage` directly instead of re-routing through `trigger_on_damage_dealt` enforces this constraint.
- The 1v1 target-finding assumption `id != actor` holds true under the current simulator architecture (where only two champion instances exist). If 2v2 or team-fights are added in the future, this target-finding logic would need to be updated.

---

## 4. Conclusion
The proposed changes represent a minimal-impact, highly modular design that fully satisfies the requirements of Milestone 1. They enable damage amplification, allow runes to emit damage events, and process those events cleanly through the existing damage pipeline.

---

## 5. Verification Method
1. **Compiler Check**: After applying the proposed changes, run:
   ```bash
   cargo check --workspace
   ```
   Verify that the workspace compiles with no errors.
2. **Unit Tests**: Run:
   ```bash
   cargo test --workspace
   ```
   Ensure all existing tests pass.
3. **Interactive Timeline Verification**: Generate a report using the dummy champions or other champions:
   ```bash
   cargo run -- simulate -a Garen -b Darius --html-out test_report.html
   ```
   Open `test_report.html` in a web browser and confirm the console contains no errors and icons resolve correctly.
