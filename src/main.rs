use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

// exclude when targeting wasm - this breaks in the browser right now
mod asset_loader;
mod camera;
mod collision_detection;
mod despawn;
#[cfg(not(target_arch = "wasm32"))]
mod diagnostic;
mod environment;
mod health;
mod movement;
mod nateroid;
mod physics;
mod schedule;
mod spaceship;
mod state;
mod utils;

use crate::{
    asset_loader::AssetLoaderPlugin, camera::CameraPlugin,
    collision_detection::CollisionDetectionPlugin, despawn::DespawnPlugin,
    environment::EnvironmentPlugin, movement::MovementPlugin,
    nateroid::Nateroid, physics::PhysicsPlugin, schedule::SchedulePlugin, spaceship::SpaceshipPlugin, state::StatePlugin,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::diagnostic::DiagnosticPlugin;


fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(CollisionDetectionPlugin)
        .add_plugins(DespawnPlugin)
        .add_plugins(EnvironmentPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(Nateroid)
        .add_plugins(PhysicsPlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(SpaceshipPlugin)
        .add_plugins(StatePlugin);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DiagnosticPlugin);

    app.run();
}
