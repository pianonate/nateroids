use crate::{
    asset_loader::SceneAssets,
    boundary::Boundary,
    config::{ColliderConfig, RenderLayer},
    health::{CollisionDamage, Health, HealthBundle},
    movement::MovingObjectBundle,
    schedule::InGameSet,
    utils::name_entity,
};
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier3d::prelude::{Collider, Velocity};
use rand::Rng;
use std::{f32::consts::PI, ops::Range};

#[derive(Resource, Debug)]
pub struct NateroidSpawnTimer {
    pub timer: Timer,
}

const ANGULAR_VELOCITY_RANGE: Range<f32> = -4.0..4.0;
const ROTATION_RANGE: Range<f32> = 0.0..2.0 * PI;

#[derive(Component, Debug)]
pub struct Nateroid;

impl Plugin for Nateroid {
    fn build(&self, app: &mut App) {
        let spawn_timer_seconds = ColliderConfig::default()
            .nateroid
            .spawn_timer_seconds
            .expect("you haven't broken the nateroid spawn timer seconds have you?");
        app.insert_resource(NateroidSpawnTimer {
            timer: Timer::from_seconds(spawn_timer_seconds, TimerMode::Repeating),
        })
        .add_systems(Update, spawn_nateroid.in_set(InGameSet::EntityUpdates));
    }
}

fn spawn_nateroid(
    mut commands: Commands,
    collider_config: Res<ColliderConfig>,
    mut spawn_timer: ResMut<NateroidSpawnTimer>,
    time: Res<Time>,
    scene_assets: Res<SceneAssets>,
    boundary: Res<Boundary>,
) {
    if !collider_config.nateroid.spawnable {
        return;
    }

    spawn_timer.timer.tick(time.delta());

    if !spawn_timer.timer.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    let boundary_min = boundary.transform.translation - boundary.transform.scale / 2.0;
    let boundary_max = boundary.transform.translation + boundary.transform.scale / 2.0;

    // todo: keep it further inside the boundary
    let spawn_translation = Vec3::new(
        rng.gen_range(boundary_min.x..boundary_max.x),
        rng.gen_range(boundary_min.y..boundary_max.y),
        0.0,
        //rng.gen_range(boundary_min.z..boundary_max.z),
    );

    let velocity = collider_config.nateroid.velocity;
    let random_velocity = random_vec3(-velocity..velocity, -velocity..velocity, 0.0..0.0);
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

    transform.scale = Vec3::splat(collider_config.nateroid.scalar);

    let nateroid_model = SceneBundle {
        scene: scene_assets.nateroid.clone(),
        transform,
        ..default()
    };

    let nateroid = commands
        .spawn(Nateroid)
        .insert(HealthBundle {
            collision_damage: CollisionDamage(collider_config.nateroid.damage),
            health: Health(collider_config.nateroid.health),
        })
        .insert(MovingObjectBundle {
            collider: Collider::ball(collider_config.nateroid.radius),
            model: nateroid_model,
            velocity: Velocity {
                linvel: random_velocity,
                angvel: random_angular_velocity,
            },
            ..default()
        })
        .insert(RenderLayers::from_layers(RenderLayer::Both.layers()))
        .id();

    name_entity(&mut commands, nateroid, collider_config.nateroid.name);
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
