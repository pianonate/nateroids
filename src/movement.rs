use crate::{
    boundary::Boundary,
    collider_config::Aabb,
    schedule::InGameSet,
};

use crate::{
    input::GlobalAction,
    utils::toggle_active,
};
use bevy::{
    color::palettes::tailwind,
    prelude::*,
};
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

#[derive(Component, Debug, Default)]
pub struct Teleporter {
    pub just_teleported:          bool,
    pub last_teleported_position: Option<Vec3>,
    pub last_teleported_normal:   Option<Dir3>,
}

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            teleport_at_boundary.in_set(InGameSet::EntityUpdates),
        );

        // app.add_systems(Update, debug_spaceship);
        app.add_systems(
            Update,
            draw_aabb_system.run_if(toggle_active(false, GlobalAction::AABBs)),
        );
    }
}

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
    pub wrappable:        Teleporter,
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
            wrappable:        Teleporter::default(),
        }
    }
}

fn teleport_at_boundary(
    boundary: Res<Boundary>,
    mut wrappable_entities: Query<(&mut Transform, &mut Teleporter)>,
) {
    for (mut transform, mut wrappable) in wrappable_entities.iter_mut() {
        let original_position = transform.translation;
        let teleported_position =
            calculate_teleport_position(original_position, &boundary.transform);
        if teleported_position != original_position {
            transform.translation = teleported_position;
            wrappable.just_teleported = true;
            wrappable.last_teleported_position = Some(teleported_position);
            wrappable.last_teleported_normal =
                Some(boundary.get_normal_for_position(teleported_position));
        } else {
            wrappable.just_teleported = false;
            wrappable.last_teleported_position = None;
            wrappable.last_teleported_normal = None;
        }
    }
}

/// given a particular point, what is the point on the opposite side of the
/// boundary?
pub fn calculate_teleport_position(position: Vec3, transform: &Transform) -> Vec3 {
    let boundary_min = transform.translation - transform.scale / 2.0;
    let boundary_max = transform.translation + transform.scale / 2.0;

    let mut wrapped_position = position;

    if position.x >= boundary_max.x {
        wrapped_position.x = boundary_min.x;
    } else if position.x <= boundary_min.x {
        wrapped_position.x = boundary_max.x;
    }

    if position.y >= boundary_max.y {
        wrapped_position.y = boundary_min.y;
    } else if position.y <= boundary_min.y {
        wrapped_position.y = boundary_max.y;
    }

    if position.z >= boundary_max.z {
        wrapped_position.z = boundary_min.z;
    } else if position.z <= boundary_min.z {
        wrapped_position.z = boundary_max.z;
    }

    wrapped_position
}

fn draw_aabb_system(mut gizmos: Gizmos, query: Query<(&Transform, &Aabb)>) {
    for (transform, aabb) in query.iter() {
        let center = transform.transform_point(aabb.center());

        // Draw the wireframe cube
        gizmos.cuboid(
            Transform::from_translation(center)
                // .with_scale(aabb.size() * transform.scale)
                .with_scale(aabb.half_extents() * 2.0 * transform.scale)
                .with_rotation(transform.rotation),
            Color::from(tailwind::GREEN_800),
        );
    }
}

// todo: #bevyqestion - attempt to try to draw what rapier is drawing but
// couldn't get       it to draw the same aabb that rapier actually draws - the
// issue is that       for cuboids, rapier is off by some pixels whereas,
// visually, my aabb is perfectly aligned       the question is why
// fn debug_spaceship(
//     query: Query<(Entity, &Transform, &Aabb), With<Spaceship>>,
//     rapier_context: Res<RapierContext>,
//     mut gizmos: Gizmos,
// ) {
//     for (entity, transform, your_aabb) in query.iter() {
//         // Draw your calculated AABB
//         let your_center = transform.transform_point(your_aabb.center());
//         gizmos.cuboid(
//             Transform::from_translation(your_center)
//                 .with_scale(your_aabb.half_extents() * 2.0 * transform.scale)
//                 .with_rotation(transform.rotation),
//             Color::from(tailwind::GREEN_800).with_alpha(0.3),
//         );
//
//         // Get the collider from the entity and draw Rapier's AABB
//         if let Some(collider_handle) =
// rapier_context.entity2collider().get(&entity) {             if let
// Some(collider) = rapier_context.colliders.get(*collider_handle) {
// let rapier_aabb = collider.compute_aabb();
//
//                 // Convert Rapier's AABB to Bevy types
//                 let aabb_half_extents = Vec3::new(
//                     rapier_aabb.half_extents().x,
//                     rapier_aabb.half_extents().y,
//                     rapier_aabb.half_extents().z
//                 );
//
//                 // Apply initial correction to align with your coordinate
// system                 let correction_z =
// Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);                 let
// correction_y = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
//
//                 let rotation =  transform.rotation; // correction_z *
// transform.rotation * correction_y;
//
//                 // Draw Rapier's AABB
//                 gizmos.cuboid(
//                     Transform::from_translation(transform.translation)
//                         .with_rotation(rotation)
//
// .with_scale(Vec3::new(aabb_half_extents.y,aabb_half_extents.z,
// aabb_half_extents.x ) * 2.0 * transform.scale),
// Color::from(tailwind::RED_800).with_alpha(0.3),                 );
//
//                 println!("your_aabb.half_extents() {}, {}, {}, rapier
// half_extents {}, {}, {}", your_aabb.half_extents().x,
// your_aabb.half_extents().y, your_aabb.half_extents().z,
// aabb_half_extents.x, aabb_half_extents.y, aabb_half_extents.z)             }
//         }
//     }
// }
