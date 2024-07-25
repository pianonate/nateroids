use bevy::prelude::{KeyCode::Space, *};
use bevy_rapier3d::prelude::{Collider, ColliderMassProperties::Mass, CollisionGroups, Velocity};

use crate::{
    asset_loader::SceneAssets,
    camera::PrimaryCamera,
    collision_detection::{CollisionDamage, GROUP_ASTEROID, GROUP_MISSILE},
    health::Health,
    movement::{LimitedDistanceMover, MovingObjectBundle},
    schedule::InGameSet,
    spaceship::{ContinuousFire, Spaceship},
    utils::name_entity,
};

const MISSILE_COLLISION_DAMAGE: f32 = 50.0;
const MISSILE_FORWARD_SPAWN_SCALAR: f32 = 3.5;
const MISSILE_HEALTH: f32 = 1.0;
const MISSILE_MASS: f32 = 0.001;
const MISSILE_NAME: &str = "Missile";
const MISSILE_RADIUS: f32 = 0.4;
const MISSILE_SPAWN_TIMER_SECONDS: f32 = 1.0 / 20.0;
const MISSILE_SPEED: f32 = 75.0;

#[derive(Component, Debug)]
pub struct Missile;

#[derive(Resource, Debug)]
pub struct MissileSpawnTimer {
    pub timer: Timer,
}

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        app.insert_resource(MissileSpawnTimer {
            timer: Timer::from_seconds(MISSILE_SPAWN_TIMER_SECONDS, TimerMode::Repeating),
        })
        .add_systems(Update, fire_missile.in_set(InGameSet::UserInput));
    }
}

// todo: #bevyquestion - how could i reduce the number of arguments here?
#[allow(clippy::too_many_arguments)]
fn fire_missile(
    mut commands: Commands,
    q_camera: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
    q_windows: Query<&Window>,
    q_spaceship: Query<(&Transform, Option<&ContinuousFire>), With<Spaceship>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    scene_assets: Res<SceneAssets>,
    mut spawn_timer: ResMut<MissileSpawnTimer>,
    time: Res<Time>,
) {
    // #todo #rustquestion - is this short circuit idiomatic rust?
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

    if continuous && keyboard_input.pressed(Space)
        || !continuous && keyboard_input.just_pressed(Space)
    {
        //if keyboard_input.just_pressed(Space) {
        let mut velocity = -transform.forward() * MISSILE_SPEED;
        velocity.y = 0.0;

        let direction = -transform.forward().as_vec3();
        let origin = transform.translation + direction * MISSILE_FORWARD_SPAWN_SCALAR;
        let limited_distance_mover =
            LimitedDistanceMover::new(origin, direction, q_windows, q_camera);

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
