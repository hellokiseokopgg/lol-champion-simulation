use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::info;

use crate::champion_data::ChampionData;
use crate::item_data::ItemData;
use crate::rune_data::RuneData;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Champion not found: {0}")]
    ChampionNotFound(String),
}

/// Utility to load JSON data files for the simulation.
pub struct DataLoader {
    data_dir: PathBuf,
}

impl DataLoader {
    /// Creates a new `DataLoader` configured to read from `data_dir`.
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }

    /// Loads a single champion's data by ID.
    /// Expects the file to be at `data_dir/champions/{id}.json`.
    pub fn load_champion(&self, id: &str) -> Result<ChampionData, DataError> {
        let file_path = self.data_dir.join("champions").join(format!("{}.json", id));
        if !file_path.exists() {
            return Err(DataError::ChampionNotFound(id.to_string()));
        }

        let content = fs::read_to_string(&file_path)?;
        let champion: ChampionData = serde_json::from_str(&content)?;
        info!("Loaded champion data for {}", champion.name);
        Ok(champion)
    }

    /// Loads all items from `data_dir/items.json`.
    /// Expects a JSON array of item objects.
    pub fn load_all_items(&self) -> Result<Vec<ItemData>, DataError> {
        let file_path = self.data_dir.join("items.json");
        let content = fs::read_to_string(&file_path)?;
        let items: Vec<ItemData> = serde_json::from_str(&content)?;
        info!("Loaded {} items", items.len());
        Ok(items)
    }

    /// Loads all runes from `data_dir/runes.json`.
    /// Expects a JSON array of rune objects.
    pub fn load_all_runes(&self) -> Result<Vec<RuneData>, DataError> {
        let file_path = self.data_dir.join("runes.json");
        let content = fs::read_to_string(&file_path)?;
        let runes: Vec<RuneData> = serde_json::from_str(&content)?;
        info!("Loaded {} runes", runes.len());
        Ok(runes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_champion() {
        let dir = tempdir().unwrap();
        let champs_dir = dir.path().join("champions");
        fs::create_dir(&champs_dir).unwrap();

        let garen_json = r#"{
            "id": "garen",
            "name": "Garen",
            "base_stats": {
                "hp": 690, "mp": 0, "hp_regen": 8, "mp_regen": 0,
                "armor": 38, "magic_resist": 32, "attack_damage": 69,
                "attack_speed": 0.625, "attack_range": 175, "move_speed": 340
            },
            "growth_stats": {
                "hp": 98, "mp": 0, "hp_regen": 0.5, "mp_regen": 0,
                "armor": 4.2, "magic_resist": 2.05, "attack_damage": 4.5,
                "attack_speed": 0.0362
            }
        }"#;
        
        let file_path = champs_dir.join("garen.json");
        let mut file = fs::File::create(file_path).unwrap();
        file.write_all(garen_json.as_bytes()).unwrap();

        let loader = DataLoader::new(dir.path());
        let champ = loader.load_champion("garen").unwrap();
        assert_eq!(champ.id, "garen");
    }

    #[test]
    fn test_load_items() {
        let dir = tempdir().unwrap();
        let items_json = r#"[
            {
                "id": "long_sword",
                "name": "Long Sword",
                "cost": 350,
                "stats": { "attack_damage": 10.0 }
            }
        ]"#;

        let file_path = dir.path().join("items.json");
        let mut file = fs::File::create(file_path).unwrap();
        file.write_all(items_json.as_bytes()).unwrap();

        let loader = DataLoader::new(dir.path());
        let items = loader.load_all_items().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "long_sword");
    }
}
