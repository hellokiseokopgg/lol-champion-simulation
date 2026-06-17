# Electrocute & Garen AutoAttack Design Analysis

This report outlines the proposed changes for implementing the **Electrocute** rune and fixing the **Garen AutoAttack trigger slot bug**.

---

## 1. Summary of Findings & Core Decisions

1. **RuneEffect Trait Signature Modification**: 
   Changing the signature of `RuneEffect::on_damage_dealt` is highly effective. It allows runes like `Electrocute` and `Taste of Blood` to access the attacker's `current` stats (from the `ThreeLayerStats` system) and `level` directly.
   
2. **Adaptive Damage Type & Scaling**:
   Electrocute requires the calculation of "bonus AD" to determine its damage and adaptive damage type. Since `StatBlock` does not split base and bonus AD, the rune will cache the champion's level-based `base_stats` during the `get_bonus_stats` phase. This allows clean, memory-safe derivation of `bonus_ad = (current_ad - base_ad).max(0.0)` in `on_damage_dealt`.

3. **Garen AutoAttack Trigger Slot Bug**:
   In `GarenAutoAttack::execute`, the slot registered to the rune manager is hardcoded to `AbilitySlot::E` rather than `slot_to_record` (which handles Q-empowered and normal AutoAttacks). We also discovered that `JudgmentTickEvent::execute` (Garen's E) mistakenly passes `AbilitySlot::Q` to the rune manager, which we propose fixing.

---

## 2. Detailed Design Proposals

### 2.1. Trait and Manager Updates (`crates/lol-core/src/rune_manager.rs`)

#### Trait & Signature Changes
We update the `RuneEffect` trait and `RuneManager` dispatcher signatures to accept `attacker_stats: &StatBlock` and `level: u32`.

```rust
// Proposed trait update in crates/lol-core/src/rune_manager.rs
pub trait RuneEffect: Debug {
    fn name(&self) -> &str;
    
    fn get_bonus_stats(&mut self, time: SimTime, base_stats: &StatBlock, level: u32) -> StatBlock;
    
    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        amount: f64,
        is_ability: bool,
        slot: crate::types::AbilitySlot,
        attacker_stats: &StatBlock,
        level: u32,
    ) -> Vec<RuneEvent>;

    fn on_tick(&mut self, time: SimTime) -> Vec<RuneEvent> {
        Vec::new()
    }
}
```

```rust
// Proposed manager update in crates/lol-core/src/rune_manager.rs
impl RuneManager {
    pub fn on_damage_dealt(
        &mut self,
        time: SimTime,
        amount: f64,
        is_ability: bool,
        slot: crate::types::AbilitySlot,
        attacker_stats: &StatBlock,
        level: u32,
    ) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        for effect in &mut self.effects {
            events.extend(effect.on_damage_dealt(time, amount, is_ability, slot, attacker_stats, level));
        }
        events
    }
}
```

#### Updating Existing Runes
- **Conqueror**, **Lethal Tempo**, and **Phase Rush** will update their `on_damage_dealt` signatures to accept the new arguments, ignoring them with leading underscores (e.g. `_attacker_stats`, `_level`). Their internal logic remains unchanged.
- **Taste of Blood** will be upgraded to use the actual LoL healing formula:
  - **Formula**: `16 - 40 (based on level) (+ 10% bonus AD) (+ 5% AP)` with a 20-second cooldown.
  - **State**: Needs fields `base_ad: f64` and `last_proc_time: f64`.

```rust
#[derive(Debug)]
pub struct TasteOfBlood {
    base_ad: f64,
    last_proc_time: f64,
}

impl TasteOfBlood {
    pub fn new() -> Self {
        Self {
            base_ad: 0.0,
            last_proc_time: -999.0,
        }
    }
}

impl RuneEffect for TasteOfBlood {
    fn name(&self) -> &str { "Taste of Blood" }

    fn get_bonus_stats(&mut self, _time: SimTime, base_stats: &StatBlock, _level: u32) -> StatBlock {
        self.base_ad = base_stats.attack_damage;
        StatBlock::new()
    }

    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        _amount: f64,
        _is_ability: bool,
        _slot: crate::types::AbilitySlot,
        attacker_stats: &StatBlock,
        level: u32,
    ) -> Vec<RuneEvent> {
        let current_time = *time.0;
        if current_time - self.last_proc_time < 20.0 {
            return Vec::new();
        }
        
        self.last_proc_time = current_time;
        let base_heal = 16.0 + (40.0 - 16.0) / 17.0 * (level.saturating_sub(1) as f64);
        let bonus_ad = (attacker_stats.attack_damage - self.base_ad).max(0.0);
        let ap = attacker_stats.ability_power;
        let heal_amount = base_heal + 0.10 * bonus_ad + 0.05 * ap;
        
        vec![RuneEvent::Healed { amount: heal_amount }]
    }
}
```

---

### 2.2. Design of the `Electrocute` Rune (`crates/lol-core/src/rune_manager.rs`)

`Electrocute` tracks unique basic attacks or abilities within a 3-second window, and deals adaptive damage.

```rust
#[derive(Debug)]
pub struct Electrocute {
    recent_hits: std::collections::VecDeque<(f64, crate::types::AbilitySlot)>,
    last_proc_time: f64,
    base_ad: f64,
}

impl Electrocute {
    pub fn new() -> Self {
        Self {
            recent_hits: std::collections::VecDeque::new(),
            last_proc_time: -999.0,
            base_ad: 0.0,
        }
    }
}

impl RuneEffect for Electrocute {
    fn name(&self) -> &str {
        "Electrocute"
    }

    fn get_bonus_stats(&mut self, _time: SimTime, base_stats: &StatBlock, _level: u32) -> StatBlock {
        self.base_ad = base_stats.attack_damage;
        StatBlock::new()
    }

    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        _amount: f64,
        _is_ability: bool,
        slot: crate::types::AbilitySlot,
        attacker_stats: &StatBlock,
        level: u32,
    ) -> Vec<RuneEvent> {
        let current_time = *time.0;

        // 1. Clean up hits older than 3 seconds
        while let Some(&(t, _)) = self.recent_hits.front() {
            if current_time - t > 3.0 {
                self.recent_hits.pop_front();
            } else {
                break;
            }
        }

        // 2. Cooldown check: 25s at lvl 1 down to 20s at lvl 18
        let cooldown = 25.0 - (25.0 - 20.0) / 17.0 * (level.saturating_sub(1) as f64);
        if current_time - self.last_proc_time < cooldown {
            return Vec::new();
        }

        // 3. Unique hit validation
        let should_add = match slot {
            crate::types::AbilitySlot::AutoAttack => {
                let last_aa_time = self.recent_hits.iter()
                    .filter(|&&(_, s)| s == crate::types::AbilitySlot::AutoAttack)
                    .map(|&(t, _)| t)
                    .last();
                match last_aa_time {
                    Some(t) => (current_time - t).abs() > 0.01,
                    None => true,
                }
            }
            _ => {
                // Abilities must use unique slots (e.g. Q, W, E, R)
                !self.recent_hits.iter().any(|&(_, s)| s == slot)
            }
        };

        if should_add {
            self.recent_hits.push_back((current_time, slot));
        }

        // 4. Trigger Electrocute if 3 unique hits
        if self.recent_hits.len() >= 3 {
            self.last_proc_time = current_time;
            self.recent_hits.clear();

            // Damage scaling: 30 - 180 (based on level) + 40% bonus AD + 25% AP
            let base_damage = 30.0 + (180.0 - 30.0) / 17.0 * (level.saturating_sub(1) as f64);
            let bonus_ad = (attacker_stats.attack_damage - self.base_ad).max(0.0);
            let ap = attacker_stats.ability_power;
            let damage = base_damage + 0.40 * bonus_ad + 0.25 * ap;

            // Adaptive damage type
            let damage_type = if bonus_ad > ap {
                crate::types::DamageType::Physical
            } else {
                crate::types::DamageType::Magic
            };

            return vec![RuneEvent::DamageDealt {
                amount: damage,
                damage_type,
                slot: crate::types::AbilitySlot::Electrocute,
            }];
        }

        Vec::new()
    }
}
```

---

### 2.3. Champion Instance Dispatch Update (`crates/lol-core/src/champion.rs`)

We modify `ChampionInstance::on_damage_dealt` to retrieve `stats.current` and `level` from the mutable state, cloning/reading them before querying the rune manager.

```rust
// Proposed change in crates/lol-core/src/champion.rs
    fn on_damage_dealt(&mut self, time: crate::types::SimTime, amount: f64, is_ability: bool, slot: crate::types::AbilitySlot) -> Vec<crate::rune_manager::RuneEvent> {
        let state = self.state_mut();
        let level = state.level;
        let stats = state.stats.current.clone(); // Cloned to satisfy borrow checker
        state.rune_manager.on_damage_dealt(time, amount, is_ability, slot, &stats, level)
    }
```

---

### 2.4. Garen Code Fixes & Rune Page Integration (`crates/lol-champions/src/garen.rs`)

#### AutoAttack Trigger Slot Bug Fix
In `GarenAutoAttack::execute`, the hardcoded slot parameter is fixed from `AbilitySlot::E` to `slot_to_record` (so that Q-empowered hits count as `Q` and normal attacks as `AutoAttack`).
In addition, in `JudgmentTickEvent::execute`, the slot parameter is fixed from `AbilitySlot::Q` to `AbilitySlot::E`.

```rust
// In GarenAutoAttack::execute
        // Before:
        // ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, is_ability, lol_core::types::AbilitySlot::E);
        
        // After:
        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, is_ability, slot_to_record);
```

```rust
// In JudgmentTickEvent::execute
        // Before:
        // ctx.trigger_on_damage_dealt(&self.attacker, damage_result.final_damage, true, lol_core::types::AbilitySlot::Q);
        
        // After:
        ctx.trigger_on_damage_dealt(&self.attacker, damage_result.final_damage, true, lol_core::types::AbilitySlot::E);
```

#### Electrocute Registration
In `GarenModule::create_instance`, add a check for the `"Electrocute"` keystone.

```rust
// In GarenModule::create_instance
        let keystone_name = config.rune_page.keystone.name();
        if keystone_name == "Conqueror" {
            state.rune_manager.add_effect(Box::new(lol_core::rune_manager::Conqueror::new(true)));
        } else if keystone_name == "Lethal Tempo" {
            state.rune_manager.add_effect(Box::new(lol_core::rune_manager::LethalTempo::new(true)));
        } else if keystone_name == "Phase Rush" {
            state.rune_manager.add_effect(Box::new(lol_core::rune_manager::PhaseRush::new(true)));
        } else if keystone_name == "Electrocute" {
            state.rune_manager.add_effect(Box::new(lol_core::rune_manager::Electrocute::new()));
        }
```

---

### 2.5. Darius Rune Page Integration (`crates/lol-champions/src/darius.rs`)

Similarly, register `Electrocute` in `DariusModule::create_instance` when equipped.

```rust
// In DariusModule::create_instance
        let keystone_name = config.rune_page.keystone.name();
        if keystone_name == "Conqueror" {
            state.rune_manager.add_effect(Box::new(lol_core::rune_manager::Conqueror::new(true)));
        } else if keystone_name == "Lethal Tempo" {
            state.rune_manager.add_effect(Box::new(lol_core::rune_manager::LethalTempo::new(true)));
        } else if keystone_name == "Phase Rush" {
            state.rune_manager.add_effect(Box::new(lol_core::rune_manager::PhaseRush::new(true)));
        } else if keystone_name == "Electrocute" {
            state.rune_manager.add_effect(Box::new(lol_core::rune_manager::Electrocute::new()));
        }
```

---

## 3. Verification Plan

1. **Compilation Check**:
   Run `cargo check` to verify that there are no syntax or type signature mismatches.
2. **Unit & Integration Tests**:
   Run `cargo test --workspace` to execute the test suite. Focus on tests in `tests/tier1_feature.rs` verifying Electrocute activation, cooldown, scaling, and timing:
   - `test_electrocute_activation_garen`
   - `test_electrocute_cooldown_garen`
   - `test_electrocute_missing_hit_garen`
   - `test_electrocute_slow_hits_garen`
   - `test_electrocute_damage_scaling_garen`
