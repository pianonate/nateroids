use bevy::{
    color::palettes::basic::BLUE,
    color::palettes::css::{GREEN, RED},
    prelude::KeyCode::F8,
    prelude::*,
};
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
const LIMITED_DISTANCE_MOVEMENT_SCALAR: f32 = 0.9;

// #todo: #rustquestion - how can i make it so that new has to be used and DrawDirection isn't constructed directly - i still need the fields visible
#[derive(Copy, Clone, Component, Debug)]
pub struct LimitedDistanceMover {
    pub direction: Vec3,
    pub total_distance: f32,
    pub traveled_distance: f32,
    pub last_position: Vec3,
}

/// take the distance to the nearest edge in front and behind and make that
/// the distance this thing will travel - it's not perfect but it will do
impl LimitedDistanceMover {
    pub fn new(
        origin: Vec3,
        direction: Vec3,
        windows: Query<&Window>,
        camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
    ) -> Self {
        let mut total_distance = 0.0;

        if let Some(dimensions) = calculate_viewable_dimensions(windows, camera_query) {
            if let Some(edge_point) = find_edge_point(origin, direction, dimensions) {
                if let Some(opposite_edge) = find_edge_point(origin, -direction, dimensions) {
                    total_distance =
                        edge_point.distance(opposite_edge) * LIMITED_DISTANCE_MOVEMENT_SCALAR;
                }
            }
        }

        LimitedDistanceMover {
            direction,
            total_distance,
            traveled_distance: 0.0,
            last_position: origin,
        }
    }
}

#[derive(Resource, Debug)]
struct MissileParty {
    enabled: bool,
}

#[derive(Component, Debug, Default)]
pub struct Wrappable {
    pub wrapped: bool,
}

#[derive(Copy, Clone, Debug)]
struct ViewableDimensions {
    pub width: f32,
    pub height: f32,
}

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                teleport_system,
                update_distance_traveled_system,
                missile_party_system,
            )
                .chain()
                .in_set(InGameSet::EntityUpdates),
        )
        .insert_resource(MissileParty { enabled: false });
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

fn update_distance_traveled_system(
    mut query: Query<(&Transform, &mut LimitedDistanceMover, &Wrappable)>,
) {
    for (transform, mut draw_direction, wrappable) in query.iter_mut() {
        let current_position = transform.translation;

        // Calculate the distance traveled since the last update
        let distance_traveled = if wrappable.wrapped {
            0.0 // Reset distance if wrapped
        } else {
            draw_direction.last_position.distance(current_position)
        };

        // Update the total traveled distance
        draw_direction.traveled_distance += distance_traveled;
        draw_direction.last_position = current_position;
    }
}

fn missile_party_system(
    camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
    direction_query: Query<&LimitedDistanceMover>,
    mut gizmos: Gizmos,
    kbd: Res<ButtonInput<KeyCode>>,
    mut missile_party: ResMut<MissileParty>,
    windows: Query<&Window>,
) {
    if kbd.just_pressed(F8) {
        missile_party.enabled = !missile_party.enabled;
        println!("missile party: {}", missile_party.enabled);
    }

    if !missile_party.enabled {
        return;
    }

    if let Some(dimensions) = calculate_viewable_dimensions(windows, camera_query) {
        for limited_distance_mover in direction_query.iter() {
            let origin = limited_distance_mover.last_position;
            let direction = limited_distance_mover.direction;

            // println!("{:?}", draw_direction);

            let (p1, p2) = calculate_perpendicular_points(origin, direction, 100.0);
            gizmos.line_gradient(p1, p2, BLUE, RED);

            if let Some(edge_point) = find_edge_point(origin, direction, dimensions) {
                gizmos
                    .sphere(edge_point, Quat::IDENTITY, 1., Color::WHITE)
                    .resolution(64);

                gizmos.line_gradient(origin, edge_point, GREEN, RED);

                let opposite_edge = calculate_wrapped_position(edge_point, dimensions);

                gizmos
                    .sphere(opposite_edge, Quat::IDENTITY, 1., Color::WHITE)
                    .resolution(64);

                if let Some(next_edge) = find_edge_point(opposite_edge, direction, dimensions) {
                    gizmos.line_gradient(opposite_edge, next_edge, RED, GREEN);
                }
            }
        }
    }
}

/// given a particular camera, what is the viewable/width and height for that camera?
fn calculate_viewable_dimensions(
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

/// given a particular point, what is the point on the opposite side of the screen?
fn calculate_wrapped_position(position: Vec3, dimensions: ViewableDimensions) -> Vec3 {
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

/// only used to help draw some groovy things to highlight what a missile is doing
fn calculate_perpendicular_points(origin: Vec3, direction: Vec3, distance: f32) -> (Vec3, Vec3) {
    // Ensure the direction vector is normalized
    let direction = direction.normalize();

    // Calculate the perpendicular direction in the xz plane
    let perpendicular = Vec3::new(-direction.z, 0.0, direction.x).normalize();

    // Calculate the two points 100.0 units away in the perpendicular direction
    let point1 = origin + perpendicular * distance;
    let point2 = origin - perpendicular * distance;

    (point1, point2)
}

/// Finds the intersection point of a ray (defined by an origin and direction) with the edges of a viewable area.
///
/// # Parameters
/// - `origin`: The starting point of the ray.
/// - `direction`: The direction vector of the ray.
/// - `dimensions`: The dimensions of the viewable area.
///
/// # Returns
/// An `Option<Vec3>` containing the intersection point if found, or `None` if no valid intersection exists.
///
/// # Method
/// - The function calculates the intersection points of the ray with the positive and negative boundaries of the viewable area along both the x and z axes.
/// - It iterates over these axes, updating the minimum intersection distance (`t_min`) if a valid intersection is found.
/// - Finally, it returns the intersection point corresponding to the minimum distance, or `None` if no valid intersection is found.
fn find_edge_point(origin: Vec3, direction: Vec3, dimensions: ViewableDimensions) -> Option<Vec3> {
    let ViewableDimensions { width, height } = dimensions;

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let mut t_min = f32::MAX;

    // learning rust - this for syntax destructures the two provided tuples into the variables
    // so we get a pas through this loop for both x and z - i like rust
    for (start, dir, pos_bound, neg_bound) in [
        (origin.x, direction.x, half_width, -half_width),
        (origin.z, direction.z, half_height, -half_height),
    ] {
        if dir != 0.0 {
            let t_positive = (pos_bound - start) / dir;
            if t_positive > 0.0 && t_positive < t_min {
                t_min = t_positive;
            }
            let t_negative = (neg_bound - start) / dir;
            if t_negative > 0.0 && t_negative < t_min {
                t_min = t_negative;
            }
        }
    }

    if t_min != f32::MAX {
        Some(origin + direction * t_min)
    } else {
        None
    }
}
