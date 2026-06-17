use crate::stats::StatBlock;

/// Represents a primary or secondary rune path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunePath {
    Precision,
    Domination,
    Sorcery,
    Resolve,
    Inspiration,
}

/// Represents an effect provided by a selected rune or stat shard.
pub trait RuneEffect {
    /// The name of the rune.
    fn name(&self) -> &str;

    /// The icon ID of the rune.
    fn icon(&self) -> &str {
        ""
    }

    /// The tree the rune belongs to (e.g. Precision).
    fn tree(&self) -> &str {
        ""
    }

    /// The static stats provided by the rune.
    fn stats(&self) -> StatBlock {
        StatBlock::new()
    }
}

/// Represents a champion's full rune page.
pub struct RunePage {
    pub primary_path: RunePath,
    pub secondary_path: RunePath,
    pub keystone: Box<dyn RuneEffect>,
    pub primary_runes: Vec<Box<dyn RuneEffect>>,
    pub secondary_runes: Vec<Box<dyn RuneEffect>>,
    pub stat_shards: Vec<Box<dyn RuneEffect>>,
}

struct EmptyRune;
impl RuneEffect for EmptyRune {
    fn name(&self) -> &str {
        "Empty"
    }
}

impl Default for RunePage {
    fn default() -> Self {
        Self {
            primary_path: RunePath::Precision,
            secondary_path: RunePath::Domination,
            keystone: Box::new(EmptyRune),
            primary_runes: vec![],
            secondary_runes: vec![],
            stat_shards: vec![],
        }
    }
}

impl RunePage {
    /// Aggregates all static stats provided by the runes and stat shards.
    pub fn aggregate_stats(&self) -> StatBlock {
        let mut total = StatBlock::new();
        total = total + self.keystone.stats();
        for r in &self.primary_runes {
            total = total + r.stats();
        }
        for r in &self.secondary_runes {
            total = total + r.stats();
        }
        for s in &self.stat_shards {
            total = total + s.stats();
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRune {
        name: String,
        ad: f64,
    }

    impl RuneEffect for TestRune {
        fn name(&self) -> &str {
            &self.name
        }
        fn stats(&self) -> StatBlock {
            StatBlock {
                attack_damage: self.ad,
                ..Default::default()
            }
        }
    }

    #[test]
    fn test_rune_page_stats() {
        let page = RunePage {
            primary_path: RunePath::Precision,
            secondary_path: RunePath::Domination,
            keystone: Box::new(TestRune {
                name: "Conqueror".to_string(),
                ad: 0.0,
            }),
            primary_runes: vec![Box::new(TestRune {
                name: "Triumph".to_string(),
                ad: 0.0,
            })],
            secondary_runes: vec![Box::new(TestRune {
                name: "Eyeball".to_string(),
                ad: 18.0,
            })],
            stat_shards: vec![
                Box::new(TestRune {
                    name: "Adaptive".to_string(),
                    ad: 5.4,
                }),
                Box::new(TestRune {
                    name: "Adaptive".to_string(),
                    ad: 5.4,
                }),
            ],
        };

        let total = page.aggregate_stats();
        // 18.0 + 5.4 + 5.4 = 28.8
        assert!((total.attack_damage - 28.8).abs() < 1e-6);
    }
}
