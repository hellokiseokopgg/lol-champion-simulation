# Handoff Report — Electrocute and Press the Attack Coverage Analysis and Adversarial Testing

## 1. Observation

During our analysis of the codebase, we inspected:
- `crates/lol-core/src/rune_manager.rs` for `Electrocute` and `PressTheAttack` implementations.
- `crates/lol-core/src/damage.rs` for resistance calculation and the mitigation pipeline.
- `crates/lol-core/src/event.rs` for simulated context, `trigger_on_damage_dealt`, and rune tick tracking.
- `tests/challenger_empirical.rs` for existing empirical tests.

Key observed details from `crates/lol-core/src/rune_manager.rs`:
- **Electrocute Cooldown Scaling Formula (line 430)**:
  `let cooldown = 25.0 - (25.0 - 20.0) / 17.0 * (level.saturating_sub(1) as f64);`
- **Press the Attack (PTA) Stack Decay Logic (line 647)**:
  `if self.stacks > 0 && time.as_f64() - self.last_attack_time > 4.0 { self.stacks = 0; }`
- **Press the Attack (PTA) Event Structure (lines 658-681)**:
  ```rust
  let base_damage = 40.0 + (180.0 - 40.0) / 17.0 * (level.saturating_sub(1) as f64);
  let bonus_ad = (attacker_stats.attack_damage - self.base_ad).max(0.0);
  let ap = attacker_stats.ability_power;
  let damage_type = if bonus_ad > ap {
      crate::types::DamageType::Physical
  } else {
      crate::types::DamageType::Magic
  };
  return vec![
      RuneEvent::StacksChanged { name: "Press the Attack".to_string(), stacks: 3 },
      RuneEvent::ApplyDebuff { name: "Press the Attack Exposure".to_string(), duration: 6.0, damage_reduction_percent: -0.08 },
      RuneEvent::DamageDealt { amount: base_damage, damage_type, slot: crate::types::AbilitySlot::PressTheAttack },
  ];
  ```
- **Press the Attack (PTA) Struct (line 593)**:
  `pub is_melee: bool` is defined but never accessed or used in the `on_damage_dealt` or `on_tick` methods.

We ran all workspace tests via:
`cargo test --workspace`
And all 17 tests in `tests/challenger_empirical.rs` passed successfully.

---

## 2. Logic Chain

From the observations, we deduced:
1. **Simultaneous Hits (Electrocute)**: While the time check cleans up old hits, three distinct hits applied at the *exact same simulation time* (e.g. comboing instantly) will correctly trigger Electrocute since `recent_hits.len() >= 3` will evaluate to true. We verified this in `test_electrocute_simultaneous_hits`.
2. **Negative Cooldown Vulnerability (Electrocute)**: In the cooldown formula, passing a high level (e.g., `level = 100`) leads to a negative cooldown value (`-4.117s`). Since `time - last_proc_time < cooldown` checks if the elapsed time is less than the cooldown, and any positive elapsed time (e.g., `0.02s`) is greater than a negative cooldown, the cooldown check is bypassed entirely. This allows Electrocute to trigger repeatedly without cooldown. We verified this in `test_electrocute_negative_and_zero_cooldown_adversarial`.
3. **Decay refresh behavior (PTA)**: PTA decay is strictly checked against `last_attack_time`, which is only updated on AutoAttacks. Consequently, dealing damage with abilities (e.g., `AbilitySlot::Q`) inside the 4-second window does not refresh the decay timer. An AutoAttack occurring 4.1s after the first AutoAttack will result in a stack decay, even if an ability hit occurred at 3.0s. We verified this in `test_pta_ability_no_reset_and_decay`.
4. **Adaptive Magic Fallback (PTA)**: When `bonus_ad` and `ap` are both exactly `0.0` (such as at level 1 with zero bonus AD and zero AP), the comparison `bonus_ad > ap` evaluates to false, causing the adaptive damage type of PTA to fall back to `Magic`. We verified this in `test_pta_zero_stat_magic_fallback`.
5. **Redundant Field (`is_melee` in PTA)**: The field `is_melee` is defined but unused, meaning melee and ranged champions share the exact same damage amplification (8%) and exposure duration (6s). We verified this in `test_pta_melee_vs_ranged_duration_and_amplification`.

---

## 3. Caveats

- We assumed that `level = 100` represents an adversarial limit since standard game level limits cap at 18.
- We did not modify any production source code in `crates/lol-core/src/`, as per the review-only constraints.
- We did not test interactions between different targets and multiple PTA users, since the simulation engine only simulates 1v1 engagements.

---

## 4. Conclusion

The adversarial tests successfully validated:
- Electrocute triggers correctly on simultaneous hits.
- Electrocute cooldown can become negative under extreme levels (level > 86) leading to zero-cooldown spamming.
- PTA decay is unaffected by ability hits.
- PTA defaults to Magic damage when there is no bonus AD or AP.
- PTA does not currently distinguish between melee and ranged for exposure duration or amplification.

---

## 5. Verification Method

To run the verification tests locally:
```bash
cargo test --test challenger_empirical
```
Inspect files:
- `tests/challenger_empirical.rs` (lines 880-1058)

---

## Attack Surface

### Hypotheses Tested
- **Hypothesis 1**: Multiple distinct abilities landing at the exact same simulation time triggers Electrocute. (Confirmed: `test_electrocute_simultaneous_hits` passes)
- **Hypothesis 2**: Extreme levels cause the Electrocute cooldown formula to return negative values, leading to infinite trigger loops. (Confirmed: `test_electrocute_negative_and_zero_cooldown_adversarial` passes)
- **Hypothesis 3**: PTA stack decay timer does not get refreshed by non-AutoAttack abilities. (Confirmed: `test_pta_ability_no_reset_and_decay` passes)
- **Hypothesis 4**: Zero bonus AD and AP falls back to Magic damage. (Confirmed: `test_pta_zero_stat_magic_fallback` passes)
- **Hypothesis 5**: The `is_melee` field is redundant and does not change PTA's output. (Confirmed: `test_pta_melee_vs_ranged_duration_and_amplification` passes)

### Vulnerabilities Found
- **Unbounded Cooldown Scaling (Electrocute)**: Level inputs above 86 yield a negative cooldown, bypassing the cooldown check entirely.
- **Redundant State (PTA)**: `is_melee` parameter has no effect on PTA performance.

### Untested Angles
- Cooldown reduction effects from items or runes (e.g. Cosmic Insight / Ingenious Hunter) since they are not implemented for rune cooldowns.
