use bevy::app::{App, Plugin, Update};
use bevy::prelude::{IntoSystemConfigs, Reflect, Res, ResMut, Resource};
use bevy_inspector_egui::{prelude::*, InspectorOptions};
use leafwing_input_manager::action_state::ActionState;

use crate::{input::GlobalAction, schedule::InGameSet};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // puts us in debug mode which can be checked anywhere
            .init_resource::<DebugMode>()
            .init_resource::<InspectorMode>()
            .add_systems(Update, toggle_debug.in_set(InGameSet::UserInput))
            .add_systems(Update, toggle_inspector.in_set(InGameSet::UserInput));
    }
}

// the default bool is false, which is what we want
#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct DebugMode {
    pub enabled: bool,
}

fn toggle_debug(user_input: Res<ActionState<GlobalAction>>, mut debug_mode: ResMut<DebugMode>) {
    if user_input.just_pressed(&GlobalAction::Debug) {
        debug_mode.enabled = !debug_mode.enabled;
        println!("DebugMode: {}", debug_mode.enabled);
    }
}

#[derive(Resource, Debug, Default)]
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
