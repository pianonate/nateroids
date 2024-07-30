use crate::schedule::InGameSet;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::Actionlike;

const CAMERA_DISTANCE: f32 = 80.0;
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_camera)
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_systems(Update, zoom_camera.in_set(InGameSet::UserInput))
            .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.15)))
            .insert_resource(AmbientLight {
                color: Color::default(),
                brightness: 1000.0,
            });
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    Zoom,
}

impl Actionlike for CameraMovement {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            CameraMovement::Zoom => InputControlKind::Axis,
        }
    }
}

#[derive(Component, Debug)]
pub struct PrimaryCamera;

fn spawn_camera(mut commands: Commands) {
    let input_map = InputMap::default()
        // This will capture the total continuous value, for direct use.
        .with_axis(CameraMovement::Zoom, MouseScrollAxis::Y);

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, CAMERA_DISTANCE, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        })
        .insert(InputManagerBundle::with_map(input_map))
        .insert(PrimaryCamera);
}

fn zoom_camera(
    window: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut event_writer: EventWriter<WindowResized>,
    mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<PrimaryCamera>>,
) {
    if let Ok((window_entity, window)) = window.get_single() {
        const CAMERA_ZOOM_RATE: f32 = 0.05;

        let (mut transform, action_state) = query.single_mut();
        // Here, we use the `action_value` method to extract the total net amount that the mouse wheel has travelled
        // Up and right axis movements are always positive by default
        let zoom_delta = action_state.value(&CameraMovement::Zoom);

        if zoom_delta == 0.0 {
            return;
        }

        let zoom_update = 1. - zoom_delta * CAMERA_ZOOM_RATE;

        transform.translation.y *= zoom_update;

        // to get the viewport properly updated with this different camera position
        // we need to hijack the resize where we also update the viewport
        event_writer.send(WindowResized {
            window: window_entity,
            width: window.width(),
            height: window.height(),
        });

        println!("zoom_delta {}", zoom_delta);
    }
}
