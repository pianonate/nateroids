//! This example shows the simplest way to create a Perf UI.
//! (using defaults for everything)

use bevy::diagnostic::Diagnostic;
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
            //.add_systems(Startup, setup)
            // We need to order our system before PerfUiSet::Setup,
            // so that iyes_perf_ui can process any new Perf UI in the same
            // frame as we spawn the entities. Otherwise, Bevy UI will complain.
            .add_systems(Update, toggle.before(iyes_perf_ui::PerfUiSet::Setup));
    }
}

#[derive(Component, Debug)]
pub struct DiagnosticCamera;

fn setup(mut commands: Commands) {
    // spawn a camera to be able to see anything
    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        })
        .insert(DiagnosticCamera);
}

fn toggle(
    mut commands: Commands,
    q_root: Query<Entity, With<PerfUiRoot>>,
    mut q_camera: Query<&mut Camera, With<DiagnosticCamera>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        if let Ok(e) = q_root.get_single() {
            // despawn the existing Perf UI
            commands.entity(e).despawn_recursive();
            if let Ok(mut camera) = q_camera.get_single_mut() {
                camera.is_active = false;
            }
        } else {
            // create a simple Perf UI with default settings
            // and all entries provided by the crate:
            commands.spawn(PerfUiCompleteBundle::default());

            // with both cameras enabled - gizmos drawn to the screen get rendered
            // by both cameras
            // #todo #bevyquestion: is there a better way to handle this?
            if let Ok(mut camera) = q_camera.get_single_mut() {
                camera.is_active = true;
            }
        }
    }
}
