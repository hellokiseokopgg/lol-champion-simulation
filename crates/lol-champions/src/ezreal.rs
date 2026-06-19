use lol_core::ability::Ability;
use lol_core::buff::{RefreshBehavior, StatusEffect};
use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};
use lol_core::damage::DamagePipeline;
use lol_core::event::SimContext;
use lol_core::stats::StatBlock;
use lol_core::types::{AbilitySlot, ChampionId, DamageType, EffectId};

pub struct EzrealModule;

impl ChampionModule for EzrealModule {
    fn id(&self) -> &str {
        "Ezreal"
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

        let mut abilities: Vec<Box<dyn Ability>> = vec![
            Box::new(EzrealAutoAttack),
            Box::new(EzrealQ),
            Box::new(EzrealW),
            Box::new(EzrealE),
            Box::new(EzrealR),
        ];

        // Register active items dynamically
        for effect in state.items.effects() {
            if let Some(active) = effect.active_ability() {
                state.abilities.register_ability(active.slot(), 1);
                abilities.push(active);
            }
        }

        Box::new(EzrealInstance {
            state,
            _config: config,
            abilities,
        })
    }
}

pub struct EzrealInstance {
    pub state: ChampionState,
    pub _config: ChampionConfig,
    pub abilities: Vec<Box<dyn Ability>>,
}

impl ChampionInstance for EzrealInstance {
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
            + self.state.rune_manager.get_bonus_stats(
                time,
                &self.state.stats.base,
                level,
                hp_ratio,
            );
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

// -----------------------------------------------------------------------------
// Buffs
// -----------------------------------------------------------------------------

pub struct EzrealPassiveBuff;
impl StatusEffect for EzrealPassiveBuff {
    fn id(&self) -> EffectId {
        EffectId("EzrealPassive".into())
    }
    fn name(&self) -> &str {
        "Rising Spell Force"
    }
    fn duration(&self) -> f64 {
        6.0
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::AddStack
    }
    fn max_stacks(&self) -> u32 {
        5
    }
    fn stat_modifiers(&self, stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        // +10% AS per stack
        stats.attack_speed = 0.10 * stacks as f64;
        stats
    }
}

pub struct EzrealWBuff;
impl StatusEffect for EzrealWBuff {
    fn id(&self) -> EffectId {
        EffectId("EzrealW".into())
    }
    fn name(&self) -> &str {
        "Essence Flux Mark"
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

// -----------------------------------------------------------------------------
// Detonation Helper
// -----------------------------------------------------------------------------

fn check_and_detonate_w(
    ctx: &mut SimContext,
    actor: &ChampionId,
    target: &ChampionId,
    triggering_cost: f64,
) {
    let has_w = if let Some(d) = ctx.champions.get(target) {
        d.borrow()
            .state()
            .buffs
            .has_effect_by_id(&EffectId("EzrealW".into()), ctx.current_time)
    } else {
        false
    };

    if has_w {
        // 1. W 버프 제거
        if let Some(d) = ctx.champions.get(target) {
            d.borrow_mut()
                .state_mut()
                .buffs
                .remove_effect(&EffectId("EzrealW".into()));
        }

        // 2. W 레벨 구하기
        let w_level = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref
                .borrow()
                .state()
                .abilities
                .get_state(AbilitySlot::W)
                .map(|s| s.level)
                .unwrap_or(1)
        } else {
            1
        };

        // 3. 데미지 계산 및 입히기
        let (attacker_stats, attacker_base) = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            (
                champ.state().stats.current.clone(),
                champ.state().stats.base.clone(),
            )
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let base_damage = 80.0 + (w_level as f64 - 1.0) * 55.0;
        let bonus_ad = (attacker_stats.attack_damage - attacker_base.attack_damage).max(0.0);
        let raw_damage = base_damage + 0.6 * bonus_ad + 0.7 * attacker_stats.ability_power;

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
                AbilitySlot::W,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, AbilitySlot::W);

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

        // 4. 마나 반환 (트리거 스킬 소모량 + 60)
        let recovered_mana = triggering_cost + 60.0;
        if let Some(champ_ref) = ctx.champions.get(actor) {
            let mut champ = champ_ref.borrow_mut();
            champ.state_mut().resource.restore(recovered_mana);

            let current_mana = champ.state().resource.current;
            let max_mana = champ.state().resource.max;
            if let Some(recorder) = &ctx.recorder {
                recorder.borrow_mut().record_resource_update(
                    ctx.current_time,
                    actor.clone(),
                    "Mana".to_string(),
                    current_mana,
                    max_mana,
                );
            }
        }

        // W 폭발도 적중으로 취급되어 패시브 스택을 쌓음
        ctx.apply_buff(actor, Box::new(EzrealPassiveBuff));
    }
}

// -----------------------------------------------------------------------------
// Cooldown Reduction Helper
// -----------------------------------------------------------------------------

fn apply_q_cooldown_reduction(ctx: &mut SimContext, actor: &ChampionId) {
    if let Some(champ_ref) = ctx.champions.get(actor) {
        let mut champ = champ_ref.borrow_mut();
        for slot in &[
            AbilitySlot::Q,
            AbilitySlot::W,
            AbilitySlot::E,
            AbilitySlot::R,
        ] {
            if let Some(state) = champ.state_mut().abilities.get_state_mut(*slot) {
                state.cooldown.reduce_cooldown(1.5);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// AutoAttack
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct EzrealAutoAttack;

impl Ability for EzrealAutoAttack {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::AutoAttack
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn windup_percent(&self) -> f64 {
        0.188
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

        let raw_damage = attacker_stats.attack_damage;
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
        ctx.trigger_on_damage_dealt(
            actor,
            damage_result.final_damage,
            false,
            AbilitySlot::AutoAttack,
        );

        // 평타로 표식 폭발 검사 (평타 소모마나 0)
        check_and_detonate_w(ctx, actor, target, 0.0);

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
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

// -----------------------------------------------------------------------------
// Q: Mystic Shot
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct EzrealQ;

impl Ability for EzrealQ {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::Q
    }
    fn cast_time(&self) -> f64 {
        0.25
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 5.5,
            2 => 5.25,
            3 => 5.0,
            4 => 4.75,
            _ => 4.5,
        }
    }
    fn cost(&self, level: u32) -> f64 {
        25.0 + level as f64 * 3.0 // 28, 31, 34, 37, 40
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            champ
                .state()
                .abilities
                .get_state(AbilitySlot::Q)
                .map(|s| s.level)
                .unwrap_or(1)
        } else {
            1
        };
        let cost = 25.0 + level as f64 * 3.0;
        ctx.consume_resource(actor, cost);

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

        let base_damage = 20.0 + (level as f64 - 1.0) * 25.0;
        let raw_damage =
            base_damage + 1.3 * attacker_stats.attack_damage + 0.15 * attacker_stats.ability_power;

        // Q applies on-hits, so it can crit if crit chance is met (modeled as physical damage)
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
                AbilitySlot::Q,
                damage_result.final_damage,
                is_critical,
            );
        }

        ctx.trigger_on_hit(actor, target, &damage_result);
        ctx.trigger_on_physical_damage(actor, target, &damage_result);
        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, false, AbilitySlot::Q);

        // Q 적중 시 패시브 스택 추가
        ctx.apply_buff(actor, Box::new(EzrealPassiveBuff));

        // Q 적중 시 쿨다운 1.5초 감소
        apply_q_cooldown_reduction(ctx, actor);

        // Q로 표식 폭발 검사
        check_and_detonate_w(ctx, actor, target, cost);

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
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

// -----------------------------------------------------------------------------
// W: Essence Flux
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct EzrealW;

impl Ability for EzrealW {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::W
    }
    fn cast_time(&self) -> f64 {
        0.25
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 12.0,
            2 => 11.0,
            3 => 10.0,
            4 => 9.0,
            _ => 8.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        50.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        ctx.consume_resource(actor, 50.0);

        // W 표식 부여
        ctx.apply_buff(target, Box::new(EzrealWBuff));

        // W 스킬 적중으로 취급되어 패시브 스택 추가
        ctx.apply_buff(actor, Box::new(EzrealPassiveBuff));
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

// -----------------------------------------------------------------------------
// E: Arcane Shift
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct EzrealE;

impl Ability for EzrealE {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::E
    }
    fn cast_time(&self) -> f64 {
        0.15
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 28.0,
            2 => 25.0,
            3 => 22.0,
            4 => 19.0,
            _ => 16.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        90.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        ctx.consume_resource(actor, 90.0);

        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            champ
                .state()
                .abilities
                .get_state(AbilitySlot::E)
                .map(|s| s.level)
                .unwrap_or(1)
        } else {
            1
        };

        let (attacker_stats, attacker_base) = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            (
                champ.state().stats.current.clone(),
                champ.state().stats.base.clone(),
            )
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let base_damage = 80.0 + (level as f64 - 1.0) * 50.0;
        let bonus_ad = (attacker_stats.attack_damage - attacker_base.attack_damage).max(0.0);
        let raw_damage = base_damage + 0.5 * bonus_ad + 0.75 * attacker_stats.ability_power;

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

        // E 적중 시 패시브 스택 추가
        ctx.apply_buff(actor, Box::new(EzrealPassiveBuff));

        // E로 표식 폭발 검사
        check_and_detonate_w(ctx, actor, target, 90.0);

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
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

// -----------------------------------------------------------------------------
// R: Trueshot Barrage
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct EzrealR;

impl Ability for EzrealR {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::R
    }
    fn cast_time(&self) -> f64 {
        1.0
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 120.0,
            2 => 105.0,
            _ => 90.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        100.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        ctx.consume_resource(actor, 100.0);

        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            champ
                .state()
                .abilities
                .get_state(AbilitySlot::R)
                .map(|s| s.level)
                .unwrap_or(1)
        } else {
            1
        };

        let (attacker_stats, attacker_base) = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            (
                champ.state().stats.current.clone(),
                champ.state().stats.base.clone(),
            )
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(target) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let base_damage = 350.0 + (level as f64 - 1.0) * 150.0;
        let bonus_ad = (attacker_stats.attack_damage - attacker_base.attack_damage).max(0.0);
        let raw_damage = base_damage + 1.0 * bonus_ad + 0.9 * attacker_stats.ability_power;

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

        // R 적중 시 패시브 스택 추가
        ctx.apply_buff(actor, Box::new(EzrealPassiveBuff));

        // R로 표식 폭발 검사
        check_and_detonate_w(ctx, actor, target, 100.0);

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
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lol_core::buff::StatusEffect;
    use lol_core::stats::StatBlock;

    #[test]
    fn test_ezreal_passive_attack_speed() {
        let passive = EzrealPassiveBuff;
        // 0 stacks
        let mods_0 = passive.stat_modifiers(0);
        assert_eq!(mods_0.attack_speed, 0.0);

        // 5 stacks
        let mods_5 = passive.stat_modifiers(5);
        assert!((mods_5.attack_speed - 0.50).abs() < 1e-6);
    }

    #[test]
    fn test_ezreal_e_mana_cost() {
        let e = EzrealE;
        assert_eq!(e.cost(1), 90.0);
        assert_eq!(e.cost(5), 90.0);
    }

    #[test]
    fn test_ezreal_w_detonation_mana_refund_auto_attack() {
        use crate::dummy::DummyModule;
        use std::collections::HashMap;

        let mut sim =
            lol_core::sim::GameSimulation::new(lol_core::sim::SimConfig { max_duration: 10.0 });

        let id_ezreal = ChampionId("Ezreal".to_string());
        let id_dummy = ChampionId("Dummy".to_string());

        let base_stats = StatBlock {
            mana: 500.0,
            ..StatBlock::new()
        };

        let config_ezreal = ChampionConfig {
            level: 1,
            item_build: lol_core::item::ItemBuild::new(),
            rune_page: lol_core::rune::RunePage::default(),
            base_stats: base_stats.clone(),
            growth_stats: StatBlock::new(),
        };

        let config_dummy = ChampionConfig {
            level: 1,
            item_build: lol_core::item::ItemBuild::new(),
            rune_page: lol_core::rune::RunePage::default(),
            base_stats: StatBlock::new(),
            growth_stats: StatBlock::new(),
        };

        let ezreal_inst = std::rc::Rc::new(std::cell::RefCell::new(
            EzrealModule.create_instance(config_ezreal),
        ));
        let dummy_inst = std::rc::Rc::new(std::cell::RefCell::new(
            DummyModule.create_instance(config_dummy),
        ));

        sim.add_actor(id_ezreal.clone(), ezreal_inst.clone());
        sim.add_actor(id_dummy.clone(), dummy_inst.clone());

        let mut champions_map = HashMap::new();
        champions_map.insert(id_ezreal.clone(), ezreal_inst.clone());
        champions_map.insert(id_dummy.clone(), dummy_inst.clone());

        let mut ctx = lol_core::event::SimContext {
            current_time: lol_core::types::SimTime::new(0.0),
            recorder: None,
            new_events: Vec::new(),
            champions: champions_map,
            is_simulation_over: false,
        };

        // Apply W mark first on dummy
        ctx.apply_buff(&id_dummy, Box::new(EzrealWBuff));
        assert!(
            dummy_inst
                .borrow()
                .state()
                .buffs
                .has_effect_by_id(&EffectId("EzrealW".into()), ctx.current_time)
        );

        // Set ezreal mana to 300
        ezreal_inst.borrow_mut().state_mut().resource.current = 300.0;

        // Detonate W using basic attack (triggering_cost = 0.0)
        // W detonation should restore 60.0 mana
        check_and_detonate_w(&mut ctx, &id_ezreal, &id_dummy, 0.0);

        let current_mana = ezreal_inst.borrow().state().resource.current;
        assert_eq!(current_mana, 360.0);
    }
}
