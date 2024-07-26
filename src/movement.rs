use bevy::prelude::*;
use bevy_rapier3d::{
    dynamics::{GravityScale, LockedAxes},
    geometry::ActiveEvents,
    prelude::{
        CoefficientCombineRule, Collider, ColliderMassProperties, ColliderMassProperties::Mass,
        CollisionGroups, Restitution, RigidBody, Velocity,
    },
};

use crate::{schedule::InGameSet, window::ViewportDimensions};

const DEFAULT_GRAVITY: f32 = 0.0;
const DEFAULT_MASS: f32 = 1.0;

#[derive(Component, Debug, Default)]
pub struct Wrappable {
    pub wrapped: bool,
}

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, teleport_system.in_set(InGameSet::EntityUpdates));
    }
}

#[derive(Bundle)]
pub struct MovingObjectBundle {
    pub active_events: ActiveEvents,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub gravity_scale: GravityScale,
    pub locked_axes: LockedAxes,
    pub mass: ColliderMassProperties,
    pub model: SceneBundle,
    pub restitution: Restitution,
    pub rigidity: RigidBody,
    pub velocity: Velocity,
    pub wrappable: Wrappable,
}

impl Default for MovingObjectBundle {
    fn default() -> Self {
        Self {
            active_events: ActiveEvents::COLLISION_EVENTS,
            collider: Collider::default(),
            collision_groups: CollisionGroups::default(),
            gravity_scale: GravityScale(DEFAULT_GRAVITY),
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Y,
            mass: Mass(DEFAULT_MASS),
            model: SceneBundle::default(),
            restitution: Restitution {
                coefficient: 1.0,
                combine_rule: CoefficientCombineRule::Max,
            },
            rigidity: RigidBody::Dynamic,
            velocity: Velocity {
                linvel: Vec3::ZERO,
                angvel: Default::default(),
            },
            wrappable: Wrappable::default(),
        }
    }
}

fn teleport_system(
    viewport: Res<ViewportDimensions>,
    mut wrappable_entities: Query<(&mut Transform, &mut Wrappable)>,
) {
    for (mut transform, mut wrappable) in wrappable_entities.iter_mut() {
        let original_position = transform.translation;
        let wrapped_position = calculate_teleport_position(original_position, &viewport);
        if wrapped_position != original_position {
            wrappable.wrapped = true;
            transform.translation = wrapped_position;
        } else {
            wrappable.wrapped = false;
        }
    }
}

/// given a particular point, what is the point on the opposite side of the screen?
pub fn calculate_teleport_position(position: Vec3, dimensions: &Res<ViewportDimensions>) -> Vec3 {
    let width = dimensions.width;
    let height = dimensions.height;

    let screen_right = width / 2.0;
    let screen_left = -screen_right;
    let screen_top = height / 2.0;
    let screen_bottom = -screen_top;

    let mut wrapped_position = position;

    if position.x >= screen_right {
        wrapped_position.x = screen_left;
    } else if position.x <= screen_left {
        wrapped_position.x = screen_right;
    }

    if position.z >= screen_top {
        wrapped_position.z = screen_bottom;
    } else if position.z <= screen_bottom {
        wrapped_position.z = screen_top;
    }

    wrapped_position
}
