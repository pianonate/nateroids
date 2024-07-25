use bevy::{
    color::palettes::basic::BLUE,
    color::palettes::css::{GREEN, RED},
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
#[derive(Component, Debug)]
pub struct LimitedDistanceMover {
    pub direction: Vec3,
    pub total_distance: f32,
    pub traveled_distance: f32,
    pub last_position: Vec3,
}

/// take the distance to the nearest edge in front and behind and make that
/// the distance this thing will travel - it's no perfect but it will do
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

                println!(
                    "dimensions:{:?} total_distance: {:?}",
                    dimensions, total_distance
                );
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

#[derive(Component, Debug)]
pub struct Wrappable {
    pub wrapped: bool,
}

impl Default for Wrappable {
    fn default() -> Self {
        Wrappable { wrapped: false }
    }
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
                draw_limited_distance_movers_system,
            )
                .chain()
                .in_set(InGameSet::EntityUpdates),
        );
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
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut LimitedDistanceMover, &Wrappable)>,
) {
    for (entity, transform, mut draw_direction, wrappable) in query.iter_mut() {
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

fn draw_limited_distance_movers_system(
    windows: Query<&Window>,
    camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
    direction_query: Query<(&Transform, &LimitedDistanceMover)>,
    mut gizmos: Gizmos,
) {
    if let Some(dimensions) = calculate_viewable_dimensions(windows, camera_query) {
        for (transform, limited_distance_mover) in direction_query.iter() {
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

                return Some(ViewableDimensions {
                    width: viewable_width,
                    height: viewable_height,
                });
            }
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

fn find_edge_point(origin: Vec3, direction: Vec3, dimensions: ViewableDimensions) -> Option<Vec3> {
    let ViewableDimensions { width, height } = dimensions;

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let x_intersection = calculate_intersection(origin.x, direction.x, half_width, -half_width);
    let z_intersection = calculate_intersection(origin.z, direction.z, half_height, -half_height);

    let mut t_min = f32::MAX;
    if let Some(t) = x_intersection {
        t_min = t_min.min(t);
    }
    if let Some(t) = z_intersection {
        t_min = t_min.min(t);
    }

    if t_min != f32::MAX {
        return Some(origin + direction * t_min);
    }
    None
}

/// used to find the nearest point on the window from the start point
///
///	1.	Initialization of closest_intersection: By starting with f32::MAX, the code ensures that any valid intersection calculated will be smaller than this initial value. This acts as a way to guarantee that the first valid intersection found will update closest_intersection.
///	2.	Boundary checks: The code checks two potential intersection points (t_positive and t_negative). It calculates where the ray (or line) starting at start and moving in direction intersects with the positive_boundary and negative_boundary.
///	3.	Valid intersection conditions:
///	•	t_positive > 0.0 && t_positive < closest_intersection: This checks if the intersection with the positive boundary is in the correct direction (i.e., not behind the start point) and closer than any previously found intersection.
///	•	t_negative > 0.0 && t_negative < closest_intersection: Similarly, this checks the intersection with the negative boundary under the same conditions.
///	4.	Final check: After checking both boundaries, if no valid intersection was found (i.e., closest_intersection remains f32::MAX), the function returns None. Otherwise, it returns the closest valid intersection.
///
fn calculate_intersection(
    start: f32,
    direction: f32,
    positive_boundary: f32,
    negative_boundary: f32,
) -> Option<f32> {
    // start at infinity
    let mut closest_intersection = f32::MAX;
    if direction != 0.0 {
        let t_positive = (positive_boundary - start) / direction;
        if t_positive > 0.0 && t_positive < closest_intersection {
            closest_intersection = t_positive;
        }
        let t_negative = (negative_boundary - start) / direction;
        if t_negative > 0.0 && t_negative < closest_intersection {
            closest_intersection = t_negative;
        }
    }
    if closest_intersection == f32::MAX {
        None
    } else {
        Some(closest_intersection)
    }
}
