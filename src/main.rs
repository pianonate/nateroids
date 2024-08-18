// exclude when targeting wasm - this breaks in the browser right now
mod actor;
mod asset_loader;
mod boundary;
mod camera;
mod collider_config;
mod collision_detection;
mod despawn;
mod health;
mod input;
mod orientation;
mod physics;
mod schedule;
mod splash;
mod state;
mod utils;

use crate::{
    actor::ActorPlugin,
    asset_loader::AssetLoaderPlugin,
    boundary::BoundaryPlugin,
    camera::CameraPlugin,
    collider_config::ColliderConfigPlugin,
    collision_detection::CollisionDetectionPlugin,
    despawn::DespawnPlugin,
    input::InputPlugin,
    orientation::OrientationPlugin,
    physics::PhysicsPlugin,
    schedule::SchedulePlugin,
    splash::SplashPlugin,
    state::StatePlugin,
};
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
use bevy::window::{
    PresentMode,
    WindowMode,
};

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

    // there's a limit to the tuple size so
    // i just split them in 2
    app.add_plugins((
        AssetLoaderPlugin,
        BoundaryPlugin,
        CameraPlugin,
        CollisionDetectionPlugin,
        ColliderConfigPlugin,
        ActorPlugin,
        DespawnPlugin,
        InputPlugin,
        OrientationPlugin,
        PhysicsPlugin,
        SchedulePlugin,
        SplashPlugin,
        StatePlugin,
    ))
    .run();
}
