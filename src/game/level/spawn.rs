use bevy::prelude::*;
use bevy::math::primitives::Cuboid;

use super::data::LevelData;

// dead code — spawned for future queries (e.g. raycasting, destruction)
#[derive(Component)]
pub struct WallBlock;

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component)]
pub struct SwitchLight;

#[derive(Component)]
pub struct LevelMusic;

#[derive(Component)]
pub struct SpeedPowerupTile;

#[derive(Component)]
pub struct LightRangePowerupTile;

#[derive(Resource, Debug, Clone)]
pub struct PlayerSpawnPoint(pub Vec3);

#[derive(Resource, Debug, Clone)]
pub struct LevelCollision {
    pub wall_centers: Vec<Vec2>,
    pub wall_half_extents: Vec2,
    pub tile_size: Vec2,
    pub switch_center: Option<Vec2>,
    pub speed_powerup_center: Option<Vec2>,
    pub light_powerup_center: Option<Vec2>,
    pub exit_center: Option<Vec2>,
    pub exit_direction: Option<Vec2>,
}

pub(super) fn spawn_level(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level: &LevelData,
) -> Option<Vec3> {
    if level.rows.is_empty() {
        warn!("Level '{}' has no rows to spawn", level.name);
        return None;
    }

    let row_width = level.width();
    if row_width == 0 {
        warn!("Level '{}' rows are empty", level.name);
        return None;
    }

    if level.rows.iter().any(|row| row.chars().count() != row_width) {
        warn!("Level '{}' has inconsistent row widths", level.name);
        return None;
    }

    let tile_size_x = (level.tile_width / 32.0).max(0.5);
    let tile_size_z = (level.tile_height / 16.0).max(0.5);
    let floor_height = 0.1;
    let wall_height = 1.55;
    let wall_scale = 0.95;

    let floor_mesh = meshes.add(Mesh::from(Cuboid::new(tile_size_x, floor_height, tile_size_z)));
    let wall_mesh = meshes.add(Mesh::from(Cuboid::new(
        tile_size_x * wall_scale,
        wall_height,
        tile_size_z * wall_scale,
    )));

    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(168, 172, 178),
        perceptual_roughness: 0.98,
        metallic: 0.0,
        ..default()
    });

    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(34, 44, 58),
        perceptual_roughness: 0.72,
        metallic: 0.03,
        ..default()
    });

    let switch_mesh = meshes.add(Mesh::from(Cuboid::new(
        tile_size_x * 0.7,
        0.25,
        tile_size_z * 0.7,
    )));
    let switch_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(255, 214, 102),
        emissive: Color::srgb_u8(255, 214, 102).into(),
        ..default()
    });

    let speed_powerup_mesh = meshes.add(Mesh::from(Cuboid::new(
        tile_size_x * 0.7,
        0.25,
        tile_size_z * 0.7,
    )));
    let speed_powerup_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(120, 255, 112),
        emissive: Color::srgb_u8(80, 255, 90).into(),
        ..default()
    });

    let light_powerup_mesh = meshes.add(Mesh::from(Cuboid::new(
        tile_size_x * 0.7,
        0.25,
        tile_size_z * 0.7,
    )));
    let light_powerup_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(255, 102, 102),
        emissive: Color::srgb_u8(255, 72, 72).into(),
        ..default()
    });

    let exit_mesh = meshes.add(Mesh::from(Cuboid::new(
        tile_size_x * 0.7,
        0.25,
        tile_size_z * 0.7,
    )));
    let exit_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(84, 160, 255),
        emissive: Color::srgb_u8(84, 160, 255).into(),
        ..default()
    });

    let center_x = (row_width as f32 - 1.0) * tile_size_x * 0.5;
    let center_z = (level.height() as f32 - 1.0) * tile_size_z * 0.5;
    let level_width_world = row_width as f32 * tile_size_x;
    let level_depth_world = level.height() as f32 * tile_size_z;
    let level_radius = 0.5 * (level_width_world * level_width_world + level_depth_world * level_depth_world).sqrt();
    let central_light_range = (level_radius * 1.35).max(18.0);
    let central_light_height = floor_height + wall_height + (level_radius * 0.45).max(3.0);
    let mut wall_centers = Vec::new();
    let mut player_spawn = None;
    let mut fallback_spawn = None;
    let mut switch_center = None;
    let mut speed_powerup_center = None;
    let mut light_powerup_center = None;
    let mut exit_center = None;
    let mut exit_direction = None;

    for (row_index, row) in level.rows.iter().enumerate() {
        for (col_index, tile) in row.chars().enumerate() {
            let x = col_index as f32 * tile_size_x - center_x;
            let z = row_index as f32 * tile_size_z - center_z;

            commands.spawn((
                LevelEntity,
                Mesh3d(floor_mesh.clone()),
                MeshMaterial3d(floor_material.clone()),
                Transform::from_xyz(x, floor_height * 0.5, z),
            ));

            if tile != '#' && fallback_spawn.is_none() {
                fallback_spawn = Some(Vec3::new(x, floor_height, z));
            }

            if tile == '#' {
                wall_centers.push(Vec2::new(x, z));
                commands.spawn((
                    LevelEntity,
                    WallBlock,
                    Mesh3d(wall_mesh.clone()),
                    MeshMaterial3d(wall_material.clone()),
                    Transform::from_xyz(x, floor_height + wall_height * 0.5, z),
                ));
            } else if tile == 'S' {
                switch_center = Some(Vec2::new(x, z));
                commands.spawn((
                    LevelEntity,
                    Mesh3d(switch_mesh.clone()),
                    MeshMaterial3d(switch_material.clone()),
                    Transform::from_xyz(x, floor_height + 0.18, z),
                ));
            } else if tile == 'G' {
                speed_powerup_center = Some(Vec2::new(x, z));
                commands.spawn((
                    LevelEntity,
                    SpeedPowerupTile,
                    Mesh3d(speed_powerup_mesh.clone()),
                    MeshMaterial3d(speed_powerup_material.clone()),
                    Transform::from_xyz(x, floor_height + 0.18, z),
                ));
            } else if tile == 'R' {
                light_powerup_center = Some(Vec2::new(x, z));
                commands.spawn((
                    LevelEntity,
                    LightRangePowerupTile,
                    Mesh3d(light_powerup_mesh.clone()),
                    MeshMaterial3d(light_powerup_material.clone()),
                    Transform::from_xyz(x, floor_height + 0.18, z),
                ));
            } else if tile == 'E' {
                let border_direction = if col_index == 0 {
                    Some(Vec2::new(-1.0, 0.0))
                } else if col_index + 1 == row_width {
                    Some(Vec2::new(1.0, 0.0))
                } else if row_index == 0 {
                    Some(Vec2::new(0.0, -1.0))
                } else if row_index + 1 == level.height() {
                    Some(Vec2::new(0.0, 1.0))
                } else {
                    None
                };

                if border_direction.is_none() {
                    warn!("Level '{}' has an exit tile ('E') that is not on the border", level.name);
                }

                if exit_center.is_none() && border_direction.is_some() {
                    exit_center = Some(Vec2::new(x, z));
                    exit_direction = border_direction;
                }

                commands.spawn((
                    LevelEntity,
                    Mesh3d(exit_mesh.clone()),
                    MeshMaterial3d(exit_material.clone()),
                    Transform::from_xyz(x, floor_height + 0.12, z),
                ));
            } else if tile == 'P' && player_spawn.is_none() {
                player_spawn = Some(Vec3::new(x, floor_height, z));
            }
        }
    }

    if switch_center.is_none() {
        warn!("Level '{}' has no light switch tile ('S')", level.name);
    }
    if exit_center.is_none() {
        warn!("Level '{}' has no border exit tile ('E')", level.name);
    }
    if player_spawn.is_none() {
        warn!("Level '{}' has no player start tile ('P')", level.name);
    }

    if switch_center.is_some() {
        commands.spawn((
            LevelEntity,
            SwitchLight,
            PointLight {
                intensity: 0.0,
                range: central_light_range,
                color: Color::srgb_u8(255, 236, 196),
                radius: 0.25,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(0.0, central_light_height, 0.0),
        ));
    }

    commands.insert_resource(LevelCollision {
        wall_centers,
        wall_half_extents: Vec2::new(tile_size_x * wall_scale * 0.5, tile_size_z * wall_scale * 0.5),
        tile_size: Vec2::new(tile_size_x, tile_size_z),
        switch_center,
        speed_powerup_center,
        light_powerup_center,
        exit_center,
        exit_direction,
    });

    let spawn_position = player_spawn.or(fallback_spawn).unwrap_or(Vec3::ZERO);
    commands.insert_resource(PlayerSpawnPoint(spawn_position));

    info!(
        "Spawned level '{}' ({}x{})",
        level.name,
        level.width(),
        level.height()
    );

    Some(spawn_position)
}


