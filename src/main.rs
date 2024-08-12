use crate::{
    asset_loader::AssetLoaderPlugin,
    boundary::BoundaryPlugin,
    camera::CameraPlugin,
    collider_config::ColliderConfigPlugin,
    collision_detection::CollisionDetectionPlugin,
    config::ConfigPlugin,
    despawn::DespawnPlugin,
    input::InputPlugin,
    missile::MissilePlugin,
    movement::MovementPlugin,
    orientation::OrientationPlugin,
    nateroid::NateroidPlugin,
    physics::PhysicsPlugin,
    schedule::SchedulePlugin,
    spaceship::SpaceshipPlugin,
    splash::SplashPlugin,
    star_twinkling::StarTwinklingPlugin,
    stars::StarsPlugin,
    state::StatePlugin,
};
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
use bevy::window::{
    PresentMode,
    WindowMode,
};

use crate::{
    debug::DebugPlugin,
    inspector::InspectorPlugin,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::diagnostic::DiagnosticPlugin;

mod debug;
mod inspector;

#[cfg(not(target_arch = "wasm32"))]
mod diagnostic;

// exclude when targeting wasm - this breaks in the browser right now
mod asset_loader;
mod boundary;
mod camera;
mod collider_config;
mod collision_detection;
mod config;
mod despawn;
mod health;
mod input;
mod missile;
mod movement;
mod nateroid;
mod orientation;
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

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync, // Reduces input lag.
                    mode: WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }),
    );

    app.add_plugins(AssetLoaderPlugin)
        .add_plugins(BoundaryPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(CollisionDetectionPlugin)
        .add_plugins(ConfigPlugin)
        .add_plugins(ColliderConfigPlugin)
        .add_plugins(DespawnPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(MissilePlugin)
        .add_plugins(NateroidPlugin)
        .add_plugins(OrientationPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(SpaceshipPlugin)
        .add_plugins(SplashPlugin)
        .add_plugins(StarsPlugin)
        .add_plugins(StarTwinklingPlugin)
        .add_plugins(StatePlugin);

    app.add_plugins(InspectorPlugin).add_plugins(DebugPlugin);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DiagnosticPlugin);

    app.run();
}
