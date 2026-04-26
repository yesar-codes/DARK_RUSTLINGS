use std::fs;

use super::data::LevelData;

pub(super) fn load_level(path: &str) -> Result<LevelData, String> {
    let contents = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    ron::de::from_str(&contents).map_err(|error| format!("{path}: {error}"))
}

