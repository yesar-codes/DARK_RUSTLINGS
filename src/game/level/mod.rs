mod data;
mod load;
mod spawn;
mod generate;

use bevy::audio::{AudioPlayer, PlaybackSettings};
pub use spawn::{
    LevelCollision, LevelEntity, LevelMusic, PlayerSpawnPoint, SwitchLight,
};

use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;

#[derive(Resource, Debug, Clone)]
pub struct LevelList(pub Vec<PathBuf>);

#[derive(Resource, Debug, Default)]
pub struct CurrentLevelIndex(pub usize);

pub fn spawn_initial_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let levels = discover_levels();
    if levels.is_empty() {
        error!("No level files found in resources/levels/");
        return;
    }

    commands.insert_resource(LevelList(levels.clone()));
    commands.insert_resource(CurrentLevelIndex(0));

    let _ = spawn_level_at_index(&mut commands, &asset_server, &mut meshes, &mut materials, 0, &levels);
}

pub fn spawn_level_at_index(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level_index: usize,
    level_list: &[PathBuf],
) -> Option<Vec3> {
    if level_index < level_list.len() {
        let level_path = &level_list[level_index];

        return match load::load_level(level_path.to_str().unwrap_or("")) {
            Ok(level) => spawn::spawn_level(commands, meshes, materials, &level).map(|spawn_position| {
                spawn_level_music(commands, asset_server);
                spawn_position
            }),
            Err(error) => {
                error!("Failed to load level: {error}");
                None
            }
        };
    }

    let level = generate::generate_level(
        "Generated Level",
        64.0,
        32.0,
        level_index,
        level_list.len(),
    );

    spawn::spawn_level(commands, meshes, materials, &level).map(|spawn_position| {
        spawn_level_music(commands, asset_server);
        spawn_position
    })
}

fn spawn_level_music(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        LevelEntity,
        LevelMusic,
        AudioPlayer::new(asset_server.load("audio/The Dark Amulet.mp3")),
        PlaybackSettings::LOOP,
    ));
}

fn discover_levels() -> Vec<PathBuf> {
    let levels_dir = "resources/levels";
    let mut levels = Vec::new();

    match fs::read_dir(levels_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                    levels.push(path);
                }
            }
        }
        Err(e) => {
            warn!("Failed to read levels directory: {}", e);
        }
    }

    levels.sort();
    levels
}

pub fn despawn_level_entities(commands: &mut Commands, level_entities: &Query<Entity, With<LevelEntity>>) {
    for entity in level_entities {
        commands.entity(entity).despawn();
    }
}


