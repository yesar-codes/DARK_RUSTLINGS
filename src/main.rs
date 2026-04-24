use bevy::prelude::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Dark Rustlings".into(),
                mode: bevy::window::WindowMode::BorderlessFullscreen(
                    bevy::window::MonitorSelection::Primary,
                ),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
) {
    let window = windows.single().unwrap();

    let width = window.width();
    let height = window.height();
    let wall_thickness = 20.0;

    // Kamera
    commands.spawn(Camera2d);

    // Raum / Hintergrund
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.05, 0.05, 0.05),
            Vec2::new(width, height),
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Obere Wand
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.2, 0.2, 0.2),
            Vec2::new(width, wall_thickness),
        ),
        Transform::from_xyz(0.0, height / 2.0 - wall_thickness / 2.0, 1.0),
    ));

    // Untere Wand
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.2, 0.2, 0.2),
            Vec2::new(width, wall_thickness),
        ),
        Transform::from_xyz(0.0, -height / 2.0 + wall_thickness / 2.0, 1.0),
    ));

    // Linke Wand
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.2, 0.2, 0.2),
            Vec2::new(wall_thickness, height),
        ),
        Transform::from_xyz(-width / 2.0 + wall_thickness / 2.0, 0.0, 1.0),
    ));

    // Rechte Wand
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.2, 0.2, 0.2),
            Vec2::new(wall_thickness, height),
        ),
        Transform::from_xyz(width / 2.0 - wall_thickness / 2.0, 0.0, 1.0),
    ));
}
