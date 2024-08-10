use bevy::{
    color::palettes::tailwind,
    ecs::system::SystemParam,
    prelude::*,
    render::view::RenderLayers,
};
use bevy_rapier3d::prelude::{
    ColliderMassProperties::Mass,
    *,
};

use crate::{
    asset_loader::SceneAssets,
    boundary::Boundary,
    collider_config::ColliderConfig,
    collision_detection::{
        GROUP_ASTEROID,
        GROUP_MISSILE,
    },
    config::RenderLayer,
    health::{
        CollisionDamage,
        Health,
        HealthBundle,
    },
    input::SpaceshipAction,
    movement::{
        MovingObjectBundle,
        Teleporter,
    },
    schedule::InGameSet,
    spaceship::{
        ContinuousFire,
        Spaceship,
    },
    utils::name_entity,
};

use crate::config::AppearanceConfig;
use leafwing_input_manager::prelude::*;

pub struct MissilePlugin;

const MISSILE_MASS: f32 = 0.001;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fire_missile.in_set(InGameSet::UserInput))
            .add_systems(
                Update,
                (
                    missile_movement,
                    // toggles the MissilePartyEnabled if the MissileParty spaceship action is
                    // pressed
                    toggle_missile_party,
                    // allows missile party to run only if the MissilePartyEnabled resource is true
                    missile_party.run_if(|enabled: Res<MissilePartyEnabled>| enabled.0),
                )
                    .chain()
                    .in_set(InGameSet::EntityUpdates),
            )
            .insert_resource(MissilePartyEnabled(true));
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
        spaceship_transform: &Transform,
        spaceship_velocity: &Velocity,
        collider_config: &ColliderConfig,
        appearance_config: Res<AppearanceConfig>,
        boundary: Res<Boundary>,
    ) -> Self {
        let forward = -spaceship_transform.forward().with_z(0.0);

        let missile_velocity = forward * collider_config.missile.velocity;

        // add spaceship velocity so that the missile fires in the direction the
        // spaceship is going - without it, they only have the missile velocity
        // and if the spaceship is moving it will look as if they are trailing
        // off to the left or right
        let velocity = spaceship_velocity.linvel + missile_velocity;

        let direction = forward;
        let origin = spaceship_transform.translation
            + direction * appearance_config.missile_forward_spawn_distance;

        Missile {
            velocity,
            total_distance: boundary.max_missile_distance,
            traveled_distance: 0.,
            remaining_distance: 0.,
            last_position: origin,
            last_teleport_position: None,
        }
    }
}

/// Logic to handle whether we're in continuous fire mode or just regular fire
/// mode if continuous we want to make sure that enough time has passed and that
/// we're holding down the fire button
fn should_fire(
    continuous_fire: Option<&ContinuousFire>,
    collider_config: &mut ResMut<ColliderConfig>,
    time: Res<Time>,
    q_input_map: Query<&ActionState<SpaceshipAction>>,
) -> bool {
    let action_state = q_input_map.single();

    if continuous_fire.is_some() {
        // We know the timer exists, so we can safely unwrap it
        let timer = collider_config.missile.spawn_timer.as_mut().unwrap();
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
    appearance_config: Res<'w, AppearanceConfig>,
    boundary:          Res<'w, Boundary>,
    scene_assets:      Res<'w, SceneAssets>,
}

// todo: #bevyquestion - in an object oriented world i think of attaching fire
// as a method to                       the spaceship - but there's a lot of
// missile logic so i have it setup in missile                       so should i
// have a simple fire method in method in spaceship that in turn calls this
//                       fn or is having it here fine?
fn fire_missile(
    mut commands: Commands,
    q_input_map: Query<&ActionState<SpaceshipAction>>,
    q_spaceship: Query<(&Transform, &Velocity, Option<&ContinuousFire>), With<Spaceship>>,
    mut collider_config: ResMut<ColliderConfig>,
    time: Res<Time>,
    res: FireResources,
) {
    if !collider_config.missile.spawnable {
        return;
    }

    let Ok((spaceship_transform, spaceship_velocity, continuous_fire)) = q_spaceship.get_single()
    else {
        return;
    };

    if !should_fire(continuous_fire, &mut collider_config, time, q_input_map) {
        return;
    }

    // extracted for readability
    spawn_missile(
        &mut commands,
        spaceship_transform,
        spaceship_velocity,
        &collider_config,
        res,
    );
}

fn spawn_missile(
    commands: &mut Commands,
    spaceship_transform: &Transform,
    spaceship_velocity: &Velocity,
    collider_config: &ColliderConfig,
    res: FireResources,
) {
    // boundary is used to set the total distance this missile can travel
    let missile = Missile::new(
        spaceship_transform,
        spaceship_velocity,
        collider_config,
        res.appearance_config,
        res.boundary,
    );

    let collider = collider_config.missile.collider.clone();

    let missile = commands
        .spawn(missile)
        .insert(HealthBundle {
            collision_damage: CollisionDamage(collider_config.missile.damage),
            health:           Health(collider_config.missile.health),
        })
        .insert(MovingObjectBundle {
            aabb: collider_config.missile.aabb.clone(),
            collider,
            collision_groups: CollisionGroups::new(GROUP_MISSILE, GROUP_ASTEROID),
            mass: Mass(MISSILE_MASS),
            model: SceneBundle {
                scene: res.scene_assets.missiles.clone(),
                transform: Transform::from_translation(missile.last_position)
                    .with_scale(Vec3::splat(collider_config.missile.scalar)),
                ..default()
            },
            velocity: Velocity {
                linvel: missile.velocity,
                angvel: Default::default(),
            },
            ..default()
        })
        .insert(RenderLayers::from_layers(RenderLayer::Game.layers()))
        //.insert(WallApproachVisual::default())
        .id();

    name_entity(commands, missile, collider_config.missile.name);
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

#[derive(Resource, Default)]
struct MissilePartyEnabled(bool);

fn toggle_missile_party(
    q_input_map: Query<&ActionState<SpaceshipAction>, With<Spaceship>>,
    mut missile_party_enabled: ResMut<MissilePartyEnabled>,
) {
    if let Ok(spaceship_action) = q_input_map.get_single() {
        if spaceship_action.just_pressed(&SpaceshipAction::MissileParty) {
            missile_party_enabled.0 = !missile_party_enabled.0;
            println!("missile party: {:?}", missile_party_enabled.0);
        }
    }
}

/// fun! with missiles!
fn missile_party(
    mut q_missile: Query<&mut Missile>,
    mut gizmos: Gizmos,
    boundary: Res<Boundary>,
    config: Res<AppearanceConfig>,
) {
    for missile in q_missile.iter_mut() {
        draw_missile_targets(&mut gizmos, &missile, &boundary, &config);
    }
}

fn draw_missile_targets(
    gizmos: &mut Gizmos,
    missile: &Missile,
    boundary: &Res<Boundary>,
    config: &AppearanceConfig,
) {
    let current_position = missile.last_position;

    if let Some(next_boundary) = boundary.find_edge_point(current_position, missile.velocity) {
        let (position, normal, color, remaining_distance) =
            if missile.remaining_distance < current_position.distance(next_boundary) {
                let end_point =
                    current_position + missile.velocity.normalize() * missile.remaining_distance;
                let circle_normal = Dir3::new(-missile.velocity.normalize()).unwrap_or(Dir3::NEG_Z);
                (
                    end_point,
                    circle_normal,
                    Color::from(tailwind::GREEN_800),
                    missile.remaining_distance,
                )
            } else {
                let boundary_normal = boundary.get_normal_for_position(next_boundary);
                let distance = current_position.distance(next_boundary);
                (
                    next_boundary,
                    boundary_normal,
                    Color::from(tailwind::BLUE_600),
                    distance,
                )
            };

        draw_variable_size_circle(config, gizmos, position, normal, color, remaining_distance);
    }

    // Draw sphere at the last teleport position if it exists
    if let Some(last_teleport_position) = missile.last_teleport_position {
        let teleport_normal = boundary.get_normal_for_position(last_teleport_position);

        gizmos.circle(
            last_teleport_position,
            teleport_normal,
            config.missile_circle_radius,
            Color::from(tailwind::YELLOW_600),
        );
    }
}

fn draw_variable_size_circle(
    config: &AppearanceConfig,
    gizmos: &mut Gizmos,
    position: Vec3,
    normal: Dir3,
    color: Color,
    remaining_distance: f32,
) {
    let min_radius = config.missile_circle_radius * 0.5; // Minimum radius to ensure visibility
    let max_radius = config.missile_circle_radius; // Maximum radius (current fixed radius)

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
