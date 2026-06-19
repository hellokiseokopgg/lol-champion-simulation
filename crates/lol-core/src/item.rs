use crate::damage::DamageResult;
use crate::event::SimContext;
use crate::stats::StatBlock;
use crate::types::ChampionId;

/// Represents an effect provided by an item (e.g., passive or active).
pub trait ItemEffect {
    fn name(&self) -> &str;

    /// Triggered when the champion hits a target with a basic attack.
    fn on_hit(
        &self,
        _sim: &mut SimContext,
        _actor: &ChampionId,
        _target: &ChampionId,
        _damage: &DamageResult,
    ) {
    }

    /// Triggered when the champion deals physical damage (from AA or Ability).
    fn on_physical_damage(
        &self,
        _sim: &mut SimContext,
        _actor: &ChampionId,
        _target: &ChampionId,
        _damage: &DamageResult,
    ) {
    }

    /// Triggered when the champion deals magic damage.
    fn on_magic_damage(
        &self,
        _sim: &mut SimContext,
        _actor: &ChampionId,
        _target: &ChampionId,
        _damage: &DamageResult,
    ) {
    }

    /// Triggered when the champion deals any damage.
    fn on_damage_dealt(
        &self,
        _sim: &mut SimContext,
        _actor: &ChampionId,
        _target: &ChampionId,
        _amount: f64,
        _is_ability: bool,
        _slot: crate::types::AbilitySlot,
    ) {
    }

    /// Triggered when the champion casts an ability.
    fn on_ability_cast(
        &self,
        _sim: &mut SimContext,
        _actor: &ChampionId,
        _slot: crate::types::AbilitySlot,
    ) {
    }

    /// Triggered when the simulation starts.
    fn on_simulation_start(&self, _sim: &mut SimContext, _actor: &ChampionId) {}

    /// Returns the active ability associated with this item, if any.
    fn active_ability(&self) -> Option<Box<dyn crate::ability::Ability>> {
        None
    }
}

/// Manages the item effects for a champion.
#[derive(Default)]
pub struct ItemManager {
    effects: Vec<Box<dyn ItemEffect>>,
}

impl ItemManager {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn add_effect(&mut self, effect: Box<dyn ItemEffect>) {
        self.effects.push(effect);
    }

    pub fn effects(&self) -> &Vec<Box<dyn ItemEffect>> {
        &self.effects
    }
}

pub struct BlackCleaverShred;

impl crate::buff::StatusEffect for BlackCleaverShred {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("BlackCleaverShred".into())
    }

    fn name(&self) -> &str {
        "Black Cleaver Shred"
    }

    fn duration(&self) -> f64 {
        6.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::AddStack
    }

    fn max_stacks(&self) -> u32 {
        6
    }

    fn stat_modifiers(&self, stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        // 4% armor reduction per stack, up to 24% at 6 stacks.
        stats.armor_reduction_percent = 0.04 * stacks as f64;
        stats
    }
}

pub struct BlackCleaverFervor;

impl crate::buff::StatusEffect for BlackCleaverFervor {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("BlackCleaverFervor".into())
    }

    fn name(&self) -> &str {
        "Black Cleaver Fervor"
    }

    fn duration(&self) -> f64 {
        2.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.movement_speed = 20.0;
        stats
    }
}

pub struct BlackCleaverEffect;

impl ItemEffect for BlackCleaverEffect {
    fn name(&self) -> &str {
        "Black Cleaver"
    }

    fn on_physical_damage(
        &self,
        ctx: &mut SimContext,
        actor: &ChampionId,
        target: &ChampionId,
        _damage: &DamageResult,
    ) {
        ctx.apply_buff(target, Box::new(BlackCleaverShred));
        ctx.apply_buff(actor, Box::new(BlackCleaverFervor));
    }
}

/// Represents a single item in the simulation.
pub struct Item {
    /// The unique identifier of the item.
    pub id: String,
    /// The human-readable name of the item.
    pub name: String,
    /// The raw stats provided by the item.
    pub stats: StatBlock,
    /// The special effects/passives the item provides.
    pub effects: Vec<Box<dyn ItemEffect>>,
}

/// Represents a champion's full item build (up to 6 standard items).
#[derive(Default)]
pub struct ItemBuild {
    /// The items currently in the build.
    pub items: Vec<Item>,
}

impl ItemBuild {
    /// Creates a new, empty item build.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds an item to the build, capping at 6 items.
    pub fn add_item(&mut self, item: Item) {
        if self.items.len() < 6 {
            self.items.push(item);
        }
    }

    /// Computes the aggregate stats from all items in the build.
    pub fn aggregate_stats(&self) -> StatBlock {
        let mut total = StatBlock::new();
        for item in &self.items {
            total = total + item.stats.clone();
        }
        // Note: Special multiplicative item effects (like Rabadon's Deathcap)
        // would be applied after the sum in a more advanced implementation.
        total
    }
}

pub struct HaltingSlashDebuff;

impl crate::buff::StatusEffect for HaltingSlashDebuff {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("HaltingSlashDebuff".into())
    }

    fn name(&self) -> &str {
        "제압의 가르기"
    }

    fn duration(&self) -> f64 {
        3.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        // -30% movement speed for the target (simplified)
        StatBlock::new()
    }
}

pub struct HeroicGaitBuff;

impl crate::buff::StatusEffect for HeroicGaitBuff {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("HeroicGaitBuff".into())
    }

    fn name(&self) -> &str {
        "비장한 걸음"
    }

    fn duration(&self) -> f64 {
        3.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        // +30% movement speed for the source (simplified)
        StatBlock::new()
    }
}

pub struct StridebreakerActive;

impl crate::ability::Ability for StridebreakerActive {
    fn slot(&self) -> crate::types::AbilitySlot {
        crate::types::AbilitySlot::Item(6631)
    }

    fn cast_time(&self) -> f64 {
        0.0
    }

    fn base_cooldown(&self, _level: u32) -> f64 {
        15.0
    }

    fn cost(&self, _level: u32) -> f64 {
        0.0
    }

    fn execute(
        &self,
        ctx: &mut crate::event::SimContext,
        actor: &crate::types::ChampionId,
        target: &crate::types::ChampionId,
    ) {
        let (attacker_stats, defender_stats, attacker_initial_ad) = {
            let attacker_cell = match ctx.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            let defender_cell = match ctx.champions.get(target) {
                Some(c) => c,
                None => return,
            };
            let attacker_ref = attacker_cell.borrow();
            let defender_ref = defender_cell.borrow();
            (
                attacker_ref.state().stats.current.clone(),
                defender_ref.state().stats.current.clone(),
                attacker_ref.state().stats.initial.attack_damage,
            )
        };

        let raw_damage = attacker_initial_ad * 0.8;

        let damage_result = crate::damage::DamagePipeline::process(
            raw_damage,
            crate::types::DamageType::Physical,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time,
                actor.clone(),
                target.clone(),
                self.slot(),
                damage_result.final_damage,
                false,
            );
        }

        ctx.trigger_on_physical_damage(actor, target, &damage_result);
        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, self.slot());

        ctx.apply_buff(target, Box::new(HaltingSlashDebuff));
        ctx.apply_buff(actor, Box::new(HeroicGaitBuff));
    }

    fn clone_box(&self) -> Box<dyn crate::ability::Ability> {
        Box::new(StridebreakerActive)
    }
}

pub struct StridebreakerEffect;

impl ItemEffect for StridebreakerEffect {
    fn name(&self) -> &str {
        "Stridebreaker"
    }

    fn active_ability(&self) -> Option<Box<dyn crate::ability::Ability>> {
        Some(Box::new(StridebreakerActive))
    }
}

/// Status effect representing Grievous Wounds, reducing healing.
pub struct GrievousWoundsBuff;

impl crate::buff::StatusEffect for GrievousWoundsBuff {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("GrievousWoundsBuff".into())
    }

    fn name(&self) -> &str {
        "Grievous Wounds"
    }

    fn duration(&self) -> f64 {
        3.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.grievous_wounds = 0.40;
        stats
    }
}

/// Item effect for Mortal Reminder, which applies Grievous Wounds on physical damage.
pub struct MortalReminderEffect;

impl ItemEffect for MortalReminderEffect {
    fn name(&self) -> &str {
        "Mortal Reminder"
    }

    fn on_physical_damage(
        &self,
        ctx: &mut SimContext,
        _actor: &crate::types::ChampionId,
        target: &crate::types::ChampionId,
        _damage: &DamageResult,
    ) {
        ctx.apply_buff(target, Box::new(GrievousWoundsBuff));
    }
}

/// Status effect representing Spectral Waltz from Phantom Dancer, granting movement speed and attack speed.
pub struct SpectralWaltzBuff;

impl crate::buff::StatusEffect for SpectralWaltzBuff {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("SpectralWaltzBuff".into())
    }

    fn name(&self) -> &str {
        "Spectral Waltz"
    }

    fn duration(&self) -> f64 {
        3.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::AddStack
    }

    fn max_stacks(&self) -> u32 {
        4
    }

    fn stat_modifiers(&self, stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.movement_speed = 24.0 * stacks as f64;
        if stacks >= 4 {
            stats.attack_speed = 0.30;
        }
        stats
    }
}

/// Item effect for Phantom Dancer, applying Spectral Waltz on basic attacks.
pub struct PhantomDancerEffect;

impl ItemEffect for PhantomDancerEffect {
    fn name(&self) -> &str {
        "Phantom Dancer"
    }

    fn on_hit(
        &self,
        sim: &mut SimContext,
        actor: &crate::types::ChampionId,
        _target: &crate::types::ChampionId,
        _damage: &DamageResult,
    ) {
        sim.apply_buff(actor, Box::new(SpectralWaltzBuff));
    }
}

pub struct NashorsToothEffect;

impl ItemEffect for NashorsToothEffect {
    fn name(&self) -> &str {
        "Nashor's Tooth"
    }

    fn on_hit(
        &self,
        ctx: &mut SimContext,
        actor: &crate::types::ChampionId,
        target: &crate::types::ChampionId,
        _damage: &DamageResult,
    ) {
        let attacker_stats = {
            let actor_cell = match ctx.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            actor_cell.borrow().state().stats.current.clone()
        };

        let defender_stats = {
            let defender_cell = match ctx.champions.get(target) {
                Some(c) => c,
                None => return,
            };
            defender_cell.borrow().state().stats.current.clone()
        };

        let ap = attacker_stats.ability_power;
        let raw_damage = 15.0 + 0.20 * ap;

        let damage_result = crate::damage::DamagePipeline::process(
            raw_damage,
            crate::types::DamageType::Magic,
            false,
            &attacker_stats,
            &defender_stats,
        );

        let (is_dead, current_hp, max_hp) = {
            let defender_cell = match ctx.champions.get(target) {
                Some(c) => c,
                None => return,
            };
            let mut defender = defender_cell.borrow_mut();
            let res = defender.take_damage(damage_result.final_damage);
            (
                res.is_dead,
                defender.state().health.current,
                defender.state().stats.current.health,
            )
        };

        let slot = crate::types::AbilitySlot::Item(3115);
        if let Some(recorder) = &ctx.recorder {
            let mut rec = recorder.borrow_mut();
            rec.record_damage(
                ctx.current_time,
                actor.clone(),
                target.clone(),
                slot,
                damage_result.final_damage,
                false,
            );
            rec.record_resource_update(
                ctx.current_time,
                target.clone(),
                "HP".to_string(),
                current_hp,
                max_hp,
            );
        }

        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, slot);

        if is_dead {
            ctx.new_events.push((
                0.0,
                Box::new(crate::event::DeathEvent {
                    target: target.clone(),
                }),
            ));
        }
    }
}

pub struct RabadonsDeathcapEffect;

impl ItemEffect for RabadonsDeathcapEffect {
    fn name(&self) -> &str {
        "Rabadon's Deathcap"
    }
}

pub struct KrakenSlayerStacks;

impl crate::buff::StatusEffect for KrakenSlayerStacks {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("KrakenSlayerStacks".into())
    }

    fn name(&self) -> &str {
        "Kraken Slayer Stacks"
    }

    fn duration(&self) -> f64 {
        3.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::AddStack
    }

    fn max_stacks(&self) -> u32 {
        2
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::default()
    }
}

pub struct KrakenSlayerEffect;

impl ItemEffect for KrakenSlayerEffect {
    fn name(&self) -> &str {
        "Kraken Slayer"
    }

    fn on_hit(
        &self,
        ctx: &mut SimContext,
        actor: &crate::types::ChampionId,
        target: &crate::types::ChampionId,
        _damage: &DamageResult,
    ) {
        let attacker_cell = match ctx.champions.get(actor) {
            Some(c) => c,
            None => return,
        };

        let stacks = {
            let attacker = attacker_cell.borrow();
            attacker
                .state()
                .buffs
                .get_stacks(&crate::types::EffectId("KrakenSlayerStacks".into()))
        };

        if stacks >= 2 {
            let (attacker_stats, level) = {
                let mut attacker = attacker_cell.borrow_mut();
                attacker
                    .state_mut()
                    .buffs
                    .remove_effect(&crate::types::EffectId("KrakenSlayerStacks".into()));
                attacker.update_stats(ctx.current_time);
                (
                    attacker.state().stats.current.clone(),
                    attacker.state().level as f64,
                )
            };

            let defender_stats = {
                let defender_cell = match ctx.champions.get(target) {
                    Some(c) => c,
                    None => return,
                };
                defender_cell.borrow().state().stats.current.clone()
            };

            let bonus_damage = 140.0 + 10.0 * (level - 1.0);

            let damage_result = crate::damage::DamagePipeline::process(
                bonus_damage,
                crate::types::DamageType::Physical,
                false,
                &attacker_stats,
                &defender_stats,
            );

            let (is_dead, current_hp, max_hp) = {
                let defender_cell = match ctx.champions.get(target) {
                    Some(c) => c,
                    None => return,
                };
                let mut defender = defender_cell.borrow_mut();
                let res = defender.take_damage(damage_result.final_damage);
                (
                    res.is_dead,
                    defender.state().health.current,
                    defender.state().stats.current.health,
                )
            };

            let slot = crate::types::AbilitySlot::Item(6672);
            if let Some(recorder) = &ctx.recorder {
                let mut rec = recorder.borrow_mut();
                rec.record_damage(
                    ctx.current_time,
                    actor.clone(),
                    target.clone(),
                    slot,
                    damage_result.final_damage,
                    false,
                );
                rec.record_resource_update(
                    ctx.current_time,
                    target.clone(),
                    "HP".to_string(),
                    current_hp,
                    max_hp,
                );
            }

            ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, slot);

            if is_dead {
                ctx.new_events.push((
                    0.0,
                    Box::new(crate::event::DeathEvent {
                        target: target.clone(),
                    }),
                ));
            }
        } else {
            ctx.apply_buff(actor, Box::new(KrakenSlayerStacks));
        }
    }
}

/// Status effect representing the bonus movement speed buff granted on-hit by Wit's End.
pub struct WitsEndMovementSpeed;

impl crate::buff::StatusEffect for WitsEndMovementSpeed {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("WitsEndMovementSpeed".into())
    }

    fn name(&self) -> &str {
        "Wit's End Movement Speed"
    }

    fn duration(&self) -> f64 {
        2.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.movement_speed = 20.0;
        stats
    }
}

/// Item effect implementation for Wit's End, dealing magic damage on-hit and granting movement speed.
pub struct WitsEndEffect;

impl ItemEffect for WitsEndEffect {
    fn name(&self) -> &str {
        "Wit's End"
    }

    fn on_hit(
        &self,
        ctx: &mut SimContext,
        actor: &crate::types::ChampionId,
        target: &crate::types::ChampionId,
        _damage: &DamageResult,
    ) {
        let (attacker_stats, level) = {
            let actor_cell = match ctx.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            let attacker = actor_cell.borrow();
            (
                attacker.state().stats.current.clone(),
                attacker.state().level as f64,
            )
        };
        let defender_stats = {
            let defender_cell = match ctx.champions.get(target) {
                Some(c) => c,
                None => return,
            };
            defender_cell.borrow().state().stats.current.clone()
        };

        let raw_damage = 15.0 + 3.8 * (level - 1.0);
        let damage_result = crate::damage::DamagePipeline::process(
            raw_damage,
            crate::types::DamageType::Magic,
            false,
            &attacker_stats,
            &defender_stats,
        );

        let (is_dead, current_hp, max_hp) = {
            let defender_cell = match ctx.champions.get(target) {
                Some(c) => c,
                None => return,
            };
            let mut defender = defender_cell.borrow_mut();
            let res = defender.take_damage(damage_result.final_damage);
            (
                res.is_dead,
                defender.state().health.current,
                defender.state().stats.current.health,
            )
        };

        let slot = crate::types::AbilitySlot::Item(3091);
        if let Some(recorder) = &ctx.recorder {
            let mut rec = recorder.borrow_mut();
            rec.record_damage(
                ctx.current_time,
                actor.clone(),
                target.clone(),
                slot,
                damage_result.final_damage,
                false,
            );
            rec.record_resource_update(
                ctx.current_time,
                target.clone(),
                "HP".to_string(),
                current_hp,
                max_hp,
            );
        }

        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, slot);
        ctx.apply_buff(actor, Box::new(WitsEndMovementSpeed));

        if is_dead {
            ctx.new_events.push((
                0.0,
                Box::new(crate::event::DeathEvent {
                    target: target.clone(),
                }),
            ));
        }
    }
}

/// Status effect representing the combat stacking damage amplification buff from Liandry's Torment.
pub struct LiandrysTormentCombat;

impl crate::buff::StatusEffect for LiandrysTormentCombat {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("LiandrysTormentCombat".into())
    }

    fn name(&self) -> &str {
        "Liandry's Torment Combat"
    }

    fn duration(&self) -> f64 {
        5.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::AddStack
    }

    fn max_stacks(&self) -> u32 {
        3
    }

    fn stat_modifiers(&self, stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.damage_amp_percent = 0.02 * stacks as f64;
        stats
    }
}

/// Status effect representing the burn damage-over-time debuff applied to targets by Liandry's Torment.
pub struct LiandrysTormentBurn;

impl crate::buff::StatusEffect for LiandrysTormentBurn {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("LiandrysTormentBurn".into())
    }

    fn name(&self) -> &str {
        "Liandry's Torment Burn"
    }

    fn duration(&self) -> f64 {
        3.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::new()
    }
}

/// Event representing a periodic tick of the Liandry's Torment burn effect.
pub struct LiandrysTormentBurnTickEvent {
    /// The champion receiving the burn damage.
    pub target: crate::types::ChampionId,
    /// The champion who applied the burn damage.
    pub attacker: crate::types::ChampionId,
}

impl crate::event::SimEvent for LiandrysTormentBurnTickEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut crate::event::EventManager) {
        let burn_id = crate::types::EffectId("LiandrysTormentBurn".into());
        let active = {
            let target_cell = match ctx.champions.get(&self.target) {
                Some(c) => c,
                None => return,
            };
            target_cell
                .borrow()
                .state()
                .buffs
                .has_effect_by_id(&burn_id, ctx.current_time)
        };

        if !active {
            return;
        }

        let attacker_stats = {
            let actor_cell = match ctx.champions.get(&self.attacker) {
                Some(c) => c,
                None => return,
            };
            actor_cell.borrow().state().stats.current.clone()
        };
        let defender_stats = {
            let defender_cell = match ctx.champions.get(&self.target) {
                Some(c) => c,
                None => return,
            };
            defender_cell.borrow().state().stats.current.clone()
        };

        let raw_damage = 0.01 * defender_stats.health;
        let damage_result = crate::damage::DamagePipeline::process(
            raw_damage,
            crate::types::DamageType::Magic,
            false,
            &attacker_stats,
            &defender_stats,
        );

        let (is_dead, current_hp, max_hp) = {
            let defender_cell = match ctx.champions.get(&self.target) {
                Some(c) => c,
                None => return,
            };
            let mut defender = defender_cell.borrow_mut();
            let res = defender.take_damage(damage_result.final_damage);
            (
                res.is_dead,
                defender.state().health.current,
                defender.state().stats.current.health,
            )
        };

        let slot = crate::types::AbilitySlot::Item(3151);
        if let Some(recorder) = &ctx.recorder {
            let mut rec = recorder.borrow_mut();
            rec.record_damage(
                ctx.current_time,
                self.attacker.clone(),
                self.target.clone(),
                slot,
                damage_result.final_damage,
                false,
            );
            rec.record_resource_update(
                ctx.current_time,
                self.target.clone(),
                "HP".to_string(),
                current_hp,
                max_hp,
            );
        }

        ctx.trigger_on_damage_dealt(&self.attacker, damage_result.final_damage, true, slot);

        if is_dead {
            ctx.new_events.push((
                0.0,
                Box::new(crate::event::DeathEvent {
                    target: self.target.clone(),
                }),
            ));
        } else {
            ctx.new_events.push((
                0.5,
                Box::new(LiandrysTormentBurnTickEvent {
                    target: self.target.clone(),
                    attacker: self.attacker.clone(),
                }),
            ));
        }
    }

    fn name(&self) -> &str {
        "LiandrysTormentBurnTickEvent"
    }
}

/// Item effect implementation for Liandry's Torment, applying the combat stack buff and the burn DOT effect.
pub struct LiandrysTormentEffect;

impl ItemEffect for LiandrysTormentEffect {
    fn name(&self) -> &str {
        "Liandry's Torment"
    }

    fn on_hit(
        &self,
        ctx: &mut SimContext,
        actor: &crate::types::ChampionId,
        _target: &crate::types::ChampionId,
        _damage: &DamageResult,
    ) {
        ctx.apply_buff(actor, Box::new(LiandrysTormentCombat));
    }

    fn on_damage_dealt(
        &self,
        sim: &mut SimContext,
        actor: &crate::types::ChampionId,
        target: &crate::types::ChampionId,
        _amount: f64,
        is_ability: bool,
        slot: crate::types::AbilitySlot,
    ) {
        if is_ability && !matches!(slot, crate::types::AbilitySlot::Item(_)) {
            sim.apply_buff(actor, Box::new(LiandrysTormentCombat));

            let burn_id = crate::types::EffectId("LiandrysTormentBurn".into());
            let already_burning = {
                let target_cell = match sim.champions.get(target) {
                    Some(c) => c,
                    None => return,
                };
                target_cell
                    .borrow()
                    .state()
                    .buffs
                    .has_effect_by_id(&burn_id, sim.current_time)
            };

            sim.apply_buff(target, Box::new(LiandrysTormentBurn));

            if !already_burning {
                sim.new_events.push((
                    0.5,
                    Box::new(LiandrysTormentBurnTickEvent {
                        target: target.clone(),
                        attacker: actor.clone(),
                    }),
                ));
            }
        }
    }
}

/// Status effect tracking the on-hit stacks build-up for Blade of the Ruined King's siphon passive.
pub struct BladeOfTheRuinedKingStacks;

impl crate::buff::StatusEffect for BladeOfTheRuinedKingStacks {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("BladeOfTheRuinedKingStacks".into())
    }

    fn name(&self) -> &str {
        "Blade of the Ruined King Stacks"
    }

    fn duration(&self) -> f64 {
        6.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::AddStack
    }

    fn max_stacks(&self) -> u32 {
        2
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::new()
    }
}

/// Status effect representing the cooldown duration of Blade of the Ruined King's siphon passive.
pub struct BladeOfTheRuinedKingCooldown;

impl crate::buff::StatusEffect for BladeOfTheRuinedKingCooldown {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("BladeOfTheRuinedKingCooldown".into())
    }

    fn name(&self) -> &str {
        "Blade of the Ruined King Cooldown"
    }

    fn duration(&self) -> f64 {
        30.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::new()
    }
}

/// Status effect representing the slow applied to the target by Blade of the Ruined King's siphon passive.
pub struct BladeOfTheRuinedKingSiphonSlow;

impl crate::buff::StatusEffect for BladeOfTheRuinedKingSiphonSlow {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("BladeOfTheRuinedKingSiphonSlow".into())
    }

    fn name(&self) -> &str {
        "Blade of the Ruined King Siphon Slow"
    }

    fn duration(&self) -> f64 {
        1.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn cc_type(&self) -> Option<crate::types::CCType> {
        Some(crate::types::CCType::Slow)
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        let mut stats = StatBlock::new();
        stats.movement_speed = -50.0;
        stats
    }
}

/// Item effect implementation for Blade of the Ruined King, dealing current health damage on-hit and managing the siphon passive.
pub struct BladeOfTheRuinedKingEffect;

impl ItemEffect for BladeOfTheRuinedKingEffect {
    fn name(&self) -> &str {
        "Blade of the Ruined King"
    }

    fn on_hit(
        &self,
        ctx: &mut SimContext,
        actor: &crate::types::ChampionId,
        target: &crate::types::ChampionId,
        _damage: &DamageResult,
    ) {
        let target_current_health = {
            let target_cell = match ctx.champions.get(target) {
                Some(c) => c,
                None => return,
            };
            target_cell.borrow().state().health.current
        };

        let raw_damage = (0.09 * target_current_health).max(15.0);

        let attacker_stats = {
            let actor_cell = match ctx.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            actor_cell.borrow().state().stats.current.clone()
        };
        let defender_stats = {
            let defender_cell = match ctx.champions.get(target) {
                Some(c) => c,
                None => return,
            };
            defender_cell.borrow().state().stats.current.clone()
        };

        let damage_result = crate::damage::DamagePipeline::process(
            raw_damage,
            crate::types::DamageType::Physical,
            false,
            &attacker_stats,
            &defender_stats,
        );

        let (is_dead, current_hp, max_hp) = {
            let defender_cell = match ctx.champions.get(target) {
                Some(c) => c,
                None => return,
            };
            let mut defender = defender_cell.borrow_mut();
            let res = defender.take_damage(damage_result.final_damage);
            (
                res.is_dead,
                defender.state().health.current,
                defender.state().stats.current.health,
            )
        };

        let slot = crate::types::AbilitySlot::Item(3153);
        if let Some(recorder) = &ctx.recorder {
            let mut rec = recorder.borrow_mut();
            rec.record_damage(
                ctx.current_time,
                actor.clone(),
                target.clone(),
                slot,
                damage_result.final_damage,
                false,
            );
            rec.record_resource_update(
                ctx.current_time,
                target.clone(),
                "HP".to_string(),
                current_hp,
                max_hp,
            );
        }

        ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, slot);

        if is_dead {
            ctx.new_events.push((
                0.0,
                Box::new(crate::event::DeathEvent {
                    target: target.clone(),
                }),
            ));
            return;
        }

        let cooldown_active = {
            let actor_cell = match ctx.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            actor_cell.borrow().state().buffs.has_effect_by_id(
                &crate::types::EffectId("BladeOfTheRuinedKingCooldown".into()),
                ctx.current_time,
            )
        };

        if !cooldown_active {
            let stacks = {
                let actor_cell = match ctx.champions.get(actor) {
                    Some(c) => c,
                    None => return,
                };
                actor_cell.borrow().state().buffs.get_stacks_by_id(
                    &crate::types::EffectId("BladeOfTheRuinedKingStacks".into()),
                    ctx.current_time,
                )
            };

            if stacks >= 2 {
                if let Some(champ_ref) = ctx.champions.get(actor) {
                    let mut champ = champ_ref.borrow_mut();
                    champ
                        .state_mut()
                        .buffs
                        .remove_effect(&crate::types::EffectId(
                            "BladeOfTheRuinedKingStacks".into(),
                        ));
                    champ.update_stats(ctx.current_time);
                }
                if let Some(recorder) = &ctx.recorder {
                    recorder.borrow_mut().record_buff_expire(
                        ctx.current_time,
                        actor.clone(),
                        "Blade of the Ruined King Stacks".to_string(),
                    );
                }

                ctx.apply_buff(actor, Box::new(BladeOfTheRuinedKingCooldown));
                ctx.apply_buff(target, Box::new(BladeOfTheRuinedKingSiphonSlow));

                let (attacker_stats, level) = {
                    let actor_cell = match ctx.champions.get(actor) {
                        Some(c) => c,
                        None => return,
                    };
                    let attacker = actor_cell.borrow();
                    (
                        attacker.state().stats.current.clone(),
                        attacker.state().level as f64,
                    )
                };
                let defender_stats = {
                    let defender_cell = match ctx.champions.get(target) {
                        Some(c) => c,
                        None => return,
                    };
                    defender_cell.borrow().state().stats.current.clone()
                };

                let raw_magic_damage = 40.0 + 6.6 * (level - 1.0);
                let damage_result = crate::damage::DamagePipeline::process(
                    raw_magic_damage,
                    crate::types::DamageType::Magic,
                    false,
                    &attacker_stats,
                    &defender_stats,
                );

                let (is_dead, current_hp, max_hp) = {
                    let defender_cell = match ctx.champions.get(target) {
                        Some(c) => c,
                        None => return,
                    };
                    let mut defender = defender_cell.borrow_mut();
                    let res = defender.take_damage(damage_result.final_damage);
                    (
                        res.is_dead,
                        defender.state().health.current,
                        defender.state().stats.current.health,
                    )
                };

                if let Some(recorder) = &ctx.recorder {
                    let mut rec = recorder.borrow_mut();
                    rec.record_damage(
                        ctx.current_time,
                        actor.clone(),
                        target.clone(),
                        slot,
                        damage_result.final_damage,
                        false,
                    );
                    rec.record_resource_update(
                        ctx.current_time,
                        target.clone(),
                        "HP".to_string(),
                        current_hp,
                        max_hp,
                    );
                }

                ctx.trigger_on_damage_dealt(actor, damage_result.final_damage, true, slot);

                if is_dead {
                    ctx.new_events.push((
                        0.0,
                        Box::new(crate::event::DeathEvent {
                            target: target.clone(),
                        }),
                    ));
                }
            } else {
                ctx.apply_buff(actor, Box::new(BladeOfTheRuinedKingStacks));
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Lich Bane
// -----------------------------------------------------------------------------

pub struct LichBaneSpellbladeBuff;

impl crate::buff::StatusEffect for LichBaneSpellbladeBuff {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("LichBaneSpellblade".into())
    }

    fn name(&self) -> &str {
        "Spellblade (Lich Bane)"
    }

    fn duration(&self) -> f64 {
        10.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::default()
    }
}

pub struct LichBaneCooldown;

impl crate::buff::StatusEffect for LichBaneCooldown {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("LichBaneCooldown".into())
    }

    fn name(&self) -> &str {
        "Lich Bane Cooldown"
    }

    fn duration(&self) -> f64 {
        1.5
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::Ignore
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::default()
    }
}

pub struct LichBaneEffect;

impl ItemEffect for LichBaneEffect {
    fn name(&self) -> &str {
        "Lich Bane"
    }

    fn on_ability_cast(
        &self,
        sim: &mut SimContext,
        actor: &ChampionId,
        slot: crate::types::AbilitySlot,
    ) {
        if matches!(
            slot,
            crate::types::AbilitySlot::Q
                | crate::types::AbilitySlot::W
                | crate::types::AbilitySlot::E
                | crate::types::AbilitySlot::R
        ) {
            let cooldown_active = {
                let actor_cell = match sim.champions.get(actor) {
                    Some(c) => c,
                    None => return,
                };
                actor_cell.borrow().state().buffs.has_effect_by_id(
                    &crate::types::EffectId("LichBaneCooldown".into()),
                    sim.current_time,
                )
            };

            if !cooldown_active {
                sim.apply_buff(actor, Box::new(LichBaneSpellbladeBuff));
                sim.apply_buff(actor, Box::new(LichBaneCooldown));
            }
        }
    }

    fn on_hit(
        &self,
        sim: &mut SimContext,
        actor: &ChampionId,
        target: &ChampionId,
        _damage: &DamageResult,
    ) {
        let has_spellblade = {
            let actor_cell = match sim.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            actor_cell.borrow().state().buffs.has_effect_by_id(
                &crate::types::EffectId("LichBaneSpellblade".into()),
                sim.current_time,
            )
        };

        if has_spellblade {
            let attacker_cell = match sim.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            let defender_cell = match sim.champions.get(target) {
                Some(c) => c,
                None => return,
            };

            // Consume the buff
            {
                let mut champ = attacker_cell.borrow_mut();
                champ
                    .state_mut()
                    .buffs
                    .remove_effect(&crate::types::EffectId("LichBaneSpellblade".into()));
                champ.update_stats(sim.current_time);
            }
            if let Some(recorder) = &sim.recorder {
                recorder.borrow_mut().record_buff_expire(
                    sim.current_time,
                    actor.clone(),
                    "Spellblade (Lich Bane)".to_string(),
                );
            }

            // Deal bonus magic damage = 150% base AD + 50% AP
            let (attacker_stats, defender_stats, base_ad) = {
                let attacker = attacker_cell.borrow();
                (
                    attacker.state().stats.current.clone(),
                    defender_cell.borrow().state().stats.current.clone(),
                    attacker.state().stats.base.attack_damage,
                )
            };

            let ap = attacker_stats.ability_power;
            let raw_damage = 1.5 * base_ad + 0.50 * ap;

            let damage_result = crate::damage::DamagePipeline::process(
                raw_damage,
                crate::types::DamageType::Magic,
                false,
                &attacker_stats,
                &defender_stats,
            );

            let (is_dead, current_hp, max_hp) = {
                let mut defender = defender_cell.borrow_mut();
                let res = defender.take_damage(damage_result.final_damage);
                (
                    res.is_dead,
                    defender.state().health.current,
                    defender.state().stats.current.health,
                )
            };

            let item_slot = crate::types::AbilitySlot::Item(3100);
            if let Some(recorder) = &sim.recorder {
                let mut rec = recorder.borrow_mut();
                rec.record_damage(
                    sim.current_time,
                    actor.clone(),
                    target.clone(),
                    item_slot,
                    damage_result.final_damage,
                    false,
                );
                rec.record_resource_update(
                    sim.current_time,
                    target.clone(),
                    "HP".to_string(),
                    current_hp,
                    max_hp,
                );
            }

            sim.trigger_on_damage_dealt(actor, damage_result.final_damage, true, item_slot);

            if is_dead {
                sim.new_events.push((
                    0.0,
                    Box::new(crate::event::DeathEvent {
                        target: target.clone(),
                    }),
                ));
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Trinity Force
// -----------------------------------------------------------------------------

pub struct TrinityForceSpellbladeBuff;

impl crate::buff::StatusEffect for TrinityForceSpellbladeBuff {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("TrinityForceSpellblade".into())
    }

    fn name(&self) -> &str {
        "Spellblade (Trinity Force)"
    }

    fn duration(&self) -> f64 {
        10.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::RefreshDuration
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::default()
    }
}

pub struct TrinityForceCooldown;

impl crate::buff::StatusEffect for TrinityForceCooldown {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("TrinityForceCooldown".into())
    }

    fn name(&self) -> &str {
        "Trinity Force Cooldown"
    }

    fn duration(&self) -> f64 {
        1.5
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::Ignore
    }

    fn max_stacks(&self) -> u32 {
        1
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::default()
    }
}

pub struct TrinityForceEffect;

impl ItemEffect for TrinityForceEffect {
    fn name(&self) -> &str {
        "Trinity Force"
    }

    fn on_ability_cast(
        &self,
        sim: &mut SimContext,
        actor: &ChampionId,
        slot: crate::types::AbilitySlot,
    ) {
        if matches!(
            slot,
            crate::types::AbilitySlot::Q
                | crate::types::AbilitySlot::W
                | crate::types::AbilitySlot::E
                | crate::types::AbilitySlot::R
        ) {
            let cooldown_active = {
                let actor_cell = match sim.champions.get(actor) {
                    Some(c) => c,
                    None => return,
                };
                actor_cell.borrow().state().buffs.has_effect_by_id(
                    &crate::types::EffectId("TrinityForceCooldown".into()),
                    sim.current_time,
                )
            };

            if !cooldown_active {
                sim.apply_buff(actor, Box::new(TrinityForceSpellbladeBuff));
                sim.apply_buff(actor, Box::new(TrinityForceCooldown));
            }
        }
    }

    fn on_hit(
        &self,
        sim: &mut SimContext,
        actor: &ChampionId,
        target: &ChampionId,
        _damage: &DamageResult,
    ) {
        let has_spellblade = {
            let actor_cell = match sim.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            actor_cell.borrow().state().buffs.has_effect_by_id(
                &crate::types::EffectId("TrinityForceSpellblade".into()),
                sim.current_time,
            )
        };

        if has_spellblade {
            let attacker_cell = match sim.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            let defender_cell = match sim.champions.get(target) {
                Some(c) => c,
                None => return,
            };

            // Consume the buff
            {
                let mut champ = attacker_cell.borrow_mut();
                champ
                    .state_mut()
                    .buffs
                    .remove_effect(&crate::types::EffectId("TrinityForceSpellblade".into()));
                champ.update_stats(sim.current_time);
            }
            if let Some(recorder) = &sim.recorder {
                recorder.borrow_mut().record_buff_expire(
                    sim.current_time,
                    actor.clone(),
                    "Spellblade (Trinity Force)".to_string(),
                );
            }

            // Deal bonus physical damage = 200% base AD
            let (attacker_stats, defender_stats, base_ad) = {
                let attacker = attacker_cell.borrow();
                (
                    attacker.state().stats.current.clone(),
                    defender_cell.borrow().state().stats.current.clone(),
                    attacker.state().stats.base.attack_damage,
                )
            };

            let raw_damage = 2.0 * base_ad;

            let damage_result = crate::damage::DamagePipeline::process(
                raw_damage,
                crate::types::DamageType::Physical,
                false,
                &attacker_stats,
                &defender_stats,
            );

            let (is_dead, current_hp, max_hp) = {
                let mut defender = defender_cell.borrow_mut();
                let res = defender.take_damage(damage_result.final_damage);
                (
                    res.is_dead,
                    defender.state().health.current,
                    defender.state().stats.current.health,
                )
            };

            let item_slot = crate::types::AbilitySlot::Item(3078);
            if let Some(recorder) = &sim.recorder {
                let mut rec = recorder.borrow_mut();
                rec.record_damage(
                    sim.current_time,
                    actor.clone(),
                    target.clone(),
                    item_slot,
                    damage_result.final_damage,
                    false,
                );
                rec.record_resource_update(
                    sim.current_time,
                    target.clone(),
                    "HP".to_string(),
                    current_hp,
                    max_hp,
                );
            }

            sim.trigger_on_damage_dealt(actor, damage_result.final_damage, true, item_slot);

            if is_dead {
                sim.new_events.push((
                    0.0,
                    Box::new(crate::event::DeathEvent {
                        target: target.clone(),
                    }),
                ));
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Luden's Companion
// -----------------------------------------------------------------------------

pub struct LudensCompanionCharges;

impl crate::buff::StatusEffect for LudensCompanionCharges {
    fn id(&self) -> crate::types::EffectId {
        crate::types::EffectId("LudensCompanionCharges".into())
    }

    fn name(&self) -> &str {
        "Luden's Companion Charges"
    }

    fn duration(&self) -> f64 {
        99999.0
    }

    fn refresh_behavior(&self) -> crate::buff::RefreshBehavior {
        crate::buff::RefreshBehavior::AddStack
    }

    fn max_stacks(&self) -> u32 {
        6
    }

    fn initial_stacks(&self) -> u32 {
        6
    }

    fn stat_modifiers(&self, _stacks: u32) -> StatBlock {
        StatBlock::default()
    }
}

pub struct LudensCompanionChargeEvent {
    pub actor: ChampionId,
}

impl crate::event::SimEvent for LudensCompanionChargeEvent {
    fn execute(&self, ctx: &mut SimContext, event_manager: &mut crate::event::EventManager) {
        let actor_cell = match ctx.champions.get(&self.actor) {
            Some(c) => c,
            None => return,
        };
        let has_luden = {
            let champ = actor_cell.borrow();
            champ
                .state()
                .items
                .effects()
                .iter()
                .any(|eff| eff.name() == "Luden's Companion")
        };

        if !has_luden {
            return;
        }

        let charges = {
            let champ = actor_cell.borrow();
            champ
                .state()
                .buffs
                .get_stacks(&crate::types::EffectId("LudensCompanionCharges".into()))
        };

        if charges == 0 {
            ctx.apply_buff(&self.actor, Box::new(LudensCompanionCharges));
            if let Some(champ_ref) = ctx.champions.get(&self.actor) {
                let mut champ = champ_ref.borrow_mut();
                champ
                    .state_mut()
                    .buffs
                    .set_stacks(&crate::types::EffectId("LudensCompanionCharges".into()), 1);
                champ.update_stats(ctx.current_time);
            }
        } else if charges < 6 {
            if let Some(champ_ref) = ctx.champions.get(&self.actor) {
                let mut champ = champ_ref.borrow_mut();
                champ.state_mut().buffs.set_stacks(
                    &crate::types::EffectId("LudensCompanionCharges".into()),
                    charges + 1,
                );
                champ.update_stats(ctx.current_time);
            }
            if let Some(recorder) = &ctx.recorder {
                recorder.borrow_mut().record_buff_apply(
                    ctx.current_time,
                    self.actor.clone(),
                    "Luden's Companion Charges".to_string(),
                );
            }
        }

        event_manager.schedule_in(
            3.0,
            Box::new(LudensCompanionChargeEvent {
                actor: self.actor.clone(),
            }),
        );
    }

    fn name(&self) -> &str {
        "LudensCompanionChargeEvent"
    }
}

pub struct LudensCompanionEffect;

impl ItemEffect for LudensCompanionEffect {
    fn name(&self) -> &str {
        "Luden's Companion"
    }

    fn on_simulation_start(&self, ctx: &mut SimContext, actor: &ChampionId) {
        ctx.apply_buff(actor, Box::new(LudensCompanionCharges));
        ctx.new_events.push((
            3.0,
            Box::new(LudensCompanionChargeEvent {
                actor: actor.clone(),
            }),
        ));
    }

    fn on_damage_dealt(
        &self,
        sim: &mut SimContext,
        actor: &ChampionId,
        target: &ChampionId,
        _amount: f64,
        is_ability: bool,
        slot: crate::types::AbilitySlot,
    ) {
        if is_ability
            && matches!(
                slot,
                crate::types::AbilitySlot::Q
                    | crate::types::AbilitySlot::W
                    | crate::types::AbilitySlot::E
                    | crate::types::AbilitySlot::R
            )
        {
            let attacker_cell = match sim.champions.get(actor) {
                Some(c) => c,
                None => return,
            };
            let charges = {
                let champ = attacker_cell.borrow();
                champ
                    .state()
                    .buffs
                    .get_stacks(&crate::types::EffectId("LudensCompanionCharges".into()))
            };

            if charges > 0 {
                let defender_cell = match sim.champions.get(target) {
                    Some(c) => c,
                    None => return,
                };

                // Consume all charges
                {
                    let mut champ = attacker_cell.borrow_mut();
                    champ
                        .state_mut()
                        .buffs
                        .remove_effect(&crate::types::EffectId("LudensCompanionCharges".into()));
                    champ.update_stats(sim.current_time);
                }

                if let Some(recorder) = &sim.recorder {
                    recorder.borrow_mut().record_buff_expire(
                        sim.current_time,
                        actor.clone(),
                        "Luden's Companion Charges".to_string(),
                    );
                }

                // Calculate bonus magic damage: (40 + 8% AP) * charges
                let (attacker_stats, defender_stats) = {
                    (
                        attacker_cell.borrow().state().stats.current.clone(),
                        defender_cell.borrow().state().stats.current.clone(),
                    )
                };

                let ap = attacker_stats.ability_power;
                let raw_damage = (40.0 + 0.08 * ap) * charges as f64;

                let damage_result = crate::damage::DamagePipeline::process(
                    raw_damage,
                    crate::types::DamageType::Magic,
                    false,
                    &attacker_stats,
                    &defender_stats,
                );

                let (is_dead, current_hp, max_hp) = {
                    let mut defender = defender_cell.borrow_mut();
                    let res = defender.take_damage(damage_result.final_damage);
                    (
                        res.is_dead,
                        defender.state().health.current,
                        defender.state().stats.current.health,
                    )
                };

                let item_slot = crate::types::AbilitySlot::Item(6655);
                if let Some(recorder) = &sim.recorder {
                    let mut rec = recorder.borrow_mut();
                    rec.record_damage(
                        sim.current_time,
                        actor.clone(),
                        target.clone(),
                        item_slot,
                        damage_result.final_damage,
                        false,
                    );
                    rec.record_resource_update(
                        sim.current_time,
                        target.clone(),
                        "HP".to_string(),
                        current_hp,
                        max_hp,
                    );
                }

                sim.trigger_on_damage_dealt(actor, damage_result.final_damage, true, item_slot);

                if is_dead {
                    sim.new_events.push((
                        0.0,
                        Box::new(crate::event::DeathEvent {
                            target: target.clone(),
                        }),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_build_stats() {
        let mut build = ItemBuild::new();

        let item1 = Item {
            id: "1038".to_string(),
            name: "B.F. Sword".to_string(),
            stats: StatBlock {
                attack_damage: 40.0,
                ..Default::default()
            },
            effects: vec![],
        };

        let item2 = Item {
            id: "1036".to_string(),
            name: "Long Sword".to_string(),
            stats: StatBlock {
                attack_damage: 10.0,
                ..Default::default()
            },
            effects: vec![],
        };

        build.add_item(item1);
        build.add_item(item2);

        let total = build.aggregate_stats();
        assert_eq!(total.attack_damage, 50.0);
    }

    #[test]
    fn test_item_build_limit() {
        let mut build = ItemBuild::new();
        for i in 0..10 {
            build.add_item(Item {
                id: i.to_string(),
                name: "Test Item".to_string(),
                stats: StatBlock::new(),
                effects: vec![],
            });
        }
        assert_eq!(build.items.len(), 6);
    }

    use crate::champion::ChampionState;
    use crate::stats::StatBlock;
    use crate::types::{DamageType, ResourceType};

    struct DummyChampionInstance {
        state: ChampionState,
    }
    impl crate::champion::ChampionInstance for DummyChampionInstance {
        fn state(&self) -> &ChampionState {
            &self.state
        }
        fn state_mut(&mut self) -> &mut ChampionState {
            &mut self.state
        }
        fn update_stats(&mut self, _time: crate::types::SimTime) {}
        fn get_ability(
            &self,
            _slot: crate::types::AbilitySlot,
        ) -> Option<&dyn crate::ability::Ability> {
            None
        }
        fn take_damage(&mut self, amount: f64) -> crate::types::TakeDamageResult {
            let is_dead = self.state.health.reduce(amount);
            crate::types::TakeDamageResult {
                actual_damage: amount,
                is_dead,
            }
        }
    }

    #[test]
    fn test_black_cleaver_shred() {
        let mut sim = SimContext {
            champions: std::collections::HashMap::new(),
            current_time: crate::types::SimTime::new(0.0),
            new_events: vec![],
            is_simulation_over: false,
            recorder: None,
        };
        let target_id = ChampionId("Target".into());

        let mut target_stats = StatBlock::new();
        target_stats.armor = 100.0;
        let mut target_state = ChampionState::new(
            1,
            target_stats,
            StatBlock::new(),
            ResourceType::None,
            StatBlock::new(),
            StatBlock::new(),
            vec![],
        );
        target_state.stats.recalculate_current(&StatBlock::new());

        sim.champions.insert(
            target_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: target_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let actor_id = ChampionId("Actor".into());
        let bc_effect = BlackCleaverEffect;
        let damage_result = crate::damage::DamageResult {
            raw_damage: 10.0,
            final_damage: 10.0,
            mitigated_damage: 0.0,
            is_critical: false,
            damage_type: DamageType::Physical,
        };

        // 1st hit
        bc_effect.on_physical_damage(&mut sim, &actor_id, &target_id, &damage_result);

        let armor_reduction = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .aggregate_stats()
            .armor_reduction_percent;
        assert_eq!(armor_reduction, 0.04); // 1 stack = 4%

        // 6th hit
        for _ in 0..5 {
            bc_effect.on_physical_damage(&mut sim, &actor_id, &target_id, &damage_result);
        }

        let armor_reduction_max = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .aggregate_stats()
            .armor_reduction_percent;
        assert_eq!(armor_reduction_max, 0.24); // 6 stacks = 24%

        // 7th hit (should not exceed 6 stacks)
        bc_effect.on_physical_damage(&mut sim, &actor_id, &target_id, &damage_result);
        let armor_reduction_cap = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .aggregate_stats()
            .armor_reduction_percent;
        assert_eq!(armor_reduction_cap, 0.24);
    }

    #[test]
    fn test_infinity_edge_crit_damage_scaling() {
        let mut attacker_stats = StatBlock::new();
        // IE adds 0.40 crit damage. Base is 1.75. So total is 2.15.
        attacker_stats.crit_damage = 1.75 + 0.40;
        let defender_stats = StatBlock::new();

        let result = crate::damage::DamagePipeline::process(
            100.0,
            crate::types::DamageType::Physical,
            true, // is_critical
            &attacker_stats,
            &defender_stats,
        );

        assert_eq!(result.raw_damage, 215.0);
    }

    #[test]
    fn test_mortal_reminder_grievous_wounds() {
        let mut sim = SimContext {
            champions: std::collections::HashMap::new(),
            current_time: crate::types::SimTime::new(0.0),
            new_events: vec![],
            is_simulation_over: false,
            recorder: None,
        };
        let target_id = ChampionId("Target".into());
        let mut target_state = ChampionState::new(
            1,
            StatBlock::new(),
            StatBlock::new(),
            ResourceType::None,
            StatBlock::new(),
            StatBlock::new(),
            vec![],
        );
        target_state.stats.recalculate_current(&StatBlock::new());

        sim.champions.insert(
            target_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: target_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let actor_id = ChampionId("Actor".into());
        let mr_effect = MortalReminderEffect;
        let damage_result = crate::damage::DamageResult {
            raw_damage: 10.0,
            final_damage: 10.0,
            mitigated_damage: 0.0,
            is_critical: false,
            damage_type: DamageType::Physical,
        };

        // Before hit, no Grievous Wounds
        let grievous_wounds_before = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .aggregate_stats()
            .grievous_wounds;
        assert_eq!(grievous_wounds_before, 0.0);

        // Apply physical damage
        mr_effect.on_physical_damage(&mut sim, &actor_id, &target_id, &damage_result);

        // After hit, Grievous Wounds buff applied (value: 0.40)
        let grievous_wounds_after = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .aggregate_stats()
            .grievous_wounds;
        assert_eq!(grievous_wounds_after, 0.40);
    }

    #[test]
    fn test_phantom_dancer_spectral_waltz() {
        let mut sim = SimContext {
            champions: std::collections::HashMap::new(),
            current_time: crate::types::SimTime::new(0.0),
            new_events: vec![],
            is_simulation_over: false,
            recorder: None,
        };
        let actor_id = ChampionId("Actor".into());
        let mut actor_state = ChampionState::new(
            1,
            StatBlock::new(),
            StatBlock::new(),
            ResourceType::None,
            StatBlock::new(),
            StatBlock::new(),
            vec![],
        );
        actor_state.stats.recalculate_current(&StatBlock::new());

        sim.champions.insert(
            actor_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: actor_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let target_id = ChampionId("Target".into());
        let pd_effect = PhantomDancerEffect;
        let damage_result = crate::damage::DamageResult {
            raw_damage: 10.0,
            final_damage: 10.0,
            mitigated_damage: 0.0,
            is_critical: false,
            damage_type: DamageType::Physical,
        };

        // 1st stack
        pd_effect.on_hit(&mut sim, &actor_id, &target_id, &damage_result);
        let buffs_stats_1 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .aggregate_stats();
        assert_eq!(buffs_stats_1.movement_speed, 24.0);
        assert_eq!(buffs_stats_1.attack_speed, 0.0);

        // 2nd stack
        pd_effect.on_hit(&mut sim, &actor_id, &target_id, &damage_result);
        // 3rd stack
        pd_effect.on_hit(&mut sim, &actor_id, &target_id, &damage_result);
        let buffs_stats_3 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .aggregate_stats();
        assert_eq!(buffs_stats_3.movement_speed, 72.0);
        assert_eq!(buffs_stats_3.attack_speed, 0.0);

        // 4th stack (should grant bonus attack speed)
        pd_effect.on_hit(&mut sim, &actor_id, &target_id, &damage_result);
        let buffs_stats_4 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .aggregate_stats();
        assert_eq!(buffs_stats_4.movement_speed, 96.0);
        assert_eq!(buffs_stats_4.attack_speed, 0.30);

        // 5th hit (should not exceed 4 stacks)
        pd_effect.on_hit(&mut sim, &actor_id, &target_id, &damage_result);
        let buffs_stats_5 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .aggregate_stats();
        assert_eq!(buffs_stats_5.movement_speed, 96.0);
        assert_eq!(buffs_stats_5.attack_speed, 0.30);
    }

    #[test]
    fn test_nashors_tooth_on_hit() {
        let mut sim = SimContext {
            champions: std::collections::HashMap::new(),
            current_time: crate::types::SimTime::new(0.0),
            new_events: vec![],
            is_simulation_over: false,
            recorder: None,
        };
        let actor_id = ChampionId("Actor".into());
        let mut actor_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        actor_state.stats.current.ability_power = 100.0;
        sim.champions.insert(
            actor_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: actor_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let target_id = ChampionId("Target".into());
        let mut target_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        target_state.health.max = 1000.0;
        target_state.health.current = 1000.0;
        sim.champions.insert(
            target_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: target_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let nashors = NashorsToothEffect;
        let dummy_damage = crate::damage::DamageResult {
            raw_damage: 10.0,
            final_damage: 10.0,
            mitigated_damage: 0.0,
            is_critical: false,
            damage_type: DamageType::Physical,
        };

        nashors.on_hit(&mut sim, &actor_id, &target_id, &dummy_damage);

        let target_hp = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;

        // 1000.0 - (15.0 + 0.20 * 100.0) = 1000.0 - 35.0 = 965.0
        assert_eq!(target_hp, 965.0);
    }

    #[test]
    fn test_kraken_slayer_on_hit() {
        let mut sim = SimContext {
            champions: std::collections::HashMap::new(),
            current_time: crate::types::SimTime::new(0.0),
            new_events: vec![],
            is_simulation_over: false,
            recorder: None,
        };
        let actor_id = ChampionId("Actor".into());
        let actor_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        sim.champions.insert(
            actor_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: actor_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let target_id = ChampionId("Target".into());
        let mut target_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        target_state.health.max = 1000.0;
        target_state.health.current = 1000.0;
        sim.champions.insert(
            target_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: target_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let kraken = KrakenSlayerEffect;
        let dummy_damage = crate::damage::DamageResult {
            raw_damage: 10.0,
            final_damage: 10.0,
            mitigated_damage: 0.0,
            is_critical: false,
            damage_type: DamageType::Physical,
        };

        // 1st hit -> 1 stack
        kraken.on_hit(&mut sim, &actor_id, &target_id, &dummy_damage);
        let stacks1 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&crate::types::EffectId("KrakenSlayerStacks".into()));
        assert_eq!(stacks1, 1);

        // 2nd hit -> 2 stacks
        kraken.on_hit(&mut sim, &actor_id, &target_id, &dummy_damage);
        let stacks2 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&crate::types::EffectId("KrakenSlayerStacks".into()));
        assert_eq!(stacks2, 2);

        // 3rd hit -> triggers extra physical damage = 140.0 + 10.0 * (1.0 - 1.0) = 140.0, and resets stacks to 0
        kraken.on_hit(&mut sim, &actor_id, &target_id, &dummy_damage);
        let stacks3 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&crate::types::EffectId("KrakenSlayerStacks".into()));
        assert_eq!(stacks3, 0);

        let target_hp = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        // 1000.0 - 140.0 = 860.0
        assert_eq!(target_hp, 860.0);
    }

    #[test]
    fn test_wits_end_on_hit() {
        let mut sim = SimContext {
            champions: std::collections::HashMap::new(),
            current_time: crate::types::SimTime::new(0.0),
            new_events: vec![],
            is_simulation_over: false,
            recorder: None,
        };
        let actor_id = ChampionId("Actor".into());
        let actor_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        sim.champions.insert(
            actor_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: actor_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let target_id = ChampionId("Target".into());
        let mut target_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        target_state.health.max = 1000.0;
        target_state.health.current = 1000.0;
        sim.champions.insert(
            target_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: target_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let wits_end = WitsEndEffect;
        let dummy_damage = crate::damage::DamageResult {
            raw_damage: 10.0,
            final_damage: 10.0,
            mitigated_damage: 0.0,
            is_critical: false,
            damage_type: DamageType::Physical,
        };

        wits_end.on_hit(&mut sim, &actor_id, &target_id, &dummy_damage);

        let target_hp = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        // Level 1 magic damage: 15.0. 1000 - 15.0 = 985.0
        assert_eq!(target_hp, 985.0);

        let has_ms_buff = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(
                &crate::types::EffectId("WitsEndMovementSpeed".into()),
                sim.current_time,
            );
        assert!(has_ms_buff);
    }

    #[test]
    fn test_liandrys_torment_passive() {
        let mut sim = SimContext {
            champions: std::collections::HashMap::new(),
            current_time: crate::types::SimTime::new(0.0),
            new_events: vec![],
            is_simulation_over: false,
            recorder: None,
        };
        let actor_id = ChampionId("Actor".into());
        let actor_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        sim.champions.insert(
            actor_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: actor_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let target_id = ChampionId("Target".into());
        let mut target_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        target_state.health.max = 1000.0;
        target_state.health.current = 1000.0;
        sim.champions.insert(
            target_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: target_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let liandrys = LiandrysTormentEffect;
        let dummy_damage = crate::damage::DamageResult {
            raw_damage: 10.0,
            final_damage: 10.0,
            mitigated_damage: 0.0,
            is_critical: false,
            damage_type: DamageType::Physical,
        };

        // Basic attack hit -> applies combat buff
        liandrys.on_hit(&mut sim, &actor_id, &target_id, &dummy_damage);
        let combat_stacks = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&crate::types::EffectId("LiandrysTormentCombat".into()));
        assert_eq!(combat_stacks, 1);

        // Ability hit -> applies combat buff & burn buff
        liandrys.on_damage_dealt(
            &mut sim,
            &actor_id,
            &target_id,
            10.0,
            true,
            crate::types::AbilitySlot::Q,
        );
        let combat_stacks_after_ability = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&crate::types::EffectId("LiandrysTormentCombat".into()));
        assert_eq!(combat_stacks_after_ability, 2);

        let has_burn = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(
                &crate::types::EffectId("LiandrysTormentBurn".into()),
                sim.current_time,
            );
        assert!(has_burn);

        // Verify scheduled tick event
        assert_eq!(sim.new_events.len(), 4); // 3 from apply_buff (two combat buff applications and one burn), 1 from new tick event
        let tick_event_exists = sim
            .new_events
            .iter()
            .any(|(_, event)| event.name() == "LiandrysTormentBurnTickEvent");
        assert!(tick_event_exists);
    }

    #[test]
    fn test_blade_of_the_ruined_king_passive() {
        let mut sim = SimContext {
            champions: std::collections::HashMap::new(),
            current_time: crate::types::SimTime::new(0.0),
            new_events: vec![],
            is_simulation_over: false,
            recorder: None,
        };
        let actor_id = ChampionId("Actor".into());
        let actor_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        sim.champions.insert(
            actor_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: actor_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let target_id = ChampionId("Target".into());
        let mut target_state = ChampionState::new(
            1,
            StatBlock::default(),
            StatBlock::default(),
            ResourceType::None,
            StatBlock::default(),
            StatBlock::default(),
            vec![],
        );
        target_state.health.max = 1000.0;
        target_state.health.current = 1000.0;
        sim.champions.insert(
            target_id.clone(),
            std::rc::Rc::new(std::cell::RefCell::new(Box::new(DummyChampionInstance {
                state: target_state,
            })
                as Box<dyn crate::champion::ChampionInstance>)),
        );

        let botrk = BladeOfTheRuinedKingEffect;
        let dummy_damage = crate::damage::DamageResult {
            raw_damage: 10.0,
            final_damage: 10.0,
            mitigated_damage: 0.0,
            is_critical: false,
            damage_type: DamageType::Physical,
        };

        // 1st attack
        botrk.on_hit(&mut sim, &actor_id, &target_id, &dummy_damage);
        // Deals 9% of 1000 = 90
        let target_hp_1 = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        assert_eq!(target_hp_1, 910.0);

        let stacks1 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&crate::types::EffectId("BladeOfTheRuinedKingStacks".into()));
        assert_eq!(stacks1, 1);

        // 2nd attack
        botrk.on_hit(&mut sim, &actor_id, &target_id, &dummy_damage);
        // Deals 9% of 910 = 81.9 -> target_hp = 910 - 81.9 = 828.1
        let target_hp_2 = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        assert!((target_hp_2 - 828.1).abs() < 0.001);

        let stacks2 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&crate::types::EffectId("BladeOfTheRuinedKingStacks".into()));
        assert_eq!(stacks2, 2);

        // 3rd attack
        botrk.on_hit(&mut sim, &actor_id, &target_id, &dummy_damage);
        // Deals 9% of 828.1 = 74.529 -> target_hp = 828.1 - 74.529 = 753.571
        // And triggers siphon magic damage 40.0 -> target_hp = 753.571 - 40.0 = 713.571
        let target_hp_3 = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .health
            .current;
        assert!((target_hp_3 - 713.571).abs() < 0.001);

        // Stacks are consumed
        let stacks3 = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .get_stacks(&crate::types::EffectId("BladeOfTheRuinedKingStacks".into()));
        assert_eq!(stacks3, 0);

        // Cooldown applied
        let has_cd = sim
            .champions
            .get(&actor_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(
                &crate::types::EffectId("BladeOfTheRuinedKingCooldown".into()),
                sim.current_time,
            );
        assert!(has_cd);

        // Slow applied to target
        let has_slow = sim
            .champions
            .get(&target_id)
            .unwrap()
            .borrow()
            .state()
            .buffs
            .has_effect_by_id(
                &crate::types::EffectId("BladeOfTheRuinedKingSiphonSlow".into()),
                sim.current_time,
            );
        assert!(has_slow);
    }
}
