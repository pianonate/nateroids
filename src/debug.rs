use crate::{
    boundary::{
        Boundary,
        PlaneConfig,
    },
    camera::{
        LightConfig,
        StarConfig,
    },
    collider_config::ColliderConfig,
    config::AppearanceConfig,
    orientation::{
        CameraOrientation,
        OrientationConfig,
    },
};
use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // puts us in debug mode which can be checked anywhere
            .add_systems(Startup, register_inspector_resources);
    }
}

fn register_inspector_resources(world: &mut World) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    type_registry.write().register::<AppearanceConfig>();
    type_registry.write().register::<Boundary>();
    type_registry.write().register::<CameraOrientation>();
    type_registry.write().register::<ColliderConfig>();
    type_registry.write().register::<LightConfig>();
    type_registry.write().register::<OrientationConfig>();
    type_registry.write().register::<PlaneConfig>();
    type_registry.write().register::<StarConfig>();
}
