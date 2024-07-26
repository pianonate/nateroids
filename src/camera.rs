use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

const CAMERA_DISTANCE: f32 = 80.0;
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_camera)
            .add_systems(Update, zoom_camera)
            .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.15)))
            .insert_resource(AmbientLight {
                color: Color::default(),
                brightness: 1000.0,
            });
    }
}

#[derive(Component, Debug)]
pub struct PrimaryCamera;

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, CAMERA_DISTANCE, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        })
        .insert(PrimaryCamera);
}

fn zoom_camera(
    mut query: Query<&mut Transform, With<PrimaryCamera>>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
) {
    for event in mouse_wheel_reader.read() {
        // scale the event magnitude per pixel or per line
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 53.0,
        };

        let mut transform = query.single_mut();
        transform.translation.y -= scroll_amount;
    }
}
