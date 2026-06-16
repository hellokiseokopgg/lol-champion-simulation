use crate::ability::AbilitySlotManager;
use crate::buff::BuffManager;
use crate::item::ItemBuild;
use crate::resource::Resource;
use crate::rune::RunePage;
use crate::stats::{StatBlock, ThreeLayerStats};
use crate::types::ResourceType;

/// Configuration for creating a champion instance.
pub struct ChampionConfig {
    /// The starting level of the champion.
    pub level: u32,
    /// The full item build equipped by the champion.
    pub item_build: ItemBuild,
    /// The rune page equipped by the champion.
    pub rune_page: RunePage,
    /// Base stats at the starting level.
    pub base_stats: StatBlock,
}

/// The core mutable state of a champion during a simulation.
pub struct ChampionState {
    /// The three-layer stats architecture for the champion.
    pub stats: ThreeLayerStats,
    /// The primary resource (e.g., HP/Mana) tracking.
    pub resource: Resource,
    /// The health resource tracking.
    pub health: Resource,
    /// Active buffs and debuffs.
    pub buffs: BuffManager,
    /// Ability state tracking (cooldowns, levels).
    pub abilities: AbilitySlotManager,
}

impl ChampionState {
    pub fn new(base_stats: StatBlock, resource_type: ResourceType) -> Self {
        Self {
            stats: ThreeLayerStats::new(base_stats.clone()),
            resource: Resource::new(base_stats.mana, resource_type),
            health: Resource::new(base_stats.health, ResourceType::None),
            buffs: BuffManager::new(),
            abilities: AbilitySlotManager::new(),
        }
    }
}

/// Trait representing an actively simulating champion.
pub trait ChampionInstance {
    /// Returns an immutable reference to the champion's state.
    fn state(&self) -> &ChampionState;
    
    /// Returns a mutable reference to the champion's state.
    fn state_mut(&mut self) -> &mut ChampionState;
    
    /// Triggers a full recalculation of stats based on base, items, runes, and buffs.
    fn update_stats(&mut self);
    
    /// Returns a reference to the ability in the given slot, if it exists.
    fn get_ability(&self, slot: crate::types::AbilitySlot) -> Option<&dyn crate::ability::Ability>;
}

/// A factory trait for generating instances of a specific champion.
pub trait ChampionModule {
    /// Returns the internal identifier of the champion (e.g., "Garen").
    fn id(&self) -> &str;
    
    /// Instantiates a new champion simulation instance given a configuration.
    fn create_instance(&self, config: ChampionConfig) -> Box<dyn ChampionInstance>;
}
