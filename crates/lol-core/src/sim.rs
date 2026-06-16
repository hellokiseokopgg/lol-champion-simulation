use crate::champion::ChampionInstance;
use crate::event::{EventManager, SimContext};
use crate::types::SimTime;

/// Configuration parameters for a simulation run.
pub struct SimConfig {
    /// The maximum duration of the simulation in seconds.
    pub max_duration: f64,
}

/// The core game simulation engine.
pub struct GameSimulation {
    event_manager: EventManager,
    config: SimConfig,
    
    // In a full implementation, this would hold multiple actors.
    // For a 1v1 sim, we would have attacker and defender.
    // Keeping it generic for now.
    // Keeping it generic for now.
    pub actors: std::collections::HashMap<crate::types::ChampionId, std::rc::Rc<std::cell::RefCell<Box<dyn ChampionInstance>>>>,
}

impl GameSimulation {
    /// Creates a new simulation with the given configuration.
    pub fn new(config: SimConfig) -> Self {
        Self {
            event_manager: EventManager::new(),
            config,
            actors: std::collections::HashMap::new(),
        }
    }

    /// Adds a champion instance to the simulation.
    pub fn add_actor(&mut self, id: crate::types::ChampionId, actor: std::rc::Rc<std::cell::RefCell<Box<dyn ChampionInstance>>>) {
        self.actors.insert(id, actor);
    }

    /// Access the event manager to queue initial events (e.g. APL execution).
    pub fn event_manager_mut(&mut self) -> &mut EventManager {
        &mut self.event_manager
    }

    /// Runs the simulation until the max duration is reached or no events remain.
    pub fn run(&mut self, recorder: Option<std::rc::Rc<std::cell::RefCell<dyn crate::event::EventRecorder>>>) {
        let mut ctx = SimContext {
            current_time: SimTime::new(0.0),
            recorder,
            new_events: Vec::new(),
            champions: self.actors.clone(),
            is_simulation_over: false,
        };
        
        let max_time = SimTime::new(self.config.max_duration);
        
        // Delegate to the EventManager's run loop
        self.event_manager.run(&mut ctx, max_time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_run() {
        let config = SimConfig {
            max_duration: 10.0,
        };
        let mut sim = GameSimulation::new(config);
        
        // Sim starts empty, should just advance time directly to max
        sim.run(None);
        
        assert_eq!(sim.event_manager_mut().current_time().as_f64(), 10.0);
    }
}
