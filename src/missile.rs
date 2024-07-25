use bevy::{
    color::palettes::basic::{BLUE, GREEN, RED},
    input::common_conditions::input_toggle_active,
    prelude::*,
};

use bevy_rapier3d::prelude::{Collider, ColliderMassProperties::Mass, CollisionGroups, Velocity};

use crate::{
    asset_loader::SceneAssets,
    camera::PrimaryCamera,
    collision_detection::{CollisionDamage, GROUP_ASTEROID, GROUP_MISSILE},
    health::Health,
    movement::{
        calculate_viewable_dimensions, calculate_wrapped_position, MovingObjectBundle,
        ViewableDimensions, Wrappable,
    },
    schedule::InGameSet,
    spaceship::{Action, ContinuousFire, Spaceship},
    utils::name_entity,
};

use leafwing_input_manager::prelude::*;

const MISSILE_COLLISION_DAMAGE: f32 = 50.0;
const MISSILE_FORWARD_SPAWN_SCALAR: f32 = 3.5;
const MISSILE_HEALTH: f32 = 1.0;
const MISSILE_MASS: f32 = 0.001;
const MISSILE_MOVEMENT_SCALAR: f32 = 0.9;
const MISSILE_NAME: &str = "Missile";
const MISSILE_RADIUS: f32 = 0.4;
const MISSILE_SPAWN_TIMER_SECONDS: f32 = 1.0 / 20.0;
const MISSILE_SPEED: f32 = 75.0;

#[derive(Component, Debug)]
struct Missile;

#[derive(Resource, Debug)]
struct MissileSpawnTimer {
    pub timer: Timer,
}

// todo: #rustquestion - how can i make it so that new has to be used and DrawDirection isn't constructed directly - i still need the fields visible
#[derive(Copy, Clone, Component, Debug)]
pub struct MissileMovement {
    pub direction: Vec3,
    pub total_distance: f32,
    pub traveled_distance: f32,
    pub last_position: Vec3,
}

/// take the distance to the nearest edge in front and behind and make that
/// the distance this thing will travel - it's not perfect but it will do
impl MissileMovement {
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
                    total_distance = edge_point.distance(opposite_edge) * MISSILE_MOVEMENT_SCALAR;
                }
            }
        }

        MissileMovement {
            direction,
            total_distance,
            traveled_distance: 0.0,
            last_position: origin,
        }
    }
}

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        app.insert_resource(MissileSpawnTimer {
            timer: Timer::from_seconds(MISSILE_SPAWN_TIMER_SECONDS, TimerMode::Repeating),
        })
        .add_systems(Update, fire_missile.in_set(InGameSet::UserInput))
        .add_systems(
            Update,
            (
                update_missile_movement,
                missile_party_system.run_if(input_toggle_active(false, KeyCode::F8)),
            )
                .chain()
                .in_set(InGameSet::EntityUpdates),
        );
    }
}

// todo: #bevyquestion - how could i reduce the number of arguments here?
#[allow(clippy::too_many_arguments)]
fn fire_missile(
    mut commands: Commands,
    q_camera: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
    q_windows: Query<&Window>,
    q_spaceship: Query<(&Transform, Option<&ContinuousFire>), With<Spaceship>>,
    input_map: Query<&ActionState<Action>>,
    scene_assets: Res<SceneAssets>,
    mut spawn_timer: ResMut<MissileSpawnTimer>,
    time: Res<Time>,
) {
    // todo: #rustquestion - is this short circuit idiomatic rust?
    let Ok((transform, continuous_fire)) = q_spaceship.get_single() else {
        return;
    };

    let continuous = match continuous_fire {
        Some(_) => {
            spawn_timer.timer.tick(time.delta());

            if !spawn_timer.timer.just_finished() {
                return;
            }
            true
        }
        None => false,
    };

    let action_state = input_map.single();

    if continuous && action_state.pressed(&Action::Fire)
        || !continuous && action_state.just_pressed(&Action::Fire)
    {
        let mut velocity = -transform.forward() * MISSILE_SPEED;
        velocity.y = 0.0;

        let direction = -transform.forward().as_vec3();
        let origin = transform.translation + direction * MISSILE_FORWARD_SPAWN_SCALAR;
        let limited_distance_mover = MissileMovement::new(origin, direction, q_windows, q_camera);

        let missile = commands
            .spawn(Missile)
            .insert(MovingObjectBundle {
                collider: Collider::ball(MISSILE_RADIUS),
                collision_damage: CollisionDamage::new(MISSILE_COLLISION_DAMAGE),
                collision_groups: CollisionGroups::new(GROUP_MISSILE, GROUP_ASTEROID),
                health: Health::new(MISSILE_HEALTH),
                mass: Mass(MISSILE_MASS),
                model: SceneBundle {
                    scene: scene_assets.missiles.clone(),
                    transform: Transform::from_translation(origin),
                    ..default()
                },
                velocity: Velocity {
                    linvel: velocity,
                    angvel: Default::default(),
                },
                ..default()
            })
            // Create the missile entity with the DrawDirection component
            .insert(limited_distance_mover)
            .id(); // to ensure we store the entity id for subsequent use

        name_entity(&mut commands, missile, MISSILE_NAME);
    }
}

fn update_missile_movement(mut query: Query<(&Transform, &mut MissileMovement, &Wrappable)>) {
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
    direction_query: Query<&MissileMovement>,
    mut gizmos: Gizmos,

    windows: Query<&Window>,
) {
    if let Some(dimensions) = crate::movement::calculate_viewable_dimensions(windows, camera_query)
    {
        for limited_distance_mover in direction_query.iter() {
            let origin = limited_distance_mover.last_position;
            let direction = limited_distance_mover.direction;

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
