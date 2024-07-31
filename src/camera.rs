use crate::boundary::Boundary;
use crate::{input::CameraMovement, schedule::InGameSet};
use bevy::color::palettes::css;
use bevy::prelude::Color::Srgba;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

const DEFAULT_CLEAR_COLOR_DARKENING_FACTOR: f32 = 0.019;
const DEFAULT_CLEAR_COLOR: Color = Srgba(css::MIDNIGHT_BLUE);

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct AppClearColor {
    color: Color,
    darkening_factor: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_camera)
            .add_systems(Update, zoom_camera.in_set(InGameSet::UserInput))
            .insert_resource(AppClearColor {
                color: DEFAULT_CLEAR_COLOR,
                darkening_factor: DEFAULT_CLEAR_COLOR_DARKENING_FACTOR,
            })
            .insert_resource(ClearColor(
                DEFAULT_CLEAR_COLOR.darker(DEFAULT_CLEAR_COLOR_DARKENING_FACTOR),
            ))
            .insert_resource(AmbientLight {
                color: Color::default(),
                brightness: 1000.0,
            })
            .add_systems(Update, update_clear_color);
    }
}

fn update_clear_color(app_clear_color: Res<AppClearColor>, mut clear_color: ResMut<ClearColor>) {
    clear_color.0 = app_clear_color
        .color
        .darker(app_clear_color.darkening_factor);
}

#[derive(Component, Debug)]
pub struct PrimaryCamera;

fn spawn_camera(mut commands: Commands, boundary: Res<Boundary>) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, boundary.transform.scale.z * 2.0)
                //transform: Transform::from_xyz(0.0, 5.0, -20.0) // -boundary.transform.scale.z) //
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
