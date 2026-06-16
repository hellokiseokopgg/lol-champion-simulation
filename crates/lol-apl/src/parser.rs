use crate::expression::Expression;
use lol_core::types::AbilitySlot;

#[derive(Debug, Clone)]
pub struct ActionPriority {
    pub slot: AbilitySlot,
    pub condition: Option<Expression>,
}

#[derive(Debug, Clone, Default)]
pub struct ActionPriorityList {
    pub actions: Vec<ActionPriority>,
    pub items: Option<Vec<String>>,
}

impl ActionPriorityList {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut actions = Vec::new();
        let mut items = None;

        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Expected format: actions+=/Q,if=condition
            // Or just: actions+=/Q
            // Or items: items=6631,3033
            if line.starts_with("items=") {
                let items_str = &line["items=".len()..];
                items = Some(items_str.split(',').map(|s| s.trim().to_string()).collect());
                continue;
            }

            if !line.starts_with("actions+=/") {
                return Err(format!("Invalid line format: {}", line));
            }

            let parts: Vec<&str> = line["actions+=/".len()..].splitn(2, ',').collect();
            let slot_str = parts[0];
            let slot = match slot_str {
                "Q" => AbilitySlot::Q,
                "W" => AbilitySlot::W,
                "E" => AbilitySlot::E,
                "R" => AbilitySlot::R,
                "AutoAttack" | "AA" => AbilitySlot::AutoAttack,
                s if s.starts_with("Item:") => {
                    let id_str = &s["Item:".len()..];
                    let id = id_str.parse::<u32>().map_err(|_| format!("Invalid item ID: {}", id_str))?;
                    AbilitySlot::Item(id)
                }
                _ => return Err(format!("Unknown ability slot: {}", slot_str)),
            };

            let condition = if parts.len() > 1 {
                let cond_str = parts[1];
                if !cond_str.starts_with("if=") {
                    return Err(format!("Invalid condition format: {}", cond_str));
                }
                Some(Expression::parse(&cond_str["if=".len()..])?)
            } else {
                None
            };

            actions.push(ActionPriority { slot, condition });
        }

        Ok(Self { actions, items })
    }
}
