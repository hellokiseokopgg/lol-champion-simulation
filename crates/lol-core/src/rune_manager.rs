use crate::stats::StatBlock;
use crate::types::SimTime;
use std::fmt::Debug;

pub enum RuneEvent {
    StacksChanged { name: String, stacks: u32 },
    Healed { amount: f64 },
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
    fn on_damage_dealt(&mut self, time: SimTime, amount: f64, is_ability: bool, slot: crate::types::AbilitySlot) -> Vec<RuneEvent>;

    /// Called periodically to allow runes to emit expiration events.
    fn on_tick(&mut self, time: SimTime) -> Vec<RuneEvent> {
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
    pub fn get_bonus_stats(&mut self, time: SimTime, base_stats: &StatBlock, level: u32) -> StatBlock {
        let mut total = StatBlock::new();
        for effect in &mut self.effects {
            total = total + effect.get_bonus_stats(time, base_stats, level);
        }
        total
    }

    /// Dispatches the damage dealt event to all runes, returning any generated RuneEvents.
    pub fn on_damage_dealt(&mut self, time: SimTime, amount: f64, is_ability: bool, slot: crate::types::AbilitySlot) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        for effect in &mut self.effects {
            events.extend(effect.on_damage_dealt(time, amount, is_ability, slot));
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
    pub is_melee: bool,
    stacks: u32,
    last_stack_time: f64,
}

impl Conqueror {
    pub fn new(is_melee: bool) -> Self {
        Self {
            is_melee,
            stacks: 0,
            last_stack_time: -999.0,
        }
    }
}

impl RuneEffect for Conqueror {
    fn name(&self) -> &str { "Conqueror" }

    fn get_bonus_stats(&mut self, time: SimTime, base_stats: &StatBlock, level: u32) -> StatBlock {
        // Expiration check
        if *time.0 - self.last_stack_time > 6.0 {
            self.stacks = 0;
        }

        let mut stats = StatBlock::new();
        if self.stacks > 0 {
            // Adaptive force per stack: 1.2 to 2.7 based on level
            // Formula: 1.2 + 0.0882 * (level - 1)
            let ad_per_stack = 1.2 + 0.0882 * (level.saturating_sub(1) as f64);
            stats.attack_damage = ad_per_stack * (self.stacks as f64);
        }
        stats
    }

    fn on_damage_dealt(&mut self, time: SimTime, amount: f64, is_ability: bool, _slot: crate::types::AbilitySlot) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        
        // Expiration check
        if *time.0 - self.last_stack_time > 5.0 {
            if self.stacks > 0 {
                events.push(RuneEvent::StacksChanged { name: "Conqueror".to_string(), stacks: 0 });
            }
            self.stacks = 0;
        }

        // Apply healing if max stacks
        if self.stacks == 12 {
            let healing = amount * if self.is_melee { 0.08 } else { 0.05 };
            if healing > 0.0 {
                events.push(RuneEvent::Healed { amount: healing });
            }
        }

        // Add stacks
        let stacks_to_add = if self.is_melee { 2 } else { 1 };
        let actual_add = if is_ability { 2 } else { stacks_to_add };
        
        let old_stacks = self.stacks;
        self.stacks = (self.stacks + actual_add).min(12);
        self.last_stack_time = *time.0;

        if self.stacks > old_stacks {
            events.push(RuneEvent::StacksChanged {
                name: "Conqueror".to_string(),
                stacks: self.stacks,
            });
        }

        events
    }

    fn on_tick(&mut self, time: SimTime) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        if self.stacks > 0 && *time.0 - self.last_stack_time > 5.0 {
            self.stacks = 0;
            events.push(RuneEvent::StacksChanged {
                name: "Conqueror".to_string(),
                stacks: 0,
            });
        }
        events
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
    fn name(&self) -> &str { "Lethal Tempo" }

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
            let as_ratio = base_stats.attack_speed_ratio.unwrap_or(base_stats.attack_speed);
            stats.attack_speed = as_per_stack * (self.stacks as f64) * as_ratio;
        }
        stats
    }

    fn on_damage_dealt(&mut self, time: SimTime, _amount: f64, is_ability: bool, _slot: crate::types::AbilitySlot) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        // Expiration check
        if *time.0 - self.last_stack_time > 6.0 {
            if self.stacks > 0 {
                events.push(RuneEvent::StacksChanged { name: "Lethal Tempo".to_string(), stacks: 0 });
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
pub struct TasteOfBlood;

impl RuneEffect for TasteOfBlood {
    fn name(&self) -> &str { "Taste of Blood" }

    fn get_bonus_stats(&mut self, _time: SimTime, _base_stats: &StatBlock, _level: u32) -> StatBlock {
        StatBlock::new()
    }

    fn on_damage_dealt(&mut self, _time: SimTime, _amount: f64, _is_ability: bool, _slot: crate::types::AbilitySlot) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        // Taste of Blood heals for 16-40 (+0.1 bonus AD) (+0.05 AP)
        // Since we don't have full stats here easily, we'll just heal for a flat 30 for now
        events.push(RuneEvent::Healed { amount: 30.0 });
        events
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
    fn name(&self) -> &str { "Phase Rush" }

    fn get_bonus_stats(&mut self, time: SimTime, _base_stats: &StatBlock, _level: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        // If activated and within 3 seconds
        if *time.0 - self.activation_time <= 3.0 {
            // Grants 25-40% MS based on level. We'll use 30% for now.
            stats.movement_speed = 30.0; // Assume we add flat MS or have a multiplier later
        }
        stats
    }

    fn on_damage_dealt(&mut self, time: SimTime, _amount: f64, _is_ability: bool, slot: crate::types::AbilitySlot) -> Vec<RuneEvent> {
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
            println!("Phase Rush hit: {:?}, total: {}", slot, self.recent_hits.len());
        } else if slot == crate::types::AbilitySlot::AutoAttack {
            // Auto attacks can proc Phase Rush multiple times
            // Let's just allow it for AutoAttacks if it's a new hit (which this function call represents)
            self.recent_hits.push_back((current_time, slot));
            println!("Phase Rush AA hit: {:?}, total: {}", slot, self.recent_hits.len());
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
