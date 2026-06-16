use crate::parser::ActionPriorityList;
use lol_core::champion::ChampionInstance;
use lol_core::event::SimContext;
use lol_core::types::AbilitySlot;

pub struct AplExecutor;

impl AplExecutor {
    /// Evaluates the APL and returns the first AbilitySlot that is ready to cast
    /// and meets all defined conditions.
    pub fn get_next_action(
        apl: &ActionPriorityList,
        ctx: &SimContext,
        champion: &dyn ChampionInstance,
        target: &dyn ChampionInstance,
    ) -> Option<AbilitySlot> {
        for action in &apl.actions {
            // Check if ability is on cooldown / unlearned
            if !champion.state().abilities.is_ready(action.slot, ctx.current_time) {
                continue;
            }

            // Check specific conditional expressions if they exist
            if let Some(cond) = &action.condition {
                if !cond.evaluate(ctx, champion, target) {
                    continue;
                }
            }

            // Valid action found
            return Some(action.slot);
        }

        None
    }
}

pub struct ActorTickEvent {
    pub actor: lol_core::types::ChampionId,
    pub target: lol_core::types::ChampionId,
    pub apl: ActionPriorityList,
}

impl lol_core::event::SimEvent for ActorTickEvent {
    fn execute(&self, ctx: &mut SimContext, _event_manager: &mut lol_core::event::EventManager) {
        let mut cast_slot = None;
        let mut cast_time = 0.0;
        let mut base_cooldown = 0.0;
        let mut level = 0;

        {
            if let Some(champ_ref) = ctx.champions.get(&self.actor) {
                if let Some(target_ref) = ctx.champions.get(&self.target) {
                    let champ = champ_ref.borrow();
                    let target_champ = target_ref.borrow();
                    let champ_inst = champ.as_ref();
                    let target_inst = target_champ.as_ref();
                    
                    if let Some(slot) = AplExecutor::get_next_action(&self.apl, ctx, champ_inst, target_inst) {
                        cast_slot = Some(slot);
                        if let Some(ability) = champ_inst.get_ability(slot) {
                            cast_time = ability.cast_time();
                            if let Some(state) = champ_inst.state().abilities.get_state(slot) {
                                level = state.level;
                            }
                            
                            if slot == lol_core::types::AbilitySlot::AutoAttack {
                                let as_stat = champ_inst.state().stats.current.attack_speed;
                                base_cooldown = 1.0 / as_stat.max(0.1);
                            } else {
                                let ah = champ_inst.state().stats.current.ability_haste;
                                let cdr = ah / (100.0 + ah);
                                base_cooldown = ability.base_cooldown(level) * (1.0 - cdr);
                            }
                        }
                    }
                }
            } else {
                return;
            }
        }
        
        if let Some(slot) = cast_slot {
            // Execute the ability by temporarily grabbing the instance again
            // We cannot hold `champ` borrow during `ability.execute(ctx)` because it might borrow `champions` internally.
            // Wait, to call `ability.execute(ctx)` we need to get `&dyn Ability` without holding `champ_ref.borrow()` if possible,
            // or `ability.execute` shouldn't touch `ctx.champions` mutably for the same actor.
            // Actually, we can just push an AbilityCastEvent!

            // Record cast (instant)
            if let Some(recorder) = &ctx.recorder {
                recorder.borrow_mut().record_cast(ctx.current_time, self.actor.clone(), slot);
            }

            // Let's invoke execute by grabbing the ability again. 
            // We assume ability.execute() doesn't panic if it borrows other champions.
            // It MUST NOT borrow `self.actor` mutably from `ctx.champions` if we hold it.
            // But we don't hold it anymore! We dropped `champ`.
            // Let's grab it, but wait: `get_ability` requires `&ChampionInstance`.
            // So we do:
            let ability_box: Option<Box<dyn lol_core::ability::Ability>> = {
                let champ_ref = ctx.champions.get(&self.actor).unwrap();
                let champ = champ_ref.borrow();
                let champ_inst = champ.as_ref();
                champ_inst.get_ability(slot).map(|a| a.clone_box())
            };

            if let Some(ability) = ability_box {
                ability.execute(ctx, &self.actor, &self.target);
            }

            // Put it on cooldown
            if let Some(champ_ref) = ctx.champions.get(&self.actor) {
                let mut champ = champ_ref.borrow_mut();
                if let Some(state) = champ.as_mut().state_mut().abilities.get_state_mut(slot) {
                    state.cooldown.start_cooldown(ctx.current_time, base_cooldown);
                }
            }

            // Schedule the next ActorTickEvent after max(cast_time, GCD)
            let gcd = 0.25;
            let delay = cast_time.max(gcd);
            
            ctx.new_events.push((
                delay,
                Box::new(ActorTickEvent {
                    actor: self.actor.clone(),
                    target: self.target.clone(),
                    apl: self.apl.clone(),
                }),
            ));
        } else {
            // No action available, wait a bit and poll again
            ctx.new_events.push((
                0.1, // 100ms polling
                Box::new(ActorTickEvent {
                    actor: self.actor.clone(),
                    target: self.target.clone(),
                    apl: self.apl.clone(),
                }),
            ));
        }
    }
    
    fn name(&self) -> &str { "ActorTick" }
}
