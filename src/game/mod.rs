pub mod camera;
pub mod gameplay;
pub mod level;
pub mod player;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<gameplay::LevelFlow>();
        app.init_resource::<gameplay::PauseState>();
        app.init_resource::<gameplay::PowerupState>();
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
        app.add_systems(Startup, gameplay::spawn_level_ui);
        app.add_systems(
            Update,
            (
                gameplay::toggle_pause_menu,
                player::move_player,
                gameplay::update_level_flow.after(player::move_player),
                gameplay::update_player_light_range.after(gameplay::update_level_flow),
                camera::follow_player.after(gameplay::update_level_flow),
                player::face_camera,
                gameplay::update_level_ui,
                gameplay::update_timer_ui,
                gameplay::handle_game_over_buttons,
                gameplay::handle_pause_buttons,
            ),
        );
    }
}

