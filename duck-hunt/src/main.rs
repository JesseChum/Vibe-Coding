use bevy::{
    prelude::*,
    window::PrimaryWindow,
};

// ─── Constants ───────────────────────────────────────────────────────────────

const WINDOW_W: f32 = 800.0;
const WINDOW_H: f32 = 600.0;

const DUCKS_PER_ROUND: u32 = 10;
const SHOTS_PER_ROUND: u32 = 3;
const DUCKS_TO_CLEAR: u32 = 6;

const DUCK_SPEED_BASE: f32 = 120.0;
const DUCK_SPEED_PER_ROUND: f32 = 20.0;

const DOG_DISPLAY_TIME: f32 = 2.0;
const ROUND_PAUSE_TIME: f32 = 1.5;

// ─── Game States ─────────────────────────────────────────────────────────────

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Title,
    Playing,
    DogReaction,
    RoundOver,
    GameOver,
}

// ─── Resources ───────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct GameData {
    score: u32,
    round: u32,
    ducks_this_round: u32,
    ducks_hit: u32,
    ducks_missed: u32,
    shots_left: u32,
    last_duck_was_hit: bool,
}

#[derive(Resource)]
struct DogTimer(Timer);

#[derive(Resource)]
struct RoundPauseTimer(Timer);

#[derive(Resource)]
struct SpawnTimer(Timer);

// ─── Components ──────────────────────────────────────────────────────────────

#[derive(Component)]
struct Duck {
    velocity: Vec2,
    flap_timer: f32,
}

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct Dog;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct RoundText;

#[derive(Component)]
struct TitleScreen;

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct HitEffect {
    timer: Timer,
}

#[derive(Component)]
struct SceneEntity;

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Duck Hunt".into(),
                resolution: (WINDOW_W, WINDOW_H).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .insert_resource(GameData::default())
        .insert_resource(ClearColor(Color::srgb(0.39, 0.70, 0.93)))
        .add_systems(Startup, setup_camera)
        // Title
        .add_systems(OnEnter(GameState::Title), spawn_title_screen)
        .add_systems(OnExit(GameState::Title), despawn_tagged::<TitleScreen>)
        // Playing
        .add_systems(OnEnter(GameState::Playing), (setup_scene, setup_hud, setup_crosshair, init_round))
        .add_systems(OnExit(GameState::Playing), despawn_tagged::<Duck>)
        // Dog
        .add_systems(OnEnter(GameState::DogReaction), spawn_dog)
        .add_systems(OnExit(GameState::DogReaction), despawn_tagged::<Dog>)
        // Game Over
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
        .add_systems(OnExit(GameState::GameOver), cleanup_scene)
        // Update loops
        .add_systems(Update, title_input.run_if(in_state(GameState::Title)))
        .add_systems(
            Update,
            (move_crosshair, spawn_duck, move_ducks, shoot, tick_hit_effects, check_round_end)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, move_crosshair.run_if(in_state(GameState::DogReaction)))
        .add_systems(Update, dog_timer_tick.run_if(in_state(GameState::DogReaction)))
        .add_systems(Update, round_pause_tick.run_if(in_state(GameState::RoundOver)))
        .add_systems(Update, game_over_input.run_if(in_state(GameState::GameOver)))
        .run();
}

// ─── Setup ────────────────────────────────────────────────────────────────────

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_scene(mut commands: Commands) {
    // Ground / dirt
    commands.spawn((
        Sprite {
            color: Color::srgb(0.55, 0.38, 0.18),
            custom_size: Some(Vec2::new(WINDOW_W, 80.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_H / 2.0 + 40.0, 0.0),
        SceneEntity,
    ));
    // Grass
    commands.spawn((
        Sprite {
            color: Color::srgb(0.13, 0.55, 0.13),
            custom_size: Some(Vec2::new(WINDOW_W, 40.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_H / 2.0 + 60.0, 1.0),
        SceneEntity,
    ));
    // Bush left
    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.45, 0.10),
            custom_size: Some(Vec2::new(120.0, 60.0)),
            ..default()
        },
        Transform::from_xyz(-280.0, -WINDOW_H / 2.0 + 70.0, 2.0),
        SceneEntity,
    ));
    // Bush right
    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.45, 0.10),
            custom_size: Some(Vec2::new(100.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(300.0, -WINDOW_H / 2.0 + 68.0, 2.0),
        SceneEntity,
    ));
}

fn setup_crosshair(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.1, 0.1),
            custom_size: Some(Vec2::new(4.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        Crosshair,
        SceneEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.1, 0.1),
            custom_size: Some(Vec2::new(32.0, 4.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        Crosshair,
        SceneEntity,
    ));
}

fn setup_hud(mut commands: Commands, game: Res<GameData>) {
    commands.spawn((
        Text::new(format!("SCORE: {}", game.score)),
        TextFont { font_size: 28.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
        SceneEntity,
    ));

    commands.spawn((
        Text::new(format!("ROUND {}", game.round + 1)),
        TextFont { font_size: 28.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        RoundText,
        SceneEntity,
    ));
}

fn init_round(mut commands: Commands, mut game: ResMut<GameData>) {
    game.ducks_this_round = 0;
    game.ducks_hit = 0;
    game.ducks_missed = 0;
    game.shots_left = SHOTS_PER_ROUND;
    commands.insert_resource(SpawnTimer(Timer::from_seconds(1.2, TimerMode::Repeating)));
}

// ─── Title Screen ─────────────────────────────────────────────────────────────

fn spawn_title_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            TitleScreen,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("DUCK HUNT"),
                TextFont { font_size: 72.0, ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.1)),
            ));
            p.spawn((
                Text::new("Click to Start"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::WHITE),
            ));
            p.spawn((
                Text::new("Left-click to shoot  |  Hit 6 / 10 ducks to advance"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

fn title_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game: ResMut<GameData>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        *game = GameData::default();
        next_state.set(GameState::Playing);
    }
}

// ─── Duck Spawning & Movement ─────────────────────────────────────────────────

fn spawn_duck(
    mut commands: Commands,
    mut game: ResMut<GameData>,
    mut spawn_timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    ducks: Query<&Duck>,
) {
    if !ducks.is_empty() || game.ducks_this_round >= DUCKS_PER_ROUND {
        return;
    }

    spawn_timer.0.tick(time.delta());
    if !spawn_timer.0.just_finished() {
        return;
    }

    game.ducks_this_round += 1;
    game.shots_left = SHOTS_PER_ROUND;

    let speed = DUCK_SPEED_BASE + game.round as f32 * DUCK_SPEED_PER_ROUND;
    let from_left = (game.ducks_this_round % 2) == 1;
    let start_x = if from_left { -WINDOW_W / 2.0 - 20.0 } else { WINDOW_W / 2.0 + 20.0 };
    let start_y = -WINDOW_H / 2.0 + 100.0 + (game.ducks_this_round as f32 * 17.0 % 100.0);
    let vx = if from_left { speed } else { -speed };

    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.55, 0.35, 0.10),
                custom_size: Some(Vec2::new(48.0, 28.0)),
                ..default()
            },
            Transform::from_xyz(start_x, start_y, 5.0),
            Duck {
                velocity: Vec2::new(vx, speed * 0.5),
                flap_timer: 0.0,
            },
        ))
        .with_children(|p| {
            // Head
            p.spawn((
                Sprite {
                    color: Color::srgb(0.20, 0.50, 0.20),
                    custom_size: Some(Vec2::new(20.0, 18.0)),
                    ..default()
                },
                Transform::from_xyz(22.0, 8.0, 0.1),
            ));
            // Bill
            p.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.65, 0.0),
                    custom_size: Some(Vec2::new(12.0, 7.0)),
                    ..default()
                },
                Transform::from_xyz(34.0, 5.0, 0.2),
            ));
            // Wing
            p.spawn((
                Sprite {
                    color: Color::srgb(0.72, 0.52, 0.25),
                    custom_size: Some(Vec2::new(38.0, 12.0)),
                    ..default()
                },
                Transform::from_xyz(-2.0, 18.0, 0.1),
            ));
        });
}

fn move_ducks(
    mut commands: Commands,
    mut ducks: Query<(Entity, &mut Duck, &mut Transform)>,
    mut game: ResMut<GameData>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    for (entity, mut duck, mut transform) in &mut ducks {
        duck.flap_timer += time.delta_secs();

        transform.translation.x += duck.velocity.x * time.delta_secs();
        transform.translation.y += duck.velocity.y * time.delta_secs();

        // Gentle sine bob
        transform.translation.y += (duck.flap_timer * 5.0).sin() * 18.0 * time.delta_secs();

        // Bounce off ceiling
        if transform.translation.y > WINDOW_H / 2.0 - 50.0 {
            duck.velocity.y = -duck.velocity.y.abs();
        }
        // Don't dip into grass
        if transform.translation.y < -WINDOW_H / 2.0 + 100.0 && duck.velocity.y < 0.0 {
            duck.velocity.y = duck.velocity.y.abs();
        }

        // Face movement direction
        transform.scale.x = if duck.velocity.x > 0.0 { 1.0 } else { -1.0 };

        // Escaped off screen
        let escaped = transform.translation.x < -WINDOW_W / 2.0 - 60.0
            || transform.translation.x > WINDOW_W / 2.0 + 60.0;

        if escaped && *state.get() == GameState::Playing {
            game.ducks_missed += 1;
            game.last_duck_was_hit = false;
            commands.entity(entity).despawn_recursive();
            commands.insert_resource(DogTimer(Timer::from_seconds(DOG_DISPLAY_TIME, TimerMode::Once)));
            next_state.set(GameState::DogReaction);
        }
    }
}

// ─── Crosshair ────────────────────────────────────────────────────────────────

fn move_crosshair(
    mut crosshairs: Query<&mut Transform, With<Crosshair>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(mut win) = window.get_single_mut() else { return };
    win.cursor_options.visible = false;

    let Some(cursor_pos) = win.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    for mut t in &mut crosshairs {
        t.translation.x = world_pos.x;
        t.translation.y = world_pos.y;
    }
}

// ─── Shooting ─────────────────────────────────────────────────────────────────

fn shoot(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut game: ResMut<GameData>,
    ducks: Query<(Entity, &Transform), With<Duck>>,
    crosshairs: Query<&Transform, With<Crosshair>>,
    mut score_texts: Query<&mut Text, With<ScoreText>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !mouse.just_pressed(MouseButton::Left) || game.shots_left == 0 {
        return;
    }

    game.shots_left -= 1;

    let cursor = crosshairs
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (entity, duck_transform) in &ducks {
        if cursor.distance(duck_transform.translation.truncate()) < 38.0 {
            game.ducks_hit += 1;
            game.score += 100 + game.round * 50;
            game.last_duck_was_hit = true;

            // Feather burst
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.85, 0.65, 0.25, 1.0),
                    custom_size: Some(Vec2::new(24.0, 24.0)),
                    ..default()
                },
                Transform::from_xyz(duck_transform.translation.x, duck_transform.translation.y, 8.0),
                HitEffect { timer: Timer::from_seconds(0.45, TimerMode::Once) },
            ));

            commands.entity(entity).despawn_recursive();

            for mut text in &mut score_texts {
                **text = format!("SCORE: {}", game.score);
            }

            commands.insert_resource(DogTimer(Timer::from_seconds(DOG_DISPLAY_TIME, TimerMode::Once)));
            next_state.set(GameState::DogReaction);
            break;
        }
    }

}

// ─── Hit Effect ───────────────────────────────────────────────────────────────

fn tick_hit_effects(
    mut commands: Commands,
    mut effects: Query<(Entity, &mut HitEffect, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut fx, mut sprite) in &mut effects {
        fx.timer.tick(time.delta());
        let alpha = 1.0 - fx.timer.fraction();
        sprite.color = Color::srgba(0.85, 0.65, 0.25, alpha);
        if fx.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ─── Dog Reaction ─────────────────────────────────────────────────────────────

fn spawn_dog(mut commands: Commands, game: Res<GameData>) {
    let body_color = Color::srgb(0.75, 0.55, 0.28);
    let ear_color = Color::srgb(0.50, 0.32, 0.12);

    commands
        .spawn((
            Sprite {
                color: body_color,
                custom_size: Some(Vec2::new(50.0, 60.0)),
                ..default()
            },
            Transform::from_xyz(-200.0, -WINDOW_H / 2.0 + 80.0, 6.0),
            Dog,
        ))
        .with_children(|p| {
            // Head
            p.spawn((
                Sprite {
                    color: body_color,
                    custom_size: Some(Vec2::new(42.0, 38.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 50.0, 0.1),
            ));
            // Ear left
            p.spawn((
                Sprite {
                    color: ear_color,
                    custom_size: Some(Vec2::new(14.0, 26.0)),
                    ..default()
                },
                Transform::from_xyz(-18.0, 62.0, 0.0),
            ));
            // Ear right
            p.spawn((
                Sprite {
                    color: ear_color,
                    custom_size: Some(Vec2::new(14.0, 26.0)),
                    ..default()
                },
                Transform::from_xyz(18.0, 62.0, 0.0),
            ));
            // Expression: duck trophy (hit) or laugh teeth (miss)
            if game.last_duck_was_hit {
                // Small duck silhouette the dog is holding up
                p.spawn((
                    Sprite {
                        color: Color::srgb(0.55, 0.35, 0.10),
                        custom_size: Some(Vec2::new(30.0, 18.0)),
                        ..default()
                    },
                    Transform::from_xyz(30.0, 30.0, 0.2),
                ));
            } else {
                // Laughing — white teeth rectangle
                p.spawn((
                    Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::new(26.0, 10.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 38.0, 0.2),
                ));
            }
        });
}

fn dog_timer_tick(
    mut commands: Commands,
    mut timer: ResMut<DogTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    game: Res<GameData>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        if game.ducks_this_round >= DUCKS_PER_ROUND {
            commands.insert_resource(RoundPauseTimer(Timer::from_seconds(ROUND_PAUSE_TIME, TimerMode::Once)));
            next_state.set(GameState::RoundOver);
        } else {
            next_state.set(GameState::Playing);
        }
    }
}

// ─── Round End ────────────────────────────────────────────────────────────────

fn check_round_end(
    game: Res<GameData>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    ducks: Query<&Duck>,
) {
    if game.ducks_this_round >= DUCKS_PER_ROUND && ducks.is_empty() {
        commands.insert_resource(RoundPauseTimer(Timer::from_seconds(ROUND_PAUSE_TIME, TimerMode::Once)));
        next_state.set(GameState::RoundOver);
    }
}

fn round_pause_tick(
    mut timer: ResMut<RoundPauseTimer>,
    time: Res<Time>,
    mut game: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        if game.ducks_hit < DUCKS_TO_CLEAR {
            next_state.set(GameState::GameOver);
        } else {
            game.round += 1;
            next_state.set(GameState::Playing);
        }
    }
}

// ─── Game Over ────────────────────────────────────────────────────────────────

fn spawn_game_over_screen(mut commands: Commands, game: Res<GameData>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            GameOverScreen,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("GAME OVER"),
                TextFont { font_size: 72.0, ..default() },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
            ));
            p.spawn((
                Text::new(format!("Final Score: {}", game.score)),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::WHITE),
            ));
            p.spawn((
                Text::new(format!("You reached Round {}", game.round + 1)),
                TextFont { font_size: 28.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
            p.spawn((
                Text::new("Click to Play Again"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.1)),
            ));
        });
}

fn game_over_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game: ResMut<GameData>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        *game = GameData::default();
        next_state.set(GameState::Playing);
    }
}

fn cleanup_scene(
    mut commands: Commands,
    entities: Query<Entity, With<SceneEntity>>,
    game_over: Query<Entity, With<GameOverScreen>>,
) {
    for e in &entities {
        commands.entity(e).despawn_recursive();
    }
    for e in &game_over {
        commands.entity(e).despawn_recursive();
    }
}

// ─── Generic Despawn Helper ───────────────────────────────────────────────────

fn despawn_tagged<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}
