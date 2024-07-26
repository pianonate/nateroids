use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::{
        KeyCode::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, KeyA, KeyD, KeyS, KeyW, F9},
        *,
    },
};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties::Mass, CollisionGroups, LockedAxes, Velocity,
};

use crate::{
    asset_loader::SceneAssets,
    collision_detection::{GROUP_ASTEROID, GROUP_SPACESHIP},
    health::{CollisionDamage, Health, HealthBundle},
    movement::MovingObjectBundle,
    schedule::InGameSet,
    state::GameState,
    utils::name_entity,
};
use leafwing_input_manager::prelude::*;

const SPACESHIP_ACCELERATION: f32 = 20.0;
const SPACESHIP_COLLISION_DAMAGE: f32 = 100.0;
const SPACESHIP_HEALTH: f32 = 100.0;
const SPACESHIP_MAX_SPEED: f32 = 40.0;
const SPACESHIP_NAME: &str = "Spaceship";
const SPACESHIP_RADIUS: f32 = 5.0;
//const SPACESHIP_ROLL_SPEED: f32 = 2.5;
const SPACESHIP_ROTATION_SPEED: f32 = 5.0;
const SPACESHIP_SCALE: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const STARTING_TRANSLATION: Vec3 = Vec3::new(0.0, 0.0, -20.0);

#[derive(Component, Debug)]
pub struct Spaceship;

#[derive(Component, Debug)]
pub struct SpaceshipShield;

#[derive(Component, Debug)]
pub struct ContinuousFire;

pub struct SpaceshipPlugin;
impl Plugin for SpaceshipPlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default())
            .add_systems(PostStartup, spawn_spaceship)
            // spawn a new Spaceship if we're in GameOver state
            .add_systems(OnEnter(GameState::GameOver), spawn_spaceship)
            .add_systems(
                Update,
                (
                    spaceship_movement_controls,
                    spaceship_shield_controls,
                    toggle_continuous_fire.run_if(input_just_pressed(F9)),
                )
                    .chain()
                    .in_set(InGameSet::UserInput),
            )
            // check if spaceship is destroyed...this will change the GameState
            .add_systems(Update, spaceship_destroyed.in_set(InGameSet::EntityUpdates));
    }
}

// This is the list of "things I want the spaceship to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    Accelerate,
    Decelerate,
    Fire,
    TurnLeft,
    TurnRight,
}

impl Action {
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        input_map.insert(Self::Accelerate, KeyW);
        input_map.insert(Self::Accelerate, ArrowUp);

        input_map.insert(Self::Decelerate, KeyS);
        input_map.insert(Self::Decelerate, ArrowDown);

        input_map.insert(Self::Fire, KeyCode::Space);

        input_map.insert(Self::TurnLeft, KeyA);
        input_map.insert(Self::TurnLeft, ArrowLeft);

        input_map.insert(Self::TurnRight, KeyD);
        input_map.insert(Self::TurnRight, ArrowRight);

        input_map
    }
}

fn toggle_continuous_fire(
    mut commands: Commands,
    q_spaceship: Query<(Entity, Option<&ContinuousFire>), With<Spaceship>>,
) {
    if let Ok((entity, continuous)) = q_spaceship.get_single() {
        if continuous.is_some() {
            println!("removing continuous");
            commands.entity(entity).remove::<ContinuousFire>();
        } else {
            println!("adding continuous");
            commands.entity(entity).insert(ContinuousFire);
        }
    }
}

fn spawn_spaceship(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    let spaceship = commands
        .spawn(Spaceship)
        .insert(HealthBundle {
            collision_damage: CollisionDamage(SPACESHIP_COLLISION_DAMAGE),
            health: Health(SPACESHIP_HEALTH),
        })
        .insert(MovingObjectBundle {
            collider: Collider::ball(SPACESHIP_RADIUS),
            collision_groups: CollisionGroups::new(GROUP_SPACESHIP, GROUP_ASTEROID),
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Y
                | LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Z,
            mass: Mass(3.0),
            model: SceneBundle {
                scene: scene_assets.spaceship.clone(),
                transform: Transform {
                    translation: STARTING_TRANSLATION,
                    scale: SPACESHIP_SCALE,
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(InputManagerBundle::with_map(Action::default_input_map()))
        .id();

    name_entity(&mut commands, spaceship, SPACESHIP_NAME);
}

fn spaceship_movement_controls(
    mut query: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    input_map: Query<&ActionState<Action>>,
    time: Res<Time>,
) {
    // we can use this because there is only exactly one spaceship - so we're not looping over the query
    if let Ok((mut transform, mut velocity)) = query.get_single_mut() {
        let action_state = input_map.single();

        let mut rotation = 0.0;
        let delta_seconds = time.delta_seconds();

        if action_state.pressed(&Action::TurnRight) {
            // right
            velocity.angvel.y = 0.0;
            rotation = -SPACESHIP_ROTATION_SPEED * delta_seconds;
        } else if action_state.pressed(&Action::TurnLeft) {
            // left
            velocity.angvel.y = 0.0;
            rotation = SPACESHIP_ROTATION_SPEED * delta_seconds;
        }

        // rotate around the y-axis
        transform.rotate_y(rotation);

        if action_state.pressed(&Action::Accelerate) {
            // down
            apply_acceleration(
                &mut velocity,
                -transform.forward().as_vec3(),
                SPACESHIP_ACCELERATION,
                SPACESHIP_MAX_SPEED,
                delta_seconds,
            );
        } else if action_state.pressed(&Action::Decelerate) {
            // up
            apply_acceleration(
                &mut velocity,
                -transform.forward().as_vec3(),
                -SPACESHIP_ACCELERATION,
                SPACESHIP_MAX_SPEED,
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

    // Force the `y` value of velocity.linvel to be 0
    velocity.linvel.y = 0.0;
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
