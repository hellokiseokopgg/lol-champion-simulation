use crate::collector::{CombatEvent, DataCollector};
use lol_core::types::{AbilitySlot, ChampionId, SimTime};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ChampionStats {
    pub total_damage: f64,
    pub dps: f64,
    pub ability_breakdown: HashMap<AbilitySlot, f64>,
}

#[derive(Debug)]
pub struct Statistics {
    pub duration: f64,
    pub champion_stats: HashMap<ChampionId, ChampionStats>,
}

impl Statistics {
    pub fn calculate(collector: &DataCollector, max_time: SimTime) -> Self {
        let mut stats = Self {
            duration: max_time.0.into_inner(),
            champion_stats: HashMap::new(),
        };

        // If no events, return empty
        if collector.events.is_empty() {
            return stats;
        }

        let mut end_time = 0.0;

        for event in &collector.events {
            match event {
                CombatEvent::Damage { time, source, amount, ability, .. } => {
                    let time_val = time.0.into_inner();
                    if time_val > end_time {
                        end_time = time_val;
                    }

                    let champ_stat = stats
                        .champion_stats
                        .entry(source.clone())
                        .or_insert_with(ChampionStats::default);

                    champ_stat.total_damage += amount;
                    *champ_stat.ability_breakdown.entry(*ability).or_insert(0.0) += amount;
                }
                CombatEvent::Cast { time, .. } | CombatEvent::Death { time, .. } => {
                    let time_val = time.0.into_inner();
                    if time_val > end_time {
                        end_time = time_val;
                    }
                }
                _ => {}
            }
        }

        // Avoid division by zero
        if end_time <= 0.0 {
            end_time = 1.0;
        }
        stats.duration = end_time;

        for champ_stat in stats.champion_stats.values_mut() {
            champ_stat.dps = champ_stat.total_damage / end_time;
        }

        stats
    }
}
