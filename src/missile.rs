use bevy::{
    color::palettes::basic::{BLUE, GREEN, RED, WHITE},
    prelude::Color::Srgba,
    prelude::*,
};

use bevy_rapier3d::{prelude::ColliderMassProperties::Mass, prelude::*};

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
const MISSILE_FORWARD_SPAWN_SCALAR: f32 = 3.6;
const MISSILE_HEALTH: f32 = 1.0;
const MISSILE_MASS: f32 = 0.001;
const MISSILE_NAME: &str = "Missile";
const MISSILE_SPAWN_TIMER_SECONDS: f32 = 1.0 / 20.0;

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
    direction: Vec3,
    velocity: Vec3,
    pub(crate) total_distance: f32,
    pub(crate) traveled_distance: f32,
    remaining_distance: f32,
    last_position: Vec3,
    last_teleport_position: Option<Vec3>, // Add this field
    edge_in_front_of_spaceship: Vec3,
    teleported_position: Vec3,
}

// rust learnings:
// boundary declared as reference because it is moved into find_edge_point so it would only be able
// to be called once if it wasn't a reference
impl Missile {
    pub fn new(
        spaceship_transform: &Transform,
        spaceship_velocity: &Velocity,
        game_scale: &Res<GameScale>,
        boundary: &Res<Boundary>,
    ) -> Self {
        let forward = -spaceship_transform.forward().with_z(0.0);

        let missile_velocity = forward * game_scale.missile.velocity;

        // clamp it to 2d for now...
        //missile_velocity.z = 0.0;

        // add spaceship velocity so that the missile fires in the direction the spaceship
        // is going - without it, they only have the missile velocity and if the spaceship
        // is moving it will look as if they are trailing off to the left or right
        let velocity = spaceship_velocity.linvel + missile_velocity;

        let direction = forward;
        let origin = spaceship_transform.translation + direction * MISSILE_FORWARD_SPAWN_SCALAR;

        let mut total_distance = 0.0;
        let mut edge_in_front_of_spaceship = origin;
        let mut teleported_position = origin;

        // Change the call to find_edge_point to pass initial_velocity
        // Find the initial edge point where the missile hits the boundary
        if let Some(calculated_edge_point) = boundary.find_edge_point(origin, velocity) {
            edge_in_front_of_spaceship = calculated_edge_point;
            println!("Initial edge point: {:?}", edge_in_front_of_spaceship);

            teleported_position =
                calculate_teleport_position(edge_in_front_of_spaceship, &boundary.transform);
            println!("Teleported position: {:?}", teleported_position);

            total_distance = boundary.longest_diagonal;
        }

        Missile {
            direction,
            velocity,
            total_distance,
            traveled_distance: 0.,
            remaining_distance: 0.,
            last_position: origin,
            last_teleport_position: None,
            edge_in_front_of_spaceship,
            teleported_position,
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
    // boundary is used to set the total distance this missile can travel
    let missile = Missile::new(
        spaceship_transform,
        spaceship_velocity,
        &game_scale,
        &boundary,
    );

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
                transform: Transform::from_translation(missile.last_position)
                    .with_scale(Vec3::splat(game_scale.missile.scalar)),
                ..default()
            },
            velocity: Velocity {
                linvel: missile.velocity,
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
        missile.remaining_distance = missile.total_distance - missile.traveled_distance;
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
    mut q_missile: Query<&mut Missile>,
    mut gizmos: Gizmos,
    boundary: Res<Boundary>,
    game_scale: Res<GameScale>,
) {
    for missile in q_missile.iter_mut() {
        let current_position = missile.last_position;
        let direction = missile.direction;

        draw_missile_perpendicular(
            &mut gizmos,
            &missile,
            current_position,
            direction,
            &game_scale,
        );
        draw_missile_ray(&mut gizmos, &missile, &boundary);
    }
}

fn draw_missile_ray(gizmos: &mut Gizmos, missile: &Missile, boundary: &Res<Boundary>) {
    draw_sphere(gizmos, missile.edge_in_front_of_spaceship, Srgba(BLUE));

    // Draw sphere at the opposite edge point
    draw_sphere(gizmos, missile.teleported_position, Srgba(RED));

    // Draw sphere at the last teleport position if it exists
    if let Some(last_teleport_position) = missile.last_teleport_position {
        if last_teleport_position.distance(missile.teleported_position) > 1. {
            draw_sphere(gizmos, last_teleport_position, Srgba(WHITE));
        }
    }

    let current_position = missile.last_position;

    if let Some(next_boundary) = boundary.find_edge_point(current_position, missile.velocity) {
        if missile.remaining_distance < current_position.distance(next_boundary) {
            let end_point =
                current_position + missile.velocity.normalize() * missile.remaining_distance;
            draw_sphere(gizmos, end_point, Srgba(GREEN));
        }
    }
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
    game_scale: &Res<GameScale>,
) {
    let distance_traveled_ratio = 1.0 - missile.traveled_distance / missile.total_distance;
    let perpendicular_length = game_scale.boundary_cell_scalar * distance_traveled_ratio;

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
