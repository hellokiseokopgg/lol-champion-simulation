# Design Analysis: Core Engine Changes for Electrocute and Press the Attack (Milestone 1)

This report details the design analysis and proposed code changes for supporting the **Electrocute** and **Press the Attack** runes in the `lol-core` simulation engine.

---

## 1. Types Module: Adding Rune Ability Slots
**File**: `crates/lol-core/src/types.rs`

To identify damage coming from Electrocute and Press the Attack, we must add new enum variants to `AbilitySlot`.

### Proposed Changes (Lines 39-41)

#### Before:
```rust
    /// Active item (stores the item ID)
    Item(u32),
}
```

#### After:
```rust
    /// Active item (stores the item ID)
    Item(u32),
    /// Electrocute rune damage source
    Electrocute,
    /// Press the Attack rune damage source
    PressTheAttack,
}
```

---

## 2. Damage Pipeline: Negative Damage Reduction (Amplify Damage)
**File**: `crates/lol-core/src/damage.rs`

Press the Attack amplifies all damage dealt to the target by a percentage (e.g. 8%) when active. This can be represented by a negative `damage_reduction_percent` value in the target's current stats (e.g. `-0.08`).

Under the current implementation, any `damage_reduction_percent <= 0.0` is ignored. We must change the check to `!= 0.0` to permit damage amplification.

### Proposed Changes (Lines 127-129)

#### Before:
```rust
        // Apply damage reduction
        let mut final_damage = mitigated_damage;
        if defender_stats.damage_reduction_percent > 0.0 {
            final_damage *= (1.0 - defender_stats.damage_reduction_percent);
        }
```

#### After:
```rust
        // Apply damage reduction
        let mut final_damage = mitigated_damage;
        if defender_stats.damage_reduction_percent != 0.0 {
            final_damage *= (1.0 - defender_stats.damage_reduction_percent);
        }
```

#### Rationale & Verification:
- If `damage_reduction_percent` is `-0.08` (8% amplification), `final_damage *= (1.0 - (-0.08))` becomes `final_damage *= 1.08`, which correctly amplifies the final damage.
- True damage is handled at the beginning of `DamagePipeline::process` (line 97) and returns early, meaning True damage will correctly ignore this amplification.

---

## 3. Rune Manager: Define RuneEvent::DamageDealt
**File**: `crates/lol-core/src/rune_manager.rs`

Runes like Electrocute and Press the Attack need to deal damage upon activation. Currently, the `RuneEvent` enum only supports stack changes and healing. We add `RuneEvent::DamageDealt` to carry the raw damage, type, and source slot.

### Proposed Changes (Lines 5-8)

#### Before:
```rust
pub enum RuneEvent {
    StacksChanged { name: String, stacks: u32 },
    Healed { amount: f64 },
}
```

#### After:
```rust
pub enum RuneEvent {
    StacksChanged { name: String, stacks: u32 },
    Healed { amount: f64 },
    DamageDealt {
        amount: f64,
        damage_type: crate::types::DamageType,
        slot: crate::types::AbilitySlot,
    },
}
```

#### Note on `RuneEffect` Trait Methods:
The `RuneEffect` trait methods (specifically `on_damage_dealt` and `on_tick`) return `Vec<RuneEvent>`. By adding `RuneEvent::DamageDealt` to the enum, **no signature changes are required** for the `RuneEffect` trait, minimizing friction for rune implementations.

---

## 4. Event Processing: Target-Finding & Damage Resolution
**File**: `crates/lol-core/src/event.rs`

We need to resolve `RuneEvent::DamageDealt` events inside the `trigger_on_damage_dealt` method of `SimContext`. This requires:
1. Identifying the target champion (opponent in the 1v1 context).
2. Fetching current stats for both champions.
3. Calculating mitigated damage via `DamagePipeline::process`.
4. Registering/recording the damage.
5. Applying the damage and triggering `DeathEvent` if necessary.

### Proposed Changes (Lines 93-134)

#### Before:
```rust
    pub fn trigger_on_damage_dealt(&mut self, actor: &crate::types::ChampionId, amount: f64, is_ability: bool, slot: crate::types::AbilitySlot) {
        let rune_events = if let Some(champ_ref) = self.champions.get(actor) {
            champ_ref.borrow_mut().on_damage_dealt(self.current_time, amount, is_ability, slot)
        } else {
            return;
        };

        for event in rune_events {
            match event {
                crate::rune_manager::RuneEvent::StacksChanged { name, stacks } => {
                    if stacks > 0 {
                        let buff_name = if name == "Conqueror" {
                            format!("정복자 ({}스택)", stacks)
                        } else if name == "Lethal Tempo" {
                            format!("치명적 속도 ({}스택)", stacks)
                        } else {
                            name.clone()
                        };
                        
                        if let Some(recorder) = &self.recorder {
                            recorder.borrow_mut().record_buff_apply(self.current_time, actor.clone(), buff_name);
                        }

                        // Schedule an expiration check event
                        let duration = if name == "Conqueror" { 5.0 } else if name == "Lethal Tempo" { 6.0 } else if name.contains("Phase Rush") { 3.0 } else { 0.0 };
                        if duration > 0.0 {
                            self.new_events.push((
                                duration + 0.001, // Slightly after expiration
                                Box::new(RuneExpireCheckEvent { target: actor.clone() }),
                            ));
                        }
                    }
                }
                crate::rune_manager::RuneEvent::Healed { amount } => {
                    if let Some(champ_ref) = self.champions.get(actor) {
                        champ_ref.borrow_mut().heal(amount);
                    }
                }
            }
        }
    }
```

#### After:
```rust
    pub fn trigger_on_damage_dealt(&mut self, actor: &crate::types::ChampionId, amount: f64, is_ability: bool, slot: crate::types::AbilitySlot) {
        // Target-finding logic: Select the opponent in 1v1 simulation
        let target_id = if let Some(id) = self.champions.keys().find(|&id| id != actor) {
            id.clone()
        } else {
            return;
        };

        let rune_events = if let Some(champ_ref) = self.champions.get(actor) {
            champ_ref.borrow_mut().on_damage_dealt(self.current_time, amount, is_ability, slot)
        } else {
            return;
        };

        for event in rune_events {
            match event {
                crate::rune_manager::RuneEvent::StacksChanged { name, stacks } => {
                    if stacks > 0 {
                        let buff_name = if name == "Conqueror" {
                            format!("정복자 ({}스택)", stacks)
                        } else if name == "Lethal Tempo" {
                            format!("치명적 속도 ({}스택)", stacks)
                        } else {
                            name.clone()
                        };
                        
                        if let Some(recorder) = &self.recorder {
                            recorder.borrow_mut().record_buff_apply(self.current_time, actor.clone(), buff_name);
                        }

                        // Schedule an expiration check event
                        let duration = if name == "Conqueror" { 5.0 } else if name == "Lethal Tempo" { 6.0 } else if name.contains("Phase Rush") { 3.0 } else { 0.0 };
                        if duration > 0.0 {
                            self.new_events.push((
                                duration + 0.001, // Slightly after expiration
                                Box::new(RuneExpireCheckEvent { target: actor.clone() }),
                            ));
                        }
                    }
                }
                crate::rune_manager::RuneEvent::Healed { amount } => {
                    if let Some(champ_ref) = self.champions.get(actor) {
                        champ_ref.borrow_mut().heal(amount);
                    }
                }
                crate::rune_manager::RuneEvent::DamageDealt { amount, damage_type, slot } => {
                    // 1. Fetch attacker and defender current stats
                    let attacker_stats = if let Some(c) = self.champions.get(actor) {
                        c.borrow().state().stats.current.clone()
                    } else {
                        continue;
                    };
                    let defender_stats = if let Some(c) = self.champions.get(&target_id) {
                        c.borrow().state().stats.current.clone()
                    } else {
                        continue;
                    };

                    // 2. Process damage calculation through mitigation pipeline
                    let damage_result = crate::damage::DamagePipeline::process(
                        amount,
                        damage_type,
                        false, // Runes do not critically strike
                        &attacker_stats,
                        &defender_stats,
                    );

                    // 3. Record rune damage to event collector
                    if let Some(recorder) = &self.recorder {
                        recorder.borrow_mut().record_damage(
                            self.current_time,
                            actor.clone(),
                            target_id.clone(),
                            slot,
                            damage_result.final_damage,
                            false,
                        );
                    }

                    // 4. Apply damage directly to target health pool
                    if let Some(champ_ref) = self.champions.get(&target_id) {
                        let is_dead = champ_ref.borrow_mut().take_damage(damage_result.final_damage).is_dead;
                        if is_dead {
                            self.new_events.push((0.0, Box::new(DeathEvent { target: target_id.clone() })));
                        }
                    }
                }
            }
        }
    }
```

---

## 5. Additional UX Improvement: Interactive Report Icon Binding
**File**: `crates/lol-report/src/report_template.html`

In the web timeline visualization report, icons are fetched from OP.GG's CDN using name matching. The debug formatting for `AbilitySlot::PressTheAttack` is `"PressTheAttack"`, but the equipped rune name is `"Press the Attack"`. To ensure the Press the Attack icon resolves correctly, we should add a quick mapping case to Javascript's `getIconData`.

### Proposed Changes (Near Line 847)

#### Before:
```javascript
                if (rune.name === "Lethal Tempo" && name.includes("치명적 속도")) {
                    return { type: 'rune', id: rune.id };
                }
```

#### After:
```javascript
                if (rune.name === "Lethal Tempo" && name.includes("치명적 속도")) {
                    return { type: 'rune', id: rune.id };
                }
                if (rune.name === "Press the Attack" && name.includes("PressTheAttack")) {
                    return { type: 'rune', id: rune.id };
                }
```
*(No change is needed for Electrocute, as `"Electrocute".includes("Electrocute")` naturally evaluates to true).*
