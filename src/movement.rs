use bevy::prelude::*;
use bevy_rapier3d::{
    dynamics::{GravityScale, LockedAxes},
    geometry::ActiveEvents,
    prelude::{
        CoefficientCombineRule, Collider, ColliderMassProperties, ColliderMassProperties::Mass,
        CollisionGroups, Restitution, RigidBody, Velocity,
    },
};

use crate::{
    camera::PrimaryCamera, collision_detection::CollisionDamage, health::Health,
    schedule::InGameSet,
};

const DEFAULT_COLLISION_DAMAGE: f32 = 100.0;
const DEFAULT_GRAVITY: f32 = 0.0;
const DEFAULT_HEALTH: f32 = 100.0;
const DEFAULT_MASS: f32 = 1.0;

#[derive(Component, Debug, Default)]
pub struct Wrappable {
    pub wrapped: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct ViewableDimensions {
    pub width: f32,
    pub height: f32,
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
    pub collision_damage: CollisionDamage,
    pub collision_groups: CollisionGroups,
    pub gravity_scale: GravityScale,
    pub health: Health,
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
            collision_damage: CollisionDamage::new(DEFAULT_COLLISION_DAMAGE),
            collision_groups: CollisionGroups::default(),
            gravity_scale: GravityScale(DEFAULT_GRAVITY),
            health: Health::new(DEFAULT_HEALTH),
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
    windows: Query<&Window>,
    camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
    mut wrappable_entities: Query<(&mut Transform, &mut Wrappable)>,
) {
    if let Some(dimensions) = calculate_viewable_dimensions(windows, camera_query) {
        for (mut transform, mut wrappable) in wrappable_entities.iter_mut() {
            let original_position = transform.translation;
            let wrapped_position = calculate_wrapped_position(original_position, dimensions);
            if wrapped_position != original_position {
                wrappable.wrapped = true;
                transform.translation = wrapped_position;
            } else {
                wrappable.wrapped = false;
            }
        }
    }
}

/// given a particular point, what is the point on the opposite side of the screen?
pub fn calculate_wrapped_position(position: Vec3, dimensions: ViewableDimensions) -> Vec3 {
    let ViewableDimensions { width, height } = dimensions;

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

/// given a particular camera, what is the viewable/width and height for that camera?
pub fn calculate_viewable_dimensions(
    windows: Query<&Window>,
    camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
) -> Option<ViewableDimensions> {
    if let Ok(window) = windows.get_single() {
        let screen_width = window.width();
        let screen_height = window.height();
        // Calculate the aspect ratio
        let aspect_ratio = screen_width / screen_height;

        if let Ok((Projection::Perspective(perspective_projection), global_transform)) =
            camera_query.get_single()
        {
            // Calculate the viewable width and height at the plane level
            let camera_distance = global_transform.translation().y;
            let viewable_height = 2.0 * (perspective_projection.fov / 2.0).tan() * camera_distance;
            let viewable_width = viewable_height * aspect_ratio;

            return Some(ViewableDimensions {
                width: viewable_width,
                height: viewable_height,
            });
        }
    }
    None
}
