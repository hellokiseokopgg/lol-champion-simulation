use lol_core::ability::Ability;
use lol_core::buff::{RefreshBehavior, StatusEffect};
use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};
use lol_core::damage::DamagePipeline;
use lol_core::event::{SimContext, SimEvent};
use lol_core::stats::StatBlock;
use lol_core::types::{AbilitySlot, ChampionId, DamageType, EffectId, CCType, SimTime};
use std::cell::RefCell;
use std::rc::Rc;

pub struct AhriModule;

impl ChampionModule for AhriModule {
    fn id(&self) -> &str {
        "Ahri"
    }

    fn create_instance(&self, mut config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let rune_stats = config.rune_page.aggregate_stats();
        let item_stats = config.item_build.aggregate_stats();
        let mut item_effects = Vec::new();
        for item in &mut config.item_build.items {
            item_effects.append(&mut item.effects);
        }

        let mut state = ChampionState::new(
            config.level,
            config.base_stats.clone(),
            config.growth_stats.clone(),
            lol_core::types::ResourceType::Mana,
            rune_stats,
            item_stats,
            item_effects,
        );

        state.rune_manager.register_runes(&config.rune_page, true);

        // Initialize abilities to rank 5 for testing (except R to 3)
        if let Some(q) = state.abilities.get_state_mut(AbilitySlot::Q) {
            q.level = 5;
        }
        if let Some(w) = state.abilities.get_state_mut(AbilitySlot::W) {
            w.level = 5;
        }
        if let Some(e) = state.abilities.get_state_mut(AbilitySlot::E) {
            e.level = 5;
        }
        if let Some(r) = state.abilities.get_state_mut(AbilitySlot::R) {
            r.level = 3;
        }

        let custom_state = Rc::new(RefCell::new(AhriCustomState {
            essence_theft_stacks: 0,
            spirit_rush_charges: 0,
            spirit_rush_window_end: 0.0,
        }));

        let mut abilities: Vec<Box<dyn Ability>> = vec![
            Box::new(AhriAutoAttack),
            Box::new(AhriQ),
            Box::new(AhriW),
            Box::new(AhriE),
            Box::new(AhriR { custom_state: custom_state.clone() }),
        ];

        // Register active items dynamically
        for effect in state.items.effects() {
            if let Some(active) = effect.active_ability() {
                state.abilities.register_ability(active.slot(), 1);
                abilities.push(active);
            }
        }

        Box::new(AhriInstance {
            state,
            _config: config,
            abilities,
            custom_state,
        })
    }
}

pub struct AhriCustomState {
    pub essence_theft_stacks: u32,
    pub spirit_rush_charges: u32,
    pub spirit_rush_window_end: f64,
}

pub struct AhriInstance {
    pub state: ChampionState,
    pub _config: ChampionConfig,
    pub abilities: Vec<Box<dyn Ability>>,
    pub custom_state: Rc<RefCell<AhriCustomState>>,
}

impl ChampionInstance for AhriInstance {
    fn state(&self) -> &ChampionState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ChampionState {
        &mut self.state
    }

    fn update_stats(&mut self, time: lol_core::types::SimTime) {
        // 1. Recalculate base stats using growth logic
        let level = self.state.level as f64;
        let growth_multiplier = (level - 1.0) * (0.7025 + 0.0175 * (level - 1.0));
        let mut new_base = self.state.base_stats.clone();
        new_base.health += self.state.growth_stats.health * growth_multiplier;
        new_base.mana += self.state.growth_stats.mana * growth_multiplier;
        new_base.health_regen += self.state.growth_stats.health_regen * growth_multiplier;
        new_base.mana_regen += self.state.growth_stats.mana_regen * growth_multiplier;
        new_base.armor += self.state.growth_stats.armor * growth_multiplier;
        new_base.magic_resist += self.state.growth_stats.magic_resist * growth_multiplier;
        new_base.attack_damage += self.state.growth_stats.attack_damage * growth_multiplier;

        let as_ratio = self
            .state
            .base_stats
            .attack_speed_ratio
            .unwrap_or(self.state.base_stats.attack_speed);
        let bonus_as_from_growth = self.state.growth_stats.attack_speed * growth_multiplier;
        new_base.attack_speed += as_ratio * bonus_as_from_growth;

        self.state.stats.base = new_base;
        self.state.current_time = time;

        // 2. Recalculate initial stats (Base + Runes + Items)
        let bonus = self.state.rune_stats.clone() + self.state.item_stats.clone();
        self.state.stats.recalculate_initial(&bonus);

        // 3. Recalculate current stats (Initial + Buffs)
        let mut total_bonus = self.state.buffs.aggregate_stats();
        let level = self.state.level;
        let hp_ratio = if self.state.stats.current.health > 0.0 {
            self.state.health.current / self.state.stats.current.health
        } else {
            1.0
        };
        total_bonus = total_bonus
            + self
                .state
                .rune_manager
                .get_bonus_stats(time, &self.state.stats.base, level, hp_ratio);
        self.state.stats.recalculate_current(&total_bonus);
    }

    fn get_ability(&self, slot: AbilitySlot) -> Option<&dyn Ability> {
        self.abilities
            .iter()
            .find(|a| a.slot() == slot)
            .map(|a| a.as_ref())
    }

    fn take_damage(&mut self, amount: f64) -> lol_core::types::TakeDamageResult {
        let is_dead = self.state.health.reduce(amount);
        lol_core::types::TakeDamageResult {
            actual_damage: amount,
            is_dead,
        }
    }

    fn on_damage_dealt(
        &mut self,
        time: SimTime,
        amount: f64,
        is_ability: bool,
        slot: AbilitySlot,
    ) -> Vec<lol_core::rune_manager::RuneEvent> {
        let state = self.state_mut();
        let level = state.level;
        let stats = state.stats.current.clone();
        let mut events = state
            .rune_manager
            .on_damage_dealt(time, amount, is_ability, slot, &stats, level);

        // Track spell hits for Essence Theft (only for Q, W, E, R)
        if is_ability && (slot == AbilitySlot::Q || slot == AbilitySlot::W || slot == AbilitySlot::E || slot == AbilitySlot::R) {
            let mut custom = self.custom_state.borrow_mut();
            if custom.essence_theft_stacks >= 8 {
                // Next spell hit consumes stacks to heal
                let heal_amount = 3.0 * level as f64 + 0.2 * stats.ability_power;
                events.push(lol_core::rune_manager::RuneEvent::Healed { amount: heal_amount });
                custom.essence_theft_stacks = 0;
            } else {
                custom.essence_theft_stacks += 1;
            }
        }

        events
    }
}

// -----------------------------------------------------------------------------
// Buffs
// -----------------------------------------------------------------------------

pub struct AhriCharmDebuff;
impl StatusEffect for AhriCharmDebuff {
    fn id(&self) -> EffectId {
        EffectId("AhriCharmDebuff".into())
    }
    fn name(&self) -> &str {
        "Charm Amplification"
    }
    fn duration(&self) -> f64 {
        5.0
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::RefreshDuration
    }
    fn max_stacks(&self) -> u32 {
        1
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        // Amplifies damage by 20%
        stats.damage_reduction_percent = -0.20;
        stats
    }
    fn cc_type(&self) -> Option<CCType> {
        Some(CCType::Stun)
    }
}

// -----------------------------------------------------------------------------
// Abilities
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct AhriAutoAttack;
impl Ability for AhriAutoAttack {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::AutoAttack
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn windup_percent(&self) -> f64 {
        0.15
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        1.0
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let attacker_stats = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let damage_result = DamagePipeline::process(
            attacker_stats.attack_damage,
            DamageType::Physical,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time,
                actor.clone(),
                target.clone(),
                AbilitySlot::AutoAttack,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_hit(actor, target, &damage_result);
        ctx.trigger_on_physical_damage(actor, target, &damage_result);
        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, false, AbilitySlot::AutoAttack);

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct AhriQ;
impl Ability for AhriQ {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::Q
    }
    fn cast_time(&self) -> f64 {
        0.25
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        7.0
    }
    fn cost(&self, level: u32) -> f64 {
        55.0 + (level as f64 - 1.0) * 5.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            champ.state().abilities.get_state(AbilitySlot::Q).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };
        let cost = 55.0 + (level as f64 - 1.0) * 5.0;
        ctx.consume_resource(actor, cost);

        let base_damage = 40.0 + (level as f64 - 1.0) * 25.0;

        let attacker_stats = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        // 1. Outgoing magic damage
        let raw_damage = base_damage + 0.45 * attacker_stats.ability_power;
        let damage_result = DamagePipeline::process(
            raw_damage,
            DamageType::Magic,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time,
                actor.clone(),
                target.clone(),
                AbilitySlot::Q,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, AbilitySlot::Q);

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
            }
        }

        // 2. Schedule returning true damage 0.5s later
        ctx.new_events.push((
            0.5,
            Box::new(AhriQReturnEvent {
                attacker: actor.clone(),
                defender: target.clone(),
                base_damage,
            }),
        ));
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

pub struct AhriQReturnEvent {
    pub attacker: ChampionId,
    pub defender: ChampionId,
    pub base_damage: f64,
}

impl SimEvent for AhriQReturnEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut lol_core::event::EventManager) {
        let attacker_stats = if let Some(champ_ref) = ctx.champions.get(&self.attacker) {
            champ_ref.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(&self.defender) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let raw_damage = self.base_damage + 0.45 * attacker_stats.ability_power;

        let damage_result = DamagePipeline::process(
            raw_damage,
            DamageType::True,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time,
                self.attacker.clone(),
                self.defender.clone(),
                AbilitySlot::Q,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_damage_dealt(&self.attacker, damage_result.final_damage, true, AbilitySlot::Q);

        if let Some(d) = ctx.champions.get(&self.defender) {
            let is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: self.defender.clone() })));
            }
        }
    }
    fn name(&self) -> &str {
        "AhriQReturn"
    }
}

#[derive(Clone)]
pub struct AhriW;
impl Ability for AhriW {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::W
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 9.0,
            2 => 8.0,
            3 => 7.0,
            4 => 6.0,
            _ => 5.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        40.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            champ.state().abilities.get_state(AbilitySlot::W).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };
        ctx.consume_resource(actor, 40.0);

        let base_damage = 50.0 + (level as f64 - 1.0) * 25.0;

        let attacker_stats = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        // Total multiplier is 1.6x: Flame 1 is 1.0x, Flame 2 and 3 are 0.3x each.
        let first_damage = base_damage + 0.3 * attacker_stats.ability_power;
        let sub_damage = 0.3 * first_damage;

        for i in 0..3 {
            let dmg = if i == 0 { first_damage } else { sub_damage };
            let damage_result = DamagePipeline::process(
                dmg,
                DamageType::Magic,
                false,
                &attacker_stats,
                &defender_stats,
            );

            if let Some(recorder) = &ctx.recorder {
                recorder.borrow_mut().record_damage(
                    ctx.current_time,
                    actor.clone(),
                    target.clone(),
                    AbilitySlot::W,
                    damage_result.final_damage,
                    false,
                );
            }

            ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, AbilitySlot::W);

            if let Some(d) = ctx.champions.get(target) {
                let is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
                if is_dead {
                    ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
                    break;
                }
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct AhriE;
impl Ability for AhriE {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::E
    }
    fn cast_time(&self) -> f64 {
        0.25
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        12.0
    }
    fn cost(&self, _level: u32) -> f64 {
        70.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            champ.state().abilities.get_state(AbilitySlot::E).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };
        ctx.consume_resource(actor, 70.0);

        let base_damage = 80.0 + (level as f64 - 1.0) * 30.0;

        let attacker_stats = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let raw_damage = base_damage + 0.6 * attacker_stats.ability_power;
        let damage_result = DamagePipeline::process(
            raw_damage,
            DamageType::Magic,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time,
                actor.clone(),
                target.clone(),
                AbilitySlot::E,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, AbilitySlot::E);
        ctx.apply_buff(target, Box::new(AhriCharmDebuff));

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct AhriR {
    custom_state: Rc<RefCell<AhriCustomState>>,
}
impl Ability for AhriR {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::R
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 130.0,
            2 => 115.0,
            _ => 100.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        100.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            champ.state().abilities.get_state(AbilitySlot::R).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };

        let base_damage = 60.0 + (level as f64 - 1.0) * 30.0;

        let attacker_stats = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let _is_first = {
            let mut custom = self.custom_state.borrow_mut();
            let is_first = custom.spirit_rush_charges == 0 || ctx.current_time.as_f64() > custom.spirit_rush_window_end;

            if is_first {
                // First dash consumes 100 mana and sets up charges
                ctx.consume_resource(actor, 100.0);
                custom.spirit_rush_charges = 2; // 1st is consumed now, 2 remaining
                custom.spirit_rush_window_end = ctx.current_time.as_f64() + 15.0;

                // Schedule window expiry event
                ctx.new_events.push((
                    15.0,
                    Box::new(SpiritRushExpiryEvent {
                        actor: actor.clone(),
                        custom_state: self.custom_state.clone(),
                        base_cooldown: self.base_cooldown(level),
                    }),
                ));

                // Start static 1.0s cooldown
                if let Some(champ_ref) = ctx.champions.get(actor) {
                    let mut champ = champ_ref.borrow_mut();
                    if let Some(r_state) = champ.state_mut().abilities.get_state_mut(AbilitySlot::R) {
                        r_state.cooldown.start_cooldown(ctx.current_time, 1.0);
                    }
                }
            } else {
                // Subsequent dash
                custom.spirit_rush_charges -= 1;
                if custom.spirit_rush_charges == 0 {
                    // Out of charges, start the full cooldown
                    if let Some(champ_ref) = ctx.champions.get(actor) {
                        let mut champ = champ_ref.borrow_mut();
                        let ah = champ.state().stats.current.ability_haste;
                        let cdr = ah / (100.0 + ah);
                        let full_cooldown = self.base_cooldown(level) * (1.0 - cdr);
                        if let Some(r_state) = champ.state_mut().abilities.get_state_mut(AbilitySlot::R) {
                            r_state.cooldown.start_cooldown(ctx.current_time, full_cooldown);
                        }
                    }
                } else {
                    // Start static 1.0s cooldown
                    if let Some(champ_ref) = ctx.champions.get(actor) {
                        let mut champ = champ_ref.borrow_mut();
                        if let Some(r_state) = champ.state_mut().abilities.get_state_mut(AbilitySlot::R) {
                            r_state.cooldown.start_cooldown(ctx.current_time, 1.0);
                        }
                    }
                }
            }
            is_first
        };

        // Deal damage
        let raw_damage = base_damage + 0.35 * attacker_stats.ability_power;
        let damage_result = DamagePipeline::process(
            raw_damage,
            DamageType::Magic,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time,
                actor.clone(),
                target.clone(),
                AbilitySlot::R,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, AbilitySlot::R);

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

pub struct SpiritRushExpiryEvent {
    pub actor: ChampionId,
    pub custom_state: Rc<RefCell<AhriCustomState>>,
    pub base_cooldown: f64,
}

impl SimEvent for SpiritRushExpiryEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut lol_core::event::EventManager) {
        let mut custom = self.custom_state.borrow_mut();
        if custom.spirit_rush_charges > 0 {
            custom.spirit_rush_charges = 0;
            if let Some(champ_ref) = ctx.champions.get(&self.actor) {
                let mut champ = champ_ref.borrow_mut();
                let ah = champ.state().stats.current.ability_haste;
                let cdr = ah / (100.0 + ah);
                let full_cooldown = self.base_cooldown * (1.0 - cdr);
                if let Some(r_state) = champ.state_mut().abilities.get_state_mut(AbilitySlot::R) {
                    r_state.cooldown.start_cooldown(ctx.current_time, full_cooldown);
                }
            }
        }
    }
    fn name(&self) -> &str {
        "SpiritRushExpiry"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lol_core::stats::StatBlock;
    use lol_core::types::DamageType;
    use lol_core::buff::StatusEffect;

    #[test]
    fn test_charm_damage_amplification() {
        let attacker = StatBlock::new();
        let mut defender = StatBlock::new();

        // Baseline
        let res_magic = DamagePipeline::process(100.0, DamageType::Magic, false, &attacker, &defender);
        assert_eq!(res_magic.final_damage, 100.0);

        // Charm debuff applied
        let charm = AhriCharmDebuff;
        let mods = charm.stat_modifiers(1);
        defender.damage_reduction_percent = mods.damage_reduction_percent;

        let res_magic_amp = DamagePipeline::process(100.0, DamageType::Magic, false, &attacker, &defender);
        assert_eq!(res_magic_amp.final_damage, 120.0);

        // Charm amplifies true damage as well
        let res_true_amp = DamagePipeline::process(100.0, DamageType::True, false, &attacker, &defender);
        assert_eq!(res_true_amp.final_damage, 120.0);
    }
}
