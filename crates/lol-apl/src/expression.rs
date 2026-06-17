use lol_core::champion::ChampionInstance;
use lol_core::event::SimContext;
use lol_core::types::AbilitySlot;

#[derive(Debug, Clone)]
pub enum Expression {
    CooldownReady(AbilitySlot),
    HealthPctLessThan(f64),
    TargetHealthPctLessThan(f64),
    TargetCasting(AbilitySlot),
    HasBuff(String),
    NotHasBuff(String),
    BuffStacksGreaterThan(String, u32),
    BuffStacksLessThan(String, u32),
    TargetBuffStacksGreaterThan(String, u32),
    TargetBuffStacksLessThan(String, u32),
    ResourcePctLessThan(f64),
    ResourcePctGreaterThan(f64),
    ResourceCurrentGreaterThan(f64),
    ResourceCurrentLessThan(f64),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if let Some((left, right)) = input.split_once('&') {
            return Ok(Expression::And(
                Box::new(Self::parse(left)?),
                Box::new(Self::parse(right)?),
            ));
        }
        if let Some((left, right)) = input.split_once('|') {
            return Ok(Expression::Or(
                Box::new(Self::parse(left)?),
                Box::new(Self::parse(right)?),
            ));
        }

        if input.starts_with("cooldown.") && input.ends_with(".ready") {
            let slot_str = &input["cooldown.".len()..input.len() - ".ready".len()];
            let slot = match slot_str {
                "Q" => AbilitySlot::Q,
                "W" => AbilitySlot::W,
                "E" => AbilitySlot::E,
                "R" => AbilitySlot::R,
                "AutoAttack" | "AA" => AbilitySlot::AutoAttack,
                s if s.starts_with("Item:") => {
                    let id_str = &s["Item:".len()..];
                    let id = id_str.parse::<u32>().map_err(|_| format!("Invalid item ID in expression: {}", id_str))?;
                    AbilitySlot::Item(id)
                }
                _ => return Err(format!("Unknown ability slot in cooldown condition: {}", slot_str)),
            };
            return Ok(Expression::CooldownReady(slot));
        }

        if input.starts_with("target.casting.") {
            let slot_str = &input["target.casting.".len()..];
            let slot = match slot_str {
                "Q" => AbilitySlot::Q,
                "W" => AbilitySlot::W,
                "E" => AbilitySlot::E,
                "R" => AbilitySlot::R,
                "AutoAttack" | "AA" => AbilitySlot::AutoAttack,
                s if s.starts_with("Item:") => {
                    let id_str = &s["Item:".len()..];
                    let id = id_str.parse::<u32>().map_err(|_| format!("Invalid item ID in expression: {}", id_str))?;
                    AbilitySlot::Item(id)
                }
                _ => return Err(format!("Unknown ability slot in casting condition: {}", slot_str)),
            };
            return Ok(Expression::TargetCasting(slot));
        }

        if input.starts_with("health.pct<") {
            let val_str = &input["health.pct<".len()..];
            let val = val_str.parse::<f64>().map_err(|_| format!("Invalid health percent: {}", val_str))?;
            return Ok(Expression::HealthPctLessThan(val));
        }

        if input.starts_with("target.health.pct<") {
            let val_str = &input["target.health.pct<".len()..];
            let val = val_str.parse::<f64>().map_err(|_| format!("Invalid health percent: {}", val_str))?;
            return Ok(Expression::TargetHealthPctLessThan(val));
        }

        if input.starts_with("buff.") && input.ends_with(".up") {
            let buff_name = &input["buff.".len()..input.len() - ".up".len()];
            return Ok(Expression::HasBuff(buff_name.to_string()));
        }

        if input.starts_with("buff.") && input.ends_with(".down") {
            let buff_name = &input["buff.".len()..input.len() - ".down".len()];
            return Ok(Expression::NotHasBuff(buff_name.to_string()));
        }

        if input.starts_with("buff.") && input.contains(".stack>") {
            let parts: Vec<&str> = input["buff.".len()..].split(".stack>").collect();
            if parts.len() == 2 {
                let val = parts[1].parse::<u32>().map_err(|_| format!("Invalid stack count: {}", parts[1]))?;
                return Ok(Expression::BuffStacksGreaterThan(parts[0].to_string(), val));
            }
        }

        if input.starts_with("buff.") && input.contains(".stack<") {
            let parts: Vec<&str> = input["buff.".len()..].split(".stack<").collect();
            if parts.len() == 2 {
                let val = parts[1].parse::<u32>().map_err(|_| format!("Invalid stack count: {}", parts[1]))?;
                return Ok(Expression::BuffStacksLessThan(parts[0].to_string(), val));
            }
        }

        if input.starts_with("target.buff.") && input.contains(".stack>") {
            let parts: Vec<&str> = input["target.buff.".len()..].split(".stack>").collect();
            if parts.len() == 2 {
                let val = parts[1].parse::<u32>().map_err(|_| format!("Invalid stack count: {}", parts[1]))?;
                return Ok(Expression::TargetBuffStacksGreaterThan(parts[0].to_string(), val));
            }
        }

        if input.starts_with("target.buff.") && input.contains(".stack<") {
            let parts: Vec<&str> = input["target.buff.".len()..].split(".stack<").collect();
            if parts.len() == 2 {
                let val = parts[1].parse::<u32>().map_err(|_| format!("Invalid stack count: {}", parts[1]))?;
                return Ok(Expression::TargetBuffStacksLessThan(parts[0].to_string(), val));
            }
        }

        if input.starts_with("resource.pct<") {
            let val = input["resource.pct<".len()..].parse::<f64>().map_err(|_| format!("Invalid resource pct: {}", input))?;
            return Ok(Expression::ResourcePctLessThan(val));
        }

        if input.starts_with("resource.pct>") {
            let val = input["resource.pct>".len()..].parse::<f64>().map_err(|_| format!("Invalid resource pct: {}", input))?;
            return Ok(Expression::ResourcePctGreaterThan(val));
        }

        if input.starts_with("resource.current<") {
            let val = input["resource.current<".len()..].parse::<f64>().map_err(|_| format!("Invalid resource current: {}", input))?;
            return Ok(Expression::ResourceCurrentLessThan(val));
        }

        if input.starts_with("resource.current>") {
            let val = input["resource.current>".len()..].parse::<f64>().map_err(|_| format!("Invalid resource current: {}", input))?;
            return Ok(Expression::ResourceCurrentGreaterThan(val));
        }

        Err(format!("Unknown expression: {}", input))
    }

    pub fn evaluate(&self, ctx: &SimContext, champion: &dyn ChampionInstance, target: &dyn ChampionInstance) -> bool {
        match self {
            Expression::CooldownReady(slot) => {
                champion.state().abilities.is_ready(*slot, ctx.current_time)
            }
            Expression::HealthPctLessThan(pct) => {
                let health = &champion.state().health;
                if health.max > 0.0 {
                    (health.current / health.max) * 100.0 < *pct
                } else {
                    false
                }
            }
            Expression::TargetHealthPctLessThan(pct) => {
                let health = &target.state().health;
                if health.max > 0.0 {
                    (health.current / health.max) * 100.0 < *pct
                } else {
                    false
                }
            }
            Expression::TargetCasting(slot) => {
                target.state().casting == Some(*slot)
            }
            Expression::HasBuff(name) => {
                champion.state().buffs.has_buff_by_name(name, ctx.current_time)
            }
            Expression::NotHasBuff(name) => {
                !champion.state().buffs.has_buff_by_name(name, ctx.current_time)
            }
            Expression::BuffStacksGreaterThan(name, val) => {
                champion.state().buffs.get_stacks_by_name(name, ctx.current_time) > *val
            }
            Expression::BuffStacksLessThan(name, val) => {
                champion.state().buffs.get_stacks_by_name(name, ctx.current_time) < *val
            }
            Expression::TargetBuffStacksGreaterThan(name, val) => {
                target.state().buffs.get_stacks_by_name(name, ctx.current_time) > *val
            }
            Expression::TargetBuffStacksLessThan(name, val) => {
                target.state().buffs.get_stacks_by_name(name, ctx.current_time) < *val
            }
            Expression::ResourcePctLessThan(pct) => {
                let resource = &champion.state().resource;
                if resource.max > 0.0 {
                    (resource.current / resource.max) * 100.0 < *pct
                } else {
                    false
                }
            }
            Expression::ResourcePctGreaterThan(pct) => {
                let resource = &champion.state().resource;
                if resource.max > 0.0 {
                    (resource.current / resource.max) * 100.0 > *pct
                } else {
                    false
                }
            }
            Expression::ResourceCurrentLessThan(val) => {
                champion.state().resource.current < *val
            }
            Expression::ResourceCurrentGreaterThan(val) => {
                champion.state().resource.current > *val
            }
            Expression::And(left, right) => {
                left.evaluate(ctx, champion, target) && right.evaluate(ctx, champion, target)
            }
            Expression::Or(left, right) => {
                left.evaluate(ctx, champion, target) || right.evaluate(ctx, champion, target)
            }
        }
    }
}
