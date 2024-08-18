use crate::actor::Aabb;

use crate::actor::Teleporter;
use bevy::prelude::*;
use bevy_rapier3d::{
    dynamics::{
        GravityScale,
        LockedAxes,
    },
    geometry::ActiveEvents,
    prelude::{
        CoefficientCombineRule,
        Collider,
        ColliderMassProperties,
        ColliderMassProperties::Mass,
        CollisionGroups,
        Restitution,
        RigidBody,
        Velocity,
    },
};

const DEFAULT_GRAVITY: f32 = 0.0;
const DEFAULT_MASS: f32 = 1.0;

#[derive(Bundle)]
pub struct MovingObjectBundle {
    pub aabb:             Aabb,
    pub active_events:    ActiveEvents,
    pub collider:         Collider,
    pub collision_groups: CollisionGroups,
    pub gravity_scale:    GravityScale,
    pub locked_axes:      LockedAxes,
    pub mass:             ColliderMassProperties,
    pub model:            SceneBundle,
    pub restitution:      Restitution,
    pub rigidity:         RigidBody,
    pub velocity:         Velocity,
    pub teleporter:       Teleporter,
}

// all of these defaults are necessary - don't get rid of them
// just because you don't see them accessed elsewhere - they're applied
// by,...default() you learned this by looking at active_events and thinking you
// can just get rid of them you definitely need all of these components
impl Default for MovingObjectBundle {
    fn default() -> Self {
        Self {
            aabb:             Aabb::default(),
            active_events:    ActiveEvents::COLLISION_EVENTS,
            collider:         Collider::default(),
            collision_groups: CollisionGroups::default(),
            gravity_scale:    GravityScale(DEFAULT_GRAVITY),
            locked_axes:      LockedAxes::TRANSLATION_LOCKED_Z,
            mass:             Mass(DEFAULT_MASS),
            model:            SceneBundle::default(),
            restitution:      Restitution {
                coefficient:  1.0,
                combine_rule: CoefficientCombineRule::Max,
            },
            rigidity:         RigidBody::Dynamic,
            velocity:         Velocity::default(),
            teleporter:       Teleporter::default(),
        }
    }
}
