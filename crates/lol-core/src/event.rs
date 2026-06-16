use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::types::SimTime;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub trait EventRecorder {
    fn record_damage(&mut self, time: SimTime, source: crate::types::ChampionId, target: crate::types::ChampionId, ability: crate::types::AbilitySlot, amount: f64, is_crit: bool);
    fn record_cast(&mut self, time: SimTime, source: crate::types::ChampionId, ability: crate::types::AbilitySlot);
    fn record_death(&mut self, time: SimTime, champion: crate::types::ChampionId);
    fn record_buff_apply(&mut self, time: SimTime, target: crate::types::ChampionId, buff_name: String);
    fn record_buff_expire(&mut self, time: SimTime, target: crate::types::ChampionId, buff_name: String);
    fn record_resource_update(&mut self, time: SimTime, target: crate::types::ChampionId, resource_type: String, amount: f64, max: f64);
}

/// Context provided to events when they are executed.
/// This acts as the state view during event execution.
pub struct SimContext {
    /// The current time of the simulation.
    pub current_time: SimTime,
    /// The event recorder to log combat metrics.
    pub recorder: Option<Rc<RefCell<dyn EventRecorder>>>,
    /// Queue for events that abilities or other events wish to schedule relative to current time.
    /// The tuple contains (delay_in_seconds, event).
    pub new_events: Vec<(f64, Box<dyn SimEvent>)>,
    /// Access to the champions in the simulation.
    pub champions: HashMap<crate::types::ChampionId, Rc<RefCell<Box<dyn crate::champion::ChampionInstance>>>>,
}

/// Trait representing an event that occurs at a specific point in simulation time.
pub trait SimEvent {
    /// Execute the event logic.
    fn execute(&self, ctx: &mut SimContext, event_manager: &mut EventManager);
    
    /// Provide a human-readable name for the event, useful for debugging.
    fn name(&self) -> &str;
}

/// Internal wrapper for queued events to enforce ordering in the priority queue.
struct QueuedEvent {
    time: SimTime,
    event: Box<dyn SimEvent>,
    event_id: u64,
}

impl PartialEq for QueuedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.event_id == other.event_id
    }
}

impl Eq for QueuedEvent {}

impl PartialOrd for QueuedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap behavior based on time.
        // If time is equal, use event_id to preserve insertion order (stable sort).
        other.time.cmp(&self.time)
            .then_with(|| other.event_id.cmp(&self.event_id))
    }
}

/// The EventManager acts as the core Timing Wheel / priority queue for the simulation engine.
/// It holds scheduled events and executes them in chronological order.
pub struct EventManager {
    queue: BinaryHeap<QueuedEvent>,
    current_time: SimTime,
    next_event_id: u64,
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EventManager {
    /// Create a new EventManager initialized at time 0.
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            current_time: SimTime::new(0.0),
            next_event_id: 0,
        }
    }

    /// Schedule an event to execute at a specific absolute `SimTime`.
    pub fn schedule(&mut self, time: SimTime, event: Box<dyn SimEvent>) {
        self.queue.push(QueuedEvent {
            time,
            event,
            event_id: self.next_event_id,
        });
        self.next_event_id += 1;
    }

    /// Schedule an event to execute at a delay relative to the current simulation time.
    pub fn schedule_in(&mut self, delay: f64, event: Box<dyn SimEvent>) {
        let time = self.current_time + delay;
        self.schedule(time, event);
    }

    /// Run the simulation event loop up to `max_time`.
    pub fn run(&mut self, ctx: &mut SimContext, max_time: SimTime) {
        while let Some(peeked) = self.queue.peek() {
            if peeked.time > max_time {
                break;
            }

            // Extract the next event
            let queued_event = self.queue.pop().expect("Queue was peeked to have elements");
            
            // Advance simulation time
            self.current_time = queued_event.time;
            ctx.current_time = self.current_time;
            
            // Execute the event
            queued_event.event.execute(ctx, self);
            
            // Drain and schedule any newly requested events
            for (delay, new_event) in ctx.new_events.drain(..) {
                self.schedule_in(delay, new_event);
            }
        }
        
        // After finishing the loop, advance time to max_time
        ctx.current_time = max_time;
        self.current_time = max_time;
    }

    /// Returns the current time of the simulation.
    pub fn current_time(&self) -> SimTime {
        self.current_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestEvent {
        name: String,
        output_log: Arc<Mutex<Vec<String>>>,
    }

    impl SimEvent for TestEvent {
        fn execute(&self, _ctx: &mut SimContext, _event_manager: &mut EventManager) {
            self.output_log.lock().unwrap().push(self.name.clone());
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_event_manager_ordering() {
        let mut manager = EventManager::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        manager.schedule(
            SimTime::new(2.0),
            Box::new(TestEvent {
                name: "Second".to_string(),
                output_log: Arc::clone(&log),
            }),
        );
        manager.schedule(
            SimTime::new(1.0),
            Box::new(TestEvent {
                name: "First".to_string(),
                output_log: Arc::clone(&log),
            }),
        );
        manager.schedule(
            SimTime::new(3.0),
            Box::new(TestEvent {
                name: "Third".to_string(),
                output_log: Arc::clone(&log),
            }),
        );

        let mut ctx = SimContext { current_time: SimTime::new(0.0), recorder: None, new_events: Vec::new(), champions: HashMap::new() };
        manager.run(&mut ctx, SimTime::new(10.0));

        let executed = log.lock().unwrap();
        assert_eq!(executed.len(), 3);
        assert_eq!(executed[0], "First");
        assert_eq!(executed[1], "Second");
        assert_eq!(executed[2], "Third");
    }

    #[test]
    fn test_stable_event_ordering() {
        let mut manager = EventManager::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        manager.schedule(
            SimTime::new(1.0),
            Box::new(TestEvent {
                name: "A".to_string(),
                output_log: Arc::clone(&log),
            }),
        );
        manager.schedule(
            SimTime::new(1.0),
            Box::new(TestEvent {
                name: "B".to_string(),
                output_log: Arc::clone(&log),
            }),
        );

        let mut ctx = SimContext { current_time: SimTime::new(0.0), recorder: None, new_events: Vec::new() };
        manager.run(&mut ctx, SimTime::new(2.0));

        let executed = log.lock().unwrap();
        assert_eq!(executed.len(), 2);
        assert_eq!(executed[0], "A");
        assert_eq!(executed[1], "B");
    }
}
