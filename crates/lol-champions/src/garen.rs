use lol_core::ability::Ability;
use lol_core::buff::{RefreshBehavior, StatusEffect};
use lol_core::champion::{ChampionConfig, ChampionInstance, ChampionModule, ChampionState};
use lol_core::damage::DamagePipeline;
use lol_core::event::{SimContext, SimEvent};
use lol_core::stats::StatBlock;
use lol_core::types::{AbilitySlot, ChampionId, EffectId, DamageType};

pub struct GarenModule;

impl ChampionModule for GarenModule {
    fn id(&self) -> &str {
        "Garen"
    }

    fn create_instance(&self, config: ChampionConfig) -> Box<dyn ChampionInstance> {
        let mut base_stats = config.base_stats.clone();
        
        // Garen W Passive: Gains 30 Armor and MR at max stacks. We assume max stacks.
        base_stats.armor += 30.0;
        base_stats.magic_resist += 30.0;
        
        let mut state = ChampionState::new(base_stats, lol_core::types::ResourceType::None);
        
        // Initialize abilities to rank 5 for testing (except R to 3)
        if let Some(q) = state.abilities.get_state_mut(AbilitySlot::Q) { q.level = 5; }
        if let Some(w) = state.abilities.get_state_mut(AbilitySlot::W) { w.level = 5; }
        if let Some(e) = state.abilities.get_state_mut(AbilitySlot::E) { e.level = 5; }
        if let Some(r) = state.abilities.get_state_mut(AbilitySlot::R) { r.level = 3; }

        Box::new(GarenInstance {
            state,
            _config: config,
            abilities: vec![
                Box::new(GarenAutoAttack),
                Box::new(GarenQ),
                Box::new(GarenW),
                Box::new(GarenE),
                Box::new(GarenR),
            ],
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

    fn update_stats(&mut self) {
        let buffs_stats = self.state.buffs.aggregate_stats();
        self.state.stats.recalculate_current(&buffs_stats);
    }
    
    fn get_ability(&self, slot: lol_core::types::AbilitySlot) -> Option<&dyn lol_core::ability::Ability> {
        self.abilities.iter().find(|a| a.slot() == slot).map(|a| a.as_ref())
    }
}

// -----------------------------------------------------------------------------
// Buffs
// -----------------------------------------------------------------------------

pub struct GarenQBuff;
impl StatusEffect for GarenQBuff {
    fn id(&self) -> EffectId { EffectId("GarenQ".into()) }
    fn name(&self) -> &str { "Decisive Strike" }
    fn duration(&self) -> f64 { 4.5 }
    fn refresh_behavior(&self) -> RefreshBehavior { RefreshBehavior::RefreshDuration }
    fn max_stacks(&self) -> u32 { 1 }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        // Provides 30% MS for simplified logic
        stats.movement_speed = 100.0; // Mock flat MS for now since we don't have % MS multiplier
        stats
    }
}

pub struct GarenWBuff;
impl StatusEffect for GarenWBuff {
    fn id(&self) -> EffectId { EffectId("GarenW".into()) }
    fn name(&self) -> &str { "Courage" }
    fn duration(&self) -> f64 { 5.0 } // 5 seconds at rank 5
    fn refresh_behavior(&self) -> RefreshBehavior { RefreshBehavior::RefreshDuration }
    fn max_stacks(&self) -> u32 { 1 }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.damage_reduction_percent = 0.3; // 30% damage reduction
        stats
    }
}

pub struct GarenEArmorShred;
impl StatusEffect for GarenEArmorShred {
    fn id(&self) -> EffectId { EffectId("GarenEShred".into()) }
    fn name(&self) -> &str { "Judgment Shred" }
    fn duration(&self) -> f64 { 6.0 }
    fn refresh_behavior(&self) -> RefreshBehavior { RefreshBehavior::RefreshDuration }
    fn max_stacks(&self) -> u32 { 1 }
    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.armor_reduction_percent = 0.25; // 25% armor reduction
        stats
    }
}

// -----------------------------------------------------------------------------
// Abilities
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct GarenQ;
impl Ability for GarenQ {
    fn slot(&self) -> AbilitySlot { AbilitySlot::Q }
    fn cast_time(&self) -> f64 { 0.0 }
    fn base_cooldown(&self, _level: u32) -> f64 { 8.0 }
    fn cost(&self, _level: u32) -> f64 { 0.0 }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, _target: &ChampionId) {
        if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow_mut().state_mut().buffs.apply_effect(Box::new(GarenQBuff), ctx.current_time);
            champ_ref.borrow_mut().update_stats();
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
}

#[derive(Clone)]
pub struct GarenW;
impl Ability for GarenW {
    fn slot(&self) -> AbilitySlot { AbilitySlot::W }
    fn cast_time(&self) -> f64 { 0.0 }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 23.0, 2 => 21.0, 3 => 19.0, 4 => 17.0, 5 => 15.0,
            _ => 15.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 { 0.0 }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, _target: &ChampionId) {
        if let Some(champ_ref) = ctx.champions.get(actor) {
            champ_ref.borrow_mut().state_mut().buffs.apply_effect(Box::new(GarenWBuff), ctx.current_time);
            champ_ref.borrow_mut().update_stats();
        }
    }
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
}

#[derive(Clone)]
pub struct GarenE;
impl Ability for GarenE {
    fn slot(&self) -> AbilitySlot { AbilitySlot::E }
    fn cast_time(&self) -> f64 { 0.0 }
    fn base_cooldown(&self, _level: u32) -> f64 { 9.0 }
    fn cost(&self, _level: u32) -> f64 { 0.0 }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let e_event = JudgmentTickEvent {
            attacker: actor.clone(),
            defender: target.clone(),
            ticks_remaining: 7,
            tick_interval: 0.5,
            base_damage: 20.0,
            ad_ratio: 0.4,
            hits_landed: 0,
        };
        ctx.new_events.push((0.0, Box::new(e_event)));
    }
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
}

#[derive(Clone)]
pub struct GarenR;
impl Ability for GarenR {
    fn slot(&self) -> AbilitySlot { AbilitySlot::R }
    fn cast_time(&self) -> f64 { 0.5 }
    fn base_cooldown(&self, level: u32) -> f64 {
        match level {
            1 => 120.0, 2 => 100.0, 3 => 80.0,
            _ => 80.0,
        }
    }
    fn cost(&self, _level: u32) -> f64 { 0.0 }
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
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
}

#[derive(Clone)]
pub struct GarenAutoAttack;
impl Ability for GarenAutoAttack {
    fn slot(&self) -> AbilitySlot { AbilitySlot::AutoAttack }
    fn cast_time(&self) -> f64 { 0.25 }
    fn base_cooldown(&self, _level: u32) -> f64 { 1.0 }
    fn cost(&self, _level: u32) -> f64 { 0.0 }
    fn execute(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId) {
        let attacker_stats = {
            if let Some(a) = ctx.champions.get(actor) {
                a.borrow().as_ref().state().stats.current.clone()
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
    }
    fn clone_box(&self) -> Box<dyn Ability> { Box::new(self.clone()) }
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

        let new_hits = self.hits_landed + 1;
        
        // Apply shred if 6 hits landed
        if new_hits == 6 {
            if let Some(d) = ctx.champions.get(&self.defender) {
                d.borrow_mut().as_mut().state_mut().buffs.apply_effect(Box::new(GarenEArmorShred), ctx.current_time);
                d.borrow_mut().as_mut().update_stats();
            }
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
    fn name(&self) -> &str { "JudgmentTick" }
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
                // In a real sim, we need current HP tracking.
                // For MVP, we assume current HP is tracked in `state.health.current` (if we implement resource)
                // We'll use 50% missing health as a mock test for now since Resource tracking is basic.
                let max_hp = d.borrow().as_ref().state().stats.current.health;
                // mock 50% missing
                let missing_hp = max_hp * 0.5; 
                missing_hp
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
    }
    fn name(&self) -> &str { "DemacianJustice" }
}
