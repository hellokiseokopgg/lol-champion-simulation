pub mod ahri;
pub mod darius;
pub mod dummy;
pub mod garen;
pub mod jinx;
pub mod zed;

use lol_core::champion::ChampionModule;
use std::collections::HashMap;

pub struct ChampionRegistry {
    modules: HashMap<String, Box<dyn ChampionModule>>,
}

impl Default for ChampionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ChampionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            modules: HashMap::new(),
        };
        registry.register(Box::new(ahri::AhriModule));
        registry.register(Box::new(darius::DariusModule));
        registry.register(Box::new(dummy::DummyModule));
        registry.register(Box::new(garen::GarenModule));
        registry.register(Box::new(jinx::JinxModule));
        registry.register(Box::new(zed::ZedModule));
        registry
    }

    pub fn register(&mut self, module: Box<dyn ChampionModule>) {
        self.modules.insert(module.id().to_string(), module);
    }

    pub fn get(&self, id: &str) -> Option<&dyn ChampionModule> {
        self.modules.get(id).map(|m| m.as_ref())
    }
}
