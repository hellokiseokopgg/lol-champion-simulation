use lol_core::ability::Ability;
use lol_core::buff::{RefreshBehavior, StatusEffect};
use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};
use lol_core::damage::DamagePipeline;
use lol_core::event::SimContext;
use lol_core::stats::StatBlock;
use lol_core::types::{AbilitySlot, ChampionId, EffectId, DamageType};

// -----------------------------------------------------------------------------
// Buffs
// -----------------------------------------------------------------------------

pub struct DariusHemorrhage;
impl StatusEffect for DariusHemorrhage {
    fn id(&self) -> EffectId { EffectId("DariusHemorrhage".into()) }
    fn name(&self) -> &str { "Hemorrhage" }
    fn duration(&self) -> f64 { 5.0 }
    fn refresh_behavior(&self) -> RefreshBehavior { RefreshBehavior::AddStack }
    fn max_stacks(&self) -> u32 { 5 }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock { StatBlock::new() }
}

pub struct NoxianMight;
impl StatusEffect for NoxianMight {
    fn id(&self) -> EffectId { EffectId("NoxianMight".into()) }
    fn name(&self) -> &str { "Noxian Might" }
    fn duration(&self) -> f64 { 5.0 }
    fn refresh_behavior(&self) -> RefreshBehavior { RefreshBehavior::RefreshDuration }
    fn max_stacks(&self) -> u32 { 1 }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.attack_damage = 30.0; // Bonus AD from Noxian Might (simplified)
        stats
    }
}

pub struct DariusWBuff;
impl StatusEffect for DariusWBuff {
    fn id(&self) -> EffectId { EffectId("DariusWBuff".into()) }
    fn name(&self) -> &str { "Crippling Strike" }
    fn duration(&self) -> f64 { 4.0 }
    fn refresh_behavior(&self) -> RefreshBehavior { RefreshBehavior::RefreshDuration }
    fn max_stacks(&self) -> u32 { 1 }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock { StatBlock::new() }
}

pub struct DariusEPassive;
impl StatusEffect for DariusEPassive {
    fn id(&self) -> EffectId { EffectId("DariusEPassive".into()) }
    fn name(&self) -> &str { "Apprehend Passive" }
    fn duration(&self) -> f64 { 9999.0 }
    fn refresh_behavior(&self) -> RefreshBehavior { RefreshBehavior::Ignore }
    fn max_stacks(&self) -> u32 { 1 }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.armor_pen_percent = 0.15; // 15% armor pen
        stats
    }
}

fn apply_hemorrhage(ctx: &mut SimContext, target: &ChampionId, attacker: &ChampionId) {
    ctx.apply_buff(target, Box::new(DariusHemorrhage));
    
    // Check if 5 stacks
    if let Some(t) = ctx.champions.get(target) {
        if t.borrow().state().buffs.get_stacks_by_name("Hemorrhage", ctx.current_time) == 5 {
            ctx.apply_buff(attacker, Box::new(NoxianMight));
        }
    }
}

// -----------------------------------------------------------------------------
// Abilities
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct DariusQ;
impl Ability for DariusQ {
    fn slot(&self) -> AbilitySlot { AbilitySlot::Q }
    fn cast_time(&self) -> f64 { 0.75 }
    fn base_cooldown(&self, _level: u32) -> f64 { 9.0 }
    fn cost(&self, _level: u32) -> f64 { 30.0 }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let attacker_stats = {
            if let Some(a) = ctx.champions.get(actor) {
                a.borrow().state().stats.current.clone()
            } else { return; }
        };

        let defender_stats = {
            if let Some(d) = ctx.champions.get(target) {
                d.borrow().state().stats.current.clone()
            } else { return; }
        };

        let raw_damage = 50.0 + (attacker_stats.attack_damage * 1.05); // Decimate outer ring
        
        let damage_result = DamagePipeline::process(
            raw_damage,
            DamageType::Physical,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time, actor.clone(), target.clone(), AbilitySlot::Q, damage_result.final_damage, false,
            );
        }

        ctx.trigger_on_physical_damage(actor, target, &damage_result);

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d.borrow_mut().take_damage(damage_result.final_damage);
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
            }
        }
        
        // Heal
        if let Some(a) = ctx.champions.get(actor) {
            let mut champ = a.borrow_mut();
            let missing_hp = champ.state().health.max - champ.state().health.current;
            champ.state_mut().health.restore(missing_hp * 0.15);
        }
        
        apply_hemorrhage(ctx, target, actor);
    }
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
}

#[derive(Clone)]
pub struct DariusW;
impl Ability for DariusW {
    fn slot(&self) -> AbilitySlot { AbilitySlot::W }
    fn cast_time(&self) -> f64 { 0.0 }
    fn base_cooldown(&self, _level: u32) -> f64 { 5.0 }
    fn cost(&self, _level: u32) -> f64 { 30.0 }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, _target: &ChampionId) {
        ctx.apply_buff(actor, Box::new(DariusWBuff));
        if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow_mut().state_mut().abilities.reset_cooldown(AbilitySlot::AutoAttack);
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
}

#[derive(Clone)]
pub struct DariusE;
impl Ability for DariusE {
    fn slot(&self) -> AbilitySlot { AbilitySlot::E }
    fn cast_time(&self) -> f64 { 0.25 }
    fn base_cooldown(&self, _level: u32) -> f64 { 24.0 }
    fn cost(&self, _level: u32) -> f64 { 45.0 }
    fn execute(&self, _ctx: &mut SimContext, _actor: &ChampionId, _target: &ChampionId) {
        // Apprehend pull effect (mostly CC, no damage)
    }
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
}

#[derive(Clone)]
pub struct DariusR;
impl Ability for DariusR {
    fn slot(&self) -> AbilitySlot { AbilitySlot::R }
    fn cast_time(&self) -> f64 { 0.25 } // Fast animation
    fn base_cooldown(&self, _level: u32) -> f64 { 120.0 }
    fn cost(&self, _level: u32) -> f64 { 100.0 }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let attacker_stats = {
            if let Some(a) = ctx.champions.get(actor) {
                a.borrow().state().stats.current.clone()
            } else { return; }
        };

        let stacks = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().buffs.get_stacks_by_name("Hemorrhage", ctx.current_time)
        } else { 0 };

        let raw_damage = 100.0 + (attacker_stats.attack_damage * 0.75);
        let final_damage = raw_damage * (1.0 + (0.2 * stacks as f64)); // 20% more damage per stack

        // True damage, ignores mitigation
        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time, actor.clone(), target.clone(), AbilitySlot::R, final_damage, false,
            );
        }

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d.borrow_mut().take_damage(final_damage);
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
                // Reset cooldown
                if let Some(a) = ctx.champions.get(actor) {
                    a.borrow_mut().state_mut().abilities.reset_cooldown(AbilitySlot::R);
                }
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
}

#[derive(Clone)]
pub struct DariusAutoAttack;
impl Ability for DariusAutoAttack {
    fn slot(&self) -> AbilitySlot { AbilitySlot::AutoAttack }
    fn cast_time(&self) -> f64 { 0.25 }
    fn base_cooldown(&self, _level: u32) -> f64 { 1.0 }
    fn cost(&self, _level: u32) -> f64 { 0.0 }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let mut has_w_buff = false;
        let attacker_stats = {
            if let Some(a) = ctx.champions.get(actor) {
                let mut champ = a.borrow_mut();
                if champ.state().buffs.has_buff_by_name("Crippling Strike", ctx.current_time) {
                    has_w_buff = true;
                    champ.state_mut().buffs.remove_effect(&EffectId("DariusWBuff".into()));
                }
                champ.state().stats.current.clone()
            } else { return; }
        };

        let defender_stats = {
            if let Some(d) = ctx.champions.get(target) {
                d.borrow().state().stats.current.clone()
            } else { return; }
        };

        let mut raw_damage = attacker_stats.attack_damage;
        if has_w_buff {
            raw_damage += attacker_stats.attack_damage * 0.40; // 40% total AD bonus
        }

        let damage_result = DamagePipeline::process(
            raw_damage,
            DamageType::Physical,
            true, // is_auto_attack
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time, actor.clone(), target.clone(), AbilitySlot::AutoAttack, damage_result.final_damage, false,
            );
        }

        ctx.trigger_on_hit(actor, target, &damage_result);
        ctx.trigger_on_physical_damage(actor, target, &damage_result);

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d.borrow_mut().take_damage(damage_result.final_damage);
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
            }
        }
        
        apply_hemorrhage(ctx, target, actor);
    }
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
}

// -----------------------------------------------------------------------------
// Module & Instance
// -----------------------------------------------------------------------------

pub struct DariusModule;

impl ChampionModule for DariusModule {
    fn id(&self) -> &str { "Darius" }

    fn create_instance(&self, mut config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let rune_stats = config.rune_page.aggregate_stats();
        let item_stats = config.item_build.aggregate_stats();
        let mut item_effects = Vec::new();
        for item in &mut config.item_build.items {
            item_effects.append(&mut item.effects);
        }

        let mut state = ChampionState::new(config.level, config.base_stats.clone(), config.growth_stats.clone(), lol_core::types::ResourceType::Mana, rune_stats, item_stats, item_effects);
        
        // Setup base stats slightly for simulation level
        state.stats.base.attack_speed_ratio = Some(0.625);
        state.stats.base.attack_speed = 0.625;
        state.stats.base.armor += 20.0;
        state.stats.base.magic_resist += 20.0;
        
        // Passive armor pen
        state.buffs.apply_effect(Box::new(DariusEPassive), lol_core::types::SimTime::new(0.0));
        
        let buffs_stats = state.buffs.aggregate_stats();
        state.stats.recalculate_current(&buffs_stats);
        
        if let Some(q) = state.abilities.get_state_mut(AbilitySlot::Q) { q.level = 5; }
        if let Some(w) = state.abilities.get_state_mut(AbilitySlot::W) { w.level = 5; }
        if let Some(e) = state.abilities.get_state_mut(AbilitySlot::E) { e.level = 5; }
        if let Some(r) = state.abilities.get_state_mut(AbilitySlot::R) { r.level = 3; }

        Box::new(DariusInstance {
            state,
            abilities: vec![
                Box::new(DariusQ),
                Box::new(DariusW),
                Box::new(DariusE),
                Box::new(DariusR),
                Box::new(DariusAutoAttack),
            ],
        })
    }
}

pub struct DariusInstance {
    state: ChampionState,
    abilities: Vec<Box<dyn Ability>>,
}

impl ChampionInstance for DariusInstance {
    fn state(&self) -> &ChampionState { &self.state }
    fn state_mut(&mut self) -> &mut ChampionState { &mut self.state }
    fn update_stats(&mut self) {
        let buffs_stats = self.state.buffs.aggregate_stats();
        self.state.stats.recalculate_current(&buffs_stats);
    }
    
    fn get_ability(&self, slot: AbilitySlot) -> Option<&dyn Ability> {
        self.abilities.iter().find(|a| a.slot() == slot).map(|a| a.as_ref())
    }
    
    fn take_damage(&mut self, amount: f64) -> bool {
        self.state.health.reduce(amount)
    }
}
