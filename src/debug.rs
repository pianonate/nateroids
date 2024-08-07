use crate::{
    boundary::Boundary, camera::Appearance, config::GameConfig, input::GlobalAction,
    schedule::InGameSet,
};
use bevy::prelude::{IntoSystemConfigs, Reflect, Res, ResMut, Resource, *};
use bevy_inspector_egui::InspectorOptions;
use leafwing_input_manager::action_state::ActionState;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // puts us in debug mode which can be checked anywhere
            .init_resource::<DebugMode>()
            .init_resource::<InspectorMode>()
            .add_systems(Startup, register_debug_resources)
            .add_systems(Update, toggle_debug.in_set(InGameSet::UserInput))
            .add_systems(Update, toggle_inspector.in_set(InGameSet::UserInput));
    }
}

fn register_debug_resources(world: &mut World) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    type_registry.write().register::<Appearance>();
    type_registry.write().register::<Boundary>();
    type_registry.write().register::<DebugMode>();
    type_registry.write().register::<GameConfig>();
}

// the default bool is false, which is what we want
#[derive(Reflect, Resource, Default, InspectorOptions)]
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

pub fn inspector_mode_enabled(inspector_mode: Res<InspectorMode>) -> bool {
    inspector_mode.enabled
}

fn toggle_inspector(
    user_input: Res<ActionState<GlobalAction>>,
    mut inspector_mode: ResMut<InspectorMode>,
) {
    if user_input.just_pressed(&GlobalAction::Inspector) {
        inspector_mode.enabled = !inspector_mode.enabled;
        println!("InspectorMode: {}", inspector_mode.enabled);
    }
}
