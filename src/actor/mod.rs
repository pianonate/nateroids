mod aabb;
mod actor_spawner;
mod actor_template;
mod collision_detection;
pub(crate) mod health;
pub mod missile;
mod nateroid;
mod spaceship;
mod spaceship_movement;
mod teleport;

use crate::actor::{
    aabb::AabbPlugin,
    actor_spawner::ActorSpawner,
    collision_detection::CollisionDetectionPlugin,
    missile::MissilePlugin,
    nateroid::NateroidPlugin,
    spaceship::SpaceshipPlugin,
    spaceship_movement::SpaceshipMovementPlugin,
    teleport::TeleportPlugin,
};
pub use crate::actor::{
    aabb::{
        get_scene_aabb,
        Aabb,
    },
    actor_spawner::ColliderType,
    teleport::Teleporter,
};

use bevy::prelude::*;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AabbPlugin)
            .add_plugins(ActorSpawner)
            .add_plugins(CollisionDetectionPlugin)
            .add_plugins(MissilePlugin)
            .add_plugins(NateroidPlugin)
            .add_plugins(SpaceshipPlugin)
            .add_plugins(SpaceshipMovementPlugin)
            .add_plugins(TeleportPlugin);
    }
}
