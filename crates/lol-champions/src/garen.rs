use lol_core::ability::Ability;
use lol_core::buff::{RefreshBehavior, StatusEffect};
use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};
use lol_core::damage::DamagePipeline;
use lol_core::event::{SimContext, SimEvent};
use lol_core::stats::StatBlock;
use lol_core::types::{AbilitySlot, ChampionId, DamageType, EffectId};

pub struct GarenModule;

impl ChampionModule for GarenModule {
    fn id(&self) -> &str {
        "Garen"
    }

    fn create_instance(&self, mut config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let mut base_stats = config.base_stats.clone();

        // Garen W Passive: Gains 30 Armor and MR at max stacks. We assume max stacks.
        base_stats.armor += 30.0;
        base_stats.magic_resist += 30.0;

        let rune_stats = config.rune_page.aggregate_stats();
        let item_stats = config.item_build.aggregate_stats();
        let mut item_effects = Vec::new();
        for item in &mut config.item_build.items {
            item_effects.append(&mut item.effects);
        }

        let mut state = ChampionState::new(
            config.level,
            base_stats,
            config.growth_stats.clone(),
            lol_core::types::ResourceType::None,
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
            Box::new(GarenAutoAttack),
            Box::new(GarenQ),
            Box::new(GarenW),
            Box::new(GarenE),
            Box::new(GarenR),
        ];

        // Register active items dynamically
        for effect in state.items.effects() {
            if let Some(active) = effect.active_ability() {
                state.abilities.register_ability(active.slot(), 1);
                abilities.push(active);
            }
        }

        Box::new(GarenInstance {
            state,
            _config: config,
            abilities,
        })
    }
}

pub struct GarenInstance {
    pub state: ChampionState,
    pub _config: ChampionConfig,
    pub abilities: Vec<Box<dyn Ability>>,
}

impl ChampionInstance for GarenInstance {
    fn state(&self) -> &ChampionState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ChampionState {
        &mut self.state
    }

    fn update_stats(&mut self, time: lol_core::types::SimTime) {
        // 1. Recalculate base stats using growth logic
        // Formula: Stat = Base + Growth * (Level - 1) * (0.7025 + 0.0175 * (Level - 1))
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

        // Attack Speed growth is a percentage of the AS Ratio.
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

    fn get_ability(
        &self,
        slot: lol_core::types::AbilitySlot,
    ) -> Option<&dyn lol_core::ability::Ability> {
        self.abilities
            .iter()
            .find(|a| a.slot() == slot)
            .map(|a| a.as_ref())
    }

    fn take_damage(&mut self, amount: f64) -> lol_core::types::TakeDamageResult {
        let time = self.state.current_time;
        let level = self.state.level;
        let mut final_damage = amount;

        // Check Bone Plating
        let has_bp_buff = self.state.buffs.has_effect_by_id(
            &lol_core::types::EffectId("BonePlatingBuff".to_string()),
            time,
        );
        if has_bp_buff {
            let blocked = 30.0 + (30.0 / 17.0) * (level as f64 - 1.0);
            final_damage = (final_damage - blocked).max(0.0);
            self.state
                .buffs
                .decrement_stacks(&lol_core::types::EffectId("BonePlatingBuff".to_string()));
            println!(
                "Garen's Bone Plating blocked {:.1} damage at time {:.3}",
                blocked,
                time.as_f64()
            );
        } else {
            let has_bp_rune = self.state.rune_manager.has_rune("Bone Plating");
            let on_cooldown = self.state.buffs.has_effect_by_id(
                &lol_core::types::EffectId("BonePlatingCooldown".to_string()),
                time,
            );
            if has_bp_rune && !on_cooldown {
                self.state.buffs.apply_effect(
                    Box::new(lol_core::buff::BonePlatingCooldown),
                    time,
                    0.0,
                );
                self.state.buffs.apply_effect(
                    Box::new(lol_core::buff::BonePlatingBuff { level }),
                    time,
                    0.0,
                );
                let blocked = 30.0 + (30.0 / 17.0) * (level as f64 - 1.0);
                final_damage = (final_damage - blocked).max(0.0);
                self.state
                    .buffs
                    .decrement_stacks(&lol_core::types::EffectId("BonePlatingBuff".to_string()));
                println!(
                    "Garen's Bone Plating blocked {:.1} damage at time {:.3}",
                    blocked,
                    time.as_f64()
                );
            }
        }

        let is_dead = self.state.health.reduce(final_damage);
        lol_core::types::TakeDamageResult {
            actual_damage: final_damage,
            is_dead,
        }
    }

    fn can_cast(
        &self,
        _slot: lol_core::types::AbilitySlot,
        _time: lol_core::types::SimTime,
    ) -> bool {
        true
    }
}

// -----------------------------------------------------------------------------
// Buffs
// -----------------------------------------------------------------------------

pub struct GarenQBuff;
impl StatusEffect for GarenQBuff {
    fn id(&self) -> EffectId {
        EffectId("GarenQ".into())
    }
    fn name(&self) -> &str {
        "Decisive Strike"
    }
    fn duration(&self) -> f64 {
        4.5
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::RefreshDuration
    }
    fn max_stacks(&self) -> u32 {
        1
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        // Provides 30% MS for simplified logic
        stats.movement_speed = 100.0; // Mock flat MS for now since we don't have % MS multiplier
        stats
    }
}

pub struct GarenQSilence;
impl StatusEffect for GarenQSilence {
    fn id(&self) -> EffectId {
        EffectId("GarenQSilence".into())
    }
    fn name(&self) -> &str {
        "Silence"
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
    fn cc_type(&self) -> Option<lol_core::types::CCType> {
        Some(lol_core::types::CCType::Silence)
    }
}

pub struct GarenWBuff;
impl StatusEffect for GarenWBuff {
    fn id(&self) -> EffectId {
        EffectId("GarenW".into())
    }
    fn name(&self) -> &str {
        "Courage"
    }
    fn duration(&self) -> f64 {
        5.0
    } // 5 seconds at rank 5
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::RefreshDuration
    }
    fn max_stacks(&self) -> u32 {
        1
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.damage_reduction_percent = 0.3; // 30% damage reduction
        stats
    }
}

pub struct GarenEArmorShred;
impl StatusEffect for GarenEArmorShred {
    fn id(&self) -> EffectId {
        EffectId("GarenEShred".into())
    }
    fn name(&self) -> &str {
        "Judgment Shred"
    }
    fn duration(&self) -> f64 {
        6.0
    }
    fn refresh_behavior(&self) -> RefreshBehavior {
        RefreshBehavior::RefreshDuration
    }
    fn max_stacks(&self) -> u32 {
        1
    }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.armor_reduction_percent = 0.25; // 25% armor reduction
        stats
    }
}

pub struct GarenEBuff;
impl StatusEffect for GarenEBuff {
    fn id(&self) -> EffectId {
        EffectId("GarenEBuff".into())
    }
    fn name(&self) -> &str {
        "Judgment"
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
    fn prevents_basic_attacks(&self) -> bool {
        false
    }
}

// -----------------------------------------------------------------------------
// Abilities
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct GarenQ;
impl Ability for GarenQ {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::Q
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        8.0
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        println!(
            "[DEBUG Q] Q execute start. actor={:?}, target={:?}",
            actor, target
        );
        ctx.apply_buff(actor, Box::new(GarenQBuff));
        if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref
                .borrow_mut()
                .state_mut()
                .abilities
                .reset_cooldown(AbilitySlot::AutoAttack);
        }
        println!("[DEBUG Q] Q calling GarenAutoAttack.execute");
        GarenAutoAttack.execute(ctx, actor, target);
        println!("[DEBUG Q] Q execute end");
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct GarenW;
impl Ability for GarenW {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::W
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 23.0,
            2 => 21.0,
            3 => 19.0,
            4 => 17.0,
            5 => 15.0,
            _ => 15.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, _target: &ChampionId) {
        ctx.apply_buff(actor, Box::new(GarenWBuff));
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct GarenE;
impl Ability for GarenE {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::E
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn base_cooldown(&self, _level: u32) -> f64 {
        9.0
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        ctx.apply_buff(actor, Box::new(GarenEBuff));

        let bonus_as_percent = if let Some(champ_ref) = ctx.champions.get(actor) {
            let champ = champ_ref.borrow();
            let stats = champ.state().stats.current.clone();
            let ratio = stats.attack_speed_ratio.unwrap_or(0.625);
            ((stats.attack_speed / ratio) - 1.0).max(0.0)
        } else {
            0.0
        };

        let extra_ticks = (bonus_as_percent / 0.25).floor() as u32;
        let ticks = 7 + extra_ticks;

        let e_event = JudgmentTickEvent {
            attacker: actor.clone(),
            defender: target.clone(),
            ticks_remaining: ticks,
            tick_interval: 3.0 / ticks as f64,
            base_damage: 20.0,
            ad_ratio: 0.4,
            hits_landed: 0,
        };
        ctx.new_events.push((0.0, Box::new(e_event)));
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct GarenR;
impl Ability for GarenR {
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
            3 => 80.0,
            _ => 80.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let r_event = DemacianJusticeEvent {
            attacker: actor.clone(),
            defender: target.clone(),
            base_damage: 150.0,
            missing_health_ratio: 0.25,
        };
        // R has 0.5s cast time, so damage applies after 0.5s
        ctx.new_events.push((0.5, Box::new(r_event)));
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct GarenAutoAttack;
impl Ability for GarenAutoAttack {
    fn slot(&self) -> AbilitySlot {
        AbilitySlot::AutoAttack
    }
    fn cast_time(&self) -> f64 {
        0.0
    }
    fn windup_percent(&self) -> f64 {
        0.20
    } // Garen's basic attack windup is 20%
    fn base_cooldown(&self, _level: u32) -> f64 {
        1.0
    }
    fn cost(&self, _level: u32) -> f64 {
        0.0
    }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        println!(
            "[DEBUG AA] AA execute start. actor={:?}, target={:?}, time={}",
            actor,
            target,
            ctx.current_time.as_f64()
        );
        let mut has_q_buff = false;
        let attacker_stats = {
            if let Some(a) = ctx.champions.get(actor) {
                let mut champ = a.borrow_mut();
                if champ
                    .state()
                    .buffs
                    .has_buff_by_name("Decisive Strike", ctx.current_time)
                {
                    has_q_buff = true;
                    champ
                        .state_mut()
                        .buffs
                        .remove_effect(&EffectId("GarenQ".into()));
                }
                println!("[DEBUG AA] has_q_buff={}", has_q_buff);
                champ.state().stats.current.clone()
            } else {
                return;
            }
        };

        let defender_stats = {
            if let Some(d) = ctx.champions.get(target) {
                d.borrow().as_ref().state().stats.current.clone()
            } else {
                return;
            }
        };

        let mut raw_damage = attacker_stats.attack_damage;
        let mut slot_to_record = AbilitySlot::AutoAttack;
        if has_q_buff {
            // Rank 5 Q bonus: 150 + 0.5 AD
            raw_damage += 150.0 + 0.5 * attacker_stats.attack_damage;
            slot_to_record = AbilitySlot::Q;
            // Apply silence to target
            ctx.apply_buff(target, Box::new(GarenQSilence));
        }

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
                slot_to_record,
                damage_result.final_damage,
                is_critical,
            );
        }

        ctx.trigger_on_hit(actor, target, &damage_result);
        ctx.trigger_on_physical_damage(actor, target, &damage_result);

        // Trigger rune events based on the damage dealt
        let is_ability = slot_to_record != AbilitySlot::AutoAttack;
        ctx.trigger_on_damage_dealt(
            actor,
            damage_result.final_damage,
            is_ability,
            slot_to_record,
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
    }
    fn clone_box(&self) -> Box<dyn Ability> {
        Box::new(self.clone())
    }
}

// -----------------------------------------------------------------------------
// Events
// -----------------------------------------------------------------------------

pub struct JudgmentTickEvent {
    pub attacker: ChampionId,
    pub defender: ChampionId,
    pub ticks_remaining: u32,
    pub tick_interval: f64,
    pub base_damage: f64,
    pub ad_ratio: f64,
    pub hits_landed: u32,
}

impl SimEvent for JudgmentTickEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut lol_core::event::EventManager) {
        // We need attacker's AD and defender's stats.
        let attacker_stats = {
            if let Some(a) = ctx.champions.get(&self.attacker) {
                a.borrow().as_ref().state().stats.current.clone()
            } else {
                return;
            }
        };

        let defender_stats = {
            if let Some(d) = ctx.champions.get(&self.defender) {
                d.borrow().as_ref().state().stats.current.clone()
            } else {
                return;
            }
        };

        // Garen E tick damage formula
        let raw_damage = self.base_damage + (attacker_stats.attack_damage * self.ad_ratio);

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
                AbilitySlot::E,
                damage_result.final_damage,
                false,
            );
        }
        ctx.trigger_on_physical_damage(&self.attacker, &self.defender, &damage_result);
        ctx.trigger_on_damage_dealt(
            &self.attacker,
            damage_result.final_damage,
            true,
            lol_core::types::AbilitySlot::E,
        );

        if let Some(d) = ctx.champions.get(&self.defender) {
            let is_dead = d
                .borrow_mut()
                .take_damage(damage_result.final_damage)
                .is_dead;
            if is_dead {
                ctx.new_events.push((
                    0.0,
                    Box::new(lol_core::event::DeathEvent {
                        target: self.defender.clone(),
                    }),
                ));
            }
        }

        let new_hits = self.hits_landed + 1;

        // Apply shred if 6 hits landed
        if new_hits == 6 {
            ctx.apply_buff(&self.defender, Box::new(GarenEArmorShred));
        }

        if self.ticks_remaining > 1 {
            ctx.new_events.push((
                self.tick_interval,
                Box::new(JudgmentTickEvent {
                    attacker: self.attacker.clone(),
                    defender: self.defender.clone(),
                    ticks_remaining: self.ticks_remaining - 1,
                    tick_interval: self.tick_interval,
                    base_damage: self.base_damage,
                    ad_ratio: self.ad_ratio,
                    hits_landed: new_hits,
                }),
            ));
        }
    }
    fn name(&self) -> &str {
        "JudgmentTick"
    }
}

pub struct DemacianJusticeEvent {
    pub attacker: ChampionId,
    pub defender: ChampionId,
    pub base_damage: f64,
    pub missing_health_ratio: f64,
}

impl SimEvent for DemacianJusticeEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut lol_core::event::EventManager) {
        let defender_health = {
            if let Some(d) = ctx.champions.get(&self.defender) {
                let max_hp = d.borrow().as_ref().state().health.max;
                let current_hp = d.borrow().as_ref().state().health.current;
                max_hp - current_hp
            } else {
                0.0
            }
        };

        let raw_damage = self.base_damage + (defender_health * self.missing_health_ratio);

        let damage_result = DamagePipeline::process(
            raw_damage,
            DamageType::True,
            false,
            &StatBlock::new(), // Attacker stats not needed for true damage
            &StatBlock::new(), // Defender stats bypassed by true damage
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

        ctx.trigger_on_damage_dealt(
            &self.attacker,
            damage_result.final_damage,
            true,
            lol_core::types::AbilitySlot::R,
        );

        if let Some(d) = ctx.champions.get(&self.defender) {
            let is_dead = d
                .borrow_mut()
                .take_damage(damage_result.final_damage)
                .is_dead;
            if is_dead {
                ctx.new_events.push((
                    0.0,
                    Box::new(lol_core::event::DeathEvent {
                        target: self.defender.clone(),
                    }),
                ));
            }
        }
    }
    fn name(&self) -> &str {
        "DemacianJustice"
    }
}
