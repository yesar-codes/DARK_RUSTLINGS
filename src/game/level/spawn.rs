use bevy::prelude::*;
use bevy::math::primitives::Cuboid;

use super::data::LevelData;

#[derive(Component)]
pub struct WallBlock;

#[derive(Resource, Debug, Clone)]
pub struct PlayerSpawnPoint(pub Vec3);

#[derive(Resource, Debug, Clone)]
pub struct LevelCollision {
    pub wall_centers: Vec<Vec2>,
    pub wall_half_extents: Vec2,
}

pub fn spawn_level(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level: &LevelData,
) {
    if level.rows.is_empty() {
        warn!("Level '{}' has no rows to spawn", level.name);
        return;
    }

    let row_width = level.width();
    if row_width == 0 {
        warn!("Level '{}' rows are empty", level.name);
        return;
    }

    if level.rows.iter().any(|row| row.chars().count() != row_width) {
        warn!("Level '{}' has inconsistent row widths", level.name);
        return;
    }

    let tile_size_x = (level.tile_width / 32.0).max(0.5);
    let tile_size_z = (level.tile_height / 16.0).max(0.5);
    let floor_height = 0.1;
    let wall_height = 2.1;
    let wall_scale = 0.95;

    let floor_mesh = meshes.add(Mesh::from(Cuboid::new(tile_size_x, floor_height, tile_size_z)));
    let wall_mesh = meshes.add(Mesh::from(Cuboid::new(
        tile_size_x * wall_scale,
        wall_height,
        tile_size_z * wall_scale,
    )));

    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(54, 61, 74),
        perceptual_roughness: 0.95,
        ..default()
    });

    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(77, 77, 77),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });

    let center_x = (row_width as f32 - 1.0) * tile_size_x * 0.5;
    let center_z = (level.height() as f32 - 1.0) * tile_size_z * 0.5;
    let mut wall_centers = Vec::new();
    let mut player_spawn = None;

    for (row_index, row) in level.rows.iter().enumerate() {
        for (col_index, tile) in row.chars().enumerate() {
            let x = col_index as f32 * tile_size_x - center_x;
            let z = row_index as f32 * tile_size_z - center_z;

            commands.spawn((
                Mesh3d(floor_mesh.clone()),
                MeshMaterial3d(floor_material.clone()),
                Transform::from_xyz(x, floor_height * 0.5, z),
            ));

            if tile == '#' {
                wall_centers.push(Vec2::new(x, z));
                commands.spawn((
                    WallBlock,
                    Mesh3d(wall_mesh.clone()),
                    MeshMaterial3d(wall_material.clone()),
                    Transform::from_xyz(x, floor_height + wall_height * 0.5, z),
                ));
            } else if player_spawn.is_none() {
                player_spawn = Some(Vec3::new(x, floor_height, z));
            }
        }
    }

    commands.insert_resource(LevelCollision {
        wall_centers,
        wall_half_extents: Vec2::new(tile_size_x * wall_scale * 0.5, tile_size_z * wall_scale * 0.5),
    });

    commands.insert_resource(PlayerSpawnPoint(player_spawn.unwrap_or(Vec3::ZERO)));

    info!(
        "Spawned level '{}' ({}x{})",
        level.name,
        level.width(),
        level.height()
    );
}


