use std::collections::HashMap;

use crate::cooldown::Cooldown;
use crate::types::{AbilitySlot, SimTime};

/// Trait defining the behavior of a champion's ability.
pub trait Ability {
    /// Returns the slot this ability occupies.
    fn slot(&self) -> AbilitySlot;

    /// Returns the base cast time of the ability.
    fn cast_time(&self) -> f64;

    /// Returns the windup percent of the total cast animation.
    /// Mostly used for AutoAttacks, representing the percentage of Attack Delay before damage hits.
    fn windup_percent(&self) -> f64 {
        0.0 // Defaults to 0 for normal abilities
    }

    /// Returns the base cooldown at a given ability level.
    fn base_cooldown(&self, level: u32) -> f64;

    /// Returns the resource cost at a given ability level.
    fn cost(&self, level: u32) -> f64;

    /// Executes the ability's logic within the simulation context.
    /// This may generate damage events, apply buffs, or spawn projectiles.
    fn execute(
        &self,
        ctx: &mut crate::event::SimContext,
        actor: &crate::types::ChampionId,
        target: &crate::types::ChampionId,
    );

    /// Clones the ability.
    fn clone_box(&self) -> Box<dyn Ability>;
}

/// The state of a specific ability during simulation.
#[derive(Debug, Clone)]
pub struct AbilityState {
    /// The current rank/level of the ability (0 means unlearned).
    pub level: u32,
    /// The cooldown tracking for the ability.
    pub cooldown: Cooldown,
}

impl Default for AbilityState {
    fn default() -> Self {
        Self::new()
    }
}

impl AbilityState {
    pub fn new() -> Self {
        Self {
            level: 0,
            cooldown: Cooldown::new(),
        }
    }
}

/// Manages all ability slots and their states for a champion.
pub struct AbilitySlotManager {
    states: HashMap<AbilitySlot, AbilityState>,
}

impl Default for AbilitySlotManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AbilitySlotManager {
    /// Creates a new manager with default slots initialized to level 0 and ready.
    pub fn new() -> Self {
        let mut states = HashMap::new();
        states.insert(AbilitySlot::Q, AbilityState::new());
        states.insert(AbilitySlot::W, AbilityState::new());
        states.insert(AbilitySlot::E, AbilityState::new());
        states.insert(AbilitySlot::R, AbilityState::new());
        // Passive usually level 1 automatically
        let mut passive = AbilityState::new();
        passive.level = 1;
        states.insert(AbilitySlot::Passive, passive);
        // Auto attack usually level 1 automatically
        let mut auto = AbilityState::new();
        auto.level = 1;
        states.insert(AbilitySlot::AutoAttack, auto);

        Self { states }
    }

    pub fn get_state(&self, slot: AbilitySlot) -> Option<&AbilityState> {
        self.states.get(&slot)
    }

    pub fn get_state_mut(&mut self, slot: AbilitySlot) -> Option<&mut AbilityState> {
        self.states.get_mut(&slot)
    }

    /// Increases the rank of the specified ability.
    pub fn level_up(&mut self, slot: AbilitySlot) {
        if let Some(state) = self.states.get_mut(&slot) {
            state.level += 1;
        }
    }

    /// Registers a new ability slot with a starting level (e.g., for items).
    pub fn register_ability(&mut self, slot: AbilitySlot, level: u32) {
        let mut state = AbilityState::new();
        state.level = level;
        self.states.insert(slot, state);
    }

    /// Resets the cooldown of the specified ability to ready.
    pub fn reset_cooldown(&mut self, slot: AbilitySlot) {
        if let Some(state) = self.states.get_mut(&slot) {
            state.cooldown.ready_at = crate::types::SimTime::new(0.0);
        }
    }

    /// Checks if an ability is learned (level > 0) and off cooldown.
    pub fn is_ready(&self, slot: AbilitySlot, current_time: SimTime) -> bool {
        if let Some(state) = self.states.get(&slot) {
            state.level > 0 && state.cooldown.is_ready(current_time)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_slot_manager() {
        let mut manager = AbilitySlotManager::new();

        // Q starts unlearned
        assert!(!manager.is_ready(AbilitySlot::Q, SimTime::new(0.0)));

        manager.level_up(AbilitySlot::Q);
        assert!(manager.is_ready(AbilitySlot::Q, SimTime::new(0.0)));

        // Put Q on cooldown
        if let Some(state) = manager.get_state_mut(AbilitySlot::Q) {
            state.cooldown.start_cooldown(SimTime::new(0.0), 5.0);
        }

        assert!(!manager.is_ready(AbilitySlot::Q, SimTime::new(2.0)));
        assert!(manager.is_ready(AbilitySlot::Q, SimTime::new(5.0)));

        // Auto attack should be ready immediately
        assert!(manager.is_ready(AbilitySlot::AutoAttack, SimTime::new(0.0)));
    }
}
