use crate::boundary::Boundary;
use crate::{input::CameraMovement, schedule::InGameSet};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_camera)
            .add_systems(Update, zoom_camera.in_set(InGameSet::UserInput))
            .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.15)))
            .insert_resource(AmbientLight {
                color: Color::default(),
                brightness: 1000.0,
            });
    }
}

#[derive(Component, Debug)]
pub struct PrimaryCamera;

fn spawn_camera(mut commands: Commands, boundary: Res<Boundary>) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, boundary.transform.scale.z * 2.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(InputManagerBundle::with_map(
            CameraMovement::camera_input_map(),
        ))
        .insert(PrimaryCamera);
}

fn zoom_camera(
    mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<PrimaryCamera>>,
) {
    if let Ok((mut transform, action_state)) = query.get_single_mut() {
        // Here, we use the `action_value` method to extract the total net amount that the mouse wheel has travelled
        // Up and right axis movements are always positive by default
        let zoom_delta = action_state.value(&CameraMovement::Zoom);

        if zoom_delta == 0.0 {
            return;
        }

        let zoom_update = 1. - zoom_delta;

        transform.translation.z *= zoom_update;

        println!("zoom_delta {}", zoom_delta);
    }
}
