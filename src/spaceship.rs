use crate::{
    asset_loader::SceneAssets,
    camera::{
        PrimaryCamera,
        RenderLayer,
    },
    collider_config::ColliderConfig,
    collision_detection::{
        GROUP_ASTEROID,
        GROUP_SPACESHIP,
    },
    health::{
        CollisionDamage,
        Health,
        HealthBundle,
    },
    input::SpaceshipAction,
    movement::MovingObjectBundle,
    schedule::InGameSet,
    state::GameState,
    utils::name_entity,
};
use bevy::{
    prelude::*,
    render::view::visibility::RenderLayers,
};
use bevy_rapier3d::prelude::{
    CollisionGroups,
    LockedAxes,
    Velocity,
};

use crate::{
    boundary::WallApproachVisual,
    orientation::{
        CameraOrientation,
        OrientationType,
    },
};
use leafwing_input_manager::prelude::*;

#[derive(Component, Debug)]
pub struct Spaceship;

#[derive(Component, Debug)]
pub struct SpaceshipShield;

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
                (
                    spaceship_movement_controls,
                    spaceship_shield_controls,
                    toggle_continuous_fire,
                )
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
            &ActionState<SpaceshipAction>,
            Option<&ContinuousFire>,
        ),
        With<Spaceship>,
    >,
) {
    if let Ok((entity, spaceship_action, continuous)) = q_spaceship.get_single() {
        if spaceship_action.just_pressed(&SpaceshipAction::ContinuousFire) {
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

fn spawn_spaceship(
    mut commands: Commands,
    collider_config: Res<ColliderConfig>,
    scene_assets: Res<SceneAssets>,
) {
    if !collider_config.spaceship.spawnable {
        return;
    }

    let spaceship_config = &collider_config.spaceship;
    let spaceship_input = InputManagerBundle::with_map(SpaceshipAction::spaceship_input_map());

    let collider = spaceship_config.collider.clone();

    let spaceship = commands
        .spawn(Spaceship)
        .insert(RenderLayers::from_layers(RenderLayer::Game.layers()))
        .insert(HealthBundle {
            collision_damage: CollisionDamage(spaceship_config.damage),
            health:           Health(spaceship_config.health),
        })
        .insert(MovingObjectBundle {
            aabb: spaceship_config.aabb.clone(),
            collider,
            collision_groups: CollisionGroups::new(GROUP_SPACESHIP, GROUP_ASTEROID),
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Z
                | LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Y,
            mass: spaceship_config.mass,
            model: SceneBundle {
                scene: scene_assets.spaceship.clone(),
                transform: Transform {
                    translation: spaceship_config.spawn_point,
                    scale:       Vec3::splat(spaceship_config.scalar),
                    rotation:    Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                },
                ..default()
            },
            restitution: spaceship_config.restitution,
            ..default()
        })
        .insert(spaceship_input)
        .insert(WallApproachVisual::default())
        .id();

    name_entity(
        &mut commands,
        spaceship,
        collider_config.spaceship.name.to_owned(),
    );
}

fn spaceship_movement_controls(
    mut q_spaceship: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    q_camera: Query<&Transform, (With<PrimaryCamera>, Without<Spaceship>)>,
    q_input_map: Query<&ActionState<SpaceshipAction>>,
    collider_config: Res<ColliderConfig>,
    time: Res<Time>,
    orientation_mode: Res<CameraOrientation>,
) {
    if let Ok(camera_transform) = q_camera.get_single() {
        // we can use this because there is only exactly one spaceship - so we're not
        // looping over the query
        if let Ok((mut spaceship_transform, mut velocity)) = q_spaceship.get_single_mut() {
            let spaceship_config = &collider_config.spaceship;

            // dynamically update from inspector while game is running
            spaceship_transform.scale = Vec3::splat(spaceship_config.scalar);

            let spaceship_action = q_input_map.single();

            let mut rotation = 0.0;
            let delta_seconds = time.delta_seconds();
            let rotation_speed = spaceship_config.rotation_speed;

            if spaceship_action.pressed(&SpaceshipAction::TurnRight) {
                // right
                velocity.angvel.z = 0.0;
                rotation = rotation_speed * delta_seconds;
            } else if spaceship_action.pressed(&SpaceshipAction::TurnLeft) {
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

            let max_speed = spaceship_config.velocity;

            let accel = spaceship_config
                .acceleration
                .expect("did you delete spaceship acceleration?");

            if spaceship_action.pressed(&SpaceshipAction::Accelerate) {
                apply_acceleration(
                    &mut velocity,
                    -spaceship_transform.forward().as_vec3(),
                    accel,
                    max_speed,
                    delta_seconds,
                    orientation_mode,
                );
            } else if spaceship_action.pressed(&SpaceshipAction::Decelerate) {
                apply_acceleration(
                    &mut velocity,
                    spaceship_transform.forward().as_vec3(),
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

fn spaceship_shield_controls(
    mut commands: Commands,
    query: Query<Entity, With<Spaceship>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(spaceship) = query.get_single() else {
        return;
    };

    if keyboard_input.pressed(KeyCode::Tab) {
        commands.entity(spaceship).insert(SpaceshipShield);
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
