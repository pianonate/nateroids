// #todo: #bug - missiles spawn rotated incorrectly center
use bevy::{
    color::palettes::tailwind,
    ecs::system::SystemParam,
    prelude::*,
    render::view::RenderLayers,
};
use bevy_rapier3d::prelude::*;

use crate::{
    asset_loader::SceneAssets,
    boundary::Boundary,
    camera::RenderLayer,
    collider_config::ColliderConfig,
    collision_detection::{
        GROUP_ASTEROID,
        GROUP_MISSILE,
    },
    health::{
        CollisionDamage,
        Health,
        HealthBundle,
    },
    input::SpaceshipAction,
    schedule::InGameSet,
    utils::name_entity,
};

use crate::{
    actor::{
        aabb::Aabb,
        actor_spawner::ActorConfig,
        actor_template::MissileConfig,
        movement::MovingObjectBundle,
        spaceship::{
            ContinuousFire,
            Spaceship,
        },
        Teleporter,
    },
    boundary::{
        BoundaryConfig,
        WallApproachVisual,
    },
    collider_config::ColliderConstant,
};

use leafwing_input_manager::prelude::*;

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fire_missile.in_set(InGameSet::UserInput))
            .add_systems(
                Update,
                (missile_movement, missile_party).in_set(InGameSet::EntityUpdates),
            );
    }
}

// todo: #rustquestion - how can i make it so that new has to be used and
// DrawDirection isn't constructed directly - i still need the fields visible
#[derive(Copy, Clone, Component, Debug)]
pub struct Missile {
    velocity:               Vec3,
    pub total_distance:     f32,
    pub traveled_distance:  f32,
    remaining_distance:     f32,
    last_position:          Vec3,
    last_teleport_position: Option<Vec3>, // Add this field
}

impl Missile {
    fn new(
        spaceship_transform: Transform,
        spaceship_velocity: &Velocity,
        spaceship_aabb: &Aabb,
        missile_config: &ColliderConstant,
        boundary: Res<Boundary>,
    ) -> Self {
        let forward = -spaceship_transform.forward();

        let missile_velocity = forward * missile_config.linear_velocity;

        // add spaceship velocity so that the missile fires in the direction the
        // spaceship is going - without it, they only have the missile velocity
        // and if the spaceship is moving it will look as if they are trailing
        // off to the left or right
        let velocity = spaceship_velocity.linvel + missile_velocity;

        // this one is actually tricky to land the firing point right at the edge of the
        // bounding box - got some help from claude.ai and it's working after some trial
        // and error
        let last_position =
            missile_config.get_forward_spawn_point(spaceship_transform, spaceship_aabb);

        Missile {
            velocity,
            total_distance: boundary.max_missile_distance,
            traveled_distance: 0.,
            remaining_distance: 0.,
            last_position,
            last_teleport_position: None,
        }
    }
}

/// Logic to handle whether we're in continuous fire mode or just regular fire
/// mode if continuous we want to make sure that enough time has passed and that
/// we're holding down the fire button
fn should_fire(
    continuous_fire: Option<&ContinuousFire>,
    missile_config: &mut ActorConfig,
    time: Res<Time>,
    q_input_map: Query<&ActionState<SpaceshipAction>>,
) -> bool {
    let action_state = q_input_map.single();

    if continuous_fire.is_some() {
        // We know the timer exists, so we can safely unwrap it
        let timer = missile_config
            .spawn_timer
            .as_mut()
            .expect("configure missile spawn timer here: impl Default for InitialEnsembleConfig");
        timer.tick(time.delta());
        if !timer.just_finished() {
            return false;
        }
        action_state.pressed(&SpaceshipAction::Fire)
    } else {
        action_state.just_pressed(&SpaceshipAction::Fire)
    }
}
#[derive(SystemParam)]
struct FireResources<'w> {
    boundary:     Res<'w, Boundary>,
    scene_assets: Res<'w, SceneAssets>,
}

// todo: #bevyquestion - in an object oriented world i think of attaching fire
// as a method to                       the spaceship - but there's a lot of
// missile logic so i have it setup in missile                       so should i
// have a simple fire method in method in spaceship that in turn calls this
//                       fn or is having it here fine?
fn fire_missile(
    mut commands: Commands,
    q_input_map: Query<&ActionState<SpaceshipAction>>,
    q_spaceship: Query<(&Transform, &Velocity, &Aabb, Option<&ContinuousFire>), With<Spaceship>>,
    collider_config: ResMut<ColliderConfig>,
    mut missile_config: ResMut<MissileConfig>,

    time: Res<Time>,
    res: FireResources,
) {
    // if !collider_config.missile.spawnable {
    //     return;
    // }

    if !missile_config.0.spawnable {
        return;
    }

    let Ok((spaceship_transform, spaceship_velocity, aabb, continuous_fire)) =
        q_spaceship.get_single()
    else {
        return;
    };

    if !should_fire(
        continuous_fire,
        //&mut collider_config.missile,
        &mut missile_config.0,
        time,
        q_input_map,
    ) {
        return;
    }

    // extracted for readability
    spawn_missile(
        &mut commands,
        *spaceship_transform,
        spaceship_velocity,
        aabb,
        &collider_config.missile,
        res,
    );
}

fn spawn_missile(
    commands: &mut Commands,
    spaceship_transform: Transform,
    spaceship_velocity: &Velocity,
    aabb: &Aabb,
    missile_config: &ColliderConstant,
    res: FireResources,
) {
    // boundary is used to set the total distance this missile can travel
    let missile = Missile::new(
        spaceship_transform,
        spaceship_velocity,
        aabb,
        missile_config,
        res.boundary,
    );

    let collider = missile_config.collider.clone();

    let missile = commands
        .spawn(missile)
        .insert(HealthBundle {
            collision_damage: CollisionDamage(missile_config.damage),
            health:           Health(missile_config.health),
        })
        .insert(MovingObjectBundle {
            aabb: missile_config.aabb.clone(),
            collider,
            collision_groups: CollisionGroups::new(GROUP_MISSILE, GROUP_ASTEROID),
            mass: missile_config.mass,
            model: SceneBundle {
                scene: res.scene_assets.missile.clone(),
                transform: Transform::from_translation(missile.last_position)
                    .with_scale(Vec3::splat(missile_config.scalar))
                    .with_rotation(spaceship_transform.rotation),
                ..default()
            },
            restitution: missile_config.restitution,
            velocity: Velocity {
                linvel: missile.velocity,
                angvel: Default::default(),
            },
            ..default()
        })
        .insert(RenderLayers::from_layers(RenderLayer::Game.layers()))
        .insert(WallApproachVisual::default())
        .id();

    name_entity(commands, missile, missile_config.name.to_owned());
}

/// we update missile movement so that it can be despawned after it has traveled
/// its total distance
fn missile_movement(mut query: Query<(&Transform, &mut Missile, &Teleporter)>) {
    for (transform, mut missile, wrappable) in query.iter_mut() {
        let current_position = transform.translation;

        // Calculate the distance traveled since the last update
        // we use wrapped as a sentinel so that we don't consider
        // the teleport of the missile at the edge of the screen to have
        // used up any distance
        let distance_traveled = if wrappable.just_teleported {
            0.0
        } else {
            missile.last_position.distance(current_position)
        };

        // Update the total traveled distance
        missile.traveled_distance += distance_traveled;
        missile.remaining_distance = missile.total_distance - missile.traveled_distance;
        missile.last_position = current_position;

        // Update the last teleport position if the missile wrapped
        if wrappable.just_teleported {
            missile.last_teleport_position = Some(missile.last_position);
        }
    }
}

/// fun! with missiles!
fn missile_party(
    mut q_missile: Query<&mut Missile>,
    mut gizmos: Gizmos,
    boundary: Res<Boundary>,
    config: Res<BoundaryConfig>,
) {
    for missile in q_missile.iter_mut() {
        let current_position = missile.last_position;

        if let Some(next_boundary) = boundary.find_edge_point(current_position, missile.velocity) {
            if missile.remaining_distance < current_position.distance(next_boundary) {
                let end_point =
                    current_position + missile.velocity.normalize() * missile.remaining_distance;
                let circle_normal = Dir3::new(-missile.velocity.normalize()).unwrap_or(Dir3::NEG_Z);

                draw_variable_size_circle(
                    &mut gizmos,
                    end_point,
                    circle_normal,
                    Color::from(tailwind::GREEN_800),
                    missile.remaining_distance,
                    config.smallest_teleport_circle,
                );
            }
        }
    }
}

fn draw_variable_size_circle(
    gizmos: &mut Gizmos,
    position: Vec3,
    normal: Dir3,
    color: Color,
    remaining_distance: f32,
    smallest_teleport_circle: f32,
) {
    let min_radius = smallest_teleport_circle * 0.5; // Minimum radius to ensure visibility
    let max_radius = smallest_teleport_circle; // Maximum radius (current fixed radius)

    // Define a distance threshold where the circle starts to shrink
    let shrink_threshold = 20.; // Adjust this value as needed

    let radius = if remaining_distance > shrink_threshold {
        max_radius
    } else {
        let scale_factor = (remaining_distance / shrink_threshold).clamp(0.0, 1.0);
        min_radius + (max_radius - min_radius) * scale_factor
    };

    gizmos.circle(position, normal, radius, color);
}
