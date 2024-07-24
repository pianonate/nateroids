use crate::{
    asset_loader::SceneAssets,
    collision_detection::CollisionDamage,
    despawn::AgedEntity,
    health::Health,
    movement::MovingObjectBundle,
    schedule::InGameSet,
    utils::{name_entity, GROUP_ASTEROID, GROUP_MISSILE},
};

use bevy::prelude::{KeyCode::Space, *};

use crate::camera::PrimaryCamera;
use crate::spaceship::Spaceship;
use crate::utils::{calculate_viewable_dimensions, ViewableDimensions};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties::Mass, CollisionGroups, LockedAxes, Velocity,
};

const MISSILE_COLLISION_DAMAGE: f32 = 20.0;
const MISSILE_FORWARD_SPAWN_SCALAR: f32 = 3.5;
const MISSILE_HEALTH: f32 = 1.0;
const MISSILE_MASS: f32 = 0.001;
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
        .add_systems(Update, draw_edge_point)
        .add_systems(Update, fire_missile.in_set(InGameSet::UserInput));
    }
}

fn fire_missile(
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

    // the following code would need to be used if we are in continuous fire mode
    // you could create a component that allows for continuous fire and then invoke this
    /*    spawn_timer.timer.tick(time.delta());

    if !spawn_timer.timer.just_finished() {
        return;
    }*/

    if keyboard_input.just_pressed(Space) {
        let mut velocity = -transform.forward() * MISSILE_SPEED;
        velocity.y = 0.0;

        let origin = transform.translation + -transform.forward() * MISSILE_FORWARD_SPAWN_SCALAR;
        let direction = transform.forward();
        let distance_traveled = 0.0;

        let missile = commands
            .spawn(Missile)
            .insert(AgedEntity::new(0))
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
            .id(); // to ensure we store the entity id for subsequent use

        name_entity(&mut commands, missile, "SpaceshipMissile");
    }
}

fn draw_edge_point(
    windows: Query<&Window>,
    camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
    spaceship_query: Query<&Transform, With<Spaceship>>,
    mut gizmos: Gizmos,
) {
    if let Ok(spaceship_transform) = spaceship_query.get_single() {
        let origin = spaceship_transform.translation;
        let direction = -spaceship_transform.forward().as_vec3();

        if let Some(edge_point) = find_edge_point(origin, direction, &windows, &camera_query) {
            // Log the edge point for debugging
            println!("Edge Point: {:?}", edge_point);

            gizmos
                .sphere(edge_point, Quat::IDENTITY, 1., Color::WHITE)
                .resolution(64);
        }
    }
}

fn find_edge_point(
    origin: Vec3,
    direction: Vec3,
    windows: &Query<&Window>,
    camera_query: &Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
) -> Option<Vec3> {
    if let Some(dimensions) = calculate_viewable_dimensions(windows, camera_query) {
        let ViewableDimensions { width, height } = dimensions;

        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let x_intersection = calculate_intersection(origin.x, direction.x, half_width, -half_width);
        let z_intersection =
            calculate_intersection(origin.z, direction.z, half_height, -half_height);

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
    }
    None
}

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
