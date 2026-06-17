use lol_core::ability::Ability;
use lol_core::buff::{RefreshBehavior, StatusEffect};
use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};
use lol_core::damage::DamagePipeline;
use lol_core::event::SimContext;
use lol_core::stats::StatBlock;
use lol_core::types::{AbilitySlot, ChampionId, DamageType, EffectId};

// -----------------------------------------------------------------------------
// Buffs
// -----------------------------------------------------------------------------

pub struct DariusHemorrhage;
impl StatusEffect for DariusHemorrhage {
    fn id(&self) -> EffectId {
        EffectId("DariusHemorrhage".into())
    }
    fn name(&self) -> &str {
        "Hemorrhage"
    }
    fn duration(&self) -> f64 {
        5.0
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::AddStack
    }
    fn max_stacks(&self) -> u32 {
        5
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::new()
    }
}

pub struct NoxianMight {
    pub level: u32,
}
impl StatusEffect for NoxianMight {
    fn id(&self) -> EffectId {
        EffectId("NoxianMight".into())
    }
    fn name(&self) -> &str {
        "Noxian Might"
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
        let ad_bonus = match self.level {
            1 => 30.0,
            2 => 35.0,
            3 => 40.0,
            4 => 45.0,
            5 => 50.0,
            6 => 55.0,
            7 => 60.0,
            8 => 65.0,
            9 => 70.0,
            10 => 75.0,
            11 => 85.0,
            12 => 95.0,
            13 => 105.0,
            14 => 130.0,
            15 => 155.0,
            16 => 180.0,
            17 => 205.0,
            _ => 230.0,
        };
        let mut stats = StatBlock::new();
        stats.attack_damage = ad_bonus;
        stats
    }
}

pub struct DariusWBuff;
impl StatusEffect for DariusWBuff {
    fn id(&self) -> EffectId {
        EffectId("DariusWBuff".into())
    }
    fn name(&self) -> &str {
        "Crippling Strike"
    }
    fn duration(&self) -> f64 {
        4.0
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

pub struct DariusWSlow;
impl StatusEffect for DariusWSlow {
    fn id(&self) -> EffectId {
        EffectId("DariusWSlow".into())
    }
    fn name(&self) -> &str {
        "Crippling Strike Slow"
    }
    fn duration(&self) -> f64 {
        1.0
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
    fn cc_type(&self) -> Option<lol_core::types::CCType> {
        Some(lol_core::types::CCType::Slow)
    }
}

pub struct DariusEPull;
impl StatusEffect for DariusEPull {
    fn id(&self) -> EffectId {
        EffectId("DariusEPull".into())
    }
    fn name(&self) -> &str {
        "Apprehend"
    }
    fn duration(&self) -> f64 {
        0.25
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
    fn cc_type(&self) -> Option<lol_core::types::CCType> {
        Some(lol_core::types::CCType::Airborne)
    }
}

fn get_hemorrhage_damage(level: u32, bonus_ad: f64) -> f64 {
    let base = 13.0 + 1.0 * (level as f64 - 1.0);
    base + (0.30 * bonus_ad)
}

pub struct HemorrhageTickEvent {
    pub target: ChampionId,
    pub attacker: ChampionId,
    pub tick_count: u32,
}

impl lol_core::event::SimEvent for HemorrhageTickEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut lol_core::event::EventManager) {
        let (stacks, level, bonus_ad) = {
            if let Some(t) = ctx.champions.get(&self.target) {
                let stacks = t
                    .borrow()
                    .state()
                    .buffs
                    .get_stacks_by_name("Hemorrhage", ctx.current_time);
                if stacks == 0 {
                    return; // Buff expired
                }

                let (lvl, ad) = if let Some(a) = ctx.champions.get(&self.attacker) {
                    let champ = a.borrow();
                    let a_state = champ.state();
                    (
                        a_state.level,
                        a_state.stats.current.attack_damage - a_state.stats.base.attack_damage,
                    )
                } else {
                    (1, 0.0)
                };

                (stacks, lvl, ad)
            } else {
                return;
            }
        };

        let total_damage_over_5s = get_hemorrhage_damage(level, bonus_ad) * (stacks as f64);
        let tick_damage = total_damage_over_5s / 4.0; // Ticks every 1.25s (4 ticks over 5s)

        let defender_stats = {
            if let Some(d) = ctx.champions.get(&self.target) {
                d.borrow().state().stats.current.clone()
            } else {
                return;
            }
        };

        let attacker_stats = {
            if let Some(a) = ctx.champions.get(&self.attacker) {
                a.borrow().state().stats.current.clone()
            } else {
                return;
            }
        };

        let damage_result = DamagePipeline::process(
            tick_damage,
            DamageType::Physical,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time,
                self.attacker.clone(),
                self.target.clone(),
                AbilitySlot::Passive,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_damage_dealt(
            &self.attacker,
            damage_result.final_damage,
            true,
            lol_core::types::AbilitySlot::Passive,
        );

        if let Some(d) = ctx.champions.get(&self.target) {
            let is_dead = d
                .borrow_mut()
                .take_damage(damage_result.final_damage)
                .is_dead;
            if is_dead {
                ctx.new_events.push((
                    0.0,
                    Box::new(lol_core::event::DeathEvent {
                        target: self.target.clone(),
                    }),
                ));
            } else {
                // Schedule next tick if it hasn't expired
                ctx.new_events.push((
                    1.25,
                    Box::new(HemorrhageTickEvent {
                        target: self.target.clone(),
                        attacker: self.attacker.clone(),
                        tick_count: self.tick_count + 1,
                    }),
                ));
            }
        }
    }

    fn name(&self) -> &str {
        "HemorrhageTickEvent"
    }
}

#[allow(clippy::collapsible_if)]
fn apply_hemorrhage(ctx: &mut SimContext, target: &ChampionId, attacker: &ChampionId) {
    let was_active = if let Some(t) = ctx.champions.get(target) {
        t.borrow()
            .state()
            .buffs
            .has_buff_by_name("Hemorrhage", ctx.current_time)
    } else {
        false
    };

    ctx.apply_buff(target, Box::new(DariusHemorrhage));

    // Check if 5 stacks
    if let Some(t) = ctx.champions.get(target) {
        if t.borrow()
            .state()
            .buffs
            .get_stacks_by_name("Hemorrhage", ctx.current_time)
            == 5
        {
            let attacker_level = {
                if let Some(a) = ctx.champions.get(attacker) {
                    a.borrow().state().level
                } else {
                    1
                }
            };
            ctx.apply_buff(
                attacker,
                Box::new(NoxianMight {
                    level: attacker_level,
                }),
            );
        }
    }

    if !was_active {
        // Schedule first tick
        ctx.new_events.push((
            1.25,
            Box::new(HemorrhageTickEvent {
                target: target.clone(),
                attacker: attacker.clone(),
                tick_count: 1,
            }),
        ));
    }
}

// -----------------------------------------------------------------------------
// Abilities
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct DariusQ;
impl Ability for DariusQ {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::Q
    }
    fn cast_time(&self) -> f64 {
        0.75
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
        30.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let (attacker_stats, q_level) = {
            if let Some(a) = ctx.champions.get(actor) {
                let champ = a.borrow();
                let state = champ.state();
                let q_lvl = state
                    .abilities
                    .get_state(AbilitySlot::Q)
                    .map(|s| s.level)
                    .unwrap_or(1);
                (state.stats.current.clone(), q_lvl)
            } else {
                return;
            }
        };

        let defender_stats = {
            if let Some(d) = ctx.champions.get(target) {
                d.borrow().state().stats.current.clone()
            } else {
                return;
            }
        };

        let base_dmg = match q_level {
            1 => 50.0,
            2 => 80.0,
            3 => 110.0,
            4 => 140.0,
            _ => 170.0,
        };
        let ratio = match q_level {
            1 => 1.0,
            2 => 1.1,
            3 => 1.2,
            4 => 1.3,
            _ => 1.4,
        };

        let raw_damage = base_dmg + (attacker_stats.attack_damage * ratio);

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
        ctx.trigger_on_damage_dealt(
            actor,
            damage_result.final_damage,
            true,
            lol_core::types::AbilitySlot::Q,
        );

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d
                .borrow_mut()
                .take_damage(damage_result.final_damage)
                .is_dead;
            if is_dead {
                ctx.new_events.push((
                    0.0,
                    Box::new(lol_core::event::DeathEvent {
                        target: target.clone(),
                    }),
                ));
            }
        }

        // Heal 15% of missing health per champion hit (1 champion here)
        if let Some(a) = ctx.champions.get(actor) {
            let mut champ = a.borrow_mut();
            let missing_hp = champ.state().health.max - champ.state().health.current;
            champ.state_mut().health.restore(missing_hp * 0.15);
        }

        apply_hemorrhage(ctx, target, actor);
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct DariusW;
impl Ability for DariusW {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::W
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        5.0
    }
    fn cost(&self, _level: u32) -> f64 {
        30.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, _target: &ChampionId) {
        ctx.apply_buff(actor, Box::new(DariusWBuff));
        if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref
                .borrow_mut()
                .state_mut()
                .abilities
                .reset_cooldown(AbilitySlot::AutoAttack);
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct DariusE;
impl Ability for DariusE {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::E
    }
    fn cast_time(&self) -> f64 {
        0.25
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 24.0,
            2 => 21.0,
            3 => 18.0,
            4 => 15.0,
            _ => 12.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        45.0
    }
    fn execute(&self, ctx: &mut SimContext, _actor: &ChampionId, target: &ChampionId) {
        ctx.apply_buff(target, Box::new(DariusEPull));
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct DariusR;
impl Ability for DariusR {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::R
    }
    fn cast_time(&self) -> f64 {
        0.25
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 120.0,
            2 => 100.0,
            _ => 80.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        100.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let (_attacker_stats, r_level, bonus_ad) = {
            if let Some(a) = ctx.champions.get(actor) {
                let champ = a.borrow();
                let state = champ.state();
                let r_lvl = state
                    .abilities
                    .get_state(AbilitySlot::R)
                    .map(|s| s.level)
                    .unwrap_or(1);
                let bonus_ad = state.stats.current.attack_damage - state.stats.base.attack_damage;
                (state.stats.current.clone(), r_lvl, bonus_ad)
            } else {
                return;
            }
        };

        let stacks = if let Some(d) = ctx.champions.get(target) {
            d.borrow()
                .state()
                .buffs
                .get_stacks_by_name("Hemorrhage", ctx.current_time)
        } else {
            0
        };

        let base_dmg = match r_level {
            1 => 150.0,
            2 => 250.0,
            _ => 300.0,
        };

        let raw_damage = base_dmg + (bonus_ad * 0.75);
        let final_damage = raw_damage * (1.0 + (0.2 * stacks as f64)); // 20% more damage per stack

        // True damage
        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time,
                actor.clone(),
                target.clone(),
                AbilitySlot::R,
                final_damage,
                false,
            );
        }

        ctx.trigger_on_damage_dealt(actor, final_damage, true, AbilitySlot::R);

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d.borrow_mut().take_damage(final_damage).is_dead;
            if is_dead {
                ctx.new_events.push((
                    0.0,
                    Box::new(lol_core::event::DeathEvent {
                        target: target.clone(),
                    }),
                ));
                // Reset cooldown
                if let Some(a) = ctx.champions.get(actor) {
                    a.borrow_mut()
                        .state_mut()
                        .abilities
                        .reset_cooldown(AbilitySlot::R);
                }
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct DariusAutoAttack;
impl Ability for DariusAutoAttack {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::AutoAttack
    }
    fn cast_time(&self) -> f64 {
        0.25
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        1.0
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let (attacker_stats, w_level) = {
            if let Some(a) = ctx.champions.get(actor) {
                let champ = a.borrow_mut();
                let w_lvl = champ
                    .state()
                    .abilities
                    .get_state(AbilitySlot::W)
                    .map(|s| s.level)
                    .unwrap_or(1);
                (champ.state().stats.current.clone(), w_lvl)
            } else {
                return;
            }
        };

        let has_w_buff = {
            if let Some(a) = ctx.champions.get(actor) {
                let champ = a.borrow();
                if champ
                    .state()
                    .buffs
                    .has_buff_by_name("Crippling Strike", ctx.current_time)
                {
                    drop(champ);
                    a.borrow_mut()
                        .state_mut()
                        .buffs
                        .remove_effect(&EffectId("DariusWBuff".into()));
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        let defender_stats = {
            if let Some(d) = ctx.champions.get(target) {
                d.borrow().state().stats.current.clone()
            } else {
                return;
            }
        };

        let mut raw_damage = attacker_stats.attack_damage;
        if has_w_buff {
            let ratio = match w_level {
                1 => 0.40,
                2 => 0.45,
                3 => 0.50,
                4 => 0.55,
                _ => 0.60,
            };
            raw_damage += attacker_stats.attack_damage * ratio;
            ctx.apply_buff(target, Box::new(DariusWSlow));
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

        let is_ability = has_w_buff;
        ctx.trigger_on_damage_dealt(
            actor,
            damage_result.final_damage,
            is_ability,
            if has_w_buff {
                lol_core::types::AbilitySlot::W
            } else {
                lol_core::types::AbilitySlot::AutoAttack
            },
        );

        if let Some(d) = ctx.champions.get(target) {
            let is_dead = d
                .borrow_mut()
                .take_damage(damage_result.final_damage)
                .is_dead;
            if is_dead {
                ctx.new_events.push((
                    0.0,
                    Box::new(lol_core::event::DeathEvent {
                        target: target.clone(),
                    }),
                ));
            }
        }

        apply_hemorrhage(ctx, target, actor);
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

// -----------------------------------------------------------------------------
// Module & Instance
// -----------------------------------------------------------------------------

pub struct DariusModule;

impl ChampionModule for DariusModule {
    fn id(&self) -> &str {
        "Darius"
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
        let keystone_name = config.rune_page.keystone.name();
        if keystone_name == "Conqueror" {
            state
                .rune_manager
                .add_effect(Box::new(lol_core::rune_manager::Conqueror::new(true)));
        } else if keystone_name == "Lethal Tempo" {
            state
                .rune_manager
                .add_effect(Box::new(lol_core::rune_manager::LethalTempo::new(true)));
        } else if keystone_name == "Phase Rush" {
            state
                .rune_manager
                .add_effect(Box::new(lol_core::rune_manager::PhaseRush::new(true)));
        } else if keystone_name == "Electrocute" {
            state
                .rune_manager
                .add_effect(Box::new(lol_core::rune_manager::Electrocute::new()));
        } else if keystone_name == "Press the Attack" {
            state
                .rune_manager
                .add_effect(Box::new(lol_core::rune_manager::PressTheAttack::new(true)));
        }

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

        let mut abilities: Vec<Box<dyn Ability>> = vec![
            Box::new(DariusQ),
            Box::new(DariusW),
            Box::new(DariusE),
            Box::new(DariusR),
            Box::new(DariusAutoAttack),
        ];

        // Register active items dynamically
        for effect in state.items.effects() {
            if let Some(active) = effect.active_ability() {
                state.abilities.register_ability(active.slot(), 1);
                abilities.push(active);
            }
        }

        Box::new(DariusInstance { state, abilities })
    }
}

pub struct DariusInstance {
    state: ChampionState,
    abilities: Vec<Box<dyn Ability>>,
}

impl ChampionInstance for DariusInstance {
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

        // Add E Passive Armor Pen
        let e_level = self
            .state
            .abilities
            .get_state(AbilitySlot::E)
            .map(|s| s.level)
            .unwrap_or(1);
        let e_armor_pen = match e_level {
            1 => 0.15,
            2 => 0.20,
            3 => 0.25,
            4 => 0.30,
            _ => 0.35,
        };
        new_base.armor_pen_percent += e_armor_pen;

        self.state.stats.base = new_base;

        // 2. Recalculate initial stats
        let bonus = self.state.rune_stats.clone() + self.state.item_stats.clone();
        self.state.stats.recalculate_initial(&bonus);

        // 3. Recalculate current stats
        let mut total_bonus = self.state.buffs.aggregate_stats();
        let level_u32 = self.state.level;
        total_bonus = total_bonus
            + self
                .state
                .rune_manager
                .get_bonus_stats(time, &self.state.stats.base, level_u32);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hemorrhage_damage() {
        // Level 1: Base 13
        assert_eq!(get_hemorrhage_damage(1, 0.0), 13.0);
        // Level 18: Base 30
        assert_eq!(get_hemorrhage_damage(18, 0.0), 30.0);
        // Level 18 + 100 Bonus AD -> 30 + 30 = 60
        assert_eq!(get_hemorrhage_damage(18, 100.0), 60.0);
    }

    #[test]
    fn test_noxian_might_bonus_ad() {
        let might = NoxianMight { level: 18 };
        let stats = might.stat_modifiers(1);
        assert_eq!(stats.attack_damage, 230.0);

        let might = NoxianMight { level: 1 };
        let stats = might.stat_modifiers(1);
        assert_eq!(stats.attack_damage, 30.0);
    }
}
