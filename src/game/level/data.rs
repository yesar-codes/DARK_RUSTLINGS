use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LevelData {
    pub name: String,
    pub tile_width: f32,
    pub tile_height: f32,
    pub rows: Vec<String>,
}

impl LevelData {
    pub fn width(&self) -> usize {
        self.rows.first().map_or(0, |row| row.chars().count())
    }

    pub fn height(&self) -> usize {
        self.rows.len()
    }
}

