## 2026-06-17T07:09:19Z

Create the integration test directory and files to implement Tier 1 tests for the Electrocute and Press the Attack runes:

1. Create directory `tests/` at the project root if it does not exist.
2. Create `tests/common/mod.rs` with the following helper functions:
```rust
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

static FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GanttEvent {
    Cast { ability: String, time_ms: u64 },
    Damage { ability: String, time_ms: u64 },
    BuffApply { buff_name: String, time_ms: u64 },
    BuffExpire { buff_name: String, time_ms: u64 },
}

pub fn run_with_apl(
    champion_a: &str,
    champion_b: &str,
    runes: Option<&str>,
    items: Option<&str>,
    apl_content: &str,
    extra_args: &[&str],
) -> String {
    let count = FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let apl_filename = format!("temp_test_{}_{}.apl", champion_a, count);
    
    std::fs::write(&apl_filename, apl_content).expect("Failed to write temp APL file");
    
    let binary = env!("CARGO_BIN_EXE_lol-champion-simulation");
    
    let mut args = vec![
        "simulate",
        "-a", champion_a,
        "-b", champion_b,
        "--apl", &apl_filename,
    ];
    
    if let Some(r) = runes {
        args.push("--runes");
        args.push(r);
    }
    if let Some(i) = items {
        args.push("--items");
        args.push(i);
    }
    
    args.extend_from_slice(extra_args);
    
    let output = Command::new(binary)
        .args(&args)
        .output()
        .expect("Failed to execute simulation binary");
        
    let _ = std::fs::remove_file(&apl_filename);
    
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn parse_total_damage(stdout: &str, champion: &str) -> Option<f64> {
    let mut in_champion = false;
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Champion:") {
            let name = line["Champion:".len()..].trim().trim_matches('"');
            in_champion = name == champion;
            continue;
        }
        if in_champion {
            if line.starts_with("Total Damage:") {
                let val_str = line["Total Damage:".len()..].trim();
                if let Ok(val) = val_str.parse::<f64>() {
                    return Some(val);
                }
            }
            if line.starts_with("-----") {
                in_champion = false;
            }
        }
    }
    None
}

pub fn parse_breakdown(stdout: &str, champion: &str) -> HashMap<String, (f64, f64)> {
    let mut breakdown = HashMap::new();
    let mut in_champion = false;
    let mut in_breakdown = false;
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Champion:") {
            let name = line["Champion:".len()..].trim().trim_matches('"');
            in_champion = name == champion;
            continue;
        }
        if in_champion {
            if line.starts_with("Ability Breakdown:") {
                in_breakdown = true;
                continue;
            }
            if in_breakdown {
                if line.starts_with("-----") || line.is_empty() {
                    in_breakdown = false;
                    in_champion = false;
                    continue;
                }
                if let Some((ability, rest)) = line.split_once(':') {
                    let rest = rest.trim();
                    if let Some((dmg_str, pct_str)) = rest.split_once('(') {
                        let dmg_val = dmg_str.trim().parse::<f64>().unwrap_or(0.0);
                        let pct_val = pct_str.trim_end_matches(')').trim_end_matches('%').trim().parse::<f64>().unwrap_or(0.0);
                        breakdown.insert(ability.trim().to_string(), (dmg_val, pct_val));
                    }
                }
            }
        }
    }
    breakdown
}

pub fn parse_gantt_events(stdout: &str, champion: &str) -> Vec<GanttEvent> {
    let mut events = Vec::new();
    let mut in_section = false;
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("section ") {
            let section_name = line["section ".len()..].trim();
            in_section = section_name == champion;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.starts_with("section ") || line.starts_with("```") {
            in_section = false;
            continue;
        }
        
        if line.starts_with("Cast ") {
            let parts: Vec<&str> = line["Cast ".len()..].split(':').collect();
            if parts.len() == 2 {
                let ability = parts[0].trim().to_string();
                let times: Vec<&str> = parts[1].split(',').collect();
                if let Ok(start_ms) = times[0].trim().parse::<u64>() {
                    events.push(GanttEvent::Cast { ability, time_ms: start_ms });
                }
            }
        } else if line.starts_with("Dmg ") {
            let parts: Vec<&str> = line["Dmg ".len()..].split(':').collect();
            if parts.len() == 2 {
                let ability = parts[0].trim().to_string();
                let times: Vec<&str> = parts[1].split(',').collect();
                if let Ok(start_ms) = times[0].trim().parse::<u64>() {
                    events.push(GanttEvent::Damage { ability, time_ms: start_ms });
                }
            }
        } else if line.starts_with("Effect ") {
            let parts: Vec<&str> = line["Effect ".len()..].split(':').collect();
            if parts.len() == 2 {
                let raw_buff = parts[0].trim();
                let times: Vec<&str> = parts[1].split(',').collect();
                if let Ok(start_ms) = times[0].trim().parse::<u64>() {
                    if raw_buff.ends_with(" 만료") {
                        let buff_name = raw_buff[..raw_buff.len() - " 만료".len()].trim().to_string();
                        events.push(GanttEvent::BuffExpire { buff_name, time_ms: start_ms });
                    } else {
                        events.push(GanttEvent::BuffApply { buff_name: raw_buff.to_string(), time_ms: start_ms });
                    }
                }
            }
        }
    }
    events
}
```

3. Create `tests/tier1_feature.rs` with the Tier 1 tests (5 Electrocute, 5 PTA tests). Ensure they use standard `#[test]` syntax and correctly load/execute.
Wait, since the new runes are NOT fully implemented yet, the tests might fail to pass when run, but they should compile correctly! Run `cargo test --test tier1_feature` to verify if they compile. (If they fail due to runes not found, that is expected, but compile errors in the test code itself must be resolved).
