use crate::{
    asset_loader::SceneAssets,
    boundary::{
        Boundary,
        WallApproachVisual,
    },
    camera::RenderLayer,
    collider_config::{
        ColliderConfig,
        ColliderConstant,
    },
    health::{
        CollisionDamage,
        Health,
        HealthBundle,
    },
    movement::MovingObjectBundle,
    schedule::InGameSet,
    utils::name_entity,
};
use bevy::{
    prelude::*,
    render::view::RenderLayers,
};
use bevy_rapier3d::prelude::Velocity;
use rand::Rng;

#[derive(Component, Debug)]
pub struct NateroidPlugin;

impl Plugin for NateroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_nateroid.in_set(InGameSet::EntityUpdates));
    }
}

fn spawn_nateroid(
    mut commands: Commands,
    mut collider_config: ResMut<ColliderConfig>,
    time: Res<Time>,
    scene_assets: Res<SceneAssets>,
    boundary: Res<Boundary>,
) {
    if !should_spawn_nateroid(&mut collider_config, time) {
        return;
    }

    let nateroid_config = &collider_config.nateroid;

    let mut rng = rand::thread_rng();

    let boundary_min = boundary.transform.translation - boundary.transform.scale / 2.0;
    let boundary_max = boundary.transform.translation + boundary.transform.scale / 2.0;

    // todo: keep it further inside the boundary
    let spawn_translation = Vec3::new(
        rng.gen_range(boundary_min.x..boundary_max.x),
        rng.gen_range(boundary_min.y..boundary_max.y),
        0.0, //rng.gen_range(boundary_min.z..boundary_max.z),
    );

    let mut transform = Transform::from_translation(spawn_translation);
    // start in a random position
    transform.rotation = ColliderConstant::random_rotation();

    // with random velocity and angular velocity
    let random_velocity = nateroid_config.random_velocity();
    let random_angular_velocity = nateroid_config.random_angular_velocity();

    transform.scale = Vec3::splat(nateroid_config.scalar);

    let nateroid_model = SceneBundle {
        scene: scene_assets.nateroid.clone(),
        transform,
        ..default()
    };

    let collider = nateroid_config.collider.clone();

    let nateroid = commands
        .spawn(NateroidPlugin)
        .insert(HealthBundle {
            collision_damage: CollisionDamage(nateroid_config.damage),
            health:           Health(nateroid_config.health),
        })
        .insert(MovingObjectBundle {
            aabb: nateroid_config.aabb.clone(),
            collider,
            mass: nateroid_config.mass,
            model: nateroid_model,
            velocity: Velocity {
                linvel: random_velocity,
                angvel: random_angular_velocity,
            },
            ..default()
        })
        .insert(RenderLayers::from_layers(RenderLayer::Both.layers()))
        .insert(WallApproachVisual::default())
        .id();

    name_entity(&mut commands, nateroid, nateroid_config.name.to_owned());
}

fn should_spawn_nateroid(collider_config: &mut ResMut<ColliderConfig>, time: Res<Time>) -> bool {
    let nateroid_config = &mut collider_config.nateroid;

    if !nateroid_config.spawnable {
        return false;
    }

    let spawn_timer = nateroid_config.spawn_timer.as_mut().unwrap();
    spawn_timer.tick(time.delta());

    if !spawn_timer.just_finished() {
        return false;
    }

    true
}
