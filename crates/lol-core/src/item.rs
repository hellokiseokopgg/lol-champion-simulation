use crate::stats::StatBlock;
use crate::event::SimContext;
use crate::types::ChampionId;
use crate::damage::DamageResult;

/// Represents an effect provided by an item (e.g., passive or active).
pub trait ItemEffect {
    fn name(&self) -> &str;
    
    /// Triggered when the champion hits a target with a basic attack.
    fn on_hit(&self, _sim: &mut SimContext, _actor: &ChampionId, _target: &ChampionId, _damage: &DamageResult) {}

    /// Triggered when the champion deals physical damage (from AA or Ability).
    fn on_physical_damage(&self, _sim: &mut SimContext, _actor: &ChampionId, _target: &ChampionId, _damage: &DamageResult) {}
    
    /// Triggered when the champion deals magic damage.
    fn on_magic_damage(&self, _sim: &mut SimContext, _actor: &ChampionId, _target: &ChampionId, _damage: &DamageResult) {}
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

    fn on_physical_damage(&self, ctx: &mut SimContext, actor: &ChampionId, target: &ChampionId, _damage: &DamageResult) {
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

    fn execute(&self, ctx: &mut crate::event::SimContext, actor: &crate::types::ChampionId, target: &crate::types::ChampionId) {
        let (attacker_stats, defender_stats) = {
            let attacker_ref = ctx.champions.get(actor).unwrap().borrow();
            let defender_ref = ctx.champions.get(target).unwrap().borrow();
            (
                attacker_ref.state().stats.current.clone(),
                defender_ref.state().stats.current.clone(),
            )
        };

        let raw_damage = attacker_stats.attack_damage * 0.8;

        let damage_result = crate::damage::DamagePipeline::process(
            raw_damage,
            crate::types::DamageType::Physical,
            false,
            &attacker_stats,
            &defender_stats,
        );

        if let Some(recorder) = &ctx.recorder {
            recorder.borrow_mut().record_damage(
                ctx.current_time, actor.clone(), target.clone(), self.slot(), damage_result.final_damage, false,
            );
        }

        ctx.trigger_on_physical_damage(actor, target, &damage_result);

        ctx.apply_buff(target, Box::new(HaltingSlashDebuff));
        ctx.apply_buff(actor, Box::new(HeroicGaitBuff));
    }

    fn clone_box(&self) -> Box<dyn crate::ability::Ability> {
        Box::new(StridebreakerActive)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::StatBlock;
    use crate::types::{ResourceType, DamageType};
    use crate::champion::ChampionState;

    struct DummyChampionInstance {
        state: ChampionState,
    }
    impl crate::champion::ChampionInstance for DummyChampionInstance {
        fn state(&self) -> &ChampionState { &self.state }
        fn state_mut(&mut self) -> &mut ChampionState { &mut self.state }
        fn update_stats(&mut self) {}
        fn get_ability(&self, _slot: crate::types::AbilitySlot) -> Option<&dyn crate::ability::Ability> { None }
        fn take_damage(&mut self, amount: f64) -> bool { self.state.health.reduce(amount) }
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
        let mut target_state = ChampionState::new(1, target_stats, StatBlock::new(), ResourceType::None, StatBlock::new(), StatBlock::new(), vec![]);
        target_state.stats.recalculate_current(&StatBlock::new());
        
        sim.champions.insert(target_id.clone(), std::rc::Rc::new(std::cell::RefCell::new(
            Box::new(DummyChampionInstance { state: target_state }) as Box<dyn crate::champion::ChampionInstance>
        )));
        
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
        
        let armor_reduction = sim.champions.get(&target_id).unwrap().borrow().state().buffs.aggregate_stats().armor_reduction_percent;
        assert_eq!(armor_reduction, 0.04); // 1 stack = 4%
        
        // 6th hit
        for _ in 0..5 {
            bc_effect.on_physical_damage(&mut sim, &actor_id, &target_id, &damage_result);
        }
        
        let armor_reduction_max = sim.champions.get(&target_id).unwrap().borrow().state().buffs.aggregate_stats().armor_reduction_percent;
        assert_eq!(armor_reduction_max, 0.24); // 6 stacks = 24%
        
        // 7th hit (should not exceed 6 stacks)
        bc_effect.on_physical_damage(&mut sim, &actor_id, &target_id, &damage_result);
        let armor_reduction_cap = sim.champions.get(&target_id).unwrap().borrow().state().buffs.aggregate_stats().armor_reduction_percent;
        assert_eq!(armor_reduction_cap, 0.24);
    }
}
}
