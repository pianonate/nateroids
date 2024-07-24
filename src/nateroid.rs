use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, Velocity};
use rand::Rng;
use std::f32::consts::PI;
use std::ops::Range;

use crate::movement::MovingObjectBundle;

use crate::{
    asset_loader::SceneAssets, collision_detection::CollisionDamage, health::Health,
    schedule::InGameSet, utils::name_entity,
};

#[derive(Resource, Debug)]
pub struct NateroidSpawnTimer {
    pub timer: Timer,
}

const ANGULAR_VELOCITY_RANGE: Range<f32> = -4.0..4.0;
const NATEROID_COLLISION_DAMAGE: f32 = 10.0;
const NATEROID_HEALTH: f32 = 50.0;
const NATEROID_RADIUS: f32 = 1.8;
const ROTATION_RANGE: Range<f32> = 0.0..2.0 * PI;
const SPAWN_RANGE_X: Range<f32> = -25.0..25.0;
const SPAWN_RANGE_Z: Range<f32> = 0.0..25.0;
const SPAWN_TIMER_SECONDS: f32 = 1.0;
const VELOCITY_RANGE: Range<f32> = -20.0..20.0;

#[derive(Component, Debug)]
pub struct Nateroid;

impl Plugin for Nateroid {
    fn build(&self, app: &mut App) {
        app.insert_resource(NateroidSpawnTimer {
            timer: Timer::from_seconds(SPAWN_TIMER_SECONDS, TimerMode::Repeating),
        })
        .add_systems(Update, spawn_nateroid.in_set(InGameSet::EntityUpdates));
    }
}

fn spawn_nateroid(
    mut commands: Commands,
    mut spawn_timer: ResMut<NateroidSpawnTimer>,
    time: Res<Time>,
    scene_assets: Res<SceneAssets>,
) {
    spawn_timer.timer.tick(time.delta());

    if !spawn_timer.timer.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    let spawn_translation = random_vec3(SPAWN_RANGE_X, 0.0..0.0, SPAWN_RANGE_Z);
    let random_velocity = random_vec3(VELOCITY_RANGE, 0.0..0.0, VELOCITY_RANGE);
    let random_angular_velocity = random_vec3(
        ANGULAR_VELOCITY_RANGE,
        ANGULAR_VELOCITY_RANGE,
        ANGULAR_VELOCITY_RANGE,
    );

    let mut transform = Transform::from_translation(spawn_translation);

    // start in a random position
    transform.rotate_local_x(rng.gen_range(ROTATION_RANGE));
    transform.rotate_local_y(rng.gen_range(ROTATION_RANGE));
    transform.rotate_local_z(rng.gen_range(ROTATION_RANGE));

    let nateroid = commands
        .spawn(Nateroid)
        .insert(MovingObjectBundle {
            collider: Collider::ball(NATEROID_RADIUS),
            collision_damage: CollisionDamage::new(NATEROID_COLLISION_DAMAGE),
            health: Health::new(NATEROID_HEALTH),
            model: SceneBundle {
                scene: scene_assets.nateroid.clone(),
                transform,
                ..default()
            },
            velocity: Velocity {
                linvel: random_velocity,
                angvel: random_angular_velocity,
            },
            ..default()
        })
        .id();

    name_entity(&mut commands, nateroid, "Nateroid");
}

fn random_vec3(range_x: Range<f32>, range_y: Range<f32>, range_z: Range<f32>) -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = if range_x.start < range_x.end {
        rng.gen_range(range_x)
    } else {
        0.0
    };
    let y = if range_y.start < range_y.end {
        rng.gen_range(range_y)
    } else {
        0.0
    };
    let z = if range_z.start < range_z.end {
        rng.gen_range(range_z)
    } else {
        0.0
    };

    Vec3::new(x, y, z)
}