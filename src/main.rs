use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::diagnostic::DiagnosticPlugin;
use crate::{
    asset_loader::AssetLoaderPlugin, camera::CameraPlugin,
    collision_detection::CollisionDetectionPlugin, despawn::DespawnPlugin, missile::MissilePlugin,
    movement::MovementPlugin, nateroid::Nateroid, physics::PhysicsPlugin, schedule::SchedulePlugin,
    spaceship::SpaceshipPlugin, state::StatePlugin, window::WindowPlugin,
};

// exclude when targeting wasm - this breaks in the browser right now
mod asset_loader;
mod camera;
mod collision_detection;
mod despawn;
#[cfg(not(target_arch = "wasm32"))]
mod diagnostic;
mod health;
mod missile;
mod movement;
mod nateroid;
mod physics;
mod schedule;
mod spaceship;
mod state;
mod utils;
mod window;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(CollisionDetectionPlugin)
        .add_plugins(DespawnPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(MissilePlugin)
        .add_plugins(Nateroid)
        .add_plugins(PhysicsPlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(SpaceshipPlugin)
        .add_plugins(StatePlugin)
        .add_plugins(WindowPlugin);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DiagnosticPlugin);
    app.run();
}
