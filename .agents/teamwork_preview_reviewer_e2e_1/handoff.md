# E2E Verification Handoff Report

## 1. Observation

### Command Outputs
* `cargo test --workspace` ran successfully and all 58 tests passed. Verbatim output:
```
     Running tests/tier1_feature.rs (target/debug/deps/tier1_feature-ef5ed3b9b1b18e26)
running 10 tests
...
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
...
     Running unittests src/lib.rs (target/debug/deps/lol_core-f47acb3569c0b1f6)
...
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
...
     Running unittests src/lib.rs (target/debug/deps/lol_data-6c5e88e90e9c9f85)
...
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

* `cargo clippy --workspace --all-targets` completed but generated **20 warnings** related to manual strip prefix warnings in `lol-apl` and `lol-report`. Verbatim:
```
warning: stripping a prefix manually
   --> crates/lol-apl/src/expression.rs:104:27
    |
104 |             let val_str = &input["target.health.pct<".len()..];
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
warning: `lol-apl` (lib) generated 14 warnings (run `cargo clippy --fix --lib -p lol-apl -- ` to apply 6 suggestions)
warning: `lol-report` (lib) generated 6 warnings
```

### Code Observations
* In `crates/lol-champions/src/darius.rs`, lines 688–693 (Darius basic attack damage trigger):
```rust
        let is_ability = has_w_buff;
        ctx.trigger_on_damage_dealt(
            actor,
            damage_result.final_damage,
            is_ability,
            lol_core::types::AbilitySlot::Passive,
        );
```
* In `crates/lol-champions/src/darius.rs`, lines 232–237 (Darius hemorrhage passive tick event):
```rust
        ctx.trigger_on_damage_dealt(
            &self.attacker,
            damage_result.final_damage,
            false,
            lol_core::types::AbilitySlot::AutoAttack,
        );
```
* In `crates/lol-core/src/item.rs`, lines 276–277 (Stridebreaker active execution):
```rust
            let attacker_ref = ctx.champions.get(actor).unwrap().borrow();
            let defender_ref = ctx.champions.get(target).unwrap().borrow();
```

---

## 2. Logic Chain

1. **Darius Combat Logic Mismatch**: 
   - Press the Attack (PTA) stacks and triggers on basic attacks (specifically checking `slot == AbilitySlot::AutoAttack && !is_ability` in `rune_manager.rs`).
   - However, in `darius.rs`, auto attacks trigger with slot `AbilitySlot::Passive` (Observation), which fails the PTA check.
   - Meanwhile, Hemorrhage (passive) DoT ticks trigger with slot `AbilitySlot::AutoAttack` and `is_ability = false` (Observation), which incorrectly increments PTA stacks on every passive DoT tick.
   - This results in Press the Attack triggering on DoT ticks rather than actual basic attacks, which directly violates League of Legends mechanics and the engine's design.

2. **Convention Violation (unwrap)**:
   - `AGENTS.md` states: `"unwrap() 금지 — Result/Option 적절히 처리"`.
   - In `item.rs`, `StridebreakerActive::execute` uses `.unwrap()` on the Option returned by `ctx.champions.get()` (Observation).
   - If either the actor or target champion is missing from the context during a Stridebreaker cast, the simulation will panic instead of returning an error or handling it gracefully.

3. **Convention Violation (clippy)**:
   - `AGENTS.md` states: `"clippy 경고 0개 유지"`.
   - Running `cargo clippy --workspace --all-targets` generates 20 warnings (Observation), which violates this convention.

---

## 3. Caveats

- We assumed that `AbilitySlot::AutoAttack` is the only slot intended to trigger PTA. This is confirmed by `rune_manager.rs` checking `slot == crate::types::AbilitySlot::AutoAttack`.
- We assumed target dummy configurations in unit/integration tests do not rely on Stridebreaker's panic behavior to pass (no tests fail when executing).
- The rest of the codebase was only reviewed statically for the specified files. Gaps may exist in other unreviewed champion modules or items.

---

## 4. Conclusion

The E2E test suite passes, but the implementation contains a **critical combat mechanics bug** in Darius's slot mapping, a **convention violation** via `unwrap()` in `item.rs`, and **clippy warnings** across the workspace.

**Verdict**: **REQUEST_CHANGES**

---

## Quality Review Report

### Review Summary
**Verdict**: REQUEST_CHANGES

### Findings

#### [Critical] Finding 1: Swapped Slots in Darius Auto Attack & Hemorrhage
- **What**: Darius basic attacks trigger `trigger_on_damage_dealt` using the `Passive` slot, while Hemorrhage DoT ticks trigger it using `AutoAttack` with `is_ability = false`.
- **Where**: `crates/lol-champions/src/darius.rs`, lines 688–693 and lines 232–237.
- **Why**: This causes Press the Attack to stack and trigger off Hemorrhage DoT ticks rather than Darius's basic attacks, violating League of Legends combat rules.
- **Suggestion**: Change the slot in `DariusAutoAttack::execute` to `AbilitySlot::AutoAttack` (or `AbilitySlot::W` if `has_w_buff` is true), and change the slot in `HemorrhageTickEvent::execute` to `AbilitySlot::Passive`.

#### [Major] Finding 2: Forbidden use of `unwrap()` in Stridebreaker implementation
- **What**: Stridebreaker active uses `.unwrap()` to fetch actor and target champions from the simulation context.
- **Where**: `crates/lol-core/src/item.rs`, lines 276–277.
- **Why**: Violates the coding convention `"unwrap() 금지"`. It could lead to a crash if actor or target champion is not found in the context.
- **Suggestion**: Use `if let (Some(a), Some(d)) = ...` or `?` to handle the options gracefully.

#### [Major] Finding 3: Clippy warnings in Workspace
- **What**: Running clippy generates 20 warnings.
- **Where**: `crates/lol-apl/src/expression.rs`, `crates/lol-apl/src/parser.rs`, and `lol-report` crate.
- **Why**: Violates the coding convention `"clippy 경고 0개 유지"`.
- **Suggestion**: Refactor code in those files to use `strip_prefix` instead of manually stripping prefixes.

### Verified Claims
- unit and E2E tests pass → verified via `cargo test --workspace` → **PASS**

### Coverage Gaps
- **Stridebreaker calls** — risk level: Low — recommendation: Accept risk (or replace unwrap with safe handlers).

---

## Adversarial Challenge Report

### Challenge Summary
**Overall risk assessment**: HIGH

### Challenges

#### [Critical] Challenge 1: PTA activation on Darius Hemorrhage DoT ticks
- **Assumption challenged**: That Press the Attack only triggers on basic attacks.
- **Attack scenario**: Darius hits an enemy with a basic attack to apply Hemorrhage. The DoT ticks 3 times without Darius landing any further basic attacks.
- **Blast radius**: PTA triggers automatically from the DoT ticks, amplifying the target's damage taken by 8% and dealing PTA base damage without Darius committing basic attacks.
- **Mitigation**: Correct slot mappings in `darius.rs` so that only `AbilitySlot::AutoAttack` invokes the basic attack trigger.

#### [High] Challenge 2: Panic on missing champion in Stridebreaker active
- **Assumption challenged**: Champions always exist in `ctx.champions`.
- **Attack scenario**: An active item is cast from a champion that has died and been removed, or is targetable but not present in `ctx.champions` due to a custom scenario.
- **Blast radius**: Entire simulation crashes with a panic.
- **Mitigation**: Replace `unwrap()` with safe conditional checks.

### Stress Test Results
- Run simulation with Darius APL doing only a single auto attack and checking if PTA triggers on subsequent Hemorrhage ticks → **FAIL** (PTA triggers on ticks).

---

## 5. Verification Method

To verify these issues independently:
1. Run `cargo clippy --workspace --all-targets` to view the 20 clippy warnings.
2. Run manual simulation with Darius and Press the Attack rune to observe PTA triggers on Hemorrhage ticks:
   `cargo run -- simulate -a Darius -b Dummy --runes press_the_attack --apl <(echo -e "runes=press_the_attack\nactions+=/AA\n")`
   Check that `Effect Press the Attack` and `Dmg PressTheAttack` trigger without Darius executing 3 auto attacks (only a single AA is executed, and Hemorrhage ticks trigger it).
3. Inspect `crates/lol-core/src/item.rs` lines 276–277 for `unwrap()` calls.
