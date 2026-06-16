use lol_core::types::{AbilitySlot, ChampionId, SimTime};

#[derive(Debug, Clone)]
pub enum CombatEvent {
    Damage {
        time: SimTime,
        source: ChampionId,
        target: ChampionId,
        ability: AbilitySlot,
        amount: f64,
        is_crit: bool,
    },
    Cast {
        time: SimTime,
        source: ChampionId,
        ability: AbilitySlot,
    },
    Death {
        time: SimTime,
        champion: ChampionId,
    },
}

#[derive(Debug, Default)]
pub struct DataCollector {
    pub events: Vec<CombatEvent>,
}

impl DataCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_damage(
        &mut self,
        time: SimTime,
        source: ChampionId,
        target: ChampionId,
        ability: AbilitySlot,
        amount: f64,
        is_crit: bool,
    ) {
        self.events.push(CombatEvent::Damage {
            time,
            source,
            target,
            ability,
            amount,
            is_crit,
        });
    }

    pub fn record_cast(&mut self, time: SimTime, source: ChampionId, ability: AbilitySlot) {
        self.events.push(CombatEvent::Cast {
            time,
            source,
            ability,
        });
    }

    pub fn record_death(&mut self, time: SimTime, champion: ChampionId) {
        self.events.push(CombatEvent::Death { time, champion });
    }
}

impl lol_core::event::EventRecorder for DataCollector {
    fn record_damage(&mut self, time: SimTime, source: ChampionId, target: ChampionId, ability: AbilitySlot, amount: f64, is_crit: bool) {
        self.record_damage(time, source, target, ability, amount, is_crit);
    }
    
    fn record_cast(&mut self, time: SimTime, source: ChampionId, ability: AbilitySlot) {
        self.record_cast(time, source, ability);
    }
    
    fn record_death(&mut self, time: SimTime, champion: ChampionId) {
        self.record_death(time, champion);
    }
}
