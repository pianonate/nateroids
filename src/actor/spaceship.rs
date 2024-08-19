use crate::{
    actor::{
        actor_spawner::spawn_actor,
        actor_template::SpaceshipConfig,
        spaceship_control::{
            SpaceshipControl,
            SpaceshipControlConfig,
        },
    },
    boundary::Boundary,
    camera::PrimaryCamera,
    orientation::{
        CameraOrientation,
        OrientationType,
    },
    schedule::InGameSet,
    state::GameState,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use leafwing_input_manager::prelude::*;

#[derive(Component, Debug)]
pub struct Spaceship;

#[derive(Component, Default)]
pub struct ContinuousFire;

pub struct SpaceshipPlugin;
impl Plugin for SpaceshipPlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        // we can enter InGame a couple of ways - when we do, spawn a spaceship
        app.add_systems(OnExit(GameState::Splash), spawn_spaceship)
            .add_systems(OnExit(GameState::GameOver), spawn_spaceship)
            .add_systems(
                Update,
                (spaceship_movement_controls, toggle_continuous_fire)
                    .chain()
                    .in_set(InGameSet::UserInput),
            )
            // check if spaceship is destroyed...this will change the GameState
            .add_systems(Update, spaceship_destroyed.in_set(InGameSet::EntityUpdates));
    }
}

// todo: how can i avoid setting this allow - i'm guessing a system param would
// be just as problematic
#[allow(clippy::type_complexity)]
fn toggle_continuous_fire(
    mut commands: Commands,
    q_spaceship: Query<
        (
            Entity,
            &ActionState<SpaceshipControl>,
            Option<&ContinuousFire>,
        ),
        With<Spaceship>,
    >,
) {
    if let Ok((entity, control, continuous)) = q_spaceship.get_single() {
        if control.just_pressed(&SpaceshipControl::ContinuousFire) {
            if continuous.is_some() {
                println!("removing continuous");
                commands.entity(entity).remove::<ContinuousFire>();
            } else {
                println!("adding continuous");
                commands.entity(entity).insert(ContinuousFire);
            }
        }
    }
}

//todo: #bug - you don't need to bring boundary in except for nateroids
// spawning
// - make it optional
fn spawn_spaceship(
    mut commands: Commands,
    spaceship_config: Res<SpaceshipConfig>,
    boundary: Res<Boundary>,
) {
    if !spaceship_config.0.spawnable {
        return;
    }

    let spaceship_input = InputManagerBundle::with_map(SpaceshipControl::generate_input_map());

    spawn_actor(&mut commands, &spaceship_config.0, None, boundary)
        .insert(spaceship_input)
        .insert(Spaceship);
}

fn spaceship_movement_controls(
    mut q_spaceship: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    q_camera: Query<&Transform, (With<PrimaryCamera>, Without<Spaceship>)>,
    q_input_map: Query<&ActionState<SpaceshipControl>>,
    spaceship_config: Res<SpaceshipConfig>,
    movement_config: Res<SpaceshipControlConfig>,
    time: Res<Time>,
    orientation_mode: Res<CameraOrientation>,
) {
    if let Ok(camera_transform) = q_camera.get_single() {
        // we can use this because there is only exactly one spaceship - so we're not
        // looping over the query
        if let Ok((mut spaceship_transform, mut velocity)) = q_spaceship.get_single_mut() {
            // dynamically update from inspector while game is running to change size
            spaceship_transform.scale = Vec3::splat(spaceship_config.0.scalar);

            let controls = q_input_map.single();

            let mut rotation = 0.0;
            let delta_seconds = time.delta_seconds();
            let rotation_speed = movement_config.rotation_speed;

            if controls.pressed(&SpaceshipControl::TurnRight) {
                // right
                velocity.angvel.z = 0.0;
                rotation = rotation_speed * delta_seconds;
            } else if controls.pressed(&SpaceshipControl::TurnLeft) {
                // left
                velocity.angvel.z = 0.0;
                rotation = -rotation_speed * delta_seconds;
            }

            let camera_forward = camera_transform.forward();
            let facing_opposite = camera_forward.dot(Vec3::new(0.0, 0.0, -1.0)) > 0.0;

            if facing_opposite {
                rotation = -rotation;
            }

            // rotate around the z-axis
            spaceship_transform.rotate_z(rotation);

            let max_speed = movement_config.max_speed;
            let accel = movement_config.acceleration;

            if controls.pressed(&SpaceshipControl::Accelerate) {
                apply_acceleration(
                    &mut velocity,
                    -spaceship_transform.forward().as_vec3(),
                    accel,
                    max_speed,
                    delta_seconds,
                    orientation_mode,
                );
            }

            /* let mut roll = 0.0;

               if keyboard_input.pressed(ShiftLeft) {
                roll = -SPACESHIP_ROLL_SPEED * time.delta_seconds();
            } else if keyboard_input.pressed(ControlLeft) {
                roll = SPACESHIP_ROLL_SPEED * time.delta_seconds();
            }*/

            // rotate around the local z-axis
            // the rotation is relative to the current rotation
            // transform.rotate_local_z(roll);
        }
    }
}

fn apply_acceleration(
    velocity: &mut Velocity,
    direction: Vec3,
    acceleration: f32,
    max_speed: f32,
    delta_seconds: f32,
    orientation: Res<CameraOrientation>,
) {
    let proposed_velocity = velocity.linvel + direction * (acceleration * delta_seconds);
    let proposed_speed = proposed_velocity.length();

    // Ensure we're not exceeding max velocity
    if proposed_speed > max_speed {
        velocity.linvel = proposed_velocity.normalize() * max_speed;
    } else {
        velocity.linvel = proposed_velocity;
    }

    //todo: #handl3d
    match orientation.orientation {
        // in 3d we can accelerate in all dirs
        OrientationType::BehindSpaceship3D => (),
        _ => velocity.linvel.z = 0.0, // Force the `z` value of velocity.linvel to be 0
    }
}

// check if spaceship exists or not - query
// if get_single() (there should only be one - returns an error then the
// spaceship doesn't exist
fn spaceship_destroyed(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<Spaceship>>,
    state: Res<State<GameState>>,
) {
    if query.get_single().is_err() {
        println!(
            "spaceship destroyed: {:?}, count {:?}",
            state,
            query.iter().count()
        );
        next_state.set(GameState::GameOver);
    }
}
