use bevy::prelude::*;
use bevy::camera::ScalingMode;

#[derive(Component)]
pub struct MainCamera;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 10.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(8.0, 8.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn spawn_lighting(
    mut commands: Commands,
    mut ambient_light: ResMut<GlobalAmbientLight>,
) {
    ambient_light.color = Color::BLACK;
    ambient_light.brightness = 0.0;
    commands.insert_resource(ClearColor(Color::BLACK));
}

