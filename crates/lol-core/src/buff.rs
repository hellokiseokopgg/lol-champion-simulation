use std::collections::HashMap;

use crate::stats::StatBlock;
use crate::types::{EffectId, SimTime};

/// How a buff behaves when re-applied while already active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshBehavior {
    /// Resets the duration to maximum, does not stack.
    RefreshDuration,
    /// Adds a stack, up to a maximum.
    AddStack,
    /// Does nothing if already active.
    Ignore,
    /// Adds the duration of the new buff to the existing duration.
    ExtendDuration,
}

/// A trait for effects that can be applied to champions (Buffs or Debuffs).
pub trait StatusEffect {
    fn id(&self) -> EffectId;
    fn name(&self) -> &str;
    fn duration(&self) -> f64;
    fn refresh_behavior(&self) -> RefreshBehavior;
    fn max_stacks(&self) -> u32;

    /// Calculate stat modifiers this effect provides based on current stacks.
    fn stat_modifiers(&self, stacks: u32) -> StatBlock;

    /// The CC type of this effect, if any.
    fn cc_type(&self) -> Option<crate::types::CCType> {
        None
    }

    /// Whether this effect prevents the champion from using basic attacks.
    fn prevents_basic_attacks(&self) -> bool {
        false
    }

    /// The number of stacks this effect has when first applied.
    fn initial_stacks(&self) -> u32 {
        1
    }
}

/// Tracks an active instance of a status effect.
pub struct ActiveEffect {
    pub effect: Box<dyn StatusEffect>,
    pub expiration_time: SimTime,
    pub stacks: u32,
}

/// Manages active buffs and debuffs on a champion.
pub struct BuffManager {
    active_effects: HashMap<EffectId, ActiveEffect>,
}

impl Default for BuffManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BuffManager {
    pub fn new() -> Self {
        Self {
            active_effects: HashMap::new(),
        }
    }

    /// Applies a status effect, taking the target's tenacity into account for CC effects.
    pub fn apply_effect(
        &mut self,
        effect: Box<dyn StatusEffect>,
        current_time: SimTime,
        target_tenacity: f64,
    ) {
        let id = effect.id();
        let mut duration = effect.duration();
        let initial_stacks = effect.initial_stacks();

        // Apply Tenacity reduction if applicable
        if effect.cc_type().is_some_and(|cc| cc.affected_by_tenacity()) {
            duration *= 1.0 - target_tenacity;
        }

        let expiration_time = current_time + duration;

        if let Some(active) = self.active_effects.get_mut(&id) {
            match effect.refresh_behavior() {
                RefreshBehavior::RefreshDuration => {
                    active.expiration_time = expiration_time;
                }
                RefreshBehavior::AddStack => {
                    active.expiration_time = expiration_time;
                    active.stacks = (active.stacks + 1).min(effect.max_stacks());
                }
                RefreshBehavior::Ignore => {}
                RefreshBehavior::ExtendDuration => {
                    active.expiration_time = active.expiration_time + duration;
                }
            }
        } else {
            self.active_effects.insert(
                id,
                ActiveEffect {
                    effect,
                    expiration_time,
                    stacks: initial_stacks,
                },
            );
        }
    }

    /// Checks if the champion is currently affected by any Hard CC (Stun, Airborne, Silence).
    pub fn has_hard_cc(&self, current_time: SimTime) -> bool {
        self.active_effects.values().any(|active| {
            active.expiration_time > current_time
                && active
                    .effect
                    .cc_type()
                    .is_some_and(|cc| cc.class() == crate::types::CCClass::Hard)
        })
    }

    /// Cleans up expired effects at the given simulation time.
    pub fn cleanup_expired(&mut self, current_time: SimTime) {
        self.active_effects
            .retain(|_, active| active.expiration_time > current_time);
    }

    /// Checks if a buff is active by name.
    pub fn has_buff_by_name(&self, name: &str, current_time: SimTime) -> bool {
        self.active_effects.values().any(|active| {
            active.effect.name().eq_ignore_ascii_case(name) && active.expiration_time > current_time
        })
    }

    /// Checks if a buff is active by id.
    pub fn has_effect_by_id(&self, id: &EffectId, current_time: SimTime) -> bool {
        if let Some(active) = self.active_effects.get(id) {
            active.expiration_time > current_time
        } else {
            false
        }
    }

    /// Checks if a buff is active by id.
    pub fn get_stacks_by_id(&self, effect_id: &EffectId, current_time: SimTime) -> u32 {
        self.active_effects
            .get(effect_id)
            .filter(|active| active.expiration_time > current_time)
            .map_or(0, |active| active.stacks)
    }

    /// Checks if any active buff prevents basic attacks.
    pub fn prevents_basic_attacks(&self, current_time: SimTime) -> bool {
        self.active_effects.values().any(|active| {
            active.expiration_time > current_time && active.effect.prevents_basic_attacks()
        })
    }

    /// Gets the number of stacks of a buff by name. Returns 0 if not found or expired.
    pub fn get_stacks_by_name(&self, name: &str, current_time: SimTime) -> u32 {
        self.active_effects
            .values()
            .find(|active| {
                active.effect.name().eq_ignore_ascii_case(name)
                    && active.expiration_time > current_time
            })
            .map(|active| active.stacks)
            .unwrap_or(0)
    }

    /// Removes a specific effect entirely.
    pub fn remove_effect(&mut self, id: &EffectId) {
        self.active_effects.remove(id);
    }

    /// Decrements the stack count of an effect by 1. If the stack count becomes 0 or if stacks <= 1, removes the effect.
    pub fn decrement_stacks(&mut self, id: &EffectId) {
        if let std::collections::hash_map::Entry::Occupied(mut entry) = self.active_effects.entry(id.clone()) {
            if entry.get().stacks > 1 {
                entry.get_mut().stacks -= 1;
            } else {
                entry.remove();
            }
        }
    }

    /// Removes a specific effect if it has expired at the given time. Returns true if removed.
    pub fn remove_effect_if_expired(&mut self, id: &EffectId, current_time: SimTime) -> bool {
        if self
            .active_effects
            .get(id)
            .is_some_and(|active| active.expiration_time <= current_time)
        {
            self.active_effects.remove(id);
            return true;
        }
        false
    }

    /// Aggregates all stat modifiers from currently active effects.
    pub fn aggregate_stats(&self) -> StatBlock {
        let mut total_stats = StatBlock::new();
        for active in self.active_effects.values() {
            total_stats = total_stats + active.effect.stat_modifiers(active.stacks);
        }
        total_stats
    }

    /// Returns the number of stacks of a specific effect, or 0 if not active.
    pub fn get_stacks(&self, id: &EffectId) -> u32 {
        self.active_effects.get(id).map_or(0, |e| e.stacks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBuff {
        id: EffectId,
        behavior: RefreshBehavior,
        max_stacks: u32,
        duration: f64,
        ad_per_stack: f64,
    }

    impl StatusEffect for TestBuff {
        fn id(&self) -> EffectId {
            self.id.clone()
        }
        fn name(&self) -> &str {
            "Test Buff"
        }
        fn duration(&self) -> f64 {
            self.duration
        }
        fn refresh_behavior(&self) -> RefreshBehavior {
            self.behavior
        }
        fn max_stacks(&self) -> u32 {
            self.max_stacks
        }
        fn stat_modifiers(&self, stacks: u32) -> StatBlock {
            let mut stats = StatBlock::new();
            stats.attack_damage = self.ad_per_stack * stacks as f64;
            stats
        }
    }

    #[test]
    fn test_buff_stacking() {
        let mut manager = BuffManager::new();
        let buff_id = EffectId("test_stack".to_string());

        let make_buff = || {
            Box::new(TestBuff {
                id: buff_id.clone(),
                behavior: RefreshBehavior::AddStack,
                max_stacks: 3,
                duration: 5.0,
                ad_per_stack: 10.0,
            })
        };

        manager.apply_effect(make_buff(), SimTime::new(0.0), 0.0);
        assert_eq!(manager.get_stacks(&buff_id), 1);
        assert_eq!(manager.aggregate_stats().attack_damage, 10.0);

        manager.apply_effect(make_buff(), SimTime::new(2.0), 0.0);
        assert_eq!(manager.get_stacks(&buff_id), 2);
        assert_eq!(manager.aggregate_stats().attack_damage, 20.0);

        // Max stacks is 3
        manager.apply_effect(make_buff(), SimTime::new(3.0), 0.0);
        manager.apply_effect(make_buff(), SimTime::new(4.0), 0.0);
        assert_eq!(manager.get_stacks(&buff_id), 3);
        assert_eq!(manager.aggregate_stats().attack_damage, 30.0);
    }

    #[test]
    fn test_buff_expiration() {
        let mut manager = BuffManager::new();
        let buff_id = EffectId("test_exp".to_string());

        let buff = Box::new(TestBuff {
            id: buff_id.clone(),
            behavior: RefreshBehavior::RefreshDuration,
            max_stacks: 1,
            duration: 5.0,
            ad_per_stack: 10.0,
        });

        manager.apply_effect(buff, SimTime::new(0.0), 0.0);

        manager.cleanup_expired(SimTime::new(4.0));
        assert_eq!(manager.get_stacks(&buff_id), 1);

        manager.cleanup_expired(SimTime::new(5.1));
        assert_eq!(manager.get_stacks(&buff_id), 0);
    }
}

/// Buff representing Bone Plating blocks
#[derive(Debug, Clone)]
pub struct BonePlatingBuff {
    pub level: u32,
}

impl StatusEffect for BonePlatingBuff {
    fn id(&self) -> EffectId {
        EffectId("BonePlatingBuff".to_string())
    }
    fn name(&self) -> &str {
        "Bone Plating"
    }
    fn duration(&self) -> f64 {
        1.5
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::Ignore
    }
    fn max_stacks(&self) -> u32 {
        3
    }
    fn initial_stacks(&self) -> u32 {
        3
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::new()
    }
}

/// Buff representing Bone Plating cooldown
#[derive(Debug, Clone)]
pub struct BonePlatingCooldown;

impl StatusEffect for BonePlatingCooldown {
    fn id(&self) -> EffectId {
        EffectId("BonePlatingCooldown".to_string())
    }
    fn name(&self) -> &str {
        "Bone Plating Cooldown"
    }
    fn duration(&self) -> f64 {
        55.0
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::Ignore
    }
    fn max_stacks(&self) -> u32 {
        1
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::new()
    }
}
