use crate::{
    boundary::Boundary,
    camera::StarConfig,
    collider_config::ColliderConfig,
    config::AppearanceConfig,
    input::GlobalAction,
    orientation::{
        CameraOrientation,
        OrientationConfig,
    },
};
use bevy::prelude::{
    Reflect,
    Res,
    ResMut,
    Resource,
    *,
};
use bevy_rapier3d::prelude::DebugRenderContext;
use leafwing_input_manager::action_state::ActionState;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // puts us in debug mode which can be checked anywhere
            .init_resource::<DebugMode>()
            .add_systems(Startup, register_debug_resources)
            .add_systems(
                Update,
                (
                    toggle_aabb_mode,
                    toggle_debug,
                    toggle_inspector,
                    toggle_physics_debug,
                ),
            );
    }
}

fn register_debug_resources(world: &mut World) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    type_registry.write().register::<AppearanceConfig>();
    type_registry.write().register::<Boundary>();
    type_registry.write().register::<DebugMode>();
    type_registry.write().register::<ColliderConfig>();
    type_registry.write().register::<OrientationConfig>();
    type_registry.write().register::<CameraOrientation>();
    type_registry.write().register::<StarConfig>();
}

// the default bool is false, which is what we want
#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct DebugMode {
    pub aabb_enabled:      bool,
    pub debug_enabled:     bool,
    pub inspector_enabled: bool,
}

fn toggle_debug(user_input: Res<ActionState<GlobalAction>>, mut debug_mode: ResMut<DebugMode>) {
    if user_input.just_pressed(&GlobalAction::Debug) {
        debug_mode.debug_enabled = !debug_mode.debug_enabled;
        println!("DebugMode: {}", debug_mode.debug_enabled);
    }
}

pub fn inspector_mode_enabled(debug_mode: Res<DebugMode>) -> bool { debug_mode.inspector_enabled }

fn toggle_inspector(user_input: Res<ActionState<GlobalAction>>, mut debug_mode: ResMut<DebugMode>) {
    if user_input.just_pressed(&GlobalAction::Inspector) {
        debug_mode.inspector_enabled = !debug_mode.inspector_enabled;
        println!("InspectorMode: {}", debug_mode.inspector_enabled);
    }
}

pub fn aabb_mode_enabled(debug_mode: Res<DebugMode>) -> bool { debug_mode.aabb_enabled }

fn toggle_aabb_mode(user_input: Res<ActionState<GlobalAction>>, mut debug_mode: ResMut<DebugMode>) {
    if user_input.just_pressed(&GlobalAction::AABBs) {
        debug_mode.aabb_enabled = !debug_mode.aabb_enabled;
        println!("AabbMode: {}", debug_mode.aabb_enabled);
    }
}

// this is the only one of our debug modes that maintains a separate enabled
// resource
fn toggle_physics_debug(
    user_input: Res<ActionState<GlobalAction>>,
    mut rapier_debug: ResMut<DebugRenderContext>,
) {
    if user_input.just_pressed(&GlobalAction::Physics) {
        rapier_debug.enabled = !rapier_debug.enabled;
        println!("DebugMode: {}", rapier_debug.enabled);
    }
}
