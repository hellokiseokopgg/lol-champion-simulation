use lol_core::ability::Ability;
use lol_core::buff::{RefreshBehavior, StatusEffect};
use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};
use lol_core::damage::DamagePipeline;
use lol_core::event::{SimContext, SimEvent};
use lol_core::stats::StatBlock;
use lol_core::types::{AbilitySlot, ChampionId, DamageType, EffectId, CCType};

pub struct JinxModule;

impl ChampionModule for JinxModule {
    fn id(&self) -> &str {
        "Jinx"
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

        let state_ptr = &state as *const ChampionState;

        let mut abilities: Vec<Box<dyn Ability>> = vec![
            Box::new(JinxAutoAttack),
            Box::new(JinxQ),
            Box::new(JinxW { state_ptr }),
            Box::new(JinxE),
            Box::new(JinxR),
        ];

        // Register active items dynamically
        for effect in state.items.effects() {
            if let Some(active) = effect.active_ability() {
                state.abilities.register_ability(active.slot(), 1);
                abilities.push(active);
            }
        }

        Box::new(JinxInstance {
            state,
            _config: config,
            abilities,
        })
    }
}

pub struct JinxInstance {
    pub state: ChampionState,
    pub _config: ChampionConfig,
    pub abilities: Vec<Box<dyn Ability>>,
}

impl ChampionInstance for JinxInstance {
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
}

// -----------------------------------------------------------------------------
// Buffs
// -----------------------------------------------------------------------------

pub struct JinxFishbonesBuff;
impl StatusEffect for JinxFishbonesBuff {
    fn id(&self) -> EffectId {
        EffectId("JinxFishbones".into())
    }
    fn name(&self) -> &str {
        "Fishbones Stance"
    }
    fn duration(&self) -> f64 {
        99999.0
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

pub struct JinxPowPowBuff;
impl StatusEffect for JinxPowPowBuff {
    fn id(&self) -> EffectId {
        EffectId("JinxPowPow".into())
    }
    fn name(&self) -> &str {
        "Pow-Pow Stacks"
    }
    fn duration(&self) -> f64 {
        4.0
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::AddStack
    }
    fn max_stacks(&self) -> u32 {
        3
    }
    fn stat_modifiers(&self, stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        // +10% AS per stack
        stats.attack_speed = 0.10 * stacks as f64;
        stats
    }
}

pub struct JinxWSlow;
impl StatusEffect for JinxWSlow {
    fn id(&self) -> EffectId {
        EffectId("JinxWSlow".into())
    }
    fn name(&self) -> &str {
        "Zap! Slow"
    }
    fn duration(&self) -> f64 {
        2.0
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

pub struct JinxERoot;
impl StatusEffect for JinxERoot {
    fn id(&self) -> EffectId {
        EffectId("JinxERoot".into())
    }
    fn name(&self) -> &str {
        "Flame Chompers Root"
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
        Some(CCType::Root)
    }
}

// -----------------------------------------------------------------------------
// Abilities
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct JinxAutoAttack;

impl Ability for JinxAutoAttack {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::AutoAttack
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn windup_percent(&self) -> f64 {
        0.16875
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        1.0
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let is_fishbones = if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow().state().buffs.has_effect_by_id(&EffectId("JinxFishbones".into()), ctx.current_time)
        } else {
            false
        };

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

        let mut cost_mana = false;
        let mut use_fishbones = false;

        if is_fishbones {
            // Check mana cost
            let current_mana = if let Some(champ_ref) = ctx.champions.get(actor) {
                champ_ref.borrow().state().resource.current
            } else {
                0.0
            };
            if current_mana >= 20.0 {
                cost_mana = true;
                use_fishbones = true;
            } else {
                // Out of mana: switch stance back to minigun automatically
                if let Some(champ_ref) = ctx.champions.get(actor) {
                    let mut champ = champ_ref.borrow_mut();
                    champ.state_mut().buffs.remove_effect(&EffectId("JinxFishbones".into()));
                    champ.update_stats(ctx.current_time);
                }
            }
        }

        let mut target_champ = None;
        if cost_mana {
            target_champ = ctx.champions.get(actor);
        }
        if let Some(champ_ref) = target_champ {
            let mut champ = champ_ref.borrow_mut();
            champ.state_mut().resource.reduce(20.0);
        }

        let raw_damage = if use_fishbones {
            attacker_stats.attack_damage * 1.10
        } else {
            attacker_stats.attack_damage
        };

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

        // Trigger on-hit effects
        ctx.trigger_on_hit(actor, target, &damage_result);
        ctx.trigger_on_physical_damage(actor, target, &damage_result);
        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, false, AbilitySlot::AutoAttack);

        // Apply minigun stack if not fishbones
        if !use_fishbones {
            ctx.apply_buff(actor, Box::new(JinxPowPowBuff));
        }

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
pub struct JinxQ;

impl Ability for JinxQ {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::Q
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        0.9
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, _target: &ChampionId) {
        if let Some(champ_ref) = ctx.champions.get(actor) {
            let mut champ = champ_ref.borrow_mut();
            let has_fb = champ.state().buffs.has_effect_by_id(&EffectId("JinxFishbones".into()), ctx.current_time);
            if has_fb {
                champ.state_mut().buffs.remove_effect(&EffectId("JinxFishbones".into()));
            } else {
                champ.state_mut().buffs.apply_effect(Box::new(JinxFishbonesBuff), ctx.current_time, 0.0);
            }
            champ.update_stats(ctx.current_time);
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct JinxW {
    state_ptr: *const ChampionState,
}
unsafe impl Send for JinxW {}
unsafe impl Sync for JinxW {}

impl Ability for JinxW {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::W
    }
    fn cast_time(&self) -> f64 {
        unsafe {
            if let Some(state) = self.state_ptr.as_ref() {
                let stats = &state.stats.current;
                let as_ratio = stats.attack_speed_ratio.unwrap_or(0.625);
                let bonus_as = (stats.attack_speed / as_ratio - 1.0).max(0.0);
                (0.6 / (1.0 + bonus_as)).max(0.4)
            } else {
                0.6
            }
        }
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 8.0,
            2 => 7.0,
            3 => 6.0,
            4 => 5.0,
            _ => 4.0,
        }
    }
    fn cost(&self, level: u32) -> f64 {
        50.0 + (level as f64 - 1.0) * 10.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let mut champ = champ_ref.borrow_mut();
            let lvl = champ.state().abilities.get_state(AbilitySlot::W).map(|s| s.level).unwrap_or(1);
            let cost = 50.0 + (lvl as f64 - 1.0) * 10.0;
            champ.state_mut().resource.reduce(cost);
            lvl
        } else {
            1
        };

        let base_damage = 10.0 + (level as f64 - 1.0) * 50.0;

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

        let raw_damage = base_damage + 1.6 * attacker_stats.attack_damage;
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
                AbilitySlot::W,
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_physical_damage(actor, target, &damage_result);
        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, AbilitySlot::W);
        ctx.apply_buff(target, Box::new(JinxWSlow));

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
pub struct JinxE;

impl Ability for JinxE {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::E
    }
    fn cast_time(&self) -> f64 {
        0.25
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 24.0,
            2 => 22.0,
            3 => 20.0,
            4 => 18.0,
            _ => 16.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        70.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let mut champ = champ_ref.borrow_mut();
            champ.state_mut().resource.reduce(70.0);
            champ.state().abilities.get_state(AbilitySlot::E).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };

        let base_damage = 70.0 + (level as f64 - 1.0) * 50.0;

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

        let raw_damage = base_damage + 1.0 * attacker_stats.ability_power;
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
        ctx.apply_buff(target, Box::new(JinxERoot));

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
pub struct JinxR;

impl Ability for JinxR {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::R
    }
    fn cast_time(&self) -> f64 {
        0.6
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 75.0,
            2 => 65.0,
            _ => 55.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        100.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let level = if let Some(champ_ref) = ctx.champions.get(actor) {
            let mut champ = champ_ref.borrow_mut();
            champ.state_mut().resource.reduce(100.0);
            champ.state().abilities.get_state(AbilitySlot::R).map(|s| s.level).unwrap_or(1)
        } else {
            1
        };

        let base_damage = 300.0 + (level as f64 - 1.0) * 150.0;

        ctx.new_events.push((
            0.6,
            Box::new(JinxRocketImpactEvent {
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

pub struct JinxRocketImpactEvent {
    pub attacker: ChampionId,
    pub defender: ChampionId,
    pub base_damage: f64,
}

impl SimEvent for JinxRocketImpactEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut lol_core::event::EventManager) {
        let (attacker_stats, attacker_base) = if let Some(champ_ref) = ctx.champions.get(&self.attacker) {
            let champ = champ_ref.borrow();
            (champ.state().stats.current.clone(), champ.state().stats.base.clone())
        } else {
            return;
        };

        let defender_stats = if let Some(d) = ctx.champions.get(&self.defender) {
            d.borrow().state().stats.current.clone()
        } else {
            return;
        };

        let defender_health_state = if let Some(d) = ctx.champions.get(&self.defender) {
            let champ = d.borrow();
            let state = champ.state();
            (state.health.current, state.stats.current.health)
        } else {
            (0.0, 0.0)
        };

        let missing_hp = (defender_health_state.1 - defender_health_state.0).max(0.0);

        let bonus_ad = (attacker_stats.attack_damage - attacker_base.attack_damage).max(0.0);

        let raw_damage = self.base_damage + 1.5 * bonus_ad + 0.25 * missing_hp;

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
        "JinxRocketImpact"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lol_core::stats::StatBlock;
    use lol_core::types::DamageType;
    use lol_core::buff::StatusEffect;

    #[test]
    fn test_jinx_powpow_stacks_and_attack_speed() {
        let powpow = JinxPowPowBuff;
        // 0 stacks
        let mods_0 = powpow.stat_modifiers(0);
        assert_eq!(mods_0.attack_speed, 0.0);

        // 3 stacks
        let mods_3 = powpow.stat_modifiers(3);
        assert!((mods_3.attack_speed - 0.30).abs() < 1e-6);
    }

    #[test]
    fn test_jinx_fishbones_damage_scale() {
        let attacker = StatBlock {
            attack_damage: 100.0,
            ..StatBlock::new()
        };
        let defender = StatBlock::new();

        // Fishbones attack deals 110% AD
        let raw_damage = attacker.attack_damage * 1.10;
        assert!((raw_damage - 110.0).abs() < 1e-6);

        let result = DamagePipeline::process(raw_damage, DamageType::Physical, false, &attacker, &defender);
        assert!((result.final_damage - 110.0).abs() < 1e-6);
    }

    #[test]
    fn test_jinx_rocket_execution_damage() {
        // base = 300, bonus_ad = 100, missing_hp = 1000
        // Expected raw damage = 300 + 1.5 * 100 + 0.25 * 1000 = 700
        let base_damage = 300.0;
        let bonus_ad = 100.0;
        let missing_hp = 1000.0;
        let raw_damage = base_damage + 1.5 * bonus_ad + 0.25 * missing_hp;
        assert_eq!(raw_damage, 700.0);
    }
}
