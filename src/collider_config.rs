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
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::prelude::Collider;

pub struct ColliderConfigPlugin;
impl Plugin for ColliderConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetsState>()
            .init_resource::<ColliderConfig>()
            .add_systems(
                Update,
                check_asset_loading.run_if(in_state(AssetsState::Loading)),
            )
            .add_systems(OnEnter(AssetsState::Loaded), extract_model_dimensions);
        //.add_systems(Update, debug_model_dimensions);
    }
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct ColliderConfig {
    pub missile:   ColliderConstant,
    pub nateroid:  ColliderConstant,
    pub spaceship: ColliderConstant,
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct ColliderConstant {
    pub name:                &'static str,
    pub damage:              f32,
    pub health:              f32,
   // pub radius:              f32,
    pub scalar:              f32,
    pub spawn_timer_seconds: Option<f32>,
    pub spawnable:           bool,
    pub velocity:            f32,
}

// these scales were set by eye-balling the game
// if you get different assets these will likely need to change
// to match the assets size
impl Default for ColliderConfig {
    fn default() -> Self {
        Self {
            missile:   ColliderConstant {
                name:                "missile",
                damage:              50.,
                health:              1.,
                scalar:              1.5,
                spawn_timer_seconds: Some(1.0 / 20.0),
                spawnable:           true,
                velocity:            85.,
            },
            nateroid:  ColliderConstant {
                name:                "nateroid",
                damage:              10.,
                health:              50.,
                scalar:              2.,
                spawn_timer_seconds: Some(2.),
                spawnable:           true,
                velocity:            30.,
            },
            spaceship: ColliderConstant {
                name:                "spaceship",
                damage:              100.,
                health:              100.,
                scalar:              0.8,
                spawn_timer_seconds: None,
                spawnable:           true,
                velocity:            60.,
            },
        }
    }
}

// nateroid flags as dead - i think because it's only accessed through a SystemParam, FireResources
// used to reduce the param list for missile fire systems
// you don't need to be warned about this instance
#[allow(dead_code)] 
#[derive(Resource, Debug)]
pub struct ModelDimensions {
    pub missile:   ModelDimension,
    pub nateroid:  ModelDimension,
    pub spaceship: ModelDimension,
}

const BLENDER_SCALE: f32 = 100.0;

#[derive(Debug, Clone)]
pub struct ModelDimension {
    pub min:           Vec3,
    pub max:           Vec3,
    pub size:          Vec3,
    pub center_offset: Vec3,
    pub cuboid:        Collider,
    pub sphere:        Collider,
}

impl ModelDimension {
    fn new(aabb: Aabb, collider_scalar: f32) -> Self {
        println!("{:?}", aabb);

        // Apply scale factor and remap axes
        let min = Vec3::new(
            aabb.min.x * BLENDER_SCALE,
            aabb.min.y * BLENDER_SCALE,
            aabb.min.z * BLENDER_SCALE,
        );
        let max = Vec3::new(
            aabb.max.x * BLENDER_SCALE,
            aabb.max.y * BLENDER_SCALE,
            aabb.max.z * BLENDER_SCALE,
        );

        let size = max - min;

       let center_offset = (min + max) / 2.0;
        let scaled_dimensions = size * collider_scalar;

        // Create a cuboid collider with an offset
        let cuboid = Collider::cuboid(
            scaled_dimensions.x / 2.0,
            scaled_dimensions.y / 2.0,
            scaled_dimensions.z / 2.0,
        );

        let sphere = Collider::ball(scaled_dimensions.length());

        println!(
            "size: {:?} sphere radius: {:?} ",
            size,
            scaled_dimensions.length()
        );

        Self {
            min,
            max,
            size,
            center_offset,
            cuboid,
            sphere,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ONE,
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

pub fn extract_model_dimensions(
    mut commands: Commands,
    scenes: Res<Assets<Scene>>,
    meshes: Res<Assets<Mesh>>,
    scene_assets: Res<SceneAssets>,
    collider_config: Res<ColliderConfig>,
) {
    let dimensions = ModelDimensions {
        missile:   ModelDimension::new(
            get_scene_aabb(&scenes, &meshes, &scene_assets.missiles),
            collider_config.missile.scalar,
        ),
        nateroid:  ModelDimension::new(
            get_scene_aabb(&scenes, &meshes, &scene_assets.nateroid),
            collider_config.nateroid.scalar,
        ),
        spaceship: ModelDimension::new(
            get_scene_aabb(&scenes, &meshes, &scene_assets.spaceship),
            collider_config.spaceship.scalar,
        ),
    };

    commands.insert_resource(dimensions);
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
