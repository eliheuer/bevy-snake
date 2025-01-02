//! A simple snake game demonstrating built with Bevy.
//! Move with arrow keys or WASD.
//! Eat the red food to grow. Don't hit walls or yourself!

use bevy::prelude::*;
use rand::{thread_rng, Rng};

// Game configuration constants
const ARENA_WIDTH: u32 = 24;
const ARENA_HEIGHT: u32 = 24;
const CELL_SIZE: f32 = 24.0;
const SNAKE_HEAD_COLOR: Color = Color::srgb(0.1, 0.8, 0.0);
const SNAKE_SEGMENT_COLOR: Color = Color::srgb(0.2, 0.7, 0.3);
const FOOD_COLOR: Color = Color::srgb(1.0, 0.1, 0.0);
const BACKGROUND_COLOR: Color = Color::srgb(0.04, 0.04, 0.04);
const SCORE_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const SNAKE_MOVE_SPEED: f32 = 0.075; // Lower number = faster speed (seconds between moves)
const GAME_FONT: &str = "fonts/Rena-BoldDisplay.ttf";

#[derive(Default, States, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Playing,
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
        .add_plugins(DefaultPlugins)
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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                snake_movement_input,
                snake_movement.run_if(in_state(GameState::Playing)),
                snake_eating.run_if(in_state(GameState::Playing)),
                snake_growth.run_if(in_state(GameState::Playing)),
                game_over.run_if(in_state(GameState::Playing)),
                update_scoreboard,
                handle_game_over.run_if(in_state(GameState::GameOver)),
            ).chain(),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2d);

    // Walls
    // Left wall
    commands.spawn((
        Wall,
        Sprite {
            color: Color::srgb(0.8, 0.8, 0.8),
            custom_size: Some(Vec2::new(CELL_SIZE, (ARENA_HEIGHT + 3) as f32 * CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            (-(ARENA_WIDTH as i32) as f32 / 2.0 - 1.0) * CELL_SIZE,
            0.0,
            0.0,
        ),
    ));

    // Right wall
    commands.spawn((
        Wall,
        Sprite {
            color: Color::srgb(0.8, 0.8, 0.8),
            custom_size: Some(Vec2::new(CELL_SIZE, (ARENA_HEIGHT + 3) as f32 * CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            (ARENA_WIDTH as f32 / 2.0 + 1.0) * CELL_SIZE,
            0.0,
            0.0,
        ),
    ));

    // Bottom wall
    commands.spawn((
        Wall,
        Sprite {
            color: Color::srgb(0.8, 0.8, 0.8),
            custom_size: Some(Vec2::new((ARENA_WIDTH + 3) as f32 * CELL_SIZE, CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            0.0,
            (-(ARENA_HEIGHT as i32) as f32 / 2.0 - 1.0) * CELL_SIZE,
            0.0,
        ),
    ));

    // Top wall
    commands.spawn((
        Wall,
        Sprite {
            color: Color::srgb(0.8, 0.8, 0.8),
            custom_size: Some(Vec2::new((ARENA_WIDTH + 3) as f32 * CELL_SIZE, CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            0.0,
            (ARENA_HEIGHT as f32 / 2.0 + 1.0) * CELL_SIZE,
            0.0,
        ),
    ));

    // First spawn the head
    let head = commands.spawn((
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
    )).id();

    // Then spawn the initial tail segment
    let segment = commands.spawn((
        Sprite {
            color: SNAKE_SEGMENT_COLOR,
            custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, -CELL_SIZE, 0.0),
        Visibility::default(),
        SnakeSegment,
    )).id();

    // Now we can safely insert the SnakeSegments resource
    commands.insert_resource(SnakeSegments(vec![head, segment]));

    // Food
    spawn_food(&mut commands);

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
        } else if keyboard_input.pressed(KeyCode::ArrowUp) 
            || keyboard_input.pressed(KeyCode::KeyW) 
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
) {
    if !timer.0.tick(time.delta()).finished() {
        return;
    }
    
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .iter()
            .skip(1) // Skip the head entity
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
        
        // Check for wall collision
        let half_arena_width = (ARENA_WIDTH as f32 * CELL_SIZE) / 2.0;
        let half_arena_height = (ARENA_HEIGHT as f32 * CELL_SIZE) / 2.0;
        if head_pos.translation.x < -half_arena_width
            || head_pos.translation.x > half_arena_width
            || head_pos.translation.y < -half_arena_height
            || head_pos.translation.y > half_arena_height
        {
            game_over_writer.send(GameOverEvent);
        }

        // Update body segments
        let head_pos = positions.get(head_entity).unwrap().translation;
        for (i, segment) in segments.iter().skip(1).enumerate() {
            *positions.get_mut(*segment).unwrap() = Transform::from_translation(
                if i == 0 {
                    head_pos
                } else {
                    segment_positions[i - 1]
                }
            );
        }
    }
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    mut score: ResMut<Score>,
    food_positions: Query<(Entity, &Transform), With<Food>>,
    head_positions: Query<&Transform, With<SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (food_entity, food_pos) in food_positions.iter() {
            if (head_pos.translation.x - food_pos.translation.x).abs() < CELL_SIZE / 2.0
                && (head_pos.translation.y - food_pos.translation.y).abs() < CELL_SIZE / 2.0
            {
                commands.entity(food_entity).despawn();
                growth_writer.send(GrowthEvent);
                score.0 += 1;
                spawn_food(&mut commands);
            }
        }
    }
}

fn spawn_food(commands: &mut Commands) {
    let mut rng = thread_rng();
    let x = (rng.gen_range(-(ARENA_WIDTH as i32) / 2..(ARENA_WIDTH as i32) / 2)) as f32 * CELL_SIZE;
    let y = (rng.gen_range(-(ARENA_HEIGHT as i32) / 2..(ARENA_HEIGHT as i32) / 2)) as f32 * CELL_SIZE;
    
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
) {
    if growth_reader.read().next().is_some() {
        segments.push(
            commands
                .spawn((
                    Sprite {
                        color: SNAKE_SEGMENT_COLOR,
                        custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::ZERO),
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
    walls: Query<Entity, With<Wall>>,
    text: Query<Entity, With<Text>>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Remove all entities
        for entity in segments.iter().chain(food.iter()).chain(walls.iter()) {
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
        setup(commands, asset_server);
        next_state.set(GameState::Playing);
    }
}

fn update_scoreboard(score: Res<Score>, mut query: Query<&mut Text>) {
    for mut text in &mut query {
        text.0 = format!("Score: {}", score.0);
    }
} 
