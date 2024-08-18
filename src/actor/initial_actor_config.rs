use crate::{
    actor::{
        actor_config::{
            ActorConfig,
            ActorType,
            SpawnPositionBehavior,
            VelocityBehavior,
        },
        ColliderType,
    },
    health::{
        CollisionDamage,
        Health,
    },
};
use bevy::prelude::*;
use bevy_rapier3d::dynamics::LockedAxes;

#[derive(Resource, Clone, Debug)]
pub struct InitialEnsembleConfig {
    pub spaceship: ActorConfig,
    pub nateroid:  ActorConfig,
    pub missile:   ActorConfig,
}

// todo: #rustquestion - why isn't rustfmt lining these up? it does if i get rid
// of ..default()...
impl Default for InitialEnsembleConfig {
    fn default() -> Self {
        Self {
            missile:   ActorConfig {
                actor_type: ActorType::Missile,
                collision_damage: CollisionDamage(50.),
                health: Health(1.),
                mass: 0.1,
                spawn_position_behavior: SpawnPositionBehavior::RelativeToParent {
                    offset: Vec3::new(0.5, 0., 0.),
                },
                scalar: 2.5,
                spawn_timer_seconds: Some(1.0 / 20.0),
                velocity_behavior: VelocityBehavior::RelativeToParent {
                    base_velocity:           85.0,
                    inherit_parent_velocity: true,
                },
                ..default()
            },
            nateroid:  ActorConfig {
                actor_type: ActorType::Nateroid,
                collider_type: ColliderType::Cuboid,
                collision_damage: CollisionDamage(10.),
                health: Health(200.),
                mass: 1.0,
                restitution: 0.3,
                // todo: handle3d - right now you're stopping the spawning in the z here
                spawn_position_behavior: SpawnPositionBehavior::RandomWithinBounds {
                    scale_factor: Vec3::new(0.5, 0.5, 0.0), 
                    random_rotation: true,
                },
                velocity_behavior: VelocityBehavior::Random {
                    linvel: 30.0,
                    angvel: 4.0,
                },
                spawn_timer_seconds: Some(2.),
                ..default()
            },
            spaceship: ActorConfig {
                actor_type: ActorType::Spaceship,
                collision_damage: CollisionDamage(50.),
                health: Health(500.),
                mass: 10.0,
                locked_axes: LockedAxes::ROTATION_LOCKED_X
                    | LockedAxes::ROTATION_LOCKED_Y
                    | LockedAxes::TRANSLATION_LOCKED_Z,
                restitution: 0.1,
                scalar: 0.8,
                spawn_position_behavior: SpawnPositionBehavior::Fixed(Vec3::new(0.0, -20.0, 0.0)),
                velocity_behavior: VelocityBehavior::Fixed(Vec3::ZERO),
                ..default()
            },
        }
    }
}
