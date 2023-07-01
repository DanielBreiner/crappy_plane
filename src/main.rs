use std::time::Duration;

use bevy::{prelude::*, sprite::collide_aabb::collide};
use rand::Rng;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flappy Plane".to_string(),
                resolution: (800., 480.).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(SpawnTimer(Timer::new(
            STARTING_SPAWN_TIMER_DURATION,
            TimerMode::Repeating,
        )))
        .add_startup_system(initialize_scene)
        .add_system(update_plane_velocity)
        .add_system(update_plane_position.after(update_plane_velocity))
        .add_system(update_obstacles)
        .add_system(spawn_obstacles)
        .add_system(check_game_over)
        .add_system(restart)
        .add_system(bevy::window::close_on_esc)
        .run();
}

const JUMP_VELOCITY: f32 = 1000.;
const MAX_VELOCITY: f32 = 800.;
const GRAVITY: f32 = -2000.;
const OBSTACLE_SPEED: f32 = 500.;
const STARTING_SPAWN_TIMER_DURATION: Duration = Duration::from_millis(500);
const SPAWN_TIMER_DECREMENT: Duration = Duration::from_millis(5);
const PLANE_INITIAL_POSITION: Vec3 = Vec3::new(-200., 100., 100.);

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Component)]
struct Plane {
    velocity: f32,
}
impl Default for Plane {
    fn default() -> Self {
        Self { velocity: 0. }
    }
}

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct ObstacleBound;

#[derive(Component)]
struct ObstacleActive;

#[derive(Component)]
struct GameOverMessage;

#[derive(Component)]
pub struct ScoreCounter {
    pub score: i32,
}
impl Default for ScoreCounter {
    fn default() -> Self {
        Self { score: -4 }
    }
}

#[derive(Component)]
pub struct Collidable {
    pub size: Vec2,
}

#[derive(Bundle)]
pub struct CollidableSpriteBundle {
    pub sprite_bundle: SpriteBundle,
    pub collidable: Collidable,
}

impl CollidableSpriteBundle {
    fn new(
        texture: Handle<Image>,
        collision_size: Vec2,
        position: Vec3,
        rotation: Option<Quat>,
    ) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                texture,
                transform: Transform {
                    translation: position,
                    rotation: rotation.unwrap_or(Quat::default()),
                    ..default()
                },
                ..default()
            },
            collidable: Collidable {
                size: collision_size,
            },
        }
    }
}

fn initialize_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let plane_texture: Handle<Image> = asset_server.load("plane.png");
    // Plane
    commands.spawn((
        CollidableSpriteBundle::new(
            plane_texture,
            Vec2::new(80., 70.),
            PLANE_INITIAL_POSITION,
            None,
        ),
        Plane::default(),
    ));
    // Background
    commands.spawn(SpriteBundle {
        texture: asset_server.load("background.png"),
        ..default()
    });
    // Bounds
    let bounds_texture = asset_server.load("ground.png");
    commands.spawn((
        CollidableSpriteBundle::new(
            bounds_texture.clone(),
            Vec2::new(800., 30.),
            Vec3::new(0., -220., 1.),
            None,
        ),
        Obstacle,
        ObstacleBound,
    ));
    commands.spawn((
        CollidableSpriteBundle::new(
            bounds_texture.clone(),
            Vec2::new(800., 30.),
            Vec3::new(0., 220., 1.),
            Some(Quat::from_rotation_z(180f32.to_radians())),
        ),
        Obstacle,
        ObstacleBound,
    ));
    commands.spawn((
        CollidableSpriteBundle::new(
            bounds_texture.clone(),
            Vec2::new(800., 30.),
            Vec3::new(800., -220., 1.),
            None,
        ),
        Obstacle,
        ObstacleBound,
    ));
    commands.spawn((
        CollidableSpriteBundle::new(
            bounds_texture,
            Vec2::new(800., 30.),
            Vec3::new(800., 220., 1.),
            Some(Quat::from_rotation_z(180f32.to_radians())),
        ),
        Obstacle,
        ObstacleBound,
    ));
    // Score
    let font = asset_server.load("font.ttf");
    commands.spawn((
        TextBundle::from_section(
            "Score: ",
            TextStyle {
                font: font,
                font_size: 36.0,
                color: Color::BLACK,
            },
        ),
        ScoreCounter::default(),
    ));
}

fn create_random_obstacle(
    texture: Handle<Image>,
) -> (CollidableSpriteBundle, Obstacle, ObstacleActive) {
    let mut rng = rand::thread_rng();
    let up = rng.gen_bool(0.5);
    (
        CollidableSpriteBundle::new(
            texture,
            Vec2::new(70., 200.),
            Vec3::new(800., if up { 200. } else { -200. }, 0.5),
            Some(if up {
                Quat::from_rotation_z(180f32.to_radians())
            } else {
                Quat::default()
            }),
        ),
        Obstacle,
        ObstacleActive,
    )
}

fn update_plane_velocity(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Plane>,
) {
    let mut plane = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        plane.velocity = (plane.velocity + JUMP_VELOCITY).min(MAX_VELOCITY);
    }
    plane.velocity += GRAVITY * time.delta_seconds();
}

fn update_plane_position(time: Res<Time>, mut query: Query<(&mut Transform, &Plane)>) {
    let (mut plane_transform, plane) = query.single_mut();
    plane_transform.translation.y +=
        plane_transform.scale.y * plane.velocity * time.delta_seconds();
}

fn check_game_over(
    mut commands: Commands,
    mut time: ResMut<Time>,
    mut obstacle_query: Query<(&Transform, &Collidable, &Obstacle)>,
    mut plane_query: Query<(&Transform, &Collidable, With<Plane>)>,
    asset_server: Res<AssetServer>,
) {
    if time.is_paused() {
        return;
    };
    let (plane_transform, plane_collidable, _) = plane_query.single_mut();
    for (obstacle_transform, obstacle_collidable, _) in obstacle_query.iter_mut() {
        let collision = collide(
            plane_transform.translation,
            plane_collidable.size,
            obstacle_transform.translation,
            obstacle_collidable.size,
        );
        if let Some(_) = collision {
            time.pause();
            let font: Handle<Font> = asset_server.load("font.ttf");
            commands.spawn((
                TextBundle::from_section(
                    "Game Over!\nRestart by pressing R".to_string(),
                    TextStyle {
                        font: font,
                        font_size: 48.0,
                        color: Color::BLACK,
                    },
                )
                .with_text_alignment(TextAlignment::Center)
                .with_style(Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    margin: UiRect::all(Val::Auto),
                    ..default()
                }),
                GameOverMessage,
            ));
        }
    }
}

fn update_obstacles(
    mut commands: Commands,
    time: Res<Time>,
    mut obstacle_query: Query<(&mut Transform, Entity, Option<&ObstacleBound>), With<Obstacle>>,
) {
    for obstacle in obstacle_query.iter_mut() {
        let mut obstacle_transform = obstacle.0;
        obstacle_transform.translation.x -= OBSTACLE_SPEED * time.delta_seconds();
        if obstacle_transform.translation.x < -800. {
            match obstacle.2 {
                Some(_) => obstacle_transform.translation.x += 1600.,
                None => commands.entity(obstacle.1).despawn(),
            }
        }
    }
}

fn spawn_obstacles(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    asset_server: Res<AssetServer>,
    mut score_query: Query<(&mut Text, &mut ScoreCounter)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        // Update score
        let (mut text, mut score) = score_query.single_mut();
        score.score += 1;
        text.sections[0].value = format!("Score: {}", score.score.max(0));
        // Spawn obstacle
        let obstacle_texture = asset_server.load("obstacle.png");
        commands.spawn(create_random_obstacle(obstacle_texture.clone()));
        let new_duration = timer.0.duration() - SPAWN_TIMER_DECREMENT;
        timer.0.set_duration(new_duration)
    }
}

fn restart(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut time: ResMut<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut despawn_query: Query<Entity, Or<(With<ObstacleActive>, With<GameOverMessage>)>>,
    mut plane_query: Query<(&mut Transform, &mut Plane)>,
    mut score_query: Query<&mut ScoreCounter>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        time.unpause();
        timer.0.set_duration(STARTING_SPAWN_TIMER_DURATION);
        for entity in despawn_query.iter_mut() {
            commands.entity(entity).despawn_recursive();
        }
        let (mut plane_transform, mut plane) = plane_query.single_mut();
        plane_transform.translation = PLANE_INITIAL_POSITION;
        plane.velocity = 0.;
        score_query.single_mut().score = ScoreCounter::default().score;
    }
}
