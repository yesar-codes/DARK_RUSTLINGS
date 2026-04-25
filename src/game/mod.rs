pub mod camera;
pub mod level;
pub mod player;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                camera::spawn_camera,
                camera::spawn_lighting,
                level::spawn_initial_level,
            ),
        );
        app.add_systems(Startup, player::spawn_player.after(level::spawn_initial_level));
        app.add_systems(Update, (player::move_player, player::face_camera));
    }
}

