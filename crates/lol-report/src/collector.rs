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
    BuffApply {
        time: SimTime,
        target: ChampionId,
        buff_name: String,
    },
    BuffExpire {
        time: SimTime,
        target: ChampionId,
        buff_name: String,
    },
    ResourceUpdate {
        time: SimTime,
        target: ChampionId,
        resource_type: String,
        amount: f64,
        max: f64,
    },
    LevelUp {
        time: SimTime,
        target: ChampionId,
        level: u32,
    },
    ItemAcquisition {
        time: SimTime,
        target: ChampionId,
        item_id: String,
        item_name: String,
    },
}

#[derive(Debug, Default)]
pub struct DataCollector {
    pub events: Vec<CombatEvent>,
    pub champion_items: std::collections::HashMap<ChampionId, Vec<(String, String)>>,
}

impl DataCollector {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            champion_items: std::collections::HashMap::new(),
        }
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

    pub fn record_buff_apply(&mut self, time: SimTime, target: ChampionId, buff_name: String) {
        self.events.push(CombatEvent::BuffApply { time, target, buff_name });
    }

    pub fn record_buff_expire(&mut self, time: SimTime, target: ChampionId, buff_name: String) {
        self.events.push(CombatEvent::BuffExpire { time, target, buff_name });
    }

    pub fn record_resource_update(&mut self, time: SimTime, target: ChampionId, resource_type: String, amount: f64, max: f64) {
        self.events.push(CombatEvent::ResourceUpdate { time, target, resource_type, amount, max });
    }

    pub fn record_item_acquisition(&mut self, time: SimTime, target: ChampionId, item_id: String, item_name: String) {
        self.events.push(CombatEvent::ItemAcquisition { time, target, item_id, item_name });
    }

    pub fn record_level_up(&mut self, time: SimTime, target: ChampionId, level: u32) {
        self.events.push(CombatEvent::LevelUp { time, target, level });
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
    
    fn record_buff_apply(&mut self, time: SimTime, target: ChampionId, buff_name: String) {
        self.record_buff_apply(time, target, buff_name);
    }
    
    fn record_buff_expire(&mut self, time: SimTime, target: ChampionId, buff_name: String) {
        self.record_buff_expire(time, target, buff_name);
    }
    
    fn record_resource_update(&mut self, time: SimTime, target: ChampionId, resource_type: String, amount: f64, max: f64) {
        self.record_resource_update(time, target, resource_type, amount, max);
    }
    
    fn record_item_acquisition(&mut self, time: SimTime, target: ChampionId, item_id: String, item_name: String) {
        self.record_item_acquisition(time, target, item_id, item_name);
    }
    
    fn record_level_up(&mut self, time: SimTime, target: ChampionId, level: u32) {
        self.record_level_up(time, target, level);
    }
}
