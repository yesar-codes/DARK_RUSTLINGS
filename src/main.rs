use bevy::asset::AssetPlugin;
use bevy::prelude::*;
mod game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "resources".into(),
            ..default()
        }))
        .add_plugins(game::GamePlugin)
        .run();
}
