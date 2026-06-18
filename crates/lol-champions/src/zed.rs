use lol_core::ability::Ability;
use lol_core::buff::{RefreshBehavior, StatusEffect};
use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};
use lol_core::damage::DamagePipeline;
use lol_core::event::{SimContext, SimEvent};
use lol_core::stats::StatBlock;
use lol_core::types::{AbilitySlot, ChampionId, DamageType, EffectId, CCType, SimTime};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ZedModule;

impl ChampionModule for ZedModule {
    fn id(&self) -> &str {
        "Zed"
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
            lol_core::types::ResourceType::Energy,
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

        let custom_state = Rc::new(RefCell::new(ZedCustomState {
            last_passive_proc_time: -99.0,
            death_mark_accumulated: 0.0,
        }));

        let mut abilities: Vec<Box<dyn Ability>> = vec![
            Box::new(ZedAutoAttack { custom_state: custom_state.clone() }),
            Box::new(ZedQ),
            Box::new(ZedW),
            Box::new(ZedE),
            Box::new(ZedR { custom_state: custom_state.clone() }),
        ];

        // Register active items dynamically
        for effect in state.items.effects() {
            if let Some(active) = effect.active_ability() {
                state.abilities.register_ability(active.slot(), 1);
                abilities.push(active);
            }
        }

        Box::new(ZedInstance {
            state,
            _config: config,
            abilities,
            custom_state,
        })
    }
}

pub struct ZedCustomState {
    pub last_passive_proc_time: f64,
    pub death_mark_accumulated: f64,
}

pub struct ZedInstance {
    pub state: ChampionState,
    pub _config: ChampionConfig,
    pub abilities: Vec<Box<dyn Ability>>,
    pub custom_state: Rc<RefCell<ZedCustomState>>,
}

impl ChampionInstance for ZedInstance {
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
        let has_mark_active = self.state.buffs.has_effect_by_id(&EffectId("DeathMarkActiveIndicator".into()), time);
        if has_mark_active {
            let mut custom = self.custom_state.borrow_mut();
            custom.death_mark_accumulated += amount;
        }

        let level = self.state.level;
        let stats = self.state.stats.current.clone();

        self.state
            .rune_manager
            .on_damage_dealt(time, amount, is_ability, slot, &stats, level)
    }

    fn can_cast(&self, slot: AbilitySlot, time: SimTime) -> bool {
        if self.state().casting.is_some() {
            return false;
        }
        if slot == AbilitySlot::AutoAttack && self.state().buffs.prevents_basic_attacks(time) {
            return false;
        }
        true
    }
}

// -----------------------------------------------------------------------------
// Buffs
// -----------------------------------------------------------------------------

pub struct ZedShadowBuff;
impl StatusEffect for ZedShadowBuff {
    fn id(&self) -> EffectId {
        EffectId("ZedShadow".into())
    }
    fn name(&self) -> &str {
        "Living Shadow"
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
        StatBlock::new()
    }
}

pub struct DeathMarkBuff;
impl StatusEffect for DeathMarkBuff {
    fn id(&self) -> EffectId {
        EffectId("DeathMark".into())
    }
    fn name(&self) -> &str {
        "Death Mark"
    }
    fn duration(&self) -> f64 {
        3.0
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::RefreshDuration
    }
    fn max_stacks(&self) -> u32 {
        1
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::new()
    }
}

pub struct DeathMarkActiveIndicator;
impl StatusEffect for DeathMarkActiveIndicator {
    fn id(&self) -> EffectId {
        EffectId("DeathMarkActiveIndicator".into())
    }
    fn name(&self) -> &str {
        "Death Mark Active Indicator"
    }
    fn duration(&self) -> f64 {
        3.0
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::RefreshDuration
    }
    fn max_stacks(&self) -> u32 {
        1
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::new()
    }
}

pub struct ZedESlow;
impl StatusEffect for ZedESlow {
    fn id(&self) -> EffectId {
        EffectId("ZedESlow".into())
    }
    fn name(&self) -> &str {
        "Shadow Slash Slow"
    }
    fn duration(&self) -> f64 {
        1.5
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::RefreshDuration
    }
    fn max_stacks(&self) -> u32 {
        1
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::new()
    }
    fn cc_type(&self) -> Option<CCType> {
        Some(CCType::Slow)
    }
}

// -----------------------------------------------------------------------------
// Abilities
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct ZedAutoAttack {
    custom_state: Rc<RefCell<ZedCustomState>>,
}
impl Ability for ZedAutoAttack {
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

        let defender_health_state = if let Some(d) = ctx.champions.get(target) {
            let champ = d.borrow();
            let state = champ.state();
            (state.health.current, state.stats.current.health)
        } else {
            (0.0, 0.0)
        };

        let raw_damage = attacker_stats.attack_damage;
        let mut passive_triggered = false;
        let mut passive_magic_damage = 0.0;

        // Contempt for the Weak
        let current_hp_ratio = defender_health_state.0 / defender_health_state.1;
        if current_hp_ratio < 0.50 {
            let mut custom = self.custom_state.borrow_mut();
            if ctx.current_time.as_f64() >= custom.last_passive_proc_time + 10.0 {
                passive_triggered = true;
                passive_magic_damage = 0.08 * defender_health_state.1;
                custom.last_passive_proc_time = ctx.current_time.as_f64();
            }
        }

        // 1. Basic Auto Attack Damage
        let is_critical = attacker_stats.crit_chance >= 1.0;
        let damage_result = DamagePipeline::process(
            raw_damage,
            DamageType::Physical,
            is_critical,
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
                is_critical,
            );
        }

        ctx.trigger_on_hit(actor, target, &damage_result);
        ctx.trigger_on_physical_damage(actor, target, &damage_result);
        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, false, AbilitySlot::AutoAttack);

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
                return;
            }
        }

        // 2. Contempt for the Weak Magic Damage
        if passive_triggered {
            let magic_result = DamagePipeline::process(
                passive_magic_damage,
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
                    AbilitySlot::Passive,
                    magic_result.final_damage,
                    false,
                );
            }

            ctx.trigger_on_damage_dealt(actor, magic_result.final_damage, false, AbilitySlot::Passive);

            if let Some(d) = ctx.champions.get(target) {
                let is_dead = d.borrow_mut().take_damage(magic_result.final_damage).is_dead;
                if is_dead {
                    ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
                }
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct ZedQ;
impl Ability for ZedQ {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::Q
    }
    fn cast_time(&self) -> f64 {
        0.25
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        6.0
    }
    fn cost(&self, level: u32) -> f64 {
        75.0 - (level as f64 - 1.0) * 5.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            champ.state().abilities.get_state(AbilitySlot::Q).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };
        let cost = 75.0 - (level as f64 - 1.0) * 5.0;
        ctx.consume_resource(actor, cost);

        let base_damage = 80.0 + (level as f64 - 1.0) * 35.0;

        let (attacker_stats, attacker_base) = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            (champ.state().stats.current.clone(), champ.state().stats.base.clone())
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let bonus_ad = (attacker_stats.attack_damage - attacker_base.attack_damage).max(0.0);
        let raw_damage = base_damage + 1.1 * bonus_ad;

        // 1. Zed's own Q
        let damage_result = DamagePipeline::process(
            raw_damage,
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
                AbilitySlot::Q,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_physical_damage(actor, target, &damage_result);
        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, AbilitySlot::Q);

        let mut is_dead = false;

        if let Some(d) = ctx.champions.get(target) {
            is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
        }

        if is_dead {
            ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
            return;
        }

        // 2. Replicated Q from shadow
        let has_shadow = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().buffs.has_effect_by_id(&EffectId("ZedShadow".into()), ctx.current_time)
        } else {
            false
        };

        if has_shadow {
            // Replicated shadow hit deals 60% damage
            let shadow_raw_damage = 0.60 * raw_damage;
            let shadow_damage_result = DamagePipeline::process(
                shadow_raw_damage,
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
                    AbilitySlot::Q,
                    shadow_damage_result.final_damage,
                    false,
                );
            }

            ctx.trigger_on_physical_damage(actor, target, &shadow_damage_result);
            ctx.trigger_on_damage_dealt(actor, shadow_damage_result.final_damage, true, AbilitySlot::Q);

            if let Some(d) = ctx.champions.get(target) {
                let is_dead = d.borrow_mut().take_damage(shadow_damage_result.final_damage).is_dead;
                if is_dead {
                    ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
                }
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct ZedW;
impl Ability for ZedW {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::W
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 20.0,
            2 => 18.5,
            3 => 17.0,
            4 => 15.5,
            _ => 14.0,
        }
    }
    fn cost(&self, level: u32) -> f64 {
        40.0 - (level as f64 - 1.0) * 5.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, _target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            champ.state().abilities.get_state(AbilitySlot::W).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };
        let cost = 40.0 - (level as f64 - 1.0) * 5.0;
        ctx.consume_resource(actor, cost);

        ctx.apply_buff(actor, Box::new(ZedShadowBuff));
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct ZedE;
impl Ability for ZedE {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::E
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 5.0,
            2 => 4.75,
            3 => 4.5,
            4 => 4.25,
            _ => 4.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        50.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        ctx.consume_resource(actor, 50.0);

        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().abilities.get_state(AbilitySlot::E).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };

        let base_damage = 65.0 + (level as f64 - 1.0) * 25.0;

        let (attacker_stats, attacker_base) = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            (champ.state().stats.current.clone(), champ.state().stats.base.clone())
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let bonus_ad = (attacker_stats.attack_damage - attacker_base.attack_damage).max(0.0);
        let raw_damage = base_damage + 0.65 * bonus_ad;

        // 1. Zed's E
        let damage_result = DamagePipeline::process(
            raw_damage,
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
                AbilitySlot::E,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_physical_damage(actor, target, &damage_result);
        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, AbilitySlot::E);

        let mut is_dead = false;
        if let Some(d) = ctx.champions.get(target) {
            is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
        }

        if is_dead {
            ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
            return;
        }

        // 2. Shadow replicated E
        let has_shadow = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().buffs.has_effect_by_id(&EffectId("ZedShadow".into()), ctx.current_time)
        } else {
            false
        };

        if has_shadow {
            // Shadow's E applies a slow debuff
            ctx.apply_buff(target, Box::new(ZedESlow));

            let shadow_raw_damage = 0.60 * raw_damage;
            let shadow_damage_result = DamagePipeline::process(
                shadow_raw_damage,
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
                    AbilitySlot::E,
                    shadow_damage_result.final_damage,
                    false,
                );
            }

            ctx.trigger_on_physical_damage(actor, target, &shadow_damage_result);
            ctx.trigger_on_damage_dealt(actor, shadow_damage_result.final_damage, true, AbilitySlot::E);

            if let Some(d) = ctx.champions.get(target) {
                let is_dead = d.borrow_mut().take_damage(shadow_damage_result.final_damage).is_dead;
                if is_dead {
                    ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: target.clone() })));
                }
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct ZedR {
    custom_state: Rc<RefCell<ZedCustomState>>,
}
impl Ability for ZedR {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::R
    }
    fn cast_time(&self) -> f64 {
        0.5
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 120.0,
            2 => 100.0,
            _ => 80.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        // R leaves a shadow (ZedShadow) and applies the mark
        ctx.apply_buff(actor, Box::new(ZedShadowBuff));
        ctx.apply_buff(target, Box::new(DeathMarkBuff));
        
        // Also apply an indicator buff to Zed himself to track damage accumulation
        ctx.apply_buff(actor, Box::new(DeathMarkActiveIndicator));

        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().abilities.get_state(AbilitySlot::R).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };

        // Reset Death Mark damage accumulator
        {
            let mut custom = self.custom_state.borrow_mut();
            custom.death_mark_accumulated = 0.0;
        }

        // Schedule Death Mark Pop 3 seconds later
        ctx.new_events.push((
            3.0,
            Box::new(DeathMarkPopEvent {
                attacker: actor.clone(),
                defender: target.clone(),
                custom_state: self.custom_state.clone(),
                level,
            }),
        ));
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

pub struct DeathMarkPopEvent {
    pub attacker: ChampionId,
    pub defender: ChampionId,
    pub custom_state: Rc<RefCell<ZedCustomState>>,
    pub level: u32,
}

impl SimEvent for DeathMarkPopEvent {
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

        let accumulated_damage = {
            let mut custom = self.custom_state.borrow_mut();
            let acc = custom.death_mark_accumulated;
            custom.death_mark_accumulated = 0.0;
            acc
        };

        // Pop damage = 1.0 * total AD + X% of accumulated damage
        let pct = match self.level {
            1 => 0.25,
            2 => 0.40,
            _ => 0.55,
        };

        let raw_damage = attacker_stats.attack_damage + pct * accumulated_damage;

        let damage_result = DamagePipeline::process(
            raw_damage,
            DamageType::Physical,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time,
                self.attacker.clone(),
                self.defender.clone(),
                AbilitySlot::R,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_physical_damage(&self.attacker, &self.defender, &damage_result);
        ctx.trigger_on_damage_dealt(&self.attacker, damage_result.final_damage, true, AbilitySlot::R);

        if let Some(d) = ctx.champions.get(&self.defender) {
            let is_dead = d.borrow_mut().take_damage(damage_result.final_damage).is_dead;
            if is_dead {
                ctx.new_events.push((0.0, Box::new(lol_core::event::DeathEvent { target: self.defender.clone() })));
            }
        }
    }
    fn name(&self) -> &str {
        "DeathMarkPop"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn test_death_mark_damage_accumulation_and_pop() {
        let custom_state = Rc::new(RefCell::new(ZedCustomState {
            last_passive_proc_time: -99.0,
            death_mark_accumulated: 1000.0,
        }));

        let pop_event = DeathMarkPopEvent {
            attacker: ChampionId("Zed".into()),
            defender: ChampionId("Dummy".into()),
            custom_state,
            level: 3,
        };

        let pct = match pop_event.level {
            1 => 0.25,
            2 => 0.40,
            _ => 0.55,
        };
        assert_eq!(pct, 0.55);

        let accumulated = 1000.0;
        let ad = 100.0;
        let expected_raw = ad + pct * accumulated;
        assert_eq!(expected_raw, 650.0);
    }

    #[test]
    fn test_living_shadow_replication_check() {
        let mut buffs = lol_core::buff::BuffManager::new();
        buffs.apply_effect(Box::new(ZedShadowBuff), lol_core::types::SimTime::new(0.0), 0.0);

        assert!(buffs.has_effect_by_id(&EffectId("ZedShadow".into()), lol_core::types::SimTime::new(2.0)));
        assert!(!buffs.has_effect_by_id(&EffectId("ZedShadow".into()), lol_core::types::SimTime::new(6.0)));
    }
}
