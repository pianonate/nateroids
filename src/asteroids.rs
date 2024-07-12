use bevy::prelude::*;
use rand::prelude::*;
use std::ops::Range;
use crate::{
    asset_loader::SceneAssets,
    collision_detection::CollisionDamage,
    collision_detection::Collider,
    health::Health,
    movement::{Acceleration, MoverType, MovingObjectBundle, Velocity},
    schedule::InGameSet
};

const ACCELERATION_SCALAR: f32 = 1.0;
const COLLISION_DAMAGE: f32 = 35.0;
const HEALTH: f32 = 80.0;
const RADIUS: f32 = 2.5;
const ROTATE_SPEED: f32 = 3.5;
const ROTATION_RANDOMIZATION_RANGE: Range<f32> = -3.0..3.0;
const SPAWN_RANGE_X: Range<f32> = -25.0..25.0;
const SPAWN_RANGE_Z: Range<f32> = 0.0..25.0;
const SPAWN_TIMER_SECONDS: f32 = 1.0;
const VELOCITY_SCALAR: f32 = 5.0;

#[derive(Component, Debug)]
pub struct Asteroid;

#[derive(Resource, Debug)]
pub struct AsteroidSpawnTimer {
    pub timer: Timer,
}

pub struct AsteroidsPlugin;

impl Plugin for AsteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AsteroidSpawnTimer {
            timer: Timer::from_seconds(SPAWN_TIMER_SECONDS, TimerMode::Repeating),
        })
        .add_systems(
            Update,
            (spawn_asteroid, rotate_asteroids).in_set(InGameSet::EntityUpdates),
        );
    }
}

fn spawn_asteroid(
    mut commands: Commands,
    mut spawn_timer: ResMut<AsteroidSpawnTimer>,
    time: Res<Time>,
    scene_assets: Res<SceneAssets>,
) {
    spawn_timer.timer.tick(time.delta());

    if !spawn_timer.timer.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    let spawn_translation = Vec3::new(
        rng.gen_range(SPAWN_RANGE_X),
        0.,
        rng.gen_range(SPAWN_RANGE_Z),
    );

    let mut random_unit_vector =
        || Vec3::new(rng.gen_range(-1.0..1.0), 0., rng.gen_range(-1.0..1.0));

    let velocity = random_unit_vector() * VELOCITY_SCALAR;
    let acceleration = random_unit_vector() * ACCELERATION_SCALAR;

    let mut transform = Transform::from_translation(spawn_translation);
    transform.rotate_local_x(rng.gen_range(ROTATION_RANDOMIZATION_RANGE));
    transform.rotate_local_y(rng.gen_range(ROTATION_RANDOMIZATION_RANGE));
    transform.rotate_local_z(rng.gen_range(ROTATION_RANDOMIZATION_RANGE));

    commands.spawn((
        MovingObjectBundle {
            mover_type: MoverType::Asteroid,
            velocity: Velocity::new(velocity),
            acceleration: Acceleration::new(acceleration),
            collider: Collider::new(RADIUS),
            model: SceneBundle {
                scene: scene_assets.asteroid.clone(),
                transform,
                ..default()
            },
        },
        Asteroid,
        Health::new(HEALTH),
        CollisionDamage::new(COLLISION_DAMAGE),
    ));
}

fn rotate_asteroids(mut query: Query<&mut Transform, With<Asteroid>>, time: Res<Time>) {
    let delta = time.delta_seconds();

    for mut transform in query.iter_mut() {
        transform.rotate_local_x(ROTATE_SPEED * delta);
        transform.rotate_local_y(ROTATE_SPEED * delta);
        transform.rotate_local_z(ROTATE_SPEED * delta);
    }
}
