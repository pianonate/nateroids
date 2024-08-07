use bevy::{prelude::*, render::view::visibility::RenderLayers};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties::Mass, CollisionGroups, LockedAxes, Velocity,
};

use crate::{
    asset_loader::SceneAssets,
    camera::PrimaryCamera,
    collision_detection::{GROUP_ASTEROID, GROUP_SPACESHIP},
    config::{GameConfig, RenderLayer},
    health::{CollisionDamage, Health, HealthBundle},
    input::SpaceshipAction,
    movement::MovingObjectBundle,
    schedule::InGameSet,
    state::GameState,
    utils::name_entity,
};

use leafwing_input_manager::prelude::*;

const SPACESHIP_ACCELERATION: f32 = 20.0;
const SPACESHIP_COLLISION_DAMAGE: f32 = 100.0;
const SPACESHIP_HEALTH: f32 = 100.0;
// const SPACESHIP_MAX_SPEED: f32 = 40.0;
//const SPACESHIP_ROLL_SPEED: f32 = 2.5;
const SPACESHIP_ROTATION_SPEED: f32 = 3.0;
const STARTING_TRANSLATION: Vec3 = Vec3::new(0.0, -20.0, 0.0);

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
        // we can come into InGame a couple of ways - when we do, spawn a spaceship
        // either when we exit splash, or when we enter GameOver
        app.add_systems(OnEnter(GameState::InGame), spawn_spaceship)
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

// todo: how can i avoid setting this allow
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
    config: Res<GameConfig>,
    scene_assets: Res<SceneAssets>,
    //  q_camera: Query<Entity, With<PrimaryCamera>>,
) {
    if !config.spaceship.spawnable {
        return;
    }

    let spaceship_input = InputManagerBundle::with_map(SpaceshipAction::spaceship_input_map());

    let spaceship = commands
        .spawn(Spaceship)
        .insert(RenderLayers::layer(RenderLayer::Game.layer()))
        .insert(HealthBundle {
            collision_damage: CollisionDamage(SPACESHIP_COLLISION_DAMAGE),
            health: Health(SPACESHIP_HEALTH),
        })
        .insert(MovingObjectBundle {
            collider: Collider::ball(config.spaceship.radius),
            collision_groups: CollisionGroups::new(GROUP_SPACESHIP, GROUP_ASTEROID),
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Z
                | LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Y,
            mass: Mass(3.0),
            model: SceneBundle {
                scene: scene_assets.spaceship.clone(),
                transform: Transform {
                    translation: STARTING_TRANSLATION,
                    scale: Vec3::splat(config.spaceship.scalar),
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                },
                ..default()
            },
            ..default()
        })
        .insert(spaceship_input)
        .id();

    // if let Ok(camera) = q_camera.get_single() {
    //     commands.entity(spaceship).add_child(camera);
    // }

    name_entity(&mut commands, spaceship, config.spaceship.name);
}

fn spaceship_movement_controls(
    mut q_spaceship: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    q_camera: Query<&Transform, (With<PrimaryCamera>, Without<Spaceship>)>,
    q_input_map: Query<&ActionState<SpaceshipAction>>,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    if let Ok(camera_transform) = q_camera.get_single() {
        // we can use this because there is only exactly one spaceship - so we're not looping over the query
        if let Ok((mut spaceship_transform, mut velocity)) = q_spaceship.get_single_mut() {
            // dynamically update from inspector while game is running
            spaceship_transform.scale = Vec3::splat(config.spaceship.scalar);

            let spaceship_action = q_input_map.single();

            let mut rotation = 0.0;
            let delta_seconds = time.delta_seconds();

            if spaceship_action.pressed(&SpaceshipAction::TurnRight) {
                // right
                velocity.angvel.z = 0.0;
                rotation = SPACESHIP_ROTATION_SPEED * delta_seconds;
            } else if spaceship_action.pressed(&SpaceshipAction::TurnLeft) {
                // left
                velocity.angvel.z = 0.0;
                rotation = -SPACESHIP_ROTATION_SPEED * delta_seconds;
            }

            let camera_forward = camera_transform.forward();
            let facing_opposite = camera_forward.dot(Vec3::new(0.0, 0.0, -1.0)) > 0.0;

            if facing_opposite {
                rotation = -rotation;
            }

            // rotate around the z-axis
            spaceship_transform.rotate_z(rotation);

            let max_speed = config.spaceship.velocity;

            if spaceship_action.pressed(&SpaceshipAction::Accelerate) {
                // down
                apply_acceleration(
                    &mut velocity,
                    -spaceship_transform.forward().as_vec3(),
                    SPACESHIP_ACCELERATION,
                    max_speed,
                    delta_seconds,
                );
            } else if spaceship_action.pressed(&SpaceshipAction::Decelerate) {
                // up
                apply_acceleration(
                    &mut velocity,
                    -spaceship_transform.forward().as_vec3(),
                    -SPACESHIP_ACCELERATION,
                    max_speed,
                    delta_seconds,
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
) {
    let proposed_velocity = velocity.linvel + direction * (acceleration * delta_seconds);
    let proposed_speed = proposed_velocity.length();

    // Ensure we're not exceeding max velocity
    if proposed_speed > max_speed {
        velocity.linvel = proposed_velocity.normalize() * max_speed;
    } else {
        velocity.linvel = proposed_velocity;
    }

    // Force the `z` value of velocity.linvel to be 0
    velocity.linvel.z = 0.0;
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
// if get_single() (there should only be one - returns an error then the spaceship doesn't exist
fn spaceship_destroyed(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<(), With<Spaceship>>,
) {
    if query.get_single().is_err() {
        next_state.set(GameState::GameOver);
    }
}
