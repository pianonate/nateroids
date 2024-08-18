mod aabb;
mod actor_spawner;
mod actor_template;
pub(crate) mod missile;
mod movement;
mod nateroid;
mod spaceship;
mod teleport;

use crate::actor::{
    aabb::AabbPlugin,
    actor_spawner::ActorSpawner,
    missile::MissilePlugin,
    nateroid::NateroidPlugin,
    spaceship::SpaceshipPlugin,
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
            .add_plugins(MissilePlugin)
            .add_plugins(NateroidPlugin)
            .add_plugins(SpaceshipPlugin)
            .add_plugins(TeleportPlugin);
    }
}
