use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

/// Represents the type of damage in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    /// Physical damage, mitigated by armor.
    Physical,
    /// Magic damage, mitigated by magic resistance.
    Magic,
    /// True damage, ignores all resistances.
    True,
}

/// The result of processing and applying damage.
#[derive(Debug, Clone, Copy)]
pub struct TakeDamageResult {
    /// The actual damage applied to the target's health.
    pub actual_damage: f64,
    /// Whether the target is dead after this damage.
    pub is_dead: bool,
}

/// Represents the slot of an ability for a champion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbilitySlot {
    /// Q ability (first basic ability)
    Q,
    /// W ability (second basic ability)
    W,
    /// E ability (third basic ability)
    E,
    /// R ability (ultimate ability)
    R,
    /// Passive ability
    Passive,
    /// Basic auto attack
    AutoAttack,
    /// Active item (stores the item ID)
    Item(u32),
}

/// Represents the resource type a champion uses for their abilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Mana resource
    Mana,
    /// Energy resource
    Energy,
    /// Rage or Fury resource
    Rage,
    /// No resource cost
    None,
}

/// Represents simulation time. Uses `OrderedFloat` to ensure valid comparison
/// and sorting within priority queues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SimTime(pub OrderedFloat<f64>);

impl SimTime {
    /// Create a new `SimTime` from a floating point value.
    pub fn new(time: f64) -> Self {
        Self(OrderedFloat(time))
    }

    /// Extract the underlying `f64` value.
    pub fn as_f64(&self) -> f64 {
        self.0.into_inner()
    }
}

impl std::ops::Add<f64> for SimTime {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        Self(OrderedFloat(self.0.into_inner() + rhs))
    }
}

/// A unique identifier for a champion.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChampionId(pub String);

/// A unique identifier for a specific ability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AbilityId(pub String);

/// A unique identifier for an active effect (e.g., buff or debuff).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EffectId(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_time_ordering() {
        let t1 = SimTime::new(1.5);
        let t2 = SimTime::new(2.0);
        let t3 = SimTime::new(1.5);

        assert!(t1 < t2);
        assert_eq!(t1, t3);
    }

    #[test]
    fn test_sim_time_add() {
        let t1 = SimTime::new(1.5);
        let t2 = t1 + 0.5;
        assert_eq!(t2, SimTime::new(2.0));
        assert_eq!(t2.as_f64(), 2.0);
    }
    
    #[test]
    fn test_serialization() {
        let json = serde_json::to_string(&DamageType::Physical).unwrap();
        assert_eq!(json, "\"Physical\"");
        
        let slot_json = serde_json::to_string(&AbilitySlot::Q).unwrap();
        assert_eq!(slot_json, "\"Q\"");
        
        let time = SimTime::new(1.5);
        let time_json = serde_json::to_string(&time).unwrap();
        assert_eq!(time_json, "1.5");
    }
}
