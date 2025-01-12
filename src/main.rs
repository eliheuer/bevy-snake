//! A simple snake game demonstrating built with Bevy.
//! Move with arrow keys or WASD.
//! Eat the red food to grow. Don't hit walls or yourself!

use bevy::prelude::*;
use rand::{thread_rng, Rng};

// Game configuration constants
const CELL_SIZE: f32 = 32.0;
const GRID_SIZE: u32 = 24;
const WINDOW_SIZE: f32 = CELL_SIZE * GRID_SIZE as f32;
const SNAKE_HEAD_COLOR: Color = Color::srgb(0.2, 0.8, 0.3);
const SNAKE_SEGMENT_COLOR: Color = Color::srgb(0.2, 0.8, 0.3);
const FOOD_COLOR: Color = Color::srgb(1.0, 0.1, 0.0);
const BACKGROUND_COLOR: Color = Color::srgb(0.04, 0.04, 0.04);
const SCORE_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const SNAKE_MOVE_SPEED: f32 = 0.075; // Lower number = faster speed (seconds between moves)
const GAME_FONT: &str = "fonts/Rena-BoldDisplay.ttf";

#[derive(Default, States, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct Wall;

#[derive(Resource, Default, Deref, DerefMut)]
struct SnakeSegments(Vec<Entity>);

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Resource)]
struct MovementTimer(Timer);

#[derive(Event)]
struct GrowthEvent;

#[derive(Event)]
struct GameOverEvent;

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WINDOW_SIZE, WINDOW_SIZE).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(SnakeSegments::default())
        .insert_resource(Score::default())
        .insert_resource(MovementTimer(Timer::from_seconds(
            SNAKE_MOVE_SPEED,
            TimerMode::Repeating,
        )))
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_systems(Startup, setup.run_if(|windows: Query<&Window>| windows.get_single().is_ok()))
        .add_systems(
            Update,
            (
                handle_pause,
                snake_movement_input.run_if(in_state(GameState::Playing)),
                snake_movement.run_if(in_state(GameState::Playing)),
                snake_eating.run_if(in_state(GameState::Playing)),
                snake_growth.run_if(in_state(GameState::Playing)),
                game_over.run_if(in_state(GameState::Playing)),
                update_scoreboard,
                handle_game_over.run_if(in_state(GameState::GameOver)),
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    // Camera
    commands.spawn(Camera2d);

    // First spawn the head
    let head = commands
        .spawn((
            Sprite {
                color: SNAKE_HEAD_COLOR,
                custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::default(),
            SnakeHead {
                direction: Direction::Up,
            },
        ))
        .id();

    // Then spawn the initial tail segment
    let segment = commands
        .spawn((
            Sprite {
                color: SNAKE_SEGMENT_COLOR,
                custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                ..default()
            },
            Transform::from_xyz(0.0, -CELL_SIZE, 0.0),
            Visibility::default(),
            SnakeSegment,
        ))
        .id();

    // Now we can safely insert the SnakeSegments resource
    commands.insert_resource(SnakeSegments(vec![head, segment]));

    // Food
    if let Ok(window) = windows.get_single() {
        spawn_food(&mut commands, window);
    }

    // Scoreboard
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font: asset_server.load(GAME_FONT),
            font_size: 56.0,
            ..default()
        },
        TextColor(SCORE_COLOR),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(24.0),
            left: Val::Px(24.0),
            ..default()
        },
    ));
}

fn snake_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut heads: Query<&mut SnakeHead>,
) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::ArrowLeft)
            || keyboard_input.pressed(KeyCode::KeyA)
        {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::ArrowDown)
            || keyboard_input.pressed(KeyCode::KeyS)
        {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW)
        {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::ArrowRight)
            || keyboard_input.pressed(KeyCode::KeyD)
        {
            Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}

fn snake_movement(
    mut game_over_writer: EventWriter<GameOverEvent>,
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Transform>,
    time: Res<Time>,
    mut timer: ResMut<MovementTimer>,
    windows: Query<&Window>,
) {
    if !timer.0.tick(time.delta()).finished() {
        return;
    }

    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .iter()
            .skip(1)
            .map(|e| positions.get_mut(*e).unwrap().translation)
            .collect::<Vec<Vec3>>();

        let mut head_pos = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            Direction::Left => head_pos.translation.x -= CELL_SIZE,
            Direction::Right => head_pos.translation.x += CELL_SIZE,
            Direction::Up => head_pos.translation.y += CELL_SIZE,
            Direction::Down => head_pos.translation.y -= CELL_SIZE,
        };

        // Check for self-collision
        if segment_positions.contains(&head_pos.translation) {
            game_over_writer.send(GameOverEvent);
        }

        // Check for window bounds collision
        let half_size = (GRID_SIZE as f32 / 2.0) * CELL_SIZE;
        if head_pos.translation.x < -half_size
            || head_pos.translation.x >= half_size
            || head_pos.translation.y < -half_size
            || head_pos.translation.y >= half_size
        {
            game_over_writer.send(GameOverEvent);
        }

        // Update body segments
        let head_pos = positions.get(head_entity).unwrap().translation;
        for (i, segment) in segments.iter().skip(1).enumerate() {
            *positions.get_mut(*segment).unwrap() = Transform::from_translation(if i == 0 {
                head_pos
            } else {
                segment_positions[i - 1]
            });
        }
    }
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    mut score: ResMut<Score>,
    food_positions: Query<(Entity, &Transform), With<Food>>,
    head_positions: Query<&Transform, With<SnakeHead>>,
    windows: Query<&Window>,
) {
    for head_pos in head_positions.iter() {
        for (food_entity, food_pos) in food_positions.iter() {
            if (head_pos.translation.x - food_pos.translation.x).abs() < CELL_SIZE / 2.0
                && (head_pos.translation.y - food_pos.translation.y).abs() < CELL_SIZE / 2.0
            {
                commands.entity(food_entity).despawn();
                growth_writer.send(GrowthEvent);
                score.0 += 1;
                if let Ok(window) = windows.get_single() {
                    spawn_food(&mut commands, window);
                }
            }
        }
    }
}

fn spawn_food(commands: &mut Commands, window: &Window) {
    let mut rng = thread_rng();
    let half_grid = (GRID_SIZE as f32 / 2.0);
    
    // Generate position in grid coordinates
    let grid_x = rng.gen_range(-half_grid..half_grid);
    let grid_y = rng.gen_range(-half_grid..half_grid);
    
    // Convert to world coordinates and ensure alignment to grid
    let x = grid_x.floor() * CELL_SIZE;
    let y = grid_y.floor() * CELL_SIZE;

    commands.spawn((
        Sprite {
            color: FOOD_COLOR,
            custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(x, y, 0.0),
        Visibility::default(),
        Food,
    ));
}

fn snake_growth(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
    positions: Query<&Transform>,
) {
    if growth_reader.read().next().is_some() {
        // Get the position of the last segment
        let last_segment_pos = if let Some(last_segment) = segments.last() {
            positions.get(*last_segment).unwrap().translation
        } else {
            Vec3::ZERO // Fallback, should never happen
        };

        segments.push(
            commands
                .spawn((
                    Sprite {
                        color: SNAKE_SEGMENT_COLOR,
                        custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(last_segment_pos),
                    Visibility::default(),
                    SnakeSegment,
                ))
                .id(),
        );
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if reader.read().next().is_some() {
        next_state.set(GameState::GameOver);
        commands.spawn((
            Text::new("Game Over! Press SPACE to restart"),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(SCORE_COLOR),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(45.0),
                left: Val::Percent(35.0),
                ..default()
            },
        ));
    }
}

fn handle_game_over(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut segments_res: ResMut<SnakeSegments>,
    mut score: ResMut<Score>,
    segments: Query<Entity, With<SnakeSegment>>,
    heads: Query<Entity, With<SnakeHead>>,
    food: Query<Entity, With<Food>>,
    text: Query<Entity, With<Text>>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Remove all entities
        for entity in segments.iter().chain(food.iter()) {
            commands.entity(entity).despawn();
        }
        for entity in heads.iter() {
            commands.entity(entity).despawn();
        }
        for entity in text.iter() {
            commands.entity(entity).despawn();
        }

        // Reset score
        score.0 = 0;

        // Clear segments
        segments_res.clear();

        // Reset game
        setup(commands, asset_server, windows);
        next_state.set(GameState::Playing);
    }
}

fn update_scoreboard(score: Res<Score>, mut query: Query<&mut Text>) {
    for mut text in &mut query {
        text.0 = format!("Score: {}", score.0);
    }
}

fn handle_pause(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => (), // Do nothing in other states
        }
    }
}
