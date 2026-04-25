pub mod animation;

use bevy::prelude::*;

use crate::game::camera::MainCamera;
use crate::game::gameplay::LevelFlow;
use crate::game::level::{LevelCollision, PlayerSpawnPoint};
use animation::load_soldier_model;

// Re-export animation functions for use in GamePlugin
pub use animation::{init_soldier_animations, setup_animations_on_load};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub(crate) struct MovementConfig {
    walk_speed: f32,
    run_speed: f32,
    acceleration: f32,
    deceleration: f32,
}

#[derive(Component, Default)]
pub(crate) struct Velocity(pub Vec2);

#[derive(Component)]
pub(crate) struct PlayerCollider {
    pub(crate) radius: f32,
}

pub(crate) fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    spawn_point: Option<Res<PlayerSpawnPoint>>,
) {
    let spawn = spawn_point
        .map(|point| point.0 + Vec3::Y * 0.8)
        .unwrap_or(Vec3::new(0.0, 0.8, 0.0));

    // Load the Soldier model
    let soldier_scene = load_soldier_model(&asset_server);

    commands.spawn((
        Player,
        MovementConfig {
            walk_speed: 4.5,
            run_speed: 7.5,
            acceleration: 18.0,
            deceleration: 24.0,
        },
        Velocity::default(),
        PlayerCollider { radius: 0.30 },
        SceneRoot(soldier_scene),
        Transform::from_translation(spawn),
    ))
    .with_children(|parent| {
        parent.spawn((
            PointLight {
                intensity: 20_400.0,
                range: 4.0,
                intensity: 2_800.0,
                range: 10.5,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(0.0, 1.6, 0.0),
        ));
    });
}

pub(crate) fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    flow: Option<Res<LevelFlow>>,
    pause_state: Res<PauseState>,
    collision: Option<Res<LevelCollision>>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut player_query: Query<
        (&mut Transform, &MovementConfig, &mut Velocity, &PlayerCollider),
        With<Player>,
    >,
) {
    if flow.as_deref().is_some_and(|flow| flow.game_over) || pause_state.paused {
        return;
    }

    let delta_seconds = time.delta_secs();
    if delta_seconds <= 0.0 {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_basis = camera_transform.compute_transform().rotation;
    let mut forward_xz = (camera_basis * -Vec3::Z).xz();
    let mut right_xz = (camera_basis * Vec3::X).xz();

    if forward_xz.length_squared() < 0.0001 || right_xz.length_squared() < 0.0001 {
        forward_xz = Vec2::new(0.0, -1.0);
        right_xz = Vec2::new(1.0, 0.0);
    } else {
        forward_xz = forward_xz.normalize();
        right_xz = right_xz.normalize();
    }

    for (mut transform, movement, mut velocity, collider) in &mut player_query {
        let mut input = Vec2::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            input.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            input.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            input.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            input.x += 1.0;
        }

        let run_pressed =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        let move_dir = (right_xz * input.x + forward_xz * input.y).normalize_or_zero();
        let target_speed = if run_pressed {
            movement.run_speed
        } else {
            movement.walk_speed
        };

        let desired_velocity = move_dir * target_speed;
        let rate = if desired_velocity.length_squared() > velocity.0.length_squared() {
            movement.acceleration
        } else {
            movement.deceleration
        };

        let delta_velocity = desired_velocity - velocity.0;
        let max_change = rate * delta_seconds;
        if delta_velocity.length() > max_change {
            velocity.0 += delta_velocity.normalize() * max_change;
        } else {
            velocity.0 = desired_velocity;
        }

        let move_step = velocity.0 * delta_seconds;
        let mut next = Vec2::new(transform.translation.x, transform.translation.z);

        // Resolve X and Z independently so movement naturally slides along walls.
        let test_x = Vec2::new(next.x + move_step.x, next.y);
        if !is_blocked(test_x, collider.radius, collision.as_deref()) {
            next.x = test_x.x;
        } else {
            velocity.0.x = 0.0;
        }

        let test_z = Vec2::new(next.x, next.y + move_step.y);
        if !is_blocked(test_z, collider.radius, collision.as_deref()) {
            next.y = test_z.y;
        } else {
            velocity.0.y = 0.0;
        }

        transform.translation.x = next.x;
        transform.translation.z = next.y;
    }
}

pub(crate) fn face_camera(
    camera_query: Query<&GlobalTransform, (With<MainCamera>, Without<Player>)>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_pos = camera_transform.translation();
    for mut player_transform in &mut player_query {
        let to_camera = Vec3::new(
            camera_pos.x - player_transform.translation.x,
            0.0,
            camera_pos.z - player_transform.translation.z,
        );

        if to_camera.length_squared() > 0.0001 {
            let yaw = to_camera.x.atan2(to_camera.z);
            player_transform.rotation = Quat::from_rotation_y(yaw);
        }
    }
}

fn is_blocked(position: Vec2, radius: f32, collision: Option<&LevelCollision>) -> bool {
    let Some(collision) = collision else {
        return false;
    };

    for center in &collision.wall_centers {
        let min = *center - collision.wall_half_extents;
        let max = *center + collision.wall_half_extents;
        let closest = position.clamp(min, max);

        if position.distance_squared(closest) <= radius * radius {
            return true;
        }
    }

    false
}

