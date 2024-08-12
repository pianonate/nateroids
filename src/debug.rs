use crate::{
    boundary::Boundary,
    collider_config::ColliderConfig,
    config::{
        AppearanceConfig,
        StarConfig,
    },
    input::GlobalAction,
    schedule::InGameSet,
};
use bevy::prelude::{
    IntoSystemConfigs,
    Reflect,
    Res,
    ResMut,
    Resource,
    *,
};
use leafwing_input_manager::action_state::ActionState;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // puts us in debug mode which can be checked anywhere
            .init_resource::<AabbMode>()
            .init_resource::<DebugMode>()
            .init_resource::<InspectorMode>()
            .add_systems(Startup, register_debug_resources)
            .add_systems(
                Update,
                (toggle_debug, toggle_inspector, toggle_aabb_mode).in_set(InGameSet::UserInput),
            );
    }
}

fn register_debug_resources(world: &mut World) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    type_registry.write().register::<AppearanceConfig>();
    type_registry.write().register::<Boundary>();
    type_registry.write().register::<DebugMode>();
    type_registry.write().register::<ColliderConfig>();
    type_registry.write().register::<StarConfig>();
}

// the default bool is false, which is what we want
#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct DebugMode {
    pub enabled: bool,
}

fn toggle_debug(user_input: Res<ActionState<GlobalAction>>, mut debug_mode: ResMut<DebugMode>) {
    if user_input.just_pressed(&GlobalAction::Debug) {
        debug_mode.enabled = !debug_mode.enabled;
        println!("DebugMode: {}", debug_mode.enabled);
    }
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct InspectorMode {
    pub enabled: bool,
}

pub fn inspector_mode_enabled(inspector_mode: Res<InspectorMode>) -> bool { inspector_mode.enabled }

fn toggle_inspector(
    user_input: Res<ActionState<GlobalAction>>,
    mut inspector_mode: ResMut<InspectorMode>,
) {
    if user_input.just_pressed(&GlobalAction::Inspector) {
        inspector_mode.enabled = !inspector_mode.enabled;
        println!("InspectorMode: {}", inspector_mode.enabled);
    }
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct AabbMode {
    pub enabled: bool,
}

pub fn aabb_mode_enabled(aabb_mode: Res<AabbMode>) -> bool { aabb_mode.enabled }

fn toggle_aabb_mode(user_input: Res<ActionState<GlobalAction>>, mut aabb_mode: ResMut<AabbMode>) {
    if user_input.just_pressed(&GlobalAction::AABBs) {
        aabb_mode.enabled = !aabb_mode.enabled;
        println!("AabbMode: {}", aabb_mode.enabled);
    }
}
