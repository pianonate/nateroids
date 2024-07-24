/// let's use just load assets once, amigos
use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::{KeyCode::F10, *},
};

use bevy_rapier3d::prelude::{
    DebugRenderContext, NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin,
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, debug_mode_off)
            .add_systems(Update, debug_mode.run_if(input_just_pressed(F10)));
    }
}
fn debug_mode_off(mut rapier_debug: ResMut<DebugRenderContext>) {
    rapier_debug.enabled = false;
}

fn debug_mode(mut rapier_debug: ResMut<DebugRenderContext>) {
    rapier_debug.enabled = !rapier_debug.enabled;
}
