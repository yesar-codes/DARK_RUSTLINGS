use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::prelude::*;

use crate::game::level::{
    self, CurrentLevelIndex, LevelCollision, LevelEntity, LevelList, LevelMusic, SwitchLight,
};
use crate::game::player::{
    PLAYER_SPAWN_HEIGHT_OFFSET, Player, PlayerCollider, PlayerLight, Velocity,
};

const LEVEL_TIME_LIMIT_SECONDS: f32 = 30.0;
const SWITCH_LIGHT_INTENSITY: f32 = 5000_0000.0;
const POWERUP_DURATION_SECONDS: f32 = 30.0;
const PLAYER_LIGHT_BASE_RANGE: f32 = 4.0;
const PLAYER_LIGHT_BOOST_MULTIPLIER: f32 = 2.0;

#[derive(Resource, Default)]
pub struct PauseState {
    pub paused: bool,
}

#[derive(Resource)]
pub struct LevelFlow {
    pub lights_on: bool,
    pub game_over: bool,
    pub won: bool,
    pub timer: Timer,
}

impl Default for LevelFlow {
    fn default() -> Self {
        Self {
            lights_on: false,
            game_over: false,
            won: false,
            timer: Timer::from_seconds(LEVEL_TIME_LIMIT_SECONDS, TimerMode::Once),
        }
    }
}

#[derive(Resource)]
pub struct PowerupState {
    pub speed_timer: Timer,
    pub light_timer: Timer,
    pub speed_active: bool,
    pub light_active: bool,
}

impl Default for PowerupState {
    fn default() -> Self {
        Self {
            speed_timer: Timer::from_seconds(POWERUP_DURATION_SECONDS, TimerMode::Once),
            light_timer: Timer::from_seconds(POWERUP_DURATION_SECONDS, TimerMode::Once),
            speed_active: false,
            light_active: false,
        }
    }
}

impl PowerupState {
    pub fn speed_multiplier(&self) -> f32 {
        if self.speed_active { 2.0 } else { 1.0 }
    }
}

#[derive(Component)]
pub(crate) struct GameOverUiRoot;

#[derive(Component)]
pub(crate) struct PauseUiRoot;

#[derive(Component)]
pub(crate) struct RetryButton;

#[derive(Component)]
pub(crate) struct PauseRetryButton;

#[derive(Component)]
pub(crate) struct QuitButton;

#[derive(Component)]
pub(crate) struct PauseQuitButton;

#[derive(Component)]
pub(crate) struct TimerText;

#[derive(Component)]
pub(crate) struct LevelText;

#[derive(Component)]
pub(crate) struct SpeedPowerupTimerText;

#[derive(Component)]
pub(crate) struct LightPowerupTimerText;

pub(crate) fn spawn_timer_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                right: Val::Px(20.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 0, 0, 120)),
        ))
        .with_children(|parent| {
            parent.spawn((
                TimerText,
                Text::new("Time: 30"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                SpeedPowerupTimerText,
                Text::new("Speed: --"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(80, 255, 90)),
            ));

            parent.spawn((
                LightPowerupTimerText,
                Text::new("Light: --"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(255, 96, 96)),
            ));
        });
}

pub(crate) fn spawn_level_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                left: Val::Px(20.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 0, 0, 120)),
        ))
        .with_child((
            LevelText,
            Text::new("Level 01"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

pub(crate) fn update_level_ui(
    level_list: Res<LevelList>,
    current_level: Res<CurrentLevelIndex>,
    mut text_query: Query<&mut Text, With<LevelText>>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    text.0 = format_current_level_label(current_level.0, level_list.0.len());
}

fn format_current_level_label(current_level_index: usize, premade_level_count: usize) -> String {
    if current_level_index < premade_level_count {
        return format!("Level {:02}", current_level_index);
    }

    let generated_index = current_level_index + 1 - premade_level_count;
    format!("Generated {:02}", generated_index)
}

pub(crate) fn update_timer_ui(
    flow: Res<LevelFlow>,
    powerups: Res<PowerupState>,
    mut text_query: Query<
        (
            &mut Text,
            Option<&TimerText>,
            Option<&SpeedPowerupTimerText>,
            Option<&LightPowerupTimerText>,
        ),
    >,
) {
    for (mut text, timer_marker, speed_marker, light_marker) in &mut text_query {
        if timer_marker.is_some() {
            if flow.game_over {
                if flow.won {
                    text.0 = "You escaped!".into();
                } else {
                    text.0 = "Time: 0".into();
                }
            } else if flow.lights_on {
                text.0 = "Lights: ON".into();
            } else {
                let remaining =
                    (LEVEL_TIME_LIMIT_SECONDS - flow.timer.elapsed_secs()).ceil().max(0.0) as i32;
                text.0 = format!("Time: {remaining}");
            }
        } else if speed_marker.is_some() {
            if powerups.speed_active {
                let remaining = (POWERUP_DURATION_SECONDS - powerups.speed_timer.elapsed_secs())
                    .ceil()
                    .max(0.0) as i32;
                text.0 = format!("Speed: {remaining}");
            } else {
                text.0 = "Speed: --".into();
            }
        } else if light_marker.is_some() {
            if powerups.light_active {
                let remaining = (POWERUP_DURATION_SECONDS - powerups.light_timer.elapsed_secs())
                    .ceil()
                    .max(0.0) as i32;
                text.0 = format!("Light: {remaining}");
            } else {
                text.0 = "Light: --".into();
            }
        }
    }
}

pub(crate) fn update_level_flow(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut flow: ResMut<LevelFlow>,
    mut powerups: ResMut<PowerupState>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_list: Res<LevelList>,
    mut current_level: ResMut<CurrentLevelIndex>,
    collision: Option<ResMut<LevelCollision>>,
    level_entities: Query<Entity, With<LevelEntity>>,
    music_entities: Query<Entity, With<LevelMusic>>,
    mut switch_lights: Query<&mut PointLight, With<SwitchLight>>,
    mut player_query: Query<(&mut Transform, &PlayerCollider, &mut Velocity), With<Player>>,
    pause_state: Res<PauseState>,
) {
    if flow.game_over || pause_state.paused {
        return;
    }

    let Ok((mut player_transform, collider, velocity)) = player_query.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation.xz();
    let trigger_distance = collider.radius + 0.45;
    let trigger_distance_sq = trigger_distance * trigger_distance;

    if let Some(mut collision) = collision {
        let switch_center = collision.switch_center;
        let speed_powerup_center = collision.speed_powerup_center;
        let light_powerup_center = collision.light_powerup_center;
        let exit_center = collision.exit_center;
        let exit_direction = collision.exit_direction;
        let tile_size = collision.tile_size;

        if !flow.lights_on {
            if let Some(switch_center) = switch_center {
                if player_pos.distance_squared(switch_center) <= trigger_distance_sq {
                    flow.lights_on = true;
                    ambient_light.color = Color::srgb_u8(225, 230, 240);
                    ambient_light.brightness = 0.35;
                    for mut switch_light in &mut switch_lights {
                        switch_light.intensity = SWITCH_LIGHT_INTENSITY + switch_light.range * 5_000.0;
                    }
                    if let Ok(music_entity) = music_entities.single() {
                        commands.entity(music_entity).despawn();
                    }
                    play_switch_sound(&mut commands, &asset_server);
                    info!("Light switch activated");
                }
            }
        }

        if let Some(speed_center) = speed_powerup_center {
            if player_pos.distance_squared(speed_center) <= trigger_distance_sq {
                powerups.speed_active = true;
                powerups.speed_timer = Timer::from_seconds(POWERUP_DURATION_SECONDS, TimerMode::Once);
                collision.speed_powerup_center = None;
                info!("Speed power-up collected");
            }
        }

        if let Some(light_center) = light_powerup_center {
            if player_pos.distance_squared(light_center) <= trigger_distance_sq {
                powerups.light_active = true;
                powerups.light_timer = Timer::from_seconds(POWERUP_DURATION_SECONDS, TimerMode::Once);
                collision.light_powerup_center = None;
                info!("Light-radius power-up collected");
            }
        }

        if let (Some(exit_center), Some(exit_direction)) = (exit_center, exit_direction) {
            let to_exit = player_pos - exit_center;
            let along_exit = to_exit.dot(exit_direction);
            let lateral = to_exit - exit_direction * along_exit;
            let lateral_limit = if exit_direction.x.abs() > 0.5 {
                tile_size.y * 0.55 + collider.radius
            } else {
                tile_size.x * 0.55 + collider.radius
            };

            let moving_through_exit = velocity.0.dot(exit_direction) > 0.0;
            if moving_through_exit && along_exit >= -0.05 && lateral.length() <= lateral_limit {
                current_level.0 += 1;
                level::despawn_level_entities(&mut commands, &level_entities);
                let spawn = level::spawn_level_at_index(
                    &mut commands,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    current_level.0,
                    &level_list.0,
                )
                    .unwrap_or(Vec3::ZERO);

                let carry_forward = Vec3::new(exit_direction.x, 0.0, exit_direction.y) * (collider.radius + 0.2);
                player_transform.translation = spawn + Vec3::Y * PLAYER_SPAWN_HEIGHT_OFFSET + carry_forward;
                reset_for_new_level(&mut flow, &mut powerups, &mut ambient_light);
                return;
            }
        }
    }

    if powerups.speed_active {
        powerups.speed_timer.tick(time.delta());
        if powerups.speed_timer.is_finished() {
            powerups.speed_active = false;
        }
    }

    if powerups.light_active {
        powerups.light_timer.tick(time.delta());
        if powerups.light_timer.is_finished() {
            powerups.light_active = false;
        }
    }

    if !flow.lights_on {
        flow.timer.tick(time.delta());
        if flow.timer.is_finished() {
            trigger_game_over(&mut commands, &mut flow, current_level.0);
        }
    }
}

pub(crate) fn update_player_light_range(
    powerups: Res<PowerupState>,
    mut player_lights: Query<&mut PointLight, With<PlayerLight>>,
) {
    let target_range = if powerups.light_active {
        PLAYER_LIGHT_BASE_RANGE * PLAYER_LIGHT_BOOST_MULTIPLIER
    } else {
        PLAYER_LIGHT_BASE_RANGE
    };

    for mut player_light in &mut player_lights {
        player_light.range = target_range;
    }
}

pub(crate) fn handle_game_over_buttons(
    mut commands: Commands,
    mut interactions: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&RetryButton>,
            Option<&QuitButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    asset_server: Res<AssetServer>,
    mut flow: ResMut<LevelFlow>,
    mut powerups: ResMut<PowerupState>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_level: Res<CurrentLevelIndex>,
    level_list: Res<LevelList>,
    level_entities: Query<Entity, With<LevelEntity>>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    ui_query: Query<Entity, With<GameOverUiRoot>>,
) {
    for (interaction, mut color, retry, quit) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb_u8(52, 152, 219));

                if retry.is_some() {
                    level::despawn_level_entities(&mut commands, &level_entities);
                    let spawn = level::spawn_level_at_index(
                        &mut commands,
                        &asset_server,
                        &mut meshes,
                        &mut materials,
                        current_level.0,
                        &level_list.0,
                    )
                        .unwrap_or(Vec3::ZERO);

                    reset_for_new_level(&mut flow, &mut powerups, &mut ambient_light);

                    if let Ok((mut player_transform, mut velocity)) = player_query.single_mut() {
                        player_transform.translation = spawn + Vec3::Y * PLAYER_SPAWN_HEIGHT_OFFSET;
                        velocity.0 = Vec2::ZERO;
                    }

                    if let Ok(ui_root) = ui_query.single() {
                        commands.entity(ui_root).despawn();
                    }
                }

                if quit.is_some() {
                    std::process::exit(0);
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb_u8(85, 95, 105));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb_u8(66, 73, 73));
            }
        }
    }
}

fn reset_for_new_level(
    flow: &mut LevelFlow,
    powerups: &mut PowerupState,
    ambient_light: &mut GlobalAmbientLight,
) {
    flow.lights_on = false;
    flow.game_over = false;
    flow.won = false;
    flow.timer = Timer::from_seconds(LEVEL_TIME_LIMIT_SECONDS, TimerMode::Once);
    powerups.speed_active = false;
    powerups.light_active = false;
    powerups.speed_timer = Timer::from_seconds(POWERUP_DURATION_SECONDS, TimerMode::Once);
    powerups.light_timer = Timer::from_seconds(POWERUP_DURATION_SECONDS, TimerMode::Once);
    ambient_light.color = Color::BLACK;
    ambient_light.brightness = 0.0;
}


fn trigger_game_over(
    commands: &mut Commands,
    flow: &mut LevelFlow,
    completed_levels_excluding_level_00: usize,
) {
    flow.game_over = true;
    flow.won = false;

    commands
        .spawn((
            GameOverUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 0, 0, 210)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Over"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("Find the switch or exit in 30 seconds"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(210, 210, 210)),
            ));

            parent.spawn((
                Text::new(format!(
                    "Completed levels: {}",
                    completed_levels_excluding_level_00
                )),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(235, 235, 235)),
            ));

            spawn_menu_button(parent, "Retry", RetryButton);
            spawn_menu_button(parent, "Quit", QuitButton);
        });
}

// dead code — kept for future use
#[allow(dead_code)]
fn trigger_win_screen(
    commands: &mut Commands,
    flow: &mut LevelFlow,
    overlay_query: &Query<Entity, With<GameOverUiRoot>>,
) {
    flow.game_over = true;
    flow.won = true;
    if !overlay_query.is_empty() {
        return;
    }

    commands
        .spawn((
            GameOverUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 0, 0, 210)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("You Win"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("You found the final exit"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(210, 210, 210)),
            ));

            spawn_menu_button(parent, "Retry", RetryButton);
            spawn_menu_button(parent, "Quit", QuitButton);
        });
}

fn spawn_menu_button<T: Component>(parent: &mut ChildSpawnerCommands, label: &str, marker: T) {
    parent
        .spawn((
            Button,
            marker,
            Node {
                width: Val::Px(220.0),
                height: Val::Px(56.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(66, 73, 73)),
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

pub(crate) fn toggle_pause_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut pause_state: ResMut<PauseState>,
    pause_ui_query: Query<Entity, With<PauseUiRoot>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    if pause_state.paused {
        pause_state.paused = false;
        if let Ok(pause_ui) = pause_ui_query.single() {
            commands.entity(pause_ui).despawn();
        }
    } else {
        pause_state.paused = true;
        spawn_pause_menu(&mut commands);
    }
}

fn spawn_pause_menu(commands: &mut Commands) {
    commands
        .spawn((
            PauseUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 0, 0, 210)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("Press Escape to Resume"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(210, 210, 210)),
            ));

            spawn_menu_button(parent, "Retry Level", PauseRetryButton);
            spawn_menu_button(parent, "Quit", PauseQuitButton);
        });
}

pub(crate) fn handle_pause_buttons(
    mut commands: Commands,
    mut interactions: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&PauseRetryButton>,
            Option<&PauseQuitButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    asset_server: Res<AssetServer>,
    mut pause_state: ResMut<PauseState>,
    mut flow: ResMut<LevelFlow>,
    mut powerups: ResMut<PowerupState>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_level: Res<CurrentLevelIndex>,
    level_list: Res<LevelList>,
    level_entities: Query<Entity, With<LevelEntity>>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    pause_ui_query: Query<Entity, With<PauseUiRoot>>,
) {
    for (interaction, mut color, retry, quit) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb_u8(52, 152, 219));

                if retry.is_some() {
                    pause_state.paused = false;
                    level::despawn_level_entities(&mut commands, &level_entities);
                    let spawn = level::spawn_level_at_index(
                        &mut commands,
                        &asset_server,
                        &mut meshes,
                        &mut materials,
                        current_level.0,
                        &level_list.0,
                    )
                        .unwrap_or(Vec3::ZERO);

                    reset_for_new_level(&mut flow, &mut powerups, &mut ambient_light);

                    if let Ok((mut player_transform, mut velocity)) = player_query.single_mut() {
                        player_transform.translation = spawn + Vec3::Y * PLAYER_SPAWN_HEIGHT_OFFSET;
                        velocity.0 = Vec2::ZERO;
                    }

                    if let Ok(pause_ui) = pause_ui_query.single() {
                        commands.entity(pause_ui).despawn();
                    }
                }

                if quit.is_some() {
                    std::process::exit(0);
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb_u8(85, 95, 105));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb_u8(66, 73, 73));
            }
        }
    }
}

fn play_switch_sound(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        LevelEntity,
        AudioPlayer::new(asset_server.load("audio/01_100-light.wav")),
        PlaybackSettings::DESPAWN,
    ));
}

