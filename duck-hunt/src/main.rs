use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;

// ─── Constants ───────────────────────────────────────────────────────────────

const WINDOW_W: f32 = 800.0;
const WINDOW_H: f32 = 600.0;

const DUCKS_PER_ROUND: u32 = 10;
const SHOTS_PER_WAVE: u32 = 3;
const DUCKS_TO_CLEAR: u32 = 6;
const MAX_MISSES: u32 = 5;

const DUCK_SPEED_BASE: f32 = 250.0;
const DUCK_SPEED_PER_ROUND: f32 = 40.0;

const DOG_DISPLAY_TIME: f32 = 2.0;
const ROUND_PAUSE_TIME: f32 = 1.5;

// ─── States ──────────────────────────────────────────────────────────────────

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
    shots_left: u32,
    total_misses: u32,
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
struct MissText;

#[derive(Component)]
struct AmmoBullet(u32);

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
        .add_systems(OnEnter(GameState::Title), spawn_title_screen)
        .add_systems(OnExit(GameState::Title), despawn_tagged::<TitleScreen>)
        .add_systems(OnEnter(GameState::Playing), (setup_scene, setup_hud, setup_crosshair, init_round))
        .add_systems(OnExit(GameState::Playing), despawn_tagged::<Duck>)
        .add_systems(OnEnter(GameState::DogReaction), spawn_dog)
        .add_systems(OnExit(GameState::DogReaction), despawn_tagged::<Dog>)
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
        .add_systems(OnExit(GameState::GameOver), cleanup_scene)
        .add_systems(Update, title_input.run_if(in_state(GameState::Title)))
        .add_systems(
            Update,
            (move_crosshair, spawn_duck, move_ducks, shoot, tick_hit_effects, check_round_end, update_hud)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, (move_crosshair, update_hud).run_if(in_state(GameState::DogReaction)))
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
    // Sky gradient suggestion — dark horizon strip
    commands.spawn((
        Sprite {
            color: Color::srgb(0.30, 0.58, 0.82),
            custom_size: Some(Vec2::new(WINDOW_W, 120.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_H / 2.0 + 120.0, 0.0),
        SceneEntity,
    ));
    // Ground / dirt
    commands.spawn((
        Sprite {
            color: Color::srgb(0.48, 0.32, 0.12),
            custom_size: Some(Vec2::new(WINDOW_W, 80.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_H / 2.0 + 40.0, 1.0),
        SceneEntity,
    ));
    // Grass strip
    commands.spawn((
        Sprite {
            color: Color::srgb(0.15, 0.58, 0.15),
            custom_size: Some(Vec2::new(WINDOW_W, 36.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_H / 2.0 + 58.0, 2.0),
        SceneEntity,
    ));
    // Grass tufts — darker strip
    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.45, 0.10),
            custom_size: Some(Vec2::new(WINDOW_W, 14.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_H / 2.0 + 72.0, 2.1),
        SceneEntity,
    ));
    // Bush left (layered for depth)
    for (ox, oy, w, h, z) in [(-290.0, 70.0, 110.0, 55.0, 3.0), (-280.0, 80.0, 80.0, 44.0, 3.1)] {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.08, 0.40, 0.08),
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_xyz(ox, -WINDOW_H / 2.0 + oy, z),
            SceneEntity,
        ));
    }
    // Bush right
    for (ox, oy, w, h, z) in [(295.0, 68.0, 100.0, 50.0, 3.0), (305.0, 78.0, 70.0, 40.0, 3.1)] {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.08, 0.40, 0.08),
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_xyz(ox, -WINDOW_H / 2.0 + oy, z),
            SceneEntity,
        ));
    }
    // HUD bottom bar
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.55),
            custom_size: Some(Vec2::new(WINDOW_W, 38.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_H / 2.0 + 19.0, 8.0),
        SceneEntity,
    ));
}

fn setup_crosshair(mut commands: Commands) {
    // Vertical bar
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.05, 0.05),
            custom_size: Some(Vec2::new(3.0, 34.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        Crosshair,
        SceneEntity,
    ));
    // Horizontal bar
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.05, 0.05),
            custom_size: Some(Vec2::new(34.0, 3.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        Crosshair,
        SceneEntity,
    ));
    // Center dot
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.05, 0.05),
            custom_size: Some(Vec2::new(5.0, 5.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.1),
        Crosshair,
        SceneEntity,
    ));
}

fn setup_hud(mut commands: Commands, game: Res<GameData>) {
    // Score — top left
    commands.spawn((
        Text::new(format!("SCORE: {}", game.score)),
        TextFont { font_size: 26.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(12.0),
            ..default()
        },
        ScoreText,
        SceneEntity,
    ));
    // Round — top right
    commands.spawn((
        Text::new(format!("ROUND {}", game.round + 1)),
        TextFont { font_size: 26.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(12.0),
            ..default()
        },
        RoundText,
        SceneEntity,
    ));
    // Misses — bottom right text
    commands.spawn((
        Text::new(format!("MISSES: {}/{}", game.total_misses, MAX_MISSES)),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgb(1.0, 0.4, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(12.0),
            ..default()
        },
        MissText,
        SceneEntity,
    ));

    // Ammo bullets — 3 bullet sprites in bottom-left of HUD bar
    for i in 0..SHOTS_PER_WAVE {
        let x = -WINDOW_W / 2.0 + 28.0 + i as f32 * 22.0;
        let y = -WINDOW_H / 2.0 + 19.0;
        // Bullet casing
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.82, 0.1),
                custom_size: Some(Vec2::new(10.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 9.0),
            AmmoBullet(i),
            SceneEntity,
        ));
        // Bullet tip
        commands.spawn((
            Sprite {
                color: Color::srgb(0.85, 0.65, 0.10),
                custom_size: Some(Vec2::new(10.0, 6.0)),
                ..default()
            },
            Transform::from_xyz(x, y + 13.0, 9.1),
            SceneEntity,
        ));
    }
}

fn init_round(mut commands: Commands, mut game: ResMut<GameData>) {
    game.ducks_this_round = 0;
    game.ducks_hit = 0;
    game.shots_left = SHOTS_PER_WAVE;
    commands.insert_resource(SpawnTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
}

fn update_hud(
    game: Res<GameData>,
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<MissText>, Without<RoundText>)>,
    mut miss_q: Query<&mut Text, (With<MissText>, Without<ScoreText>, Without<RoundText>)>,
    mut bullets: Query<(&AmmoBullet, &mut Sprite)>,
) {
    for mut t in &mut score_q {
        **t = format!("SCORE: {}", game.score);
    }
    for mut t in &mut miss_q {
        **t = format!("MISSES: {}/{}", game.total_misses, MAX_MISSES);
    }
    for (b, mut sprite) in &mut bullets {
        sprite.color = if b.0 < game.shots_left {
            Color::srgb(1.0, 0.82, 0.1) // loaded — gold
        } else {
            Color::srgb(0.22, 0.22, 0.22) // spent — dark
        };
    }
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
                row_gap: Val::Px(22.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            TitleScreen,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("DUCK HUNT"),
                TextFont { font_size: 80.0, ..default() },
                TextColor(Color::srgb(1.0, 0.88, 0.1)),
            ));
            p.spawn((
                Text::new("Click to Start"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::WHITE),
            ));
            p.spawn((
                Text::new("Shoot 6/10 ducks to advance   |   5 misses = Game Over"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.75, 0.75, 0.75)),
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

// ─── Duck ─────────────────────────────────────────────────────────────────────

fn spawn_duck(
    mut commands: Commands,
    mut game: ResMut<GameData>,
    mut spawn_timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    ducks: Query<&Duck>,
) {
    if game.ducks_this_round >= DUCKS_PER_ROUND {
        return;
    }
    let max_simultaneous = (1 + game.round).min(3);
    let duck_count = ducks.iter().count() as u32;
    if duck_count >= max_simultaneous {
        return;
    }

    spawn_timer.0.tick(time.delta());
    if !spawn_timer.0.just_finished() {
        return;
    }

    game.ducks_this_round += 1;
    if duck_count == 0 {
        game.shots_left = SHOTS_PER_WAVE;
    }

    let speed = DUCK_SPEED_BASE + game.round as f32 * DUCK_SPEED_PER_ROUND;
    let mut rng = rand::thread_rng();
    let side = rng.gen_range(0..3_u32);

    let (start_x, start_y, velocity) = match side {
        0 => {
            let y = rng.gen_range(-WINDOW_H / 2.0 + 120.0..WINDOW_H / 2.0 - 80.0);
            let angle = rng.gen_range(-25.0_f32..45.0_f32).to_radians();
            (-WINDOW_W / 2.0 - 30.0, y, Vec2::new(speed * angle.cos(), speed * angle.sin()))
        }
        1 => {
            let y = rng.gen_range(-WINDOW_H / 2.0 + 120.0..WINDOW_H / 2.0 - 80.0);
            let angle = rng.gen_range(135.0_f32..205.0_f32).to_radians();
            (WINDOW_W / 2.0 + 30.0, y, Vec2::new(speed * angle.cos(), speed * angle.sin()))
        }
        _ => {
            let x = rng.gen_range(-WINDOW_W / 2.0 + 60.0..WINDOW_W / 2.0 - 60.0);
            let angle = rng.gen_range(50.0_f32..130.0_f32).to_radians();
            (x, -WINDOW_H / 2.0 + 95.0, Vec2::new(speed * angle.cos(), speed * angle.sin()))
        }
    };

    // Body color varies by "species"
    let (body_col, head_col) = match rng.gen_range(0..3_u32) {
        0 => (Color::srgb(0.52, 0.33, 0.10), Color::srgb(0.10, 0.42, 0.18)), // mallard
        1 => (Color::srgb(0.18, 0.40, 0.32), Color::srgb(0.14, 0.30, 0.25)), // teal
        _ => (Color::srgb(0.62, 0.55, 0.40), Color::srgb(0.48, 0.38, 0.22)), // female/brown
    };

    commands
        .spawn((
            Sprite { color: body_col, custom_size: Some(Vec2::new(58.0, 24.0)), ..default() },
            Transform::from_xyz(start_x, start_y, 5.0),
            Duck { velocity, flap_timer: 0.0 },
        ))
        .with_children(|p| {
            // Tail feathers (back)
            p.spawn((Sprite {
                color: Color::srgb(body_col.to_srgba().red * 0.7,
                                   body_col.to_srgba().green * 0.7,
                                   body_col.to_srgba().blue * 0.7),
                custom_size: Some(Vec2::new(18.0, 14.0)), ..default()
            }, Transform::from_xyz(-34.0, 8.0, -0.1)));
            // Wing highlight (lighter stripe across body)
            p.spawn((Sprite {
                color: Color::srgb(
                    (body_col.to_srgba().red + 0.18).min(1.0),
                    (body_col.to_srgba().green + 0.12).min(1.0),
                    (body_col.to_srgba().blue + 0.08).min(1.0),
                ),
                custom_size: Some(Vec2::new(46.0, 10.0)), ..default()
            }, Transform::from_xyz(-4.0, 14.0, 0.1)));
            // Neck
            p.spawn((Sprite {
                color: body_col,
                custom_size: Some(Vec2::new(14.0, 20.0)), ..default()
            }, Transform::from_xyz(26.0, 8.0, 0.0)));
            // White collar ring (mallard-style)
            p.spawn((Sprite {
                color: Color::srgb(0.88, 0.88, 0.88),
                custom_size: Some(Vec2::new(14.0, 5.0)), ..default()
            }, Transform::from_xyz(26.0, 0.0, 0.15)));
            // Head
            p.spawn((Sprite {
                color: head_col,
                custom_size: Some(Vec2::new(24.0, 22.0)), ..default()
            }, Transform::from_xyz(38.0, 20.0, 0.1)));
            // Eye
            p.spawn((Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(5.0, 5.0)), ..default()
            }, Transform::from_xyz(46.0, 24.0, 0.3)));
            // Eye glint
            p.spawn((Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(2.0, 2.0)), ..default()
            }, Transform::from_xyz(47.0, 25.0, 0.4)));
            // Upper bill
            p.spawn((Sprite {
                color: Color::srgb(0.95, 0.68, 0.05),
                custom_size: Some(Vec2::new(20.0, 7.0)), ..default()
            }, Transform::from_xyz(52.0, 17.0, 0.2)));
            // Lower bill
            p.spawn((Sprite {
                color: Color::srgb(0.80, 0.55, 0.04),
                custom_size: Some(Vec2::new(20.0, 5.0)), ..default()
            }, Transform::from_xyz(52.0, 11.0, 0.2)));
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
        // Sine bob
        transform.translation.y += (duck.flap_timer * 5.5).sin() * 16.0 * time.delta_secs();

        if transform.translation.y > WINDOW_H / 2.0 - 50.0 {
            duck.velocity.y = -duck.velocity.y.abs();
        }
        if transform.translation.y < -WINDOW_H / 2.0 + 110.0 && duck.velocity.y < 0.0 {
            duck.velocity.y = duck.velocity.y.abs();
        }
        transform.scale.x = if duck.velocity.x >= 0.0 { 1.0 } else { -1.0 };

        let escaped = transform.translation.x < -WINDOW_W / 2.0 - 70.0
            || transform.translation.x > WINDOW_W / 2.0 + 70.0;

        if escaped && *state.get() == GameState::Playing {
            game.total_misses += 1;
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
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !mouse.just_pressed(MouseButton::Left) || game.shots_left == 0 {
        return;
    }
    game.shots_left -= 1;

    let cursor = crosshairs.iter().next().map(|t| t.translation.truncate()).unwrap_or(Vec2::ZERO);

    for (entity, dt) in &ducks {
        if cursor.distance(dt.translation.truncate()) < 40.0 {
            game.ducks_hit += 1;
            game.score += 100 + game.round * 50;
            game.last_duck_was_hit = true;

            // Feather burst
            for i in 0..5_u32 {
                let ox = (i as f32 - 2.0) * 10.0;
                let oy = (i % 2) as f32 * 12.0;
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.80, 0.60, 0.20, 1.0),
                        custom_size: Some(Vec2::new(8.0, 14.0)),
                        ..default()
                    },
                    Transform::from_xyz(dt.translation.x + ox, dt.translation.y + oy, 8.0),
                    HitEffect { timer: Timer::from_seconds(0.5, TimerMode::Once) },
                ));
            }

            commands.entity(entity).despawn_recursive();
            commands.insert_resource(DogTimer(Timer::from_seconds(DOG_DISPLAY_TIME, TimerMode::Once)));
            next_state.set(GameState::DogReaction);
            break;
        }
    }
}

fn tick_hit_effects(
    mut commands: Commands,
    mut effects: Query<(Entity, &mut HitEffect, &mut Sprite, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut fx, mut sprite, mut transform) in &mut effects {
        fx.timer.tick(time.delta());
        let frac = fx.timer.fraction();
        sprite.color = Color::srgba(0.80, 0.60, 0.20, 1.0 - frac);
        transform.translation.y += 40.0 * time.delta_secs(); // feathers drift upward
        if fx.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ─── Dog ──────────────────────────────────────────────────────────────────────

fn spawn_dog(mut commands: Commands, game: Res<GameData>) {
    let tan = Color::srgb(0.78, 0.58, 0.30);
    let tan_light = Color::srgb(0.90, 0.78, 0.58);
    let brown_dark = Color::srgb(0.38, 0.22, 0.08);
    let base_y = -WINDOW_H / 2.0 + 90.0;

    commands
        .spawn((
            // Body
            Sprite { color: tan, custom_size: Some(Vec2::new(58.0, 78.0)), ..default() },
            Transform::from_xyz(-180.0, base_y, 6.0),
            Dog,
        ))
        .with_children(|p| {
            // Belly patch
            p.spawn((Sprite { color: tan_light, custom_size: Some(Vec2::new(34.0, 36.0)), ..default() },
                Transform::from_xyz(0.0, -12.0, 0.1)));
            // Left front leg
            p.spawn((Sprite { color: tan, custom_size: Some(Vec2::new(14.0, 30.0)), ..default() },
                Transform::from_xyz(-16.0, -38.0, 0.05)));
            // Right front leg
            p.spawn((Sprite { color: tan, custom_size: Some(Vec2::new(14.0, 30.0)), ..default() },
                Transform::from_xyz(16.0, -38.0, 0.05)));
            // Left paw
            p.spawn((Sprite { color: tan_light, custom_size: Some(Vec2::new(18.0, 10.0)), ..default() },
                Transform::from_xyz(-16.0, -55.0, 0.1)));
            // Right paw
            p.spawn((Sprite { color: tan_light, custom_size: Some(Vec2::new(18.0, 10.0)), ..default() },
                Transform::from_xyz(16.0, -55.0, 0.1)));
            // Red collar
            p.spawn((Sprite { color: Color::srgb(0.80, 0.10, 0.10), custom_size: Some(Vec2::new(54.0, 10.0)), ..default() },
                Transform::from_xyz(0.0, 26.0, 0.2)));
            // Collar tag (gold)
            p.spawn((Sprite { color: Color::srgb(0.95, 0.78, 0.10), custom_size: Some(Vec2::new(8.0, 10.0)), ..default() },
                Transform::from_xyz(0.0, 21.0, 0.3)));
            // Head
            p.spawn((Sprite { color: tan, custom_size: Some(Vec2::new(52.0, 46.0)), ..default() },
                Transform::from_xyz(0.0, 68.0, 0.1)));
            // Left ear (floppy, hangs down)
            p.spawn((Sprite { color: brown_dark, custom_size: Some(Vec2::new(16.0, 42.0)), ..default() },
                Transform::from_xyz(-28.0, 54.0, 0.0)));
            // Right ear
            p.spawn((Sprite { color: brown_dark, custom_size: Some(Vec2::new(16.0, 42.0)), ..default() },
                Transform::from_xyz(28.0, 54.0, 0.0)));
            // Muzzle
            p.spawn((Sprite { color: tan_light, custom_size: Some(Vec2::new(32.0, 22.0)), ..default() },
                Transform::from_xyz(0.0, 54.0, 0.2)));
            // Nose
            p.spawn((Sprite { color: Color::srgb(0.12, 0.08, 0.06), custom_size: Some(Vec2::new(14.0, 9.0)), ..default() },
                Transform::from_xyz(0.0, 47.0, 0.3)));
            // Left eye white
            p.spawn((Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(12.0, 12.0)), ..default() },
                Transform::from_xyz(-14.0, 74.0, 0.2)));
            // Right eye white
            p.spawn((Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(12.0, 12.0)), ..default() },
                Transform::from_xyz(14.0, 74.0, 0.2)));
            // Left pupil
            p.spawn((Sprite { color: Color::BLACK, custom_size: Some(Vec2::new(7.0, 7.0)), ..default() },
                Transform::from_xyz(-14.0, 74.0, 0.3)));
            // Right pupil
            p.spawn((Sprite { color: Color::BLACK, custom_size: Some(Vec2::new(7.0, 7.0)), ..default() },
                Transform::from_xyz(14.0, 74.0, 0.3)));

            if game.last_duck_was_hit {
                // Happy: eyebrows up, holding duck overhead
                // Eyebrows (raised)
                p.spawn((Sprite { color: brown_dark, custom_size: Some(Vec2::new(10.0, 3.0)), ..default() },
                    Transform::from_xyz(-14.0, 82.0, 0.4)));
                p.spawn((Sprite { color: brown_dark, custom_size: Some(Vec2::new(10.0, 3.0)), ..default() },
                    Transform::from_xyz(14.0, 82.0, 0.4)));
                // Duck held above head — small silhouette
                p.spawn((Sprite { color: Color::srgb(0.52, 0.33, 0.10), custom_size: Some(Vec2::new(38.0, 18.0)), ..default() },
                    Transform::from_xyz(0.0, 120.0, 0.4)));
                p.spawn((Sprite { color: Color::srgb(0.10, 0.42, 0.18), custom_size: Some(Vec2::new(18.0, 16.0)), ..default() },
                    Transform::from_xyz(18.0, 130.0, 0.5)));
                p.spawn((Sprite { color: Color::srgb(0.95, 0.68, 0.05), custom_size: Some(Vec2::new(14.0, 6.0)), ..default() },
                    Transform::from_xyz(30.0, 125.0, 0.5)));
                // Arms raised
                p.spawn((Sprite { color: tan, custom_size: Some(Vec2::new(10.0, 30.0)), ..default() },
                    Transform::from_xyz(-32.0, 95.0, 0.2)));
                p.spawn((Sprite { color: tan, custom_size: Some(Vec2::new(10.0, 30.0)), ..default() },
                    Transform::from_xyz(32.0, 95.0, 0.2)));
            } else {
                // Laughing: mouth open showing teeth, eyes squinting
                // Squint lines over eyes
                p.spawn((Sprite { color: tan, custom_size: Some(Vec2::new(13.0, 5.0)), ..default() },
                    Transform::from_xyz(-14.0, 77.0, 0.4)));
                p.spawn((Sprite { color: tan, custom_size: Some(Vec2::new(13.0, 5.0)), ..default() },
                    Transform::from_xyz(14.0, 77.0, 0.4)));
                // Open mouth (dark)
                p.spawn((Sprite { color: Color::srgb(0.20, 0.08, 0.05), custom_size: Some(Vec2::new(24.0, 12.0)), ..default() },
                    Transform::from_xyz(0.0, 53.0, 0.4)));
                // Teeth
                p.spawn((Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(22.0, 5.0)), ..default() },
                    Transform::from_xyz(0.0, 57.0, 0.5)));
                // "HA HA" implied by raised paw
                p.spawn((Sprite { color: tan, custom_size: Some(Vec2::new(10.0, 24.0)), ..default() },
                    Transform::from_xyz(36.0, 52.0, 0.2)));
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
        if game.total_misses >= MAX_MISSES {
            next_state.set(GameState::GameOver);
        } else if game.ducks_this_round >= DUCKS_PER_ROUND {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.68)),
            GameOverScreen,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("GAME OVER"),
                TextFont { font_size: 76.0, ..default() },
                TextColor(Color::srgb(1.0, 0.18, 0.18)),
            ));
            p.spawn((
                Text::new(format!("Score: {}   Round: {}", game.score, game.round + 1)),
                TextFont { font_size: 34.0, ..default() },
                TextColor(Color::WHITE),
            ));
            p.spawn((
                Text::new(format!("Ducks escaped: {}/{}", game.total_misses, MAX_MISSES)),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::srgb(1.0, 0.5, 0.5)),
            ));
            p.spawn((
                Text::new("Click to Play Again"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(Color::srgb(1.0, 0.88, 0.1)),
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
    scene_q: Query<Entity, With<SceneEntity>>,
    go_q: Query<Entity, With<GameOverScreen>>,
) {
    for e in &scene_q { commands.entity(e).despawn_recursive(); }
    for e in &go_q { commands.entity(e).despawn_recursive(); }
}

fn despawn_tagged<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for e in &query { commands.entity(e).despawn_recursive(); }
}
