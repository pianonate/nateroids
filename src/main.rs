use bevy::prelude::*;

use crate::{
    // comment to force cargo fmt to output these with a newline
    asset_loader::AssetLoaderPlugin,
    boundary::BoundaryPlugin,
    camera::CameraPlugin,
    collision_detection::CollisionDetectionPlugin,
    config::ConfigPlugin,
    despawn::DespawnPlugin,
    input::InputPlugin,
    missile::MissilePlugin,
    movement::MovementPlugin,
    nateroid::Nateroid,
    physics::PhysicsPlugin,
    schedule::SchedulePlugin,
    spaceship::SpaceshipPlugin,
    splash::SplashPlugin,
    star_twinkling::StarTwinklingPlugin,
    stars::StarsPlugin,
    state::StatePlugin,
};

#[cfg(debug_assertions)]
use crate::{debug::DebugPlugin, inspector::InspectorPlugin};

//#[cfg(debug_assertions)]
use crate::diagnostic::DiagnosticPlugin;

#[cfg(debug_assertions)]
mod debug;

// #[cfg(debug_assertions)]
mod diagnostic;
#[cfg(debug_assertions)]
mod inspector;

// exclude when targeting wasm - this breaks in the browser right now
mod asset_loader;
mod boundary;
mod camera;
mod collision_detection;
mod config;
mod despawn;
mod health;
mod input;
mod missile;
mod movement;
mod nateroid;
mod physics;
mod schedule;
mod spaceship;
mod splash;
mod star_twinkling;
mod stars;
mod state;
mod utils;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(BoundaryPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(CollisionDetectionPlugin)
        .add_plugins(ConfigPlugin)
        .add_plugins(DespawnPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(MissilePlugin)
        .add_plugins(Nateroid)
        .add_plugins(PhysicsPlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(SpaceshipPlugin)
        .add_plugins(SplashPlugin)
        .add_plugins(StarsPlugin)
        .add_plugins(StarTwinklingPlugin)
        .add_plugins(StatePlugin);

    #[cfg(debug_assertions)]
    app.add_plugins(InspectorPlugin).add_plugins(DebugPlugin);

    // #[cfg(debug_assertions)]
    app.add_plugins(DiagnosticPlugin);

    app.run();
}
