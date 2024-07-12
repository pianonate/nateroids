use bevy::prelude::*;

mod asset_loader;
mod asteroids;
mod camera;
mod collision_detection;
//mod debug;
mod despawn;
mod movement;
mod spaceship;
mod schedule;
mod state;
mod health;


use asset_loader::AssetLoaderPlugin;
use asteroids::AsteroidsPlugin;
use camera::CameraPlugin;
use collision_detection::CollisionDetectionPlugin;
//use debug::DebugPlugin;
use despawn::DespawnPlugin;
use movement::MovementPlugin;
use spaceship::SpaceshipPlugin;
use schedule::SchedulePlugin;
use state::StatePlugin;

fn main() {


    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.15)))
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 1000.0,
        })
        .add_plugins(DefaultPlugins)
        // user defined
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(CollisionDetectionPlugin)
        .add_plugins(MovementPlugin)
        // .add_plugins(DebugPlugin)
        .add_plugins(SpaceshipPlugin)
        .add_plugins(AsteroidsPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(DespawnPlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(StatePlugin)
        .run();
}
