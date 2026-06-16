use lol_core::champion::ChampionInstance;
use lol_core::event::SimContext;
use lol_core::types::AbilitySlot;

#[derive(Debug, Clone)]
pub enum Expression {
    CooldownReady(AbilitySlot),
    HealthPctLessThan(f64),
    TargetHealthPctLessThan(f64),
    HasBuff(String),
    NotHasBuff(String),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();
        // A very simple parser that handles only flat expressions without parentheses
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

        // Parse base conditions
        if input.starts_with("cooldown.") && input.ends_with(".ready") {
            let slot_str = &input["cooldown.".len()..input.len() - ".ready".len()];
            let slot = match slot_str {
                "Q" => AbilitySlot::Q,
                "W" => AbilitySlot::W,
                "E" => AbilitySlot::E,
                "R" => AbilitySlot::R,
                _ => return Err(format!("Unknown ability slot in cooldown condition: {}", slot_str)),
            };
            return Ok(Expression::CooldownReady(slot));
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
            Expression::HasBuff(name) => {
                champion.state().buffs.has_buff_by_name(name, ctx.current_time)
            }
            Expression::NotHasBuff(name) => {
                !champion.state().buffs.has_buff_by_name(name, ctx.current_time)
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
