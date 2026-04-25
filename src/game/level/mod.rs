mod data;
mod load;
mod spawn;

pub use spawn::{LevelCollision, PlayerSpawnPoint};

use bevy::prelude::*;

pub fn spawn_initial_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    match load::load_level("resources/levels/level_01.ron") {
        Ok(level) => spawn::spawn_level(&mut commands, &mut meshes, &mut materials, &level),
        Err(error) => {
            error!("Failed to load level: {error}");
        }
    }
}

