use bevy::prelude::*;
use std::time::Duration;

use crate::game::camera::MainCamera;
use crate::game::gameplay::{LevelFlow, PauseState, PowerupState};
use crate::game::level::{LevelCollision, PlayerSpawnPoint};

pub const PLAYER_SPAWN_HEIGHT_OFFSET: f32 = 0.05;

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

#[derive(Resource)]
pub(crate) struct PlayerModelHandle(Handle<bevy::gltf::Gltf>);

#[derive(Resource)]
pub(crate) struct PlayerAnimations {
    graph: Handle<AnimationGraph>,
    indices: Vec<AnimationNodeIndex>,
}

#[derive(Component)]
pub(crate) struct PlayerAnimationConfigured;

#[derive(Component)]
pub(crate) struct HiddenModelExtra;

const MODEL_PATH: &str = "models/run.glb";
const MODEL_SCALE: f32 = 2.0;

pub(crate) fn load_player_model(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlayerModelHandle(asset_server.load(MODEL_PATH)));
}

#[derive(Component)]
pub struct PlayerLight;

pub(crate) fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    spawn_point: Option<Res<PlayerSpawnPoint>>,
) {
    let spawn = spawn_point
        .map(|point| point.0 + Vec3::Y * PLAYER_SPAWN_HEIGHT_OFFSET)
        .unwrap_or(Vec3::new(0.0, PLAYER_SPAWN_HEIGHT_OFFSET, 0.0));

    commands
        .spawn((
            Player,
            MovementConfig {
                walk_speed: 4.5,
                run_speed: 7.5,
                acceleration: 18.0,
                deceleration: 24.0,
            },
            Velocity::default(),
            PlayerCollider { radius: 0.30 },
            SceneRoot(asset_server.load(format!("{MODEL_PATH}#Scene0"))),
            Transform::from_translation(spawn).with_scale(Vec3::splat(MODEL_SCALE)),
        ))
        .with_children(|parent| {
            parent.spawn((
                PointLight {
                    intensity: 300_000.0,
                    range: 6.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_xyz(0.0, 2.0, 0.0),
            ));
        });
}

pub(crate) fn setup_player_animations(
    mut commands: Commands,
    model_handle: Option<Res<PlayerModelHandle>>,
    animations: Option<Res<PlayerAnimations>>,
    gltf_assets: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut unconfigured: Query<
        (Entity, &mut AnimationPlayer),
        Without<PlayerAnimationConfigured>,
    >,
) {
    if animations.is_none() {
        let Some(handle) = model_handle else { return };
        let Some(gltf) = gltf_assets.get(&handle.0) else { return };
        if gltf.animations.is_empty() {
            warn!("Player model has no animations — inserting empty graph");
            commands.insert_resource(PlayerAnimations {
                graph: graphs.add(AnimationGraph::new()),
                indices: vec![],
            });
            return;
        }

        for (name, _) in &gltf.named_animations {
            info!("Player animation found: {name}");
        }

        let (graph, indices) =
            AnimationGraph::from_clips(gltf.animations.iter().cloned());
        commands.insert_resource(PlayerAnimations {
            graph: graphs.add(graph),
            indices,
        });
        return;
    }

    let animations = animations.unwrap();
    for (entity, mut player) in &mut unconfigured {
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, animations.indices[0], Duration::ZERO)
            .repeat();
        commands.entity(entity).insert((
            AnimationGraphHandle(animations.graph.clone()),
            transitions,
            PlayerAnimationConfigured,
        ));
    }
}

pub(crate) fn update_player_animation(
    animations: Option<Res<PlayerAnimations>>,
    player_query: Query<&Velocity, With<Player>>,
    mut anim_query: Query<&mut AnimationPlayer, With<PlayerAnimationConfigured>>,
) {
    let Some(animations) = animations else { return };
    let Ok(velocity) = player_query.single() else { return };
    if animations.indices.is_empty() {
        return;
    }

    let move_speed = velocity.0.length();
    let anim_speed = if move_speed < 0.5 {
        0.0
    } else if move_speed < 5.0 {
        0.6
    } else {
        1.0
    };

    let idx = animations.indices[0];
    for mut player in &mut anim_query {
        if let Some(active) = player.animation_mut(idx) {
            active.set_speed(anim_speed);
        }
    }
}

pub(crate) fn hide_model_extras(
    mut commands: Commands,
    extras: Query<(Entity, &Name), Without<HiddenModelExtra>>,
) {
    for (entity, name) in &extras {
        if name.as_str() == "Cube" {
            commands
                .entity(entity)
                .insert((Visibility::Hidden, HiddenModelExtra));
        }
    }
}

pub(crate) fn face_movement_direction(
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &Velocity), With<Player>>,
) {
    for (mut transform, velocity) in &mut player_query {
        if velocity.0.length_squared() < 0.01 {
            continue;
        }
        let yaw = velocity.0.x.atan2(velocity.0.y);
        let target = Quat::from_rotation_y(yaw);
        transform.rotation =
            transform.rotation.slerp(target, (time.delta_secs() * 10.0).min(1.0));
    }
}

pub(crate) fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    flow: Option<Res<LevelFlow>>,
    pause_state: Res<PauseState>,
    powerup_state: Option<Res<PowerupState>>,
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
        let base_target_speed = if run_pressed {
            movement.run_speed
        } else {
            movement.walk_speed
        };
        let speed_multiplier = powerup_state
            .as_deref()
            .map_or(1.0, |state| state.speed_multiplier());
        let target_speed = base_target_speed * speed_multiplier;

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
        let origin = Vec2::new(transform.translation.x, transform.translation.z);

        let step_count = ((move_step.length() / collider.radius).ceil() as u32).max(1).min(8);
        let sub_step = move_step / step_count as f32;
        let mut pos = origin;

        for _ in 0..step_count {
            let candidate = pos + sub_step;
            pos = resolve_collisions(candidate, collider.radius, collision.as_deref());
        }

        let actual_move = pos - origin;
        if actual_move.length_squared() > 1e-8 {
            let allowed_dir = actual_move.normalize();
            let original_speed = velocity.0.length();
            velocity.0 = allowed_dir * original_speed;
        } else {
            velocity.0 = Vec2::ZERO;
        }

        transform.translation.x = pos.x;
        transform.translation.z = pos.y;
    }
}

fn resolve_collisions(mut position: Vec2, radius: f32, collision: Option<&LevelCollision>) -> Vec2 {
    let Some(collision) = collision else {
        return position;
    };

    for _ in 0..4 {
        let mut any = false;
        for center in &collision.wall_centers {
            let half = collision.wall_half_extents;
            let min = *center - half;
            let max = *center + half;
            let closest = position.clamp(min, max);
            let diff = position - closest;
            let dist_sq = diff.length_squared();

            if dist_sq >= radius * radius {
                continue;
            }

            any = true;
            if dist_sq > 1e-8 {
                let dist = dist_sq.sqrt();
                position += (diff / dist) * (radius - dist);
            } else {
                let dl = position.x - min.x;
                let dr = max.x - position.x;
                let db = position.y - min.y;
                let dt = max.y - position.y;
                let m = dl.min(dr).min(db).min(dt);
                if m == dl {
                    position.x = min.x - radius;
                } else if m == dr {
                    position.x = max.x + radius;
                } else if m == db {
                    position.y = min.y - radius;
                } else {
                    position.y = max.y + radius;
                }
            }
        }
        if !any {
            break;
        }
    }

    position
}
