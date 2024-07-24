use crate::{
    camera::PrimaryCamera, collision_detection::CollisionDamage, health::Health,
    schedule::InGameSet,
};
use bevy::prelude::*;

use bevy_rapier3d::{
    dynamics::{GravityScale, LockedAxes},
    geometry::ActiveEvents,
    prelude::{
        CoefficientCombineRule, Collider, ColliderMassProperties, ColliderMassProperties::Mass,
        CollisionGroups, Restitution, RigidBody, Velocity,
    },
};

const DEFAULT_COLLISION_DAMAGE: f32 = 100.0;
const DEFAULT_GRAVITY: f32 = 0.0;
const DEFAULT_HEALTH: f32 = 100.0;
const DEFAULT_MASS: f32 = 1.0;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (/*update_velocity, update_position,*/wrap_position) // these should happen in order
                .chain()
                .in_set(InGameSet::EntityUpdates), // use system sets to put this into an enum that controls ordering
        );
    }
}

#[derive(Component, Debug)]
pub struct Wrappable;

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
            wrappable: Wrappable,
        }
    }
}

// needs to move from world to screen
// remember this game is currently operating int he x-z plane and the a camera is above it
// on the y axis
fn wrap_position(
    windows: Query<&Window>,
    camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
    mut wrappable_entities: Query<&mut Transform, With<Wrappable>>,
) {
    if let Ok(window) = windows.get_single() {
        if let Ok((projection, global_transform)) = camera_query.get_single() {
            if let Projection::Perspective(perspective_projection) = projection {
                let screen_width = window.width() as f32;
                let screen_height = window.height() as f32;

                // Calculate the aspect ratio
                let aspect_ratio = screen_width / screen_height;

                // Calculate the viewable width and height at the plane level
                let camera_distance = global_transform.translation().y;
                let viewable_height =
                    2.0 * (perspective_projection.fov / 2.0).tan() * camera_distance;
                let viewable_width = viewable_height * aspect_ratio;

                for mut transform in wrappable_entities.iter_mut() {
                    let x = transform.translation.x;
                    let z = transform.translation.z;

                    let screen_right = viewable_width / 2.0;
                    let screen_left = -screen_right;
                    let screen_top = viewable_height / 2.0;
                    let screen_bottom = -screen_top;

                    if x > screen_right {
                        transform.translation.x = screen_left;
                    } else if x < screen_left {
                        transform.translation.x = screen_right;
                    }

                    if z > screen_top {
                        transform.translation.z = screen_bottom;
                    } else if z < screen_bottom {
                        transform.translation.z = screen_top;
                    }
                }
            } else {
                println!("Projection is not PerspectiveProjection");
            }
        } else {
            println!("Failed to get camera components");
        }
    }
}
