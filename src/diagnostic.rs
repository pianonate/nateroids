//! This example shows the simplest way to create a Perf UI.
//! (using defaults for everything)
use bevy::prelude::*;
use iyes_perf_ui::prelude::*;

pub struct DiagnosticPlugin;

impl Plugin for DiagnosticPlugin {
    fn build(&self, app: &mut App) {
        app
            // we want Bevy to measure these values for us:
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
            .add_plugins(PerfUiPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    // spawn a camera to be able to see anything
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        ..default()
    });

    // create a simple Perf UI with default settings
    // and all entries provided by the crate:
    commands.spawn(PerfUiCompleteBundle::default());
}
