use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

// exclude when targeting wasm - this breaks in the browser right now
#[cfg(not(target_arch = "wasm32"))]
mod asset_loader;
mod asteroid;
mod camera;
mod collision_detection;
mod despawn;
#[cfg(not(target_arch = "wasm32"))]
mod diagnostic;
mod health;
mod movement;
mod schedule;
mod spaceship;
mod state;
mod utils;

use crate::{
    asset_loader::AssetLoaderPlugin, asteroid::AsteroidPlugin, camera::CameraPlugin,
    collision_detection::CollisionDetectionPlugin, despawn::DespawnPlugin,
    diagnostic::DiagnosticPlugin, movement::MovementPlugin, schedule::SchedulePlugin,
    spaceship::SpaceshipPlugin, state::StatePlugin,
};

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.15)))
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 1000.0,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        // user defined
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(CollisionDetectionPlugin)
        .add_plugins(DespawnPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(AsteroidPlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(SpaceshipPlugin)
        .add_plugins(StatePlugin);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DiagnosticPlugin);

    app.run();
}
