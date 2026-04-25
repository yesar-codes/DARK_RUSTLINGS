mod data;
mod load;
mod spawn;

pub use spawn::{LevelCollision, LevelEntity, PlayerSpawnPoint};

use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct LevelList(pub Vec<&'static str>);

#[derive(Resource, Debug, Default)]
pub struct CurrentLevelIndex(pub usize);

pub fn spawn_initial_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(LevelList(vec![
        "resources/levels/level_01.ron",
        "resources/levels/level_02.ron",
    ]));
    commands.insert_resource(CurrentLevelIndex(0));

    let _ = spawn_level_at_index(&mut commands, &mut meshes, &mut materials, 0);
}

pub fn spawn_level_at_index(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level_index: usize,
) -> Option<Vec3> {
    let level_path = match level_index {
        0 => "resources/levels/level_01.ron",
        1 => "resources/levels/level_02.ron",
        _ => {
            warn!("Requested invalid level index {level_index}");
            return None;
        }
    };

    match load::load_level(level_path) {
        Ok(level) => spawn::spawn_level(commands, meshes, materials, &level),
        Err(error) => {
            error!("Failed to load level: {error}");
            None
        }
    }
}


pub fn despawn_level_entities(commands: &mut Commands, level_entities: &Query<Entity, With<LevelEntity>>) {
    for entity in level_entities {
        commands.entity(entity).despawn();
    }
}


