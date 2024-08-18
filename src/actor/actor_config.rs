use crate::{
    actor::{
        get_scene_aabb,
        initial_actor_config::InitialEnsembleConfig,
        Aabb,
        Teleporter,
    },
    asset_loader::{
        AssetsState,
        SceneAssets,
    },
    boundary::WallApproachVisual,
    camera::RenderLayer,
    health::{
        CollisionDamage,
        Health,
    },
    input::GlobalAction,
    utils::{
        name_entity,
        random_vec3,
        toggle_active,
    },
};
use bevy::{
    prelude::*,
    render::view::RenderLayers,
};
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::fmt;
use crate::boundary::Boundary;

// this is how far off we are from blender for the assets we're loading
// we need to get them scaled up to generate a usable aabb
const BLENDER_SCALE: f32 = 100.;

// call flow is to initialize the ensemble config which has the defaults
// for an actor - configure defaults in initial_actor_config.rs
pub struct ActorConfigPlugin;
impl Plugin for ActorConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetsState>()
            .register_type::<EnsembleConfig>()
            .add_systems(OnEnter(AssetsState::Loaded), initialize_ensemble_config)
            .add_plugins(
                ResourceInspectorPlugin::<EnsembleConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::ActorInspector)),
            );
    }
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub enum ColliderType {
    Ball,
    Cuboid,
}

#[derive(Reflect, Debug, Clone)]
pub enum SpawnPositionBehavior {
    Fixed(Vec3),
    RandomWithinBounds {
        scale_factor: Vec3,
        random_rotation: bool,
    },
    RelativeToParent {
        offset: Vec3,
    },
}

#[derive(Reflect, Debug, Clone)]
pub enum VelocityBehavior {
    Fixed(Vec3),
    Random {
        linvel: f32,
        angvel: f32,
    },
    RelativeToParent {
        base_velocity:           f32,
        inherit_parent_velocity: bool,
    },
}

impl VelocityBehavior {
    fn calculate_velocity(
        &self,
        parent_velocity: Option<&Velocity>,
        parent_transform: Option<&Transform>,
    ) -> Velocity {
        match self {
            VelocityBehavior::Fixed(velocity) => Velocity::linear(*velocity),
            VelocityBehavior::Random { linvel, angvel } => Velocity {
                linvel: random_vec3(-*linvel..*linvel, -*linvel..*linvel, 0.0..0.0),
                angvel: random_vec3(-*angvel..*angvel, -*angvel..*angvel, -*angvel..*angvel),
            },
            VelocityBehavior::RelativeToParent {
                base_velocity,
                inherit_parent_velocity,
            } => {
                if let (Some(parent_velocity), Some(parent_transform)) =
                    (parent_velocity, parent_transform)
                {
                    let forward = -parent_transform.forward();
                    let mut velocity = forward * *base_velocity;
                    if *inherit_parent_velocity {
                        velocity += parent_velocity.linvel;
                    }
                    Velocity::linear(velocity)
                } else {
                    Velocity::zero()
                }
            },
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct ActorConfig {
    pub spawnable:                bool,
    #[reflect(ignore)]
    pub aabb:                     Aabb,
    #[reflect(ignore)]
    pub actor_type:               ActorType,
    #[reflect(ignore)]
    pub collider:                 Collider,
    pub collider_type:            ColliderType,
    pub collision_damage:         CollisionDamage,
    #[reflect(ignore)]
    pub collision_groups:         CollisionGroups,
    pub gravity_scale:            f32,
    pub health:                   Health,
    pub locked_axes:              LockedAxes,
    #[inspector(min = 0.0, max = 20.0, display = NumberDisplay::Slider)]
    pub mass:                     f32,
    pub render_layer:             RenderLayer,
    #[inspector(min = 0.1, max = 1.0, display = NumberDisplay::Slider)]
    pub restitution:              f32,
    pub restitution_combine_rule: CoefficientCombineRule,
    pub rigid_body:               RigidBody,
    #[inspector(min = 0.1, max = 10.0, display = NumberDisplay::Slider)]
    pub scalar:                   f32,
    pub spawn_position_behavior:  SpawnPositionBehavior,
    pub spawn_timer_seconds:      Option<f32>,
    #[reflect(ignore)]
    pub spawn_timer:              Option<Timer>,
    pub velocity_behavior:        VelocityBehavior,
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            spawnable:                true,
            actor_type:               ActorType::default(),
            aabb:                     Aabb::default(),
            collider:                 Collider::cuboid(0.5, 0.5, 0.5),
            collider_type:            ColliderType::Cuboid,
            collision_damage:         CollisionDamage(0.),
            collision_groups:         CollisionGroups::default(),
            gravity_scale:            0.,
            health:                   Health(0.),
            locked_axes:              LockedAxes::TRANSLATION_LOCKED_Z,
            mass:                     1.,
            render_layer:             RenderLayer::Both,
            restitution:              1.,
            restitution_combine_rule: CoefficientCombineRule::Max,
            rigid_body:               RigidBody::Dynamic,
            scalar:                   1.,
            spawn_position_behavior:  SpawnPositionBehavior::Fixed(Vec3::ZERO),
            spawn_timer_seconds:      None,
            spawn_timer:              None,
            velocity_behavior:        VelocityBehavior::Fixed(Vec3::ZERO),
        }
    }
}

impl ActorConfig {
    fn calculate_spawn_transform(&self, boundary: Res<Boundary>) -> Transform {
       
        match &self.spawn_position_behavior {
            SpawnPositionBehavior::Fixed(position) => {
                Transform::from_translation(*position).with_scale(Vec3::splat(self.scalar))
            },
            
            SpawnPositionBehavior::RandomWithinBounds { scale_factor, random_rotation } => {
                
                let bounds = Transform {
                    translation: boundary.transform.translation,
                    scale: boundary.transform.scale * *scale_factor,
                    ..default()
                };
                let position = get_random_position_within_bounds(&bounds);
                
                let mut transform = Transform::from_translation(position)
                    .with_scale(Vec3::splat(self.scalar));
                
                if *random_rotation {
                    transform.rotation = get_random_rotation();
                }
                
                transform
            },
            
            SpawnPositionBehavior::RelativeToParent { offset } => {
                // Implementation remains the same
                Transform::from_translation(*offset).with_scale(Vec3::splat(self.scalar))
            },
        }
    }
}

#[derive(Bundle)]
pub struct ActorBundle {
    pub actor_type:       ActorType,
    pub aabb:             Aabb,
    pub active_events:    ActiveEvents,
    pub collider:         Collider,
    pub collision_damage: CollisionDamage,
    pub collision_groups: CollisionGroups,
    pub gravity_scale:    GravityScale,
    pub health:           Health,
    pub locked_axes:      LockedAxes,
    pub rigid_body:       RigidBody,
    pub restitution:      Restitution,
    pub mass_properties:  ColliderMassProperties,
    pub render_layers:    RenderLayers,
    pub scene_bundle:     SceneBundle,
    pub teleporter:       Teleporter,
    pub velocity:         Velocity,
    pub wall_visualizer:  WallApproachVisual,
}

impl ActorBundle {
    pub fn new(
        config: &ActorConfig,
        scene: Handle<Scene>,
        parent: Option<(&Transform, &Velocity)>,
        boundary: Res<Boundary>,

    ) -> Self {
        let (parent_transform, parent_velocity) = parent.unzip();
        let transform = config.calculate_spawn_transform(boundary);
        let velocity = config
            .velocity_behavior
            .calculate_velocity(parent_velocity, parent_transform);

        Self {
            actor_type: config.actor_type,
            aabb: config.aabb.clone(),
            active_events: ActiveEvents::COLLISION_EVENTS,
            collider: config.collider.clone(),
            collision_damage: config.collision_damage.clone(),
            collision_groups: config.collision_groups,
            gravity_scale: GravityScale(config.gravity_scale),
            health: config.health.clone(),
            locked_axes: config.locked_axes,
            rigid_body: config.rigid_body,
            restitution: Restitution {
                coefficient:  config.restitution,
                combine_rule: config.restitution_combine_rule,
            },
            mass_properties: ColliderMassProperties::Mass(config.mass),
            render_layers: RenderLayers::from_layers(config.render_layer.layers()),
            scene_bundle: SceneBundle {
                scene,
                transform,
                ..default()
            },
            teleporter: Teleporter::default(),
            velocity,
            wall_visualizer: WallApproachVisual::default(),
        }
    }
}

fn get_random_position_within_bounds(bounds: &Transform) -> Vec3 {
    let mut rng = rand::thread_rng();
    let half_scale = bounds.scale.abs() / 2.0; // Use absolute value to ensure positive scale
    let min = bounds.translation - half_scale;
    let max = bounds.translation + half_scale;

    Vec3::new(
        get_random_component(min.x, max.x, &mut rng),
        get_random_component(min.y, max.y, &mut rng),
        get_random_component(min.z, max.z, &mut rng), 
    )
}

fn get_random_component(min: f32, max: f32, rng: &mut impl Rng) -> f32 {
    if (max - min).abs() < f32::EPSILON {
        min // If the range is effectively zero, just return the min value
    } else {
        rng.gen_range(min.min(max)..=min.max(max)) // Ensure min is always less
                                                   // than max
    }
}

fn get_random_rotation() -> Quat {
    let mut rng = rand::thread_rng();
    Quat::from_euler(
        EulerRot::XYZ,
        rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI),
        rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI),
        rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI)
    )
}

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
pub enum ActorType {
    #[default]
    Missile,
    Nateroid,
    Spaceship,
}

impl fmt::Display for ActorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActorType::Missile => write!(f, "Missile"),
            ActorType::Nateroid => write!(f, "Nateroid"),
            ActorType::Spaceship => write!(f, "Spaceship"),
        }
    }
}

// Create configurations for different entity types
#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource, InspectorOptions)]
pub struct EnsembleConfig {
    pub(crate) missile:  ActorConfig,
    pub(crate) nateroid: ActorConfig,
    spaceship:           ActorConfig,
}

impl EnsembleConfig {
    pub fn get_actor_config(&self, actor_type: ActorType) -> &ActorConfig {
        match actor_type {
            ActorType::Spaceship => &self.spaceship,
            ActorType::Nateroid => &self.nateroid,
            ActorType::Missile => &self.missile,
        }
    }
}

fn initialize_ensemble_config(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    scenes: Res<Assets<Scene>>,
    scene_assets: Res<SceneAssets>,
) {
    let initial_config = InitialEnsembleConfig::default();

    let ensemble_config = EnsembleConfig {
        spaceship: initialize_actor_config(
            initial_config.spaceship,
            &scenes,
            &meshes,
            &scene_assets.spaceship,
        ),
        nateroid:  initialize_actor_config(
            initial_config.nateroid,
            &scenes,
            &meshes,
            &scene_assets.nateroid,
        ),
        missile:   initialize_actor_config(
            initial_config.missile,
            &scenes,
            &meshes,
            &scene_assets.missile,
        ),
    };

    commands.insert_resource(ensemble_config);
}

fn initialize_actor_config(
    mut config: ActorConfig,
    scenes: &Assets<Scene>,
    meshes: &Assets<Mesh>,
    scene_handle: &Handle<Scene>,
) -> ActorConfig {
    let aabb = get_scene_aabb(scenes, meshes, scene_handle);
    let adjusted_aabb = aabb.scale(BLENDER_SCALE);

    // Calculate the size based on the adjusted AABB
    let size = adjusted_aabb.size();
    let half_extents = adjusted_aabb.half_extents();

    let collider = match config.collider_type {
        ColliderType::Ball => {
            let radius = size.length() / 3.;
            Collider::ball(radius)
        },
        ColliderType::Cuboid => Collider::cuboid(half_extents.x, half_extents.y, half_extents.z),
    };

    let spawn_timer = config
        .spawn_timer_seconds
        .map(|seconds| Timer::from_seconds(seconds, TimerMode::Repeating));

    config.aabb = adjusted_aabb;
    config.collider = collider;
    config.spawn_timer = spawn_timer;
    config
}

pub fn spawn_actor(
    commands: &mut Commands,
    config: &ActorConfig,
    scene: Handle<Scene>,
    parent: Option<(Entity, &Transform, &Velocity)>,
    boundary: Res<Boundary>,
) {
    let bundle = ActorBundle::new(
        config,
        scene,
        parent.map(|(_, transform, velocity)| (transform, velocity)),
        boundary,
    );

    let entity = commands.spawn(bundle).id();

    name_entity(commands, entity, config.actor_type.to_string());
}
