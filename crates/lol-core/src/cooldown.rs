use crate::types::SimTime;

/// Calculates the effective cooldown given a base cooldown and ability haste.
pub fn effective_cooldown(base_cooldown: f64, ability_haste: f64) -> f64 {
    base_cooldown * (100.0 / (100.0 + ability_haste))
}

/// Tracks the cooldown state of an ability or effect.
#[derive(Debug, Clone)]
pub struct Cooldown {
    /// The simulation time when the cooldown will expire.
    pub ready_at: SimTime,
}

impl Cooldown {
    /// Creates a new cooldown that is ready immediately.
    pub fn new() -> Self {
        Self {
            ready_at: SimTime::new(0.0),
        }
    }

    /// Checks if the cooldown is currently ready at the given simulation time.
    pub fn is_ready(&self, current_time: SimTime) -> bool {
        current_time >= self.ready_at
    }

    /// Puts the ability on cooldown by setting the `ready_at` time.
    pub fn start_cooldown(&mut self, current_time: SimTime, effective_cd: f64) {
        self.ready_at = current_time + effective_cd;
    }

    /// Reduces the remaining cooldown by a flat amount of seconds.
    pub fn reduce_cooldown(&mut self, amount: f64) {
        let current_ready = self.ready_at.as_f64();
        let new_ready = current_ready - amount;
        self.ready_at = SimTime::new(new_ready.max(0.0));
    }
}

impl Default for Cooldown {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_cooldown() {
        // 0 AH = 100% CD
        assert_eq!(effective_cooldown(10.0, 0.0), 10.0);
        // 100 AH = 50% CD
        assert_eq!(effective_cooldown(10.0, 100.0), 5.0);
        // 200 AH = 33.3% CD
        assert!((effective_cooldown(10.0, 200.0) - 3.333333).abs() < 1e-5);
    }

    #[test]
    fn test_cooldown_tracking() {
        let mut cd = Cooldown::new();
        let current_time = SimTime::new(10.0);
        
        assert!(cd.is_ready(current_time));
        
        cd.start_cooldown(current_time, 5.0);
        assert!(!cd.is_ready(current_time));
        assert!(cd.is_ready(SimTime::new(15.0)));

        cd.reduce_cooldown(2.0);
        assert!(cd.is_ready(SimTime::new(13.0)));
    }
}
