use bevy::{
    color::palettes::basic::{BLUE, GREEN, RED, WHITE},
    prelude::Color::Srgba,
    prelude::*,
};

use bevy_rapier3d::prelude::ColliderMassProperties::Mass;
use bevy_rapier3d::prelude::*;

use crate::{
    asset_loader::SceneAssets,
    collision_detection::{GROUP_ASTEROID, GROUP_MISSILE},
    health::{CollisionDamage, Health, HealthBundle},
    input::SpaceshipAction,
    movement::{calculate_teleport_position, MovingObjectBundle, Wrappable},
    schedule::InGameSet,
    spaceship::{ContinuousFire, Spaceship},
    utils::name_entity,
};

use crate::boundary::Boundary;
use crate::game_scale::GameScale;
use leafwing_input_manager::prelude::*;

pub struct MissilePlugin;

const MISSILE_COLLISION_DAMAGE: f32 = 50.0;
const MISSILE_FORWARD_SPAWN_SCALAR: f32 = 3.5;
const MISSILE_HEALTH: f32 = 1.0;
const MISSILE_MASS: f32 = 0.001;
const MISSILE_MOVEMENT_SCALAR: f32 = 0.9;
const MISSILE_NAME: &str = "Missile";
const MISSILE_SPAWN_TIMER_SECONDS: f32 = 1.0 / 20.0;
const MISSILE_PERPENDICULAR_LENGTH: f32 = 10.0;

impl Plugin for MissilePlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        app.insert_resource(MissileSpawnTimer {
            timer: Timer::from_seconds(MISSILE_SPAWN_TIMER_SECONDS, TimerMode::Repeating),
        })
        .add_systems(Startup, config_gizmo_line_width)
        .add_systems(Update, fire_missile.in_set(InGameSet::UserInput))
        .add_systems(
            Update,
            (
                missile_movement,
                // toggles the MissilePartyEnabled if the MissileParty spaceship action is pressed
                toggle_missile_party,
                // allows missile party to run only if the MissilePartyEnabled resource is true
                missile_party.run_if(|enabled: Res<MissilePartyEnabled>| enabled.0),
            )
                .chain()
                .in_set(InGameSet::EntityUpdates),
        )
        .insert_resource(MissilePartyEnabled(false));
    }
}

#[derive(Resource, Debug)]
struct MissileSpawnTimer {
    pub timer: Timer,
}

// todo: #rustquestion - how can i make it so that new has to be used and DrawDirection isn't constructed directly - i still need the fields visible
#[derive(Copy, Clone, Component, Debug)]
pub struct Missile {
    pub direction: Vec3,
    pub total_distance: f32,
    pub traveled_distance: f32,
    pub last_position: Vec3,
    pub last_teleport_position: Option<Vec3>, // Add this field
}

// rust learnings:
// boundary declared as reference because it is moved into find_edge_point so it would only be able
// to be called once if it wasn't a reference
impl Missile {
    pub fn new(origin: Vec3, direction: Vec3, boundary: &Res<Boundary>) -> Self {
        let mut total_distance = 0.0;

        if let Some(edge_point) = find_edge_point(origin, direction, boundary) {
            if let Some(opposite_edge) = find_edge_point(origin, -direction, boundary) {
                total_distance = edge_point.distance(opposite_edge) * MISSILE_MOVEMENT_SCALAR;
            }
        }

        Missile {
            direction,
            total_distance,
            traveled_distance: 0.0,
            last_position: origin,
            last_teleport_position: None,
        }
    }
}

/// Logic to handle whether we're in continuous fire mode or just regular fire mode
/// if continuous we want to make sure that enough time has passed and that we're
/// holding down the fire button
fn should_fire(
    continuous_fire: Option<&ContinuousFire>,
    mut spawn_timer: ResMut<MissileSpawnTimer>,
    time: Res<Time>,
    q_input_map: Query<&ActionState<SpaceshipAction>>,
) -> bool {
    let action_state = q_input_map.single();

    if continuous_fire.is_some() {
        spawn_timer.timer.tick(time.delta());
        if !spawn_timer.timer.just_finished() {
            return false;
        }
        action_state.pressed(&SpaceshipAction::Fire)
    } else {
        action_state.just_pressed(&SpaceshipAction::Fire)
    }
}

// todo: #bevyquestion - how could i reduce the number of arguments here?
// todo: #bevyquestion - in an object oriented world i think of attaching fire as a method to
//                       the spaceship - but there's a lot of missile logic so i have it setup in missile
//                       so should i have a simple fire method in method in spaceship that in turn calls this
//                       fn or is having it here fine?
#[allow(clippy::too_many_arguments)]
fn fire_missile(
    mut commands: Commands,
    game_scale: Res<GameScale>,
    spawn_timer: ResMut<MissileSpawnTimer>,
    q_input_map: Query<&ActionState<SpaceshipAction>>,
    q_spaceship: Query<(&Transform, &Velocity, Option<&ContinuousFire>), With<Spaceship>>,
    scene_assets: Res<SceneAssets>,
    time: Res<Time>,
    boundary: Res<Boundary>,
) {
    if !game_scale.missile.spawnable {
        return;
    }

    let Ok((spaceship_transform, spaceship_velocity, continuous_fire)) = q_spaceship.get_single()
    else {
        return;
    };

    if !should_fire(continuous_fire, spawn_timer, time, q_input_map) {
        return;
    }

    // extracted for readability
    spawn_missile(
        &mut commands,
        game_scale,
        scene_assets,
        spaceship_transform,
        spaceship_velocity,
        boundary,
    );
}

fn spawn_missile(
    commands: &mut Commands,
    game_scale: Res<GameScale>,
    scene_assets: Res<SceneAssets>,
    spaceship_transform: &Transform,
    spaceship_velocity: &Velocity,
    boundary: Res<Boundary>,
) {
    let forward = -spaceship_transform.forward();

    let mut missile_velocity = forward * game_scale.missile.velocity;

    // clamp it to 2d for now...
    missile_velocity.z = 0.0;

    // add these so that the missile fires in the direction the spaceship
    // when it is going in a direction but it has turned
    // without this it looks as if the missiles are trailing off to the
    // left or to the right from where the spaceship is currently pointing
    let initial_velocity = spaceship_velocity.linvel + missile_velocity;

    let direction = forward.as_vec3();
    let origin = spaceship_transform.translation + direction * MISSILE_FORWARD_SPAWN_SCALAR;
    // boundary is used to set the total distance this missile can travel
    let missile = Missile::new(origin, direction, &boundary);

    let missile = commands
        .spawn(missile)
        .insert(HealthBundle {
            collision_damage: CollisionDamage(MISSILE_COLLISION_DAMAGE),
            health: Health(MISSILE_HEALTH),
        })
        .insert(MovingObjectBundle {
            collider: Collider::ball(game_scale.missile.radius),
            collision_groups: CollisionGroups::new(GROUP_MISSILE, GROUP_ASTEROID),
            mass: Mass(MISSILE_MASS),
            model: SceneBundle {
                scene: scene_assets.missiles.clone(),
                transform: Transform::from_translation(origin)
                    .with_scale(Vec3::splat(game_scale.missile.scalar)),
                ..default()
            },
            velocity: Velocity {
                linvel: initial_velocity,
                angvel: Default::default(),
            },
            ..default()
        })
        .id();

    name_entity(commands, missile, MISSILE_NAME);
}

/// we update missile movement so that it can be despawned after it has traveled its total distance
fn missile_movement(mut query: Query<(&Transform, &mut Missile, &Wrappable)>) {
    for (transform, mut missile, wrappable) in query.iter_mut() {
        let current_position = transform.translation;

        // Calculate the distance traveled since the last update
        // we use wrapped as a sentinel so that we don't consider
        // the teleport of the missile at the edge of the screen to have
        // used up any distance
        let distance_traveled = if wrappable.wrapped {
            0.0
        } else {
            missile.last_position.distance(current_position)
        };

        // Update the total traveled distance
        missile.traveled_distance += distance_traveled;
        missile.last_position = current_position;

        // Update the last teleport position if the missile wrapped
        if wrappable.wrapped {
            missile.last_teleport_position = Some(missile.last_position);
        }
    }
}

#[derive(Resource, Default)]
struct MissilePartyEnabled(bool);

fn toggle_missile_party(
    q_input_map: Query<&ActionState<SpaceshipAction>, With<Spaceship>>,
    mut missile_party_enabled: ResMut<MissilePartyEnabled>,
) {
    if let Ok(spaceship_action) = q_input_map.get_single() {
        if spaceship_action.just_pressed(&SpaceshipAction::MissileParty) {
            missile_party_enabled.0 = !missile_party_enabled.0;
        }
    }
}

fn config_gizmo_line_width(mut config_store: ResMut<GizmoConfigStore>) {
    for (_, config, _) in config_store.iter_mut() {
        // change default from 2.
        config.line_width = 1.;
    }
}

/// fun! with missiles!
fn missile_party(
    q_missile: Query<&Missile>,
    mut gizmos: Gizmos,
    //viewport: Res<ViewportWorldDimensions>,
    boundary: Res<Boundary>,
) {
    for missile in q_missile.iter() {
        let current_position = missile.last_position;
        let direction = missile.direction;

        draw_missile_perpendicular(&mut gizmos, missile, current_position, direction);
        //draw_missile_ray(&mut gizmos, &viewport, missile, current_position, direction);
        draw_missile_ray(&mut gizmos, &boundary, missile, current_position, direction);
    }
}

fn draw_missile_ray(
    gizmos: &mut Gizmos,
    //viewport: &Res<ViewportWorldDimensions>,
    boundary: &Res<Boundary>,
    missile: &Missile,
    current_position: Vec3,
    direction: Vec3,
) {
    let remaining_distance = missile.total_distance - missile.traveled_distance;

    //if let Some(edge_point) = find_edge_point(current_position, direction, viewport) {
    if let Some(edge_point) = find_edge_point(current_position, direction, boundary) {
        let distance_to_edge = current_position.distance(edge_point);

        if remaining_distance > distance_to_edge {
            // Draw line to the edge point and a sphere at the edge point
            draw_line(gizmos, current_position, edge_point);
            draw_sphere(gizmos, edge_point, Srgba(BLUE));

            // Calculate the opposite edge point
            //let opposite_edge = calculate_teleport_position(edge_point, viewport);
            let opposite_edge = calculate_teleport_position(edge_point, &boundary.transform);

            draw_sphere(gizmos, opposite_edge, Srgba(RED));

            // Draw a sphere at the last teleport position if it exists
            if let Some(last_teleport_position) = missile.last_teleport_position {
                draw_sphere(gizmos, last_teleport_position, Srgba(WHITE));
            }
        } else {
            // Draw the final segment of the line
            let final_point = current_position + direction * remaining_distance;
            draw_line(gizmos, current_position, final_point);

            // Draw a sphere at the final point
            draw_sphere(gizmos, final_point, Srgba(GREEN));
        }
    }
}

fn draw_line(gizmos: &mut Gizmos, current_position: Vec3, final_point: Vec3) {
    gizmos.line_gradient(current_position, final_point, BLUE, RED);
}

fn draw_sphere(gizmos: &mut Gizmos, position: Vec3, color: Color) {
    gizmos
        .sphere(position, Quat::IDENTITY, 1., color)
        .resolution(16);
}

fn draw_missile_perpendicular(
    gizmos: &mut Gizmos,
    missile: &Missile,
    current_position: Vec3,
    direction: Vec3,
) {
    let distance_traveled_ratio = 1.0 - missile.traveled_distance / missile.total_distance;
    let perpendicular_length = MISSILE_PERPENDICULAR_LENGTH * distance_traveled_ratio;

    let (p1, p2) =
        calculate_perpendicular_points(current_position, direction, perpendicular_length);
    gizmos.line_gradient(p1, p2, BLUE, RED);
}

/// only used to help draw some groovy things to highlight what a missile is doing
fn calculate_perpendicular_points(origin: Vec3, direction: Vec3, distance: f32) -> (Vec3, Vec3) {
    // Ensure the direction vector is normalized
    let direction = direction.normalize();

    // Calculate the perpendicular direction in the xz plane
    let perpendicular = Vec3::new(-direction.y, direction.x, 0.0).normalize();

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
fn find_edge_point(origin: Vec3, direction: Vec3, boundary: &Res<Boundary>) -> Option<Vec3> {
    let boundary_min = boundary.transform.translation - boundary.transform.scale / 2.0;
    let boundary_max = boundary.transform.translation + boundary.transform.scale / 2.0;

    let mut t_min = f32::MAX;

    for (start, dir, pos_bound, neg_bound) in [
        (origin.x, direction.x, boundary_max.x, boundary_min.x),
        (origin.y, direction.y, boundary_max.y, boundary_min.y),
        (origin.z, direction.z, boundary_max.z, boundary_min.z),
    ] {
        if dir != 0.0 {
            let t_positive = (pos_bound - start) / dir;
            let point_positive = origin + direction * t_positive;
            if t_positive > 0.0
                && t_positive < t_min
                && is_in_bounds(point_positive, start, origin, boundary_min, boundary_max)
            {
                t_min = t_positive;
            }

            let t_negative = (neg_bound - start) / dir;
            let point_negative = origin + direction * t_negative;
            if t_negative > 0.0
                && t_negative < t_min
                && is_in_bounds(point_negative, start, origin, boundary_min, boundary_max)
            {
                t_min = t_negative;
            }
        }
    }

    if t_min != f32::MAX {
        let edge_point = origin + direction * t_min;
        return Some(edge_point);
    }
    None
}

fn is_in_bounds(
    point: Vec3,
    start: f32,
    origin: Vec3,
    boundary_min: Vec3,
    boundary_max: Vec3,
) -> bool {
    if start == origin.x {
        point.y >= boundary_min.y
            && point.y <= boundary_max.y
            && point.z >= boundary_min.z
            && point.z <= boundary_max.z
    } else if start == origin.y {
        point.x >= boundary_min.x
            && point.x <= boundary_max.x
            && point.z >= boundary_min.z
            && point.z <= boundary_max.z
    } else {
        point.x >= boundary_min.x
            && point.x <= boundary_max.x
            && point.y >= boundary_min.y
            && point.y <= boundary_max.y
    }
}
