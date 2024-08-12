use crate::asset_loader::SceneAssets;
use bevy::{
    asset::LoadState,
    prelude::*,
    render::mesh::{
        Mesh,
        VertexAttributeValues,
    },
    scene::Scene,
};
use bevy_rapier3d::prelude::Collider;

pub struct ColliderConfigPlugin;
impl Plugin for ColliderConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetsState>()
            .add_systems(
                Update,
                check_asset_loading.run_if(in_state(AssetsState::Loading)),
            )
            .add_systems(OnEnter(AssetsState::Loaded), extract_model_dimensions);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColliderType {
    Ball,
    Cuboid,
}

#[derive(Debug, Clone, Resource)]
struct InitialColliderConfig {
    missile:   InitialColliderConstant,
    nateroid:  InitialColliderConstant,
    spaceship: InitialColliderConstant,
}

#[derive(Debug, Clone)]
struct InitialColliderConstant {
    name:                &'static str,
    damage:              f32,
    health:              f32,
    scalar:              f32,
    spawn_timer_seconds: Option<f32>,
    spawnable:           bool,
    velocity:            f32,
    collider_type:       ColliderType,
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
    pub aabb:        Aabb,
    #[reflect(ignore)]
    pub collider:    Collider,
    pub damage:      f32,
    pub health:      f32,
    pub name:        &'static str,
    pub scalar:      f32,
    pub size:        Vec3,
    #[reflect(ignore)]
    pub spawn_timer: Option<Timer>,
    pub spawnable:   bool,
    pub velocity:    f32,
}

impl Default for InitialColliderConfig {
    fn default() -> Self {
        Self {
            missile:   InitialColliderConstant {
                collider_type:       ColliderType::Cuboid,
                damage:              50.,
                health:              1.,
                name:                "missile",
                scalar:              1.5,
                spawn_timer_seconds: Some(1.0 / 20.0),
                spawnable:           true,
                velocity:            85.,
            },
            nateroid:  InitialColliderConstant {
                collider_type:       ColliderType::Ball,
                damage:              10.,
                health:              50.,
                name:                "nateroid",
                scalar:              1.,
                spawn_timer_seconds: Some(2.),
                spawnable:           true,
                velocity:            30.,
            },
            spaceship: InitialColliderConstant {
                collider_type:       ColliderType::Cuboid,
                damage:              100.,
                health:              100.,
                name:                "spaceship",
                scalar:              0.8,
                spawn_timer_seconds: None,
                spawnable:           true,
                velocity:            60.,
            },
        }
    }
}

const BLENDER_SCALE: f32 = 100.;

impl InitialColliderConstant {
    fn initialize(&self, aabb: Aabb) -> ColliderConstant {
        let original_aabb = aabb;
        let adjusted_aabb = original_aabb.scale(BLENDER_SCALE);

        // Calculate the size based on the adjusted AABB
        let size = adjusted_aabb.size();

        let collider = match self.collider_type {
            ColliderType::Ball => {
                let radius = size.length() / 3.;
                println!("Creating Ball collider with radius: {}", radius);
                Collider::ball(radius)
            },
            ColliderType::Cuboid => {
                println!(
                    "Creating Cuboid collider with half extents: {:?}",
                    size / 2.0
                );
                Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
            },
        };

        println!(
            "Collider type after creation: {:?}",
            collider.raw.shape_type()
        );

        let spawn_timer = self
            .spawn_timer_seconds
            .map(|seconds| Timer::from_seconds(seconds, TimerMode::Repeating));

        ColliderConstant {
            aabb: adjusted_aabb,
            collider,
            damage: self.damage,
            health: self.health,
            name: self.name,
            scalar: self.scalar,
            size,
            spawn_timer,
            spawnable: self.spawnable,
            velocity: self.velocity,
        }
    }
}

pub fn extract_model_dimensions(
    mut commands: Commands,
    scenes: Res<Assets<Scene>>,
    meshes: Res<Assets<Mesh>>,
    scene_assets: Res<SceneAssets>,
) {
    let initial_config = InitialColliderConfig::default();

    let collider_config = ColliderConfig {
        missile:   initial_config.missile.initialize(get_scene_aabb(
            &scenes,
            &meshes,
            &scene_assets.missiles,
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

#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    //pub fn new(min: Vec3, max: Vec3) -> Self { Self { min, max } }

    //pub fn half_extents(&self) -> Vec3 { (self.max - self.min) * 0.5 }

    pub fn size(&self) -> Vec3 { self.max - self.min }

    pub fn center(&self) -> Vec3 { (self.min + self.max) / 2.0 }

    pub fn scale(&self, scale: f32) -> Self {
        Self {
            min: self.min * scale,
            max: self.max * scale,
        }
    }
}

pub fn check_asset_loading(
    mut next_state: ResMut<NextState<AssetsState>>,
    asset_server: Res<AssetServer>,
    scene_assets: Res<SceneAssets>,
) {
    let missile_loaded =
        asset_server.get_load_state(scene_assets.missiles.id()) == Some(LoadState::Loaded);
    let nateroid_loaded =
        asset_server.get_load_state(scene_assets.nateroid.id()) == Some(LoadState::Loaded);
    let spaceship_loaded =
        asset_server.get_load_state(scene_assets.spaceship.id()) == Some(LoadState::Loaded);

    if missile_loaded && nateroid_loaded && spaceship_loaded {
        // println!("All assets loaded, transitioning to Loaded state");
        next_state.set(AssetsState::Loaded);
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AssetsState {
    #[default]
    Loading,
    Loaded,
}

fn get_scene_aabb(scenes: &Assets<Scene>, meshes: &Assets<Mesh>, handle: &Handle<Scene>) -> Aabb {
    if let Some(scene) = scenes.get(handle) {
        let mut aabb = None;
        for entity in scene.world.iter_entities() {
            if let Some(mesh_handle) = entity.get::<Handle<Mesh>>() {
                if let Some(mesh) = meshes.get(mesh_handle) {
                    let mesh_aabb = get_mesh_aabb(mesh);
                    aabb = Some(match aabb {
                        Some(existing) => combine_aabb(existing, mesh_aabb),
                        None => mesh_aabb,
                    });
                }
            }
        }
        aabb.unwrap_or(Aabb {
            min: Vec3::ZERO,
            max: Vec3::ONE,
        })
    } else {
        Aabb {
            min: Vec3::ZERO,
            max: Vec3::ONE,
        }
    }
}

fn get_mesh_aabb(mesh: &Mesh) -> Aabb {
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for position in positions.iter() {
            min = min.min(Vec3::from(*position));
            max = max.max(Vec3::from(*position));
        }
        Aabb { min, max }
    } else {
        // Default to a unit cube if no vertex data is found
        Aabb {
            min: Vec3::splat(-0.5),
            max: Vec3::splat(0.5),
        }
    }
}

fn combine_aabb(a: Aabb, b: Aabb) -> Aabb {
    Aabb {
        min: a.min.min(b.min),
        max: a.max.max(b.max),
    }
}
