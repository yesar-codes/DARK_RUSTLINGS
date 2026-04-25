pub mod camera;
pub mod gameplay;
pub mod level;
pub mod player;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<gameplay::LevelFlow>();
        app.add_systems(
            Startup,
            (
                camera::spawn_camera,
                camera::spawn_lighting,
                level::spawn_initial_level,
            ),
        );
        app.add_systems(Startup, player::spawn_player.after(level::spawn_initial_level));
        app.add_systems(Startup, gameplay::spawn_timer_ui);
        app.add_systems(
            Update,
            (
                player::move_player,
                gameplay::update_level_flow.after(player::move_player),
                player::face_camera,
                gameplay::update_timer_ui,
                gameplay::handle_game_over_buttons,
            ),
        );
    }
}

