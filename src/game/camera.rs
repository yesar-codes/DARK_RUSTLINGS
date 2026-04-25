use bevy::prelude::*;
use bevy::camera::ScalingMode;

use crate::game::player::Player;

#[derive(Component)]
pub struct MainCamera;

const CAMERA_OFFSET: Vec3 = Vec3::new(30.0, 30.0, 30.0);

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 36.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(30.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let target = player_transform.translation;
    camera_transform.translation = target + CAMERA_OFFSET;
    camera_transform.look_at(target, Vec3::Y);
}

pub fn spawn_lighting(
    mut commands: Commands,
    mut ambient_light: ResMut<GlobalAmbientLight>,
) {
    ambient_light.color = Color::BLACK;
    ambient_light.brightness = 0.0;
    commands.insert_resource(ClearColor(Color::BLACK));
}

