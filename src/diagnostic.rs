//! This example shows the simplest way to create a Perf UI.
//! (using defaults for everything)
//! note this doesn't work with the wasm target

use crate::input::{
    GlobalAction,
    GlobalAction::Diagnostics,
};
use bevy::prelude::*;
use iyes_perf_ui::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub struct DiagnosticPlugin;

impl Plugin for DiagnosticPlugin {
    fn build(&self, app: &mut App) {
        app
            // we want Bevy to measure these values for us:
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
            .add_plugins(PerfUiPlugin)
            // We need to order our system before PerfUiSet::Setup,
            // so that iyes_perf_ui can process any new Perf UI in the same
            // frame as we spawn the entities. Otherwise, Bevy UI will complain.
            .add_systems(
                Update,
                toggle_diagnostics.before(iyes_perf_ui::PerfUiSet::Setup),
            );
    }
}

fn toggle_diagnostics(
    mut commands: Commands,
    q_root: Query<Entity, With<PerfUiRoot>>,
    user_input: Res<ActionState<GlobalAction>>,
) {
    if user_input.just_pressed(&Diagnostics) {
        if let Ok(e) = q_root.get_single() {
            // despawn the existing Perf UI
            commands.entity(e).despawn_recursive();
        } else {
            // create a simple Perf UI with default settings
            // and all entries provided by the crate:
            commands.spawn(PerfUiCompleteBundle::default());
        }
    }
}
