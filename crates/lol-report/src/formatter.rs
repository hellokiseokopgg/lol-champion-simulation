use crate::statistics::Statistics;
use crate::collector::{DataCollector, CombatEvent};
use std::fmt::Write;

pub struct Formatter;

impl Formatter {
    pub fn format_text(stats: &Statistics) -> String {
        let mut output = String::new();
        writeln!(output, "--- Combat Simulation Report ---").unwrap();
        writeln!(output, "Combat Duration: {:.2}s", stats.duration).unwrap();
        writeln!(output, "--------------------------------").unwrap();

        for (champion, champ_stat) in &stats.champion_stats {
            writeln!(output, "Champion: {:?}", champion).unwrap();
            writeln!(output, "  Total Damage: {:.1}", champ_stat.total_damage).unwrap();
            writeln!(output, "  DPS: {:.1}", champ_stat.dps).unwrap();
            writeln!(output, "  Ability Breakdown:").unwrap();
            
            let mut breakdown: Vec<_> = champ_stat.ability_breakdown.iter().collect();
            // Sort by damage descending
            breakdown.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (slot, damage) in breakdown {
                let pct = if champ_stat.total_damage > 0.0 {
                    (damage / champ_stat.total_damage) * 100.0
                } else {
                    0.0
                };
                writeln!(output, "    {:?}: {:.1} ({:.1}%)", slot, damage, pct).unwrap();
            }
            writeln!(output, "--------------------------------").unwrap();
        }

        output
    }

    pub fn format_gantt(collector: &DataCollector) -> String {
        let mut output = String::new();
        writeln!(output, "```mermaid").unwrap();
        writeln!(output, "gantt").unwrap();
        writeln!(output, "    title Combat Skill Timeline").unwrap();
        writeln!(output, "    dateFormat x").unwrap();
        writeln!(output, "    axisFormat %S.%L").unwrap();
        writeln!(output, "").unwrap();

        // Group events by source champion
        let mut events_by_champ: std::collections::HashMap<String, Vec<&CombatEvent>> = std::collections::HashMap::new();
        for event in &collector.events {
            match event {
                CombatEvent::Damage { source, .. } | CombatEvent::Cast { source, .. } => {
                    events_by_champ.entry(source.0.clone()).or_default().push(event);
                }
                _ => {}
            }
        }

        for (champ, events) in events_by_champ {
            writeln!(output, "    section {}", champ).unwrap();
            let mut event_counter = 0;
            for event in events {
                match event {
                    CombatEvent::Cast { time, ability, .. } => {
                        let start_ms = (time.as_f64() * 1000.0) as u64;
                        let end_ms = start_ms + 200; // 200ms visual duration for casts
                        writeln!(output, "    Cast {:?} : {}, {}", ability, start_ms, end_ms).unwrap();
                        event_counter += 1;
                    }
                    CombatEvent::Damage { time, ability, .. } => {
                        let start_ms = (time.as_f64() * 1000.0) as u64;
                        let end_ms = start_ms + 100; // 100ms visual duration for damage ticks
                        writeln!(output, "    Dmg {:?} : {}, {}", ability, start_ms, end_ms).unwrap();
                        event_counter += 1;
                    }
                    _ => {}
                }
            }
            if event_counter == 0 {
                writeln!(output, "    No Actions : 0, 0").unwrap();
            }
        }

        writeln!(output, "```").unwrap();
        output
    }
}
