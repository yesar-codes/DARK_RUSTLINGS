use bevy::app::AppExit;
use bevy::prelude::*;

use crate::game::level::{
    self, CurrentLevelIndex, LevelCollision, LevelEntity, LevelList,
};
use crate::game::player::{Player, PlayerCollider, Velocity};

const LEVEL_TIME_LIMIT_SECONDS: f32 = 30.0;

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

#[derive(Component)]
pub(crate) struct GameOverUiRoot;

#[derive(Component)]
pub(crate) struct RetryButton;

#[derive(Component)]
pub(crate) struct QuitButton;

#[derive(Component)]
pub(crate) struct TimerText;

pub(crate) fn spawn_timer_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                right: Val::Px(20.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 0, 0, 120)),
        ))
        .with_child((
            TimerText,
            Text::new("Time: 30"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

pub(crate) fn update_timer_ui(
    flow: Res<LevelFlow>,
    mut text_query: Query<&mut Text, With<TimerText>>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    if flow.game_over {
        if flow.won {
            text.0 = "You escaped!".into();
        } else {
            text.0 = "Time: 0".into();
        }
        return;
    }

    if flow.lights_on {
        text.0 = "Lights: ON".into();
        return;
    }

    let remaining = (LEVEL_TIME_LIMIT_SECONDS - flow.timer.elapsed_secs()).ceil().max(0.0) as i32;
    text.0 = format!("Time: {remaining}");
}

pub(crate) fn update_level_flow(
    mut commands: Commands,
    time: Res<Time>,
    mut flow: ResMut<LevelFlow>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_list: Res<LevelList>,
    mut current_level: ResMut<CurrentLevelIndex>,
    collision: Option<Res<LevelCollision>>,
    level_entities: Query<Entity, With<LevelEntity>>,
    mut player_query: Query<(&mut Transform, &PlayerCollider, &mut Velocity), With<Player>>,
    overlay_query: Query<Entity, With<GameOverUiRoot>>,
) {
    if flow.game_over {
        return;
    }

    let Ok((mut player_transform, collider, mut velocity)) = player_query.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation.xz();
    let trigger_distance = collider.radius + 0.45;
    let trigger_distance_sq = trigger_distance * trigger_distance;

    if let Some(collision) = collision.as_deref() {
        if !flow.lights_on {
            if let Some(switch_center) = collision.switch_center {
                if player_pos.distance_squared(switch_center) <= trigger_distance_sq {
                    flow.lights_on = true;
                    ambient_light.color = Color::WHITE;
                    ambient_light.brightness = 55.0;
                    info!("Light switch activated");
                }
            }
        }

        if let Some(exit_center) = collision.exit_center {
            if player_pos.distance_squared(exit_center) <= trigger_distance_sq {
                if current_level.0 + 1 < level_list.0.len() {
                    current_level.0 += 1;
                    level::despawn_level_entities(&mut commands, &level_entities);
                    let spawn = level::spawn_level_at_index(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        current_level.0,
                    )
                    .unwrap_or(Vec3::ZERO);

                    player_transform.translation = spawn + Vec3::Y * 0.8;
                    velocity.0 = Vec2::ZERO;
                    reset_for_new_level(&mut flow, &mut ambient_light);
                } else {
                    trigger_win_screen(&mut commands, &mut flow, &overlay_query);
                }
                return;
            }
        }
    }

    if !flow.lights_on {
        flow.timer.tick(time.delta());
        if flow.timer.is_finished() {
            trigger_game_over(&mut commands, &mut flow, &overlay_query);
        }
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
    mut app_exit: MessageWriter<AppExit>,
    mut flow: ResMut<LevelFlow>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_level: ResMut<CurrentLevelIndex>,
    level_entities: Query<Entity, With<LevelEntity>>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    ui_query: Query<Entity, With<GameOverUiRoot>>,
) {
    for (interaction, mut color, retry, quit) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb_u8(52, 152, 219));

                if retry.is_some() {
                    current_level.0 = 0;
                    level::despawn_level_entities(&mut commands, &level_entities);
                    let spawn = level::spawn_level_at_index(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        current_level.0,
                    )
                    .unwrap_or(Vec3::ZERO);

                    reset_for_new_level(&mut flow, &mut ambient_light);

                    if let Ok((mut player_transform, mut velocity)) = player_query.single_mut() {
                        player_transform.translation = spawn + Vec3::Y * 0.8;
                        velocity.0 = Vec2::ZERO;
                    }

                    if let Ok(ui_root) = ui_query.single() {
                        commands.entity(ui_root).despawn();
                    }
                }

                if quit.is_some() {
                    app_exit.write(AppExit::Success);
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

fn reset_for_new_level(flow: &mut LevelFlow, ambient_light: &mut GlobalAmbientLight) {
    flow.lights_on = false;
    flow.game_over = false;
    flow.won = false;
    flow.timer = Timer::from_seconds(LEVEL_TIME_LIMIT_SECONDS, TimerMode::Once);
    ambient_light.color = Color::BLACK;
    ambient_light.brightness = 0.0;
}

fn trigger_game_over(
    commands: &mut Commands,
    flow: &mut LevelFlow,
    overlay_query: &Query<Entity, With<GameOverUiRoot>>,
) {
    flow.game_over = true;
    flow.won = false;
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

            spawn_menu_button(parent, "Retry", RetryButton);
            spawn_menu_button(parent, "Quit", QuitButton);
        });
}

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

