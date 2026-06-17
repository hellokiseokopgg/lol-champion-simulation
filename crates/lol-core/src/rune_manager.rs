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
    fn on_damage_dealt(&mut self, time: SimTime, amount: f64, is_ability: bool) -> Vec<RuneEvent>;
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
    pub fn on_damage_dealt(&mut self, time: SimTime, amount: f64, is_ability: bool) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        for effect in &mut self.effects {
            events.extend(effect.on_damage_dealt(time, amount, is_ability));
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

    fn on_damage_dealt(&mut self, time: SimTime, amount: f64, is_ability: bool) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        
        // Expiration check
        if *time.0 - self.last_stack_time > 6.0 {
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

    fn on_damage_dealt(&mut self, time: SimTime, _amount: f64, is_ability: bool) -> Vec<RuneEvent> {
        let mut events = Vec::new();
        // Expiration check
        if *time.0 - self.last_stack_time > 6.0 {
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
}
