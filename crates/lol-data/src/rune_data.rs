use serde::{Deserialize, Serialize};

/// Represents all static data associated with a rune.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuneData {
    /// The unique identifier of the rune.
    pub id: String,
    /// The display name of the rune.
    pub name: String,
    /// The rune tree (e.g., "Precision", "Domination").
    pub tree: String,
    /// The ID of the icon to be used for display.
    #[serde(default)]
    pub icon: String,
    /// Static stats provided by the rune (if any).
    #[serde(default)]
    pub stats: RuneStats,
}

/// Statistics provided by a rune.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RuneStats {
    #[serde(default)]
    pub adaptive_force: f32,
    #[serde(default)]
    pub attack_speed: f32,
    #[serde(default)]
    pub ability_haste: f32,
    #[serde(default)]
    pub armor: f32,
    #[serde(default)]
    pub magic_resist: f32,
    #[serde(default)]
    pub hp: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_rune() {
        let json = r#"{
            "id": "conqueror",
            "name": "Conqueror",
            "tree": "Precision"
        }"#;

        let rune: RuneData = serde_json::from_str(json).unwrap();
        assert_eq!(rune.id, "conqueror");
        assert_eq!(rune.name, "Conqueror");
        assert_eq!(rune.tree, "Precision");
        assert_eq!(rune.stats.adaptive_force, 0.0);
    }
}
