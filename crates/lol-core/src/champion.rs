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
    /// Stat growth per level.
    pub growth_stats: StatBlock,
}

impl ChampionConfig {
    pub fn aggregate_bonus_stats(&self) -> StatBlock {
        self.item_build.aggregate_stats() + self.rune_page.aggregate_stats()
    }
}

/// The core mutable state of a champion during a simulation.
pub struct ChampionState {
    /// The current level of the champion.
    pub level: u32,
    /// The base stats at level 1.
    pub base_stats: StatBlock,
    /// The stat growth per level.
    pub growth_stats: StatBlock,
    /// Stats provided by runes.
    pub rune_stats: StatBlock,
    /// Stats provided by items.
    pub item_stats: StatBlock,
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
    /// Item effect management.
    pub items: crate::item::ItemManager,
    /// Manages dynamic rune effects.
    pub rune_manager: crate::rune_manager::RuneManager,
}

impl ChampionState {
    pub fn new(level: u32, base_stats: StatBlock, growth_stats: StatBlock, resource_type: ResourceType, rune_stats: StatBlock, item_stats: StatBlock, item_effects: Vec<Box<dyn crate::item::ItemEffect>>) -> Self {
        let leveled_stats = base_stats.calculate_growth(&growth_stats, level);
        let mut stats = ThreeLayerStats::new(leveled_stats);
        stats.recalculate_initial(&(rune_stats.clone() + item_stats.clone()));
        stats.recalculate_current(&StatBlock::new()); // Make sure current reflects initial
        
        let mut item_manager = crate::item::ItemManager::new();
        for effect in item_effects {
            item_manager.add_effect(effect);
        }
        
        Self {
            level,
            base_stats: base_stats.clone(),
            growth_stats,
            rune_stats,
            item_stats,
            resource: Resource::new(stats.current.mana, resource_type),
            health: Resource::new(stats.current.health, ResourceType::None),
            stats,
            buffs: BuffManager::new(),
            abilities: AbilitySlotManager::new(),
            items: item_manager,
            rune_manager: crate::rune_manager::RuneManager::new(),
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
    fn update_stats(&mut self, time: crate::types::SimTime);
    
    /// Returns a reference to the ability in the given slot, if it exists.
    fn get_ability(&self, slot: crate::types::AbilitySlot) -> Option<&dyn crate::ability::Ability>;
    
    /// Applies damage to the champion's health pool. Returns the detailed damage result.
    fn take_damage(&mut self, amount: f64) -> crate::types::TakeDamageResult;

    /// Called when this champion deals damage. Routes to RuneManager and other passives.
    fn on_damage_dealt(&mut self, time: crate::types::SimTime, amount: f64, is_ability: bool) -> Vec<crate::rune_manager::RuneEvent> {
        let state = self.state_mut();
        state.rune_manager.on_damage_dealt(time, amount, is_ability)
    }

    /// Heals the champion.
    fn heal(&mut self, amount: f64) {
        let state = self.state_mut();
        state.health.add(amount);
    }

    /// Checks if the champion can currently cast the specified ability.
    /// Used for champion-specific restrictions (e.g. Garen cannot AutoAttack during E).
    fn can_cast(&self, _slot: crate::types::AbilitySlot, _time: crate::types::SimTime) -> bool {
        true
    }
}

/// A factory trait for generating instances of a specific champion.
pub trait ChampionModule {
    /// Returns the internal identifier of the champion (e.g., "Garen").
    fn id(&self) -> &str;
    
    /// Instantiates a new champion simulation instance given a configuration.
    fn create_instance(&self, config: ChampionConfig) -> Box<dyn ChampionInstance>;
}
