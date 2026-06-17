use crate::stats::StatBlock;
use crate::types::SimTime;
use std::fmt::Debug;

pub enum RuneEvent {
    StacksChanged {
        name: String,
        stacks: u32,
    },
    Healed {
        amount: f64,
    },
    DamageDealt {
        amount: f64,
        damage_type: crate::types::DamageType,
        slot: crate::types::AbilitySlot,
    },
    ApplyDebuff {
        name: String,
        duration: f64,
        damage_reduction_percent: f64,
    },
}

/// Trait for a dynamic rune effect (e.g. Conqueror, Lethal Tempo).
pub trait RuneEffect: Debug {
    /// Returns the name of the rune.
    fn name(&self) -> &str;

    /// Called when calculating current stats.
    /// The rune should check for expiration (using `time`) and return its bonus stats based on current stacks and level.
    fn get_bonus_stats(&mut self, time: SimTime, base_stats: &StatBlock, level: u32) -> StatBlock;

    /// Called when the champion deals damage to an enemy.
    /// Returns a list of rune events (like stack changes or healing).
    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        amount: f64,
        is_ability: bool,
        slot: crate::types::AbilitySlot,
        attacker_stats: &crate::stats::StatBlock,
        level: u32,
    ) -> Vec<RuneEvent>;

    /// Called periodically to allow runes to emit expiration events.
    fn on_tick(&mut self, _time: SimTime) -> Vec<RuneEvent> {
        Vec::new() // Default empty implementation
    }
}

/// Manages the dynamic rune effects for a champion during simulation.
#[derive(Debug, Default)]
pub struct RuneManager {
    effects: Vec<Box<dyn RuneEffect>>,
}

impl RuneManager {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn add_effect(&mut self, effect: Box<dyn RuneEffect>) {
        self.effects.push(effect);
    }

    /// Aggregates all bonus stats from active rune effects.
    pub fn get_bonus_stats(
        &mut self,
        time: SimTime,
        base_stats: &StatBlock,
        level: u32,
    ) -> StatBlock {
        let mut total = StatBlock::new();
        for effect in &mut self.effects {
            total = total + effect.get_bonus_stats(time, base_stats, level);
        }
        total
    }

    /// Dispatches the damage dealt event to all runes, returning any generated RuneEvents.
    pub fn on_damage_dealt(
        &mut self,
        time: SimTime,
        amount: f64,
        is_ability: bool,
        slot: crate::types::AbilitySlot,
        attacker_stats: &crate::stats::StatBlock,
        level: u32,
    ) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        for effect in &mut self.effects {
            events.extend(effect.on_damage_dealt(
                time,
                amount,
                is_ability,
                slot,
                attacker_stats,
                level,
            ));
        }
        events
    }

    /// Ticks the runes to check for expiration.
    pub fn on_tick(&mut self, time: SimTime) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        for effect in &mut self.effects {
            events.extend(effect.on_tick(time));
        }
        events
    }
}

#[derive(Debug)]
pub struct Conqueror {
    pub stacks: u32,
    pub duration: f64,
    pub last_trigger_time: f64,
    pub is_adaptive_ad: Option<bool>,
    pub is_melee: bool,
    pub last_stack_time_per_slot: std::collections::HashMap<crate::types::AbilitySlot, f64>,
}

impl Conqueror {
    pub fn new(is_melee: bool) -> Self {
        Self {
            stacks: 0,
            duration: 6.0,
            last_trigger_time: -999.0,
            is_adaptive_ad: None,
            is_melee,
            last_stack_time_per_slot: std::collections::HashMap::new(),
        }
    }
}

impl RuneEffect for Conqueror {
    fn name(&self) -> &str {
        "Conqueror"
    }

    fn get_bonus_stats(&mut self, time: SimTime, base_stats: &StatBlock, level: u32) -> StatBlock {
        let mut bonus = StatBlock::new();
        
        if self.is_adaptive_ad.is_none() {
            let total_bonus_ad = base_stats.attack_damage; 
            let total_ap = base_stats.ability_power;
            self.is_adaptive_ad = Some(total_bonus_ad >= total_ap);
        }

        if self.stacks == 0 || time.as_f64() - self.last_trigger_time >= self.duration {
            self.stacks = 0;
            return bonus;
        }

        let ad_per_stack = 1.2 + (2.7 - 1.2) * ((level as f64 - 1.0) / 17.0);
        let ap_per_stack = 2.0 + (4.5 - 2.0) * ((level as f64 - 1.0) / 17.0);

        if self.is_adaptive_ad.unwrap_or(true) {
            bonus.attack_damage = ad_per_stack * (self.stacks as f64);
        } else {
            bonus.ability_power = ap_per_stack * (self.stacks as f64);
        }

        bonus
    }

    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        amount: f64,
        is_ability: bool,
        slot: crate::types::AbilitySlot,
        _attacker_stats: &crate::stats::StatBlock,
        _level: u32,
    ) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        let current_time = time.as_f64();

        // Expire first if needed
        if self.stacks > 0 && current_time - self.last_trigger_time >= self.duration {
            self.stacks = 0;
            events.push(RuneEvent::StacksChanged {
                name: "Conqueror".to_string(),
                stacks: 0,
            });
        }

        // Add stacks
        let mut add_stacks = 0;
        if is_ability {
            // Check for DoT
            let last_stack_time = self.last_stack_time_per_slot.get(&slot).copied().unwrap_or(-999.0);
            if current_time - last_stack_time >= 5.0 {
                add_stacks = 2;
                self.last_stack_time_per_slot.insert(slot, current_time);
            }
        } else {
            add_stacks = if self.is_melee { 2 } else { 1 };
        }

        if add_stacks > 0 || is_ability {
            // basic attacks or abilities refresh duration
            self.last_trigger_time = current_time;
            if add_stacks > 0 {
                self.stacks = std::cmp::min(12, self.stacks + add_stacks);
                events.push(RuneEvent::StacksChanged {
                    name: "Conqueror".to_string(),
                    stacks: self.stacks,
                });
            }
        } else if !is_ability {
            // For ranged auto attacks it adds 1 stack and refreshes. The above handles it.
            self.last_trigger_time = current_time;
        }

        // Heal if full stacks
        if self.stacks == 12 {
            let heal_ratio = if self.is_melee { 0.08 } else { 0.05 };
            events.push(RuneEvent::Healed {
                amount: amount * heal_ratio,
            });
        }

        events
    }

    fn on_tick(&mut self, time: SimTime) -> Vec<RuneEvent> {
        if self.stacks > 0 && time.as_f64() - self.last_trigger_time >= self.duration {
            self.stacks = 0;
            return vec![RuneEvent::StacksChanged {
                name: "Conqueror".to_string(),
                stacks: 0,
            }];
        }
        Vec::new()
    }
}

#[derive(Debug)]
pub struct LethalTempo {
    pub is_melee: bool,
    stacks: u32,
    last_stack_time: f64,
}

impl LethalTempo {
    pub fn new(is_melee: bool) -> Self {
        Self {
            is_melee,
            stacks: 0,
            last_stack_time: -999.0,
        }
    }
}

impl RuneEffect for LethalTempo {
    fn name(&self) -> &str {
        "Lethal Tempo"
    }

    fn get_bonus_stats(&mut self, time: SimTime, base_stats: &StatBlock, level: u32) -> StatBlock {
        // Expiration check
        if *time.0 - self.last_stack_time > 6.0 {
            self.stacks = 0;
        }

        let mut stats = StatBlock::new();
        if self.stacks > 0 {
            // Melee: 5-16% AS per stack
            // Ranged: 4-8% AS per stack
            let as_per_stack = if self.is_melee {
                5.0 + (16.0 - 5.0) / 17.0 * (level.saturating_sub(1) as f64)
            } else {
                4.0 + (8.0 - 4.0) / 17.0 * (level.saturating_sub(1) as f64)
            };
            let as_ratio = base_stats
                .attack_speed_ratio
                .unwrap_or(base_stats.attack_speed);
            stats.attack_speed = as_per_stack * (self.stacks as f64) * as_ratio;
        }
        stats
    }

    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        _amount: f64,
        is_ability: bool,
        _slot: crate::types::AbilitySlot,
        _attacker_stats: &crate::stats::StatBlock,
        _level: u32,
    ) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        // Expiration check
        if *time.0 - self.last_stack_time > 6.0 {
            if self.stacks > 0 {
                events.push(RuneEvent::StacksChanged {
                    name: "Lethal Tempo".to_string(),
                    stacks: 0,
                });
            }
            self.stacks = 0;
        }

        // Only basic attacks grant Lethal Tempo stacks
        if !is_ability {
            let old_stacks = self.stacks;
            self.stacks = (self.stacks + 1).min(6);
            self.last_stack_time = *time.0;

            if self.stacks > old_stacks {
                events.push(RuneEvent::StacksChanged {
                    name: "Lethal Tempo".to_string(),
                    stacks: self.stacks,
                });
            }
        }

        events
    }

    fn on_tick(&mut self, time: SimTime) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        if self.stacks > 0 && *time.0 - self.last_stack_time > 6.0 {
            self.stacks = 0;
            events.push(RuneEvent::StacksChanged {
                name: "Lethal Tempo".to_string(),
                stacks: 0,
            });
        }
        events
    }
}

#[derive(Debug)]
pub struct TasteOfBlood {
    pub base_ad: f64,
    pub last_proc_time: f64,
}

impl Default for TasteOfBlood {
    fn default() -> Self {
        Self {
            base_ad: 0.0,
            last_proc_time: -999.0,
        }
    }
}

impl TasteOfBlood {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RuneEffect for TasteOfBlood {
    fn name(&self) -> &str {
        "Taste of Blood"
    }

    fn get_bonus_stats(
        &mut self,
        _time: SimTime,
        base_stats: &StatBlock,
        _level: u32,
    ) -> StatBlock {
        self.base_ad = base_stats.attack_damage;
        StatBlock::new()
    }

    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        _amount: f64,
        _is_ability: bool,
        _slot: crate::types::AbilitySlot,
        attacker_stats: &crate::stats::StatBlock,
        level: u32,
    ) -> Vec<RuneEvent> {
        if time.as_f64() - self.last_proc_time < 20.0 {
            return Vec::new();
        }
        self.last_proc_time = time.as_f64();

        // Base heal: 16.0 + (40.0 - 16.0) / 17.0 * (level.saturating_sub(1) as f64)
        let base_heal = 16.0 + (40.0 - 16.0) / 17.0 * (level.saturating_sub(1) as f64);
        // Bonus AD: (attacker_stats.attack_damage - self.base_ad).max(0.0)
        let bonus_ad = (attacker_stats.attack_damage - self.base_ad).max(0.0);
        // AP: attacker_stats.ability_power
        let ap = attacker_stats.ability_power;
        // Heal amount: base_heal + 0.10 * bonus_ad + 0.05 * ap
        let heal_amount = base_heal + 0.10 * bonus_ad + 0.05 * ap;

        vec![RuneEvent::Healed {
            amount: heal_amount,
        }]
    }
}

/// The Electrocute rune effect.
#[derive(Debug)]
pub struct Electrocute {
    pub recent_hits: std::collections::VecDeque<(f64, crate::types::AbilitySlot)>,
    pub last_proc_time: f64,
    pub base_ad: f64,
}

impl Default for Electrocute {
    fn default() -> Self {
        Self {
            recent_hits: std::collections::VecDeque::new(),
            last_proc_time: -999.0,
            base_ad: 0.0,
        }
    }
}

impl Electrocute {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RuneEffect for Electrocute {
    fn name(&self) -> &str {
        "Electrocute"
    }

    fn get_bonus_stats(
        &mut self,
        _time: SimTime,
        base_stats: &StatBlock,
        _level: u32,
    ) -> StatBlock {
        self.base_ad = base_stats.attack_damage;
        StatBlock::new()
    }

    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        _amount: f64,
        _is_ability: bool,
        slot: crate::types::AbilitySlot,
        attacker_stats: &crate::stats::StatBlock,
        level: u32,
    ) -> Vec<RuneEvent> {
        if matches!(slot, crate::types::AbilitySlot::Item(_)) {
            return Vec::new();
        }

        // Cooldown: 25.0 - (25.0 - 20.0) / 17.0 * (level.saturating_sub(1) as f64)
        let cooldown = 25.0 - (25.0 - 20.0) / 17.0 * (level.saturating_sub(1) as f64);
        if time.as_f64() - self.last_proc_time < cooldown {
            return Vec::new();
        }

        // Clean up self.recent_hits by removing hits older than 3 seconds (where time.as_f64() - t > 3.0)
        let current_time = time.as_f64();
        self.recent_hits.retain(|&(t, _)| current_time - t <= 3.15);

        // Update timestamp for this slot: if self.recent_hits already contains an entry for slot, remove it.
        self.recent_hits.retain(|&(_, s)| s != slot);

        // Push (time.as_f64(), slot) to the back of self.recent_hits
        self.recent_hits.push_back((current_time, slot));

        // If self.recent_hits.len() >= 3:
        if self.recent_hits.len() >= 3 {
            self.last_proc_time = current_time;
            self.recent_hits.clear();

            // Base damage: 30.0 + (180.0 - 30.0) / 17.0 * (level.saturating_sub(1) as f64)
            let base_damage = 30.0 + (180.0 - 30.0) / 17.0 * (level.saturating_sub(1) as f64);
            // Bonus AD: (attacker_stats.attack_damage - self.base_ad).max(0.0)
            let bonus_ad = (attacker_stats.attack_damage - self.base_ad).max(0.0);
            // AP: attacker_stats.ability_power
            let ap = attacker_stats.ability_power;
            // Damage: base_damage + 0.40 * bonus_ad + 0.25 * ap
            let damage = base_damage + 0.40 * bonus_ad + 0.25 * ap;
            // Damage Type (Adaptive): physical if bonus_ad > ap, magic otherwise
            let damage_type = if bonus_ad > ap {
                crate::types::DamageType::Physical
            } else {
                crate::types::DamageType::Magic
            };

            vec![RuneEvent::DamageDealt {
                amount: damage,
                damage_type,
                slot: crate::types::AbilitySlot::Electrocute,
            }]
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug)]
pub struct PhaseRush {
    pub is_melee: bool,
    /// List of (timestamp, slot) for the recent hits.
    pub recent_hits: std::collections::VecDeque<(f64, crate::types::AbilitySlot)>,
    /// When Phase Rush was activated.
    pub activation_time: f64,
    pub is_active: bool,
}

impl PhaseRush {
    pub fn new(is_melee: bool) -> Self {
        Self {
            is_melee,
            recent_hits: std::collections::VecDeque::new(),
            activation_time: -999.0,
            is_active: false,
        }
    }
}

impl RuneEffect for PhaseRush {
    fn name(&self) -> &str {
        "Phase Rush"
    }

    fn get_bonus_stats(
        &mut self,
        time: SimTime,
        _base_stats: &StatBlock,
        _level: u32,
    ) -> StatBlock {
        let mut stats = StatBlock::new();
        // If activated and within 3 seconds
        if *time.0 - self.activation_time <= 3.0 {
            // Grants 25-40% MS based on level. We'll use 30% for now.
            stats.movement_speed = 30.0; // Assume we add flat MS or have a multiplier later
        }
        stats
    }

    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        _amount: f64,
        _is_ability: bool,
        slot: crate::types::AbilitySlot,
        _attacker_stats: &crate::stats::StatBlock,
        _level: u32,
    ) -> Vec<RuneEvent> {
        let current_time = *time.0;

        // Clean up hits older than 4 seconds
        while let Some(&(t, _)) = self.recent_hits.front() {
            if current_time - t > 4.0 {
                self.recent_hits.pop_front();
            } else {
                break;
            }
        }

        // Check cooldown (15s)
        if current_time - self.activation_time < 15.0 {
            return Vec::new();
        }

        // Check if this slot is already in recent hits within the window
        // For Phase Rush, it's 3 *separate* attacks or abilities.
        let has_slot = self.recent_hits.iter().any(|&(_, s)| s == slot);
        if !has_slot {
            self.recent_hits.push_back((current_time, slot));
            println!(
                "Phase Rush hit: {:?}, total: {}",
                slot,
                self.recent_hits.len()
            );
        } else if slot == crate::types::AbilitySlot::AutoAttack {
            // Auto attacks can proc Phase Rush multiple times
            // Let's just allow it for AutoAttacks if it's a new hit (which this function call represents)
            self.recent_hits.push_back((current_time, slot));
            println!(
                "Phase Rush AA hit: {:?}, total: {}",
                slot,
                self.recent_hits.len()
            );
        }

        if self.recent_hits.len() >= 3 {
            self.activation_time = current_time;
            self.is_active = true;
            self.recent_hits.clear();
            return vec![RuneEvent::StacksChanged {
                name: "Phase Rush Activated".to_string(),
                stacks: 1,
            }];
        }

        Vec::new()
    }

    fn on_tick(&mut self, time: SimTime) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        // Check for expiration
        if self.is_active && *time.0 - self.activation_time > 3.0 {
            self.is_active = false;
            events.push(RuneEvent::StacksChanged {
                name: "Phase Rush Activated".to_string(),
                stacks: 0,
            });
        }
        events
    }
}

/// The Press the Attack rune effect.
#[derive(Debug)]
pub struct PressTheAttack {
    pub is_melee: bool,
    pub stacks: u32,
    pub last_attack_time: f64,
    pub last_trigger_time: f64,
    pub was_exposed: bool,
    pub base_ad: f64,
}

impl PressTheAttack {
    pub fn new(is_melee: bool) -> Self {
        Self {
            is_melee,
            stacks: 0,
            last_attack_time: -999.0,
            last_trigger_time: -999.0,
            was_exposed: false,
            base_ad: 0.0,
        }
    }
}

impl RuneEffect for PressTheAttack {
    fn name(&self) -> &str {
        "Press the Attack"
    }

    fn get_bonus_stats(
        &mut self,
        _time: SimTime,
        base_stats: &StatBlock,
        _level: u32,
    ) -> StatBlock {
        self.base_ad = base_stats.attack_damage;
        StatBlock::new()
    }

    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        _amount: f64,
        is_ability: bool,
        slot: crate::types::AbilitySlot,
        attacker_stats: &crate::stats::StatBlock,
        level: u32,
    ) -> Vec<RuneEvent> {
        if self.was_exposed {
            if time.as_f64() - self.last_trigger_time >= 6.0 {
                self.was_exposed = false;
                self.stacks = 0;
            } else {
                return Vec::new();
            }
        }

        if self.stacks > 0 && time.as_f64() - self.last_attack_time > 4.0 {
            self.stacks = 0;
        }

        if slot == crate::types::AbilitySlot::AutoAttack && !is_ability {
            self.last_attack_time = time.as_f64();
            self.stacks += 1;
            if self.stacks >= 3 {
                self.last_trigger_time = time.as_f64();
                self.was_exposed = true;
                self.stacks = 0;
                let base_damage = 40.0 + (180.0 - 40.0) / 17.0 * (level.saturating_sub(1) as f64);
                let bonus_ad = (attacker_stats.attack_damage - self.base_ad).max(0.0);
                let ap = attacker_stats.ability_power;
                let damage_type = if bonus_ad > ap {
                    crate::types::DamageType::Physical
                } else {
                    crate::types::DamageType::Magic
                };
                return vec![
                    RuneEvent::StacksChanged {
                        name: "Press the Attack".to_string(),
                        stacks: 3,
                    },
                    RuneEvent::ApplyDebuff {
                        name: "Press the Attack Exposure".to_string(),
                        duration: 6.0,
                        damage_reduction_percent: -0.08,
                    },
                    RuneEvent::DamageDealt {
                        amount: base_damage,
                        damage_type,
                        slot: crate::types::AbilitySlot::PressTheAttack,
                    },
                ];
            }
        }

        Vec::new()
    }

    fn on_tick(&mut self, time: SimTime) -> Vec<RuneEvent> {
        if self.was_exposed && time.as_f64() - self.last_trigger_time >= 6.0 {
            self.was_exposed = false;
            self.stacks = 0;
            return vec![RuneEvent::StacksChanged {
                name: "Press the Attack".to_string(),
                stacks: 0,
            }];
        }
        Vec::new()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AbilitySlot;

    #[test]
    fn test_taste_of_blood_scaling_and_cooldown() {
        let mut rune = TasteOfBlood::new();
        let mut base_stats = StatBlock::new();
        base_stats.attack_damage = 80.0;

        // Caches base_ad
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1);

        let mut attacker_stats = StatBlock::new();
        attacker_stats.attack_damage = 100.0; // 20 bonus AD
        attacker_stats.ability_power = 40.0; // 40 AP

        // First proc at t=0.0
        let events = rune.on_damage_dealt(
            SimTime::new(0.0),
            100.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        assert_eq!(events.len(), 1);
        match events[0] {
            RuneEvent::Healed { amount } => {
                // level 1 base_heal: 16.0
                // 0.10 * 20.0 = 2.0
                // 0.05 * 40.0 = 2.0
                // Total = 20.0
                assert!(
                    (amount - 20.0).abs() < 1e-5,
                    "Expected 20.0 heal, got {}",
                    amount
                );
            }
            _ => panic!("Expected Healed event"),
        }

        // Second hit at t=10.0 (on cooldown)
        let events = rune.on_damage_dealt(
            SimTime::new(10.0),
            100.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        assert!(events.is_empty());

        // Third hit at t=20.0 (cooldown finished)
        let events = rune.on_damage_dealt(
            SimTime::new(20.0),
            100.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_electrocute_proc_and_adaptive_damage() {
        let mut rune = Electrocute::new();
        let mut base_stats = StatBlock::new();
        base_stats.attack_damage = 70.0;

        // Cache base_ad
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1);

        let mut attacker_stats = StatBlock::new();
        attacker_stats.attack_damage = 120.0; // 50 bonus AD
        attacker_stats.ability_power = 10.0; // 10 AP

        // Hit 1: Q at t=0.0
        let events = rune.on_damage_dealt(
            SimTime::new(0.0),
            10.0,
            true,
            AbilitySlot::Q,
            &attacker_stats,
            1,
        );
        assert!(events.is_empty());

        // Hit 2: W at t=1.0
        let events = rune.on_damage_dealt(
            SimTime::new(1.0),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            1,
        );
        assert!(events.is_empty());

        // Duplicate Hit: W at t=2.0 (should update W's timestamp and not count as new slot)
        let events = rune.on_damage_dealt(
            SimTime::new(2.0),
            10.0,
            true,
            AbilitySlot::W,
            &attacker_stats,
            1,
        );
        assert!(events.is_empty());

        // Hit 3: AutoAttack at t=2.5
        let events = rune.on_damage_dealt(
            SimTime::new(2.5),
            10.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            RuneEvent::DamageDealt {
                amount,
                damage_type,
                slot,
            } => {
                assert_eq!(*slot, AbilitySlot::Electrocute);
                // base_damage = 30.0 (level 1)
                // bonus_ad = 50.0 -> 0.40 * 50.0 = 20.0
                // ap = 10.0 -> 0.25 * 10.0 = 2.5
                // total = 52.5
                assert!(
                    (amount - 52.5).abs() < 1e-5,
                    "Expected 52.5 damage, got {}",
                    amount
                );
                // bonus_ad (50.0) > ap (10.0) -> physical damage
                assert_eq!(*damage_type, crate::types::DamageType::Physical);
            }
            _ => panic!("Expected DamageDealt event"),
        }
    }

    #[test]
    fn test_press_the_attack_activation_and_exposure() {
        let mut rune = PressTheAttack::new(true);
        let mut base_stats = StatBlock::new();
        base_stats.attack_damage = 80.0;

        // Caches base_ad
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1);

        let mut attacker_stats = StatBlock::new();
        attacker_stats.attack_damage = 110.0; // 30 bonus AD
        attacker_stats.ability_power = 20.0; // 20 AP

        // 1st attack at t=0.0 -> no events
        let events = rune.on_damage_dealt(
            SimTime::new(0.0),
            80.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert!(events.is_empty());
        assert_eq!(rune.stacks, 1);

        // 2nd attack at t=1.0 -> no events
        let events = rune.on_damage_dealt(
            SimTime::new(1.0),
            80.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert!(events.is_empty());
        assert_eq!(rune.stacks, 2);

        // 3rd attack at t=2.0 -> triggers Press the Attack
        let events = rune.on_damage_dealt(
            SimTime::new(2.0),
            80.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert_eq!(events.len(), 3);
        assert_eq!(rune.stacks, 0);
        assert!(rune.was_exposed);

        match &events[0] {
            RuneEvent::StacksChanged { name, stacks } => {
                assert_eq!(name, "Press the Attack");
                assert_eq!(*stacks, 3);
            }
            _ => panic!("Expected StacksChanged"),
        }

        match &events[1] {
            RuneEvent::ApplyDebuff {
                name,
                duration,
                damage_reduction_percent,
            } => {
                assert_eq!(name, "Press the Attack Exposure");
                assert_eq!(*duration, 6.0);
                assert_eq!(*damage_reduction_percent, -0.08);
            }
            _ => panic!("Expected ApplyDebuff"),
        }

        match &events[2] {
            RuneEvent::DamageDealt {
                amount,
                damage_type,
                slot,
            } => {
                assert_eq!(*slot, AbilitySlot::PressTheAttack);
                // Level 1 base damage: 40.0
                assert!((amount - 40.0).abs() < 1e-5);
                // bonus_ad (30.0) > ap (20.0) -> physical
                assert_eq!(*damage_type, crate::types::DamageType::Physical);
            }
            _ => panic!("Expected DamageDealt"),
        }

        // 4th attack at t=3.0 -> no events (exposure is active)
        let events = rune.on_damage_dealt(
            SimTime::new(3.0),
            80.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert!(events.is_empty());

        // Check tick at t=8.0 (exposure not yet expired, 8.0 - 2.0 = 6.0, wait, duration is 6.0, so at 8.0 it is expired)
        // Let's tick at 5.0 first (not expired)
        let events = rune.on_tick(SimTime::new(5.0));
        assert!(events.is_empty());
        assert!(rune.was_exposed);

        // Tick at t=8.0 (expired, 8.0 - 2.0 = 6.0 >= 6.0)
        let events = rune.on_tick(SimTime::new(8.0));
        assert_eq!(events.len(), 1);
        assert!(!rune.was_exposed);
        match &events[0] {
            RuneEvent::StacksChanged { name, stacks } => {
                assert_eq!(name, "Press the Attack");
                assert_eq!(*stacks, 0);
            }
            _ => panic!("Expected StacksChanged"),
        }
    }
    #[test]
    fn test_conqueror_activation_and_healing() {
        let mut rune = Conqueror::new(true); // Melee
        let mut base_stats = StatBlock::new();
        base_stats.attack_damage = 80.0;
        
        let _ = rune.get_bonus_stats(SimTime::new(0.0), &base_stats, 1);
        
        let mut attacker_stats = StatBlock::new();
        attacker_stats.attack_damage = 110.0;
        
        // 1st attack (Melee = 2 stacks)
        let events = rune.on_damage_dealt(
            SimTime::new(0.0),
            100.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(rune.stacks, 2);
        
        // Stack to 10
        for i in 1..=4 {
            rune.on_damage_dealt(
                SimTime::new(i as f64),
                100.0,
                false,
                AbilitySlot::AutoAttack,
                &attacker_stats,
                1,
            );
        }
        assert_eq!(rune.stacks, 10);
        
        // Hit 6: Reaches 12 stacks
        let events = rune.on_damage_dealt(
            SimTime::new(5.0),
            100.0,
            false,
            AbilitySlot::AutoAttack,
            &attacker_stats,
            1,
        );
        assert_eq!(events.len(), 2); // StacksChanged + Healed
        assert_eq!(rune.stacks, 12);
        
        let mut healed = false;
        for event in events {
            if let RuneEvent::Healed { amount } = event {
                // 8% of 100 = 8.0
                assert!((amount - 8.0).abs() < 1e-5);
                healed = true;
            }
        }
        assert!(healed);
    }
}
