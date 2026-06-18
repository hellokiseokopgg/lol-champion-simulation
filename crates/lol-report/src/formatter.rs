use crate::collector::{CombatEvent, DataCollector};
use crate::statistics::Statistics;
use std::fmt::Write;

pub struct Formatter;

impl Formatter {
    pub fn format_text(stats: &Statistics, collector: &DataCollector) -> String {
        let mut output = String::new();
        writeln!(output, "--- Combat Simulation Report ---").unwrap();
        writeln!(output, "Combat Duration: {:.2}s", stats.duration).unwrap();
        writeln!(output, "--------------------------------").unwrap();

        let mut sorted_stats: Vec<_> = stats.champion_stats.iter().collect();
        sorted_stats.sort_by(|a, b| {
            b.1.total_damage
                .partial_cmp(&a.1.total_damage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (champion, champ_stat) in sorted_stats {
            writeln!(output, "Champion: {:?}", champion.0).unwrap();

            if let Some(items) = collector.champion_items.get(champion) {
                if !items.is_empty() {
                    let item_names: Vec<String> =
                        items.iter().map(|(_, name)| name.clone()).collect();
                    writeln!(output, "  Items: {}", item_names.join(", ")).unwrap();
                } else {
                    writeln!(output, "  Items: None").unwrap();
                }
            }
            writeln!(output, "  Total Damage: {:.1}", champ_stat.total_damage).unwrap();
            writeln!(output, "  Damage Taken: {:.1}", champ_stat.damage_taken).unwrap();
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

    pub fn format_gantt(collector: &DataCollector, translator: &crate::i18n::Translator) -> String {
        let mut out = String::new();
        out.push_str("```mermaid\ngantt\n    title Combat Skill Timeline\n    dateFormat x\n    axisFormat %S.%L\n\n");

        // Group events by actor
        let mut actor_events = std::collections::HashMap::new();
        for event in &collector.events {
            match event {
                CombatEvent::Cast { source, .. } => {
                    actor_events
                        .entry(source.clone())
                        .or_insert_with(Vec::new)
                        .push(event.clone());
                }
                CombatEvent::Damage { source, .. } => {
                    actor_events
                        .entry(source.clone())
                        .or_insert_with(Vec::new)
                        .push(event.clone());
                }
                CombatEvent::BuffApply { target, .. } => {
                    actor_events
                        .entry(target.clone())
                        .or_insert_with(Vec::new)
                        .push(event.clone());
                }
                CombatEvent::BuffExpire { target, .. } => {
                    actor_events
                        .entry(target.clone())
                        .or_insert_with(Vec::new)
                        .push(event.clone());
                }
                _ => {}
            }
        }

        for (actor, events) in actor_events {
            out.push_str(&format!("    section {}\n", actor.0));

            let mut last_buff_times: std::collections::HashMap<String, f64> =
                std::collections::HashMap::new();

            for event in events {
                match event {
                    CombatEvent::Cast { time, ability, .. } => {
                        let ms = (time.as_f64() * 1000.0) as u64;
                        let ability_str = format!("{:?}", ability);
                        out.push_str(&format!(
                            "    Cast {} : {}, {}\n",
                            ability_str,
                            ms,
                            ms + 200
                        ));
                    }
                    CombatEvent::Damage { time, ability, .. } => {
                        let ms = (time.as_f64() * 1000.0) as u64;
                        let ability_str = format!("{:?}", ability);
                        out.push_str(&format!("    Dmg {} : {}, {}\n", ability_str, ms, ms + 100));
                    }
                    CombatEvent::BuffApply {
                        time, buff_name, ..
                    } => {
                        let ms = (time.as_f64() * 1000.0) as u64;
                        let localized = translator.translate_buff(&buff_name);

                        let should_print = if let Some(&last_time) = last_buff_times.get(&localized)
                        {
                            time.as_f64() - last_time >= 1.0 // Only print if 1s has passed since last effect
                        } else {
                            true
                        };

                        if should_print {
                            last_buff_times.insert(localized.clone(), time.as_f64());
                            out.push_str(&format!(
                                "    Effect {} : {}, {}\n",
                                localized,
                                ms,
                                ms + 150
                            ));
                        }
                    }
                    CombatEvent::BuffExpire {
                        time, buff_name, ..
                    } => {
                        let ms = (time.as_f64() * 1000.0) as u64;
                        let localized = translator.translate_buff(&buff_name);
                        out.push_str(&format!(
                            "    Effect {} 만료 : {}, {}\n",
                            localized,
                            ms,
                            ms + 150
                        ));
                    }
                    _ => {}
                }
            }
        }
        out.push_str("```\n");
        out
    }

    pub fn format_html(
        collector: &DataCollector,
        apl_script: &str,
        translator: &crate::i18n::Translator,
        stats: &Statistics,
    ) -> String {
        let mut json_events = String::from("[\n");
        let mut filtered_json_strs = Vec::new();
        let mut last_buff_times_html: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();

        for event in &collector.events {
            let json_str = match event {
                CombatEvent::Cast {
                    time,
                    source,
                    ability,
                    cost,
                    resource_type,
                } => Some(format!(
                    r#"  {{ "type": "cast", "time": {}, "source": "{}", "ability": "{:?}", "cost": {}, "resource_type": "{}" }}"#,
                    time.as_f64(),
                    source.0,
                    ability,
                    cost,
                    resource_type
                )),
                CombatEvent::Damage {
                    time,
                    source,
                    ability,
                    amount,
                    ..
                } => Some(format!(
                    r#"  {{ "type": "damage", "time": {}, "source": "{}", "ability": "{:?}", "amount": {} }}"#,
                    time.as_f64(),
                    source.0,
                    ability,
                    amount
                )),
                CombatEvent::Death { time, champion } => Some(format!(
                    r#"  {{ "type": "death", "time": {}, "source": "{}" }}"#,
                    time.as_f64(),
                    champion.0
                )),
                CombatEvent::BuffApply {
                    time,
                    target,
                    buff_name,
                } => {
                    let localized = translator.translate_buff(buff_name);
                    let key = format!("{}_{}", target.0, localized);
                    let should_print = if let Some(&last_time) = last_buff_times_html.get(&key) {
                        time.as_f64() - last_time >= 1.0
                    } else {
                        true
                    };

                    if should_print {
                        last_buff_times_html.insert(key, time.as_f64());
                        Some(format!(
                            r#"  {{ "type": "buff_apply", "time": {}, "target": "{}", "buff_name": "{}" }}"#,
                            time.as_f64(),
                            target.0,
                            localized
                        ))
                    } else {
                        None
                    }
                }
                CombatEvent::BuffExpire {
                    time,
                    target,
                    buff_name,
                } => {
                    let localized = translator.translate_buff(buff_name);
                    Some(format!(
                        r#"  {{ "type": "buff_expire", "time": {}, "target": "{}", "buff_name": "{}" }}"#,
                        time.as_f64(),
                        target.0,
                        localized
                    ))
                }
                CombatEvent::ResourceUpdate {
                    time,
                    target,
                    resource_type,
                    amount,
                    max,
                } => Some(format!(
                    r#"  {{ "type": "resource_update", "time": {}, "target": "{}", "resource_type": "{}", "amount": {}, "max": {} }}"#,
                    time.as_f64(),
                    target.0,
                    resource_type,
                    amount,
                    max
                )),

                CombatEvent::ItemAcquisition {
                    time,
                    target,
                    item_id,
                    item_name,
                } => {
                    let localized = translator.translate_buff(item_name);
                    Some(format!(
                        r#"  {{ "type": "item_acquisition", "time": {}, "target": "{}", "item_id": "{}", "item_name": "{}" }}"#,
                        time.as_f64(),
                        target.0,
                        item_id,
                        localized
                    ))
                }
                CombatEvent::Heal {
                    time,
                    source,
                    target,
                    amount,
                } => Some(format!(
                    r#"  {{ "type": "heal", "time": {}, "source": "{}", "target": "{}", "amount": {} }}"#,
                    time.as_f64(),
                    source.0,
                    target.0,
                    amount
                )),
            };
            if let Some(s) = json_str {
                filtered_json_strs.push(s);
            }
        }

        json_events.push_str(&filtered_json_strs.join(",\n"));
        json_events.push('\n');
        json_events.push(']');

        let mut json_items = String::from("{\n");

        // Sort champions by total damage dealt so the primary attacker appears first
        let mut sorted_champs: Vec<_> = collector.champion_items.iter().collect();
        sorted_champs.sort_by(|a, b| {
            let dmg_a = stats
                .champion_stats
                .get(a.0)
                .map_or(0.0, |s| s.total_damage);
            let dmg_b = stats
                .champion_stats
                .get(b.0)
                .map_or(0.0, |s| s.total_damage);
            dmg_b
                .partial_cmp(&dmg_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (i, (champ, items)) in sorted_champs.iter().enumerate() {
            let items_list = items
                .iter()
                .map(|(id, name)| {
                    let localized = translator.translate_buff(name);
                    format!("{{\"id\": \"{}\", \"name\": \"{}\"}}", id, localized)
                })
                .collect::<Vec<_>>()
                .join(", ");
            json_items.push_str(&format!(r#"  "{}": [{}]"#, champ.0, items_list));
            if i < sorted_champs.len() - 1 {
                json_items.push_str(",\n");
            } else {
                json_items.push('\n');
            }
        }
        json_items.push('}');

        let mut json_runes = String::from("{\n");
        let mut sorted_champs_runes: Vec<_> = collector.champion_runes.iter().collect();
        sorted_champs_runes.sort_by(|a, b| {
            let dmg_a = stats
                .champion_stats
                .get(a.0)
                .map_or(0.0, |s| s.total_damage);
            let dmg_b = stats
                .champion_stats
                .get(b.0)
                .map_or(0.0, |s| s.total_damage);
            dmg_b
                .partial_cmp(&dmg_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (i, (champ, runes)) in sorted_champs_runes.iter().enumerate() {
            let runes_list = runes
                .iter()
                .map(|(id, name, tree)| {
                    let localized = translator.translate_buff(name);
                    let tree_id = match tree.as_str() {
                        "Precision" => "8000",
                        "Domination" => "8100",
                        "Sorcery" => "8200",
                        "Resolve" => "8400",
                        "Inspiration" => "8300",
                        _ => "8000",
                    };
                    format!(
                        "{{\"id\": \"{}\", \"name\": \"{}\", \"tree_id\": \"{}\"}}",
                        id, localized, tree_id
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            json_runes.push_str(&format!(r#"  "{}": [{}]"#, champ.0, runes_list));
            if i < sorted_champs_runes.len() - 1 {
                json_runes.push_str(",\n");
            } else {
                json_runes.push('\n');
            }
        }
        json_runes.push('}');

        let json_stats = serde_json::to_string(&collector.champion_initial_stats).unwrap_or_else(|_| "{}".to_string());
        let template = include_str!("report_template.html");
        let rune_trees_json = include_str!("runesReforged.json");
        template
            .replace("/* __EVENTS_JSON__ */", &json_events)
            .replace("<!-- __APL_SCRIPT__ -->", apl_script)
            .replace("/* __ITEMS_JSON__ */", &json_items)
            .replace("/* __RUNES_JSON__ */", &json_runes)
            .replace("/* __RUNE_TREES_JSON__ */", rune_trees_json)
            .replace("/* __STATS_JSON__ */", &json_stats)
    }
}
