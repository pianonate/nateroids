use bevy::prelude::*;

use crate::{
    asset_loader::AssetLoaderPlugin, boundary::BoundaryPlugin, camera::CameraPlugin,
    collision_detection::CollisionDetectionPlugin, despawn::DespawnPlugin,
    game_scale::GameScalePlugin, input::InputPlugin, missile::MissilePlugin,
    movement::MovementPlugin, nateroid::Nateroid, physics::PhysicsPlugin, schedule::SchedulePlugin,
    spaceship::SpaceshipPlugin, state::StatePlugin,
};

#[cfg(debug_assertions)]
use crate::{debug::DebugPlugin, diagnostic::DiagnosticPlugin, inspector::InspectorPlugin};

#[cfg(debug_assertions)]
mod debug;
#[cfg(debug_assertions)]
mod diagnostic;
#[cfg(debug_assertions)]
mod inspector;

// exclude when targeting wasm - this breaks in the browser right now
mod asset_loader;
mod boundary;
mod camera;
mod collision_detection;
mod despawn;
mod game_scale;
mod health;
mod input;
mod missile;
mod movement;
mod nateroid;
mod physics;
mod schedule;
mod spaceship;
mod state;
mod utils;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(BoundaryPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(CollisionDetectionPlugin)
        .add_plugins(DespawnPlugin)
        .add_plugins(GameScalePlugin)
        .add_plugins(InputPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(MissilePlugin)
        .add_plugins(Nateroid)
        .add_plugins(PhysicsPlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(SpaceshipPlugin)
        .add_plugins(StatePlugin);

    #[cfg(debug_assertions)]
    app.add_plugins(InspectorPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(DiagnosticPlugin);

    app.run();
}
