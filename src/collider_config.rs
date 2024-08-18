use crate::{
    actor::{
        get_scene_aabb,
        Aabb,
        ColliderType,
    },
    asset_loader::{
        AssetsState,
        SceneAssets,
    },
};
use bevy::{
    prelude::*,
    render::mesh::Mesh,
    scene::Scene,
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule,
    Collider,
    ColliderMassProperties,
    ColliderMassProperties::Mass,
    Restitution,
};

// todo: #bevyquestion - where should this go
const BLENDER_SCALE: f32 = 100.;

pub struct ColliderConfigPlugin;
impl Plugin for ColliderConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetsState::Loaded), initialize_configuration);
    }
}

#[derive(Debug, Clone, Resource)]
struct InitialColliderConfig {
    missile:   InitialColliderConstant,
    nateroid:  InitialColliderConstant,
    spaceship: InitialColliderConstant,
}

#[derive(Debug, Clone)]
struct InitialColliderConstant {
    acceleration:   Option<f32>,
    angvel:         Option<f32>,
    collider_type:  ColliderType,
    damage:         f32,
    health:         f32,
    mass:           ColliderMassProperties,
    name:           &'static str,
    restitution:    f32,
    rotation_speed: f32,
    scalar:         f32,
    spawn_point:    Vec3,
    spawnable:      bool,
    velocity:       f32,
}

impl Default for InitialColliderConfig {
    fn default() -> Self {
        Self {
            missile:   InitialColliderConstant {
                acceleration:   None,
                angvel:         None,
                collider_type:  ColliderType::Cuboid,
                damage:         50.,
                health:         1.,
                mass:           Mass(0.001),
                name:           "missile",
                restitution:    0.,
                rotation_speed: 0.,
                scalar:         2.5,
                spawn_point:    Vec3::new(0.5, 0., 0.),
                spawnable:      true,
                velocity:       85.,
            },
            nateroid:  InitialColliderConstant {
                acceleration:   None,
                angvel:         Some(4.),
                collider_type:  ColliderType::Ball,
                damage:         10.,
                health:         200.,
                mass:           Mass(1.0),
                name:           "nateroid",
                restitution:    1.0,
                rotation_speed: 0.,
                scalar:         1.,
                spawn_point:    Vec3::ZERO,
                spawnable:      true,
                velocity:       30.,
            },
            spaceship: InitialColliderConstant {
                acceleration:   Some(60.),
                angvel:         None,
                collider_type:  ColliderType::Cuboid,
                damage:         50.,
                health:         500.,
                mass:           Mass(3.0),
                name:           "spaceship",
                restitution:    0.3,
                rotation_speed: 5.,
                scalar:         0.8,
                spawn_point:    Vec3::new(0.0, -20.0, 0.0),
                spawnable:      true,
                velocity:       80.,
            },
        }
    }
}

impl InitialColliderConstant {
    fn initialize(&self, aabb: Aabb) -> ColliderConstant {
        let original_aabb = aabb;
        let adjusted_aabb = original_aabb.scale(BLENDER_SCALE);

        // Calculate the size based on the adjusted AABB
        let size = adjusted_aabb.size();
        let half_extents = adjusted_aabb.half_extents();

        let collider = match self.collider_type {
            ColliderType::Ball => {
                let radius = size.length() / 3.;
                Collider::ball(radius)
            },
            ColliderType::Cuboid => {
                Collider::cuboid(half_extents.x, half_extents.y, half_extents.z)
            },
        };

        let restitution = Restitution {
            coefficient:  self.restitution,
            combine_rule: CoefficientCombineRule::Min,
        };

        ColliderConstant {
            aabb: adjusted_aabb,
            acceleration: self.acceleration,
            angular_velocity: self.angvel,
            collider,
            damage: self.damage,
            health: self.health,
            mass: self.mass,
            name: self.name.to_string(),
            restitution,
            rotation_speed: self.rotation_speed,
            scalar: self.scalar,
            spawn_point: self.spawn_point,
            spawnable: self.spawnable,
            linear_velocity: self.velocity,
        }
    }
}
#[derive(Debug, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct ColliderConfig {
    pub missile:   ColliderConstant,
    pub nateroid:  ColliderConstant,
    pub spaceship: ColliderConstant,
}

#[derive(Debug, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct ColliderConstant {
    pub aabb:             Aabb,
    pub acceleration:     Option<f32>,
    pub angular_velocity: Option<f32>,
    #[reflect(ignore)]
    pub collider:         Collider,
    pub damage:           f32,
    pub health:           f32,
    pub linear_velocity:  f32,
    pub mass:             ColliderMassProperties,
    pub name:             String,
    #[reflect(ignore)]
    pub restitution:      Restitution,
    pub rotation_speed:   f32,
    pub scalar:           f32,
    pub spawn_point:      Vec3,
    #[reflect(ignore)]
    pub spawnable:        bool,
}

impl ColliderConstant {
    pub fn get_forward_spawn_point(
        &self,
        spaceship_transform: Transform,
        spaceship_aabb: &Aabb,
    ) -> Vec3 {
        // Step 1: Determine the forward vector of the box in world space
        let forward = -spaceship_transform.forward();

        // Step 2: Get the half extents of the AABB
        let half_extents = spaceship_aabb.half_extents();

        // Step 3: Transform the half extents to world space
        let world_half_extents =
            spaceship_transform.rotation * (half_extents * spaceship_transform.scale);

        // Step 4: Project the world half extents onto the forward vector
        let forward_extent = forward.dot(world_half_extents);

        // Step 5: Compute the point on the edge of the box in the forward direction + a
        // buffer from the missile         we're overloading the spawn_point
        // from the missile as it is not otherwise used
        spaceship_transform.translation + forward * (forward_extent + self.spawn_point.length())
    }
}

fn initialize_configuration(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    scenes: Res<Assets<Scene>>,
    scene_assets: Res<SceneAssets>,
) {
    let initial_config = InitialColliderConfig::default();

    let collider_config = ColliderConfig {
        missile:   initial_config.missile.initialize(get_scene_aabb(
            &scenes,
            &meshes,
            &scene_assets.missile,
        )),
        nateroid:  initial_config.nateroid.initialize(get_scene_aabb(
            &scenes,
            &meshes,
            &scene_assets.nateroid,
        )),
        spaceship: initial_config.spaceship.initialize(get_scene_aabb(
            &scenes,
            &meshes,
            &scene_assets.spaceship,
        )),
    };

    commands.insert_resource(collider_config);
}
