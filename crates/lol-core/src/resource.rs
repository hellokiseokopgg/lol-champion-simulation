use crate::types::ResourceType;

/// Tracks a champion's resource (e.g., HP, Mana, Energy).
#[derive(Debug, Clone)]
pub struct Resource {
    /// The current amount of the resource.
    pub current: f64,
    /// The maximum capacity of the resource.
    pub max: f64,
    /// The type of resource.
    pub resource_type: ResourceType,
}

impl Resource {
    /// Creates a new resource with a specific max value and type. Starts full.
    pub fn new(max: f64, resource_type: ResourceType) -> Self {
        Self {
            current: max,
            max,
            resource_type,
        }
    }

    /// Consumes a given amount of the resource. Returns true if fully successful, false if it had to be capped at 0.
    pub fn consume(&mut self, amount: f64) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            self.current = 0.0;
            false
        }
    }

    /// Restores a given amount of the resource, capped at max.
    pub fn restore(&mut self, amount: f64) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Updates the maximum capacity of the resource, preserving the current amount (unless it exceeds the new max).
    pub fn update_max(&mut self, new_max: f64) {
        self.max = new_max;
        self.current = self.current.min(self.max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_consumption() {
        let mut mana = Resource::new(100.0, ResourceType::Mana);
        assert!(mana.consume(40.0));
        assert_eq!(mana.current, 60.0);
        assert!(!mana.consume(70.0));
        assert_eq!(mana.current, 60.0);
    }

    #[test]
    fn test_resource_restoration() {
        let mut mana = Resource::new(100.0, ResourceType::Mana);
        mana.consume(60.0);
        mana.restore(30.0);
        assert_eq!(mana.current, 70.0);
        mana.restore(50.0);
        assert_eq!(mana.current, 100.0); // Capped at max
    }
}
