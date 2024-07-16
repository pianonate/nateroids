use bevy::prelude::*;

use bevy::prelude::KeyCode::{
    ArrowDown, ArrowLeft, ArrowRight, ArrowUp, ControlLeft, KeyA, KeyD, KeyS, KeyW, ShiftLeft,
    Space,
};

use crate::{
    asset_loader::SceneAssets,
    collision_detection::{CollisionDamage, OldCollider},
    despawn::Mortal,
    health::Health,
    movement::{Acceleration, MovingObjectBundle, Velocity, Wrappable},
    schedule::InGameSet,
    state::GameState,
    utils::name_entity,
};

const MISSILE_COLLISION_DAMAGE: f32 = 20.0;
const MISSILE_FORWARD_SPAWN_SCALAR: f32 = 3.5;
const MISSILE_HEALTH: f32 = 1.0;
const MISSILE_RADIUS: f32 = 1.0;
const MISSILE_SPAWN_TIMER_SECONDS: f32 = 1.0 / 20.0;
const MISSILE_SPEED: f32 = 45.0;

const SPACESHIP_COLLISION_DAMAGE: f32 = 100.0;
const SPACESHIP_HEALTH: f32 = 100.0;
const SPACESHIP_RADIUS: f32 = 5.0;
const SPACESHIP_ROLL_SPEED: f32 = 2.5;
const SPACESHIP_ROTATION_SPEED: f32 = 2.5;
const SPACESHIP_SCALE: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const SPACESHIP_SPEED: f32 = 35.0;
const STARTING_TRANSLATION: Vec3 = Vec3::new(0.0, 0.0, -20.0);

#[derive(Component, Debug)]
pub struct Spaceship;

#[derive(Component, Debug)]
pub struct SpaceshipShield;

#[derive(Component, Debug)]
pub struct SpaceshipMissile;

#[derive(Resource, Debug)]
pub struct MissileSpawnTimer {
    pub timer: Timer,
}

pub struct SpaceshipPlugin;

impl Plugin for SpaceshipPlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        app.insert_resource(MissileSpawnTimer {
            timer: Timer::from_seconds(MISSILE_SPAWN_TIMER_SECONDS, TimerMode::Repeating),
        })
        .add_systems(PostStartup, spawn_spaceship)
        // spawn a new Spaceship if we're in GameOver state
        .add_systems(OnEnter(GameState::GameOver), spawn_spaceship)
        .add_systems(
            Update,
            (
                spaceship_movement_controls,
                spaceship_weapon_controls,
                spaceship_shield_controls,
            )
                .chain()
                .in_set(InGameSet::UserInput),
        )
        // check if spaceship is destroyed...this will change the GameState
        .add_systems(Update, spaceship_destroyed.in_set(InGameSet::EntityUpdates));
    }
}

fn spawn_spaceship(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    let entity = commands
        .spawn(MovingObjectBundle {
            velocity: Velocity::new(Vec3::ZERO),
            acceleration: Acceleration::new(Vec3::ZERO),
            collider: OldCollider::new(SPACESHIP_RADIUS),
            model: SceneBundle {
                scene: scene_assets.spaceship.clone(),
                // transform: Transform::from_translation(STARTING_TRANSLATION),
                transform: Transform {
                    translation: STARTING_TRANSLATION,
                    scale: SPACESHIP_SCALE,
                    ..Default::default()
                },
                ..default()
            },
        })
        .insert(Spaceship)
        .insert(Name::new("Spaceship"))
        .insert(Health::new(SPACESHIP_HEALTH))
        .insert(CollisionDamage::new(SPACESHIP_COLLISION_DAMAGE))
        .insert(Wrappable)
        .id();

    name_entity(&mut commands, entity, "Spaceship");
}

fn spaceship_movement_controls(
    mut query: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // we can use this because there is only exactly one spaceship - so we're not looping over the query
    let Ok((mut transform, mut velocity)) = query.get_single_mut() else {
        return;
    };

    let mut rotation = 0.0;
    let mut roll = 0.0;
    let mut movement = 0.0;

    if keyboard_input.any_pressed([KeyD, ArrowRight]) {
        // right
        rotation = -SPACESHIP_ROTATION_SPEED * time.delta_seconds();
    } else if keyboard_input.any_pressed([KeyA, ArrowLeft]) {
        // left
        rotation = SPACESHIP_ROTATION_SPEED * time.delta_seconds();
    }

    // we don't need to multiply time time.delta_seconds() because we already do this in Movement
    if keyboard_input.any_pressed([KeyS, ArrowDown]) {
        // down
        movement = -SPACESHIP_SPEED;
    } else if keyboard_input.any_pressed([KeyW, ArrowUp]) {
        // up
        movement = SPACESHIP_SPEED;
    }

    if keyboard_input.pressed(ShiftLeft) {
        roll = -SPACESHIP_ROLL_SPEED * time.delta_seconds();
    } else if keyboard_input.pressed(ControlLeft) {
        roll = SPACESHIP_ROLL_SPEED * time.delta_seconds();
    }

    // rotate around the y-axis
    // ignores the z-axis rotation applied below
    transform.rotate_y(rotation);

    // rotate around the local z-axis
    // the rotation is relative to the current rotation
    transform.rotate_local_z(roll);

    // update the spaceship's velocity based on new direction
    // the model has a different orientation than bevy uses (typically the ones that come from bevy)
    velocity.value = -transform.forward() * movement;
}

fn spaceship_weapon_controls(
    mut commands: Commands,
    query: Query<&Transform, With<Spaceship>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    scene_assets: Res<SceneAssets>,
    mut spawn_timer: ResMut<MissileSpawnTimer>,
    time: Res<Time>,
) {
    let Ok(transform) = query.get_single() else {
        return;
    };

    spawn_timer.timer.tick(time.delta());

    if !spawn_timer.timer.just_finished() {
        return;
    }

    if keyboard_input.pressed(Space) {
        let entity = commands
            .spawn(MovingObjectBundle {
                velocity: Velocity::new(-transform.forward() * MISSILE_SPEED),
                acceleration: Acceleration::new(Vec3::ZERO),
                collider: OldCollider::new(MISSILE_RADIUS),
                model: SceneBundle {
                    scene: scene_assets.missiles.clone(),
                    transform: Transform::from_translation(
                        transform.translation + -transform.forward() * MISSILE_FORWARD_SPAWN_SCALAR,
                    ),
                    ..default()
                },
            })
            .insert(CollisionDamage::new(MISSILE_COLLISION_DAMAGE))
            .insert(Health::new(MISSILE_HEALTH))
            .insert(Mortal::new(0))
            .insert(Name::new("SpaceshipMissile"))
            .insert(SpaceshipMissile)
            .insert(Wrappable)
            .id(); // to ensure we store the entity id for subsequent use

        name_entity(&mut commands, entity, "Missile");
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
// if get single (there should only be one - returns an error then the spaceship doesn't exist
fn spaceship_destroyed(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<(), With<Spaceship>>,
) {
    if query.get_single().is_err() {
        next_state.set(GameState::GameOver);
    }
}
