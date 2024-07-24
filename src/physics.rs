/// let's use just load assets once, amigos
use bevy::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin    {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default());
    }
}
