use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct LevelData {
    pub(super) name: String,
    pub(super) tile_width: f32,
    pub(super) tile_height: f32,
    pub(super) rows: Vec<String>,
}

impl LevelData {
    pub(super) fn width(&self) -> usize {
        self.rows.first().map_or(0, |row| row.chars().count())
    }

    pub(super) fn height(&self) -> usize {
        self.rows.len()
    }
}

