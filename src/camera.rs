use crate::stars::{StarsCamera, GAME_CAMERA_ORDER, GAME_LAYER};
use crate::{boundary::Boundary, input::CameraMovement, schedule::InGameSet};
use bevy::render::view::RenderLayers;
use bevy::{
    color::palettes::css,
    //core_pipeline::Skybox,
    prelude::{Color::Srgba, *},
};
use leafwing_input_manager::prelude::*;

const DEFAULT_CLEAR_COLOR_DARKENING_FACTOR: f32 = 0.019;
const DEFAULT_CLEAR_COLOR: Color = Srgba(css::MIDNIGHT_BLUE);
const DEFAULT_AMBIENT_LIGHT_BRIGHTNESS: f32 = 1_000.;

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct Appearance {
    color: Color,
    darkening_factor: f32,
    ambient_light_brightness: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Appearance {
            color: DEFAULT_CLEAR_COLOR,
            darkening_factor: DEFAULT_CLEAR_COLOR_DARKENING_FACTOR,
            ambient_light_brightness: DEFAULT_AMBIENT_LIGHT_BRIGHTNESS,
        })
        .insert_resource(ClearColor(
            DEFAULT_CLEAR_COLOR.darker(DEFAULT_CLEAR_COLOR_DARKENING_FACTOR),
        ))
        .insert_resource(AmbientLight {
            color: default(),
            brightness: 0.2,
        })
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, zoom_camera.in_set(InGameSet::UserInput))
        .add_systems(Update, update_clear_color.in_set(InGameSet::EntityUpdates));
    }
}

// this allows us to use Inspector reflection to manually update ClearColor to different values
// while the game is running from the ui_for_resources provided by bevy_inspector_egui
fn update_clear_color(
    app_clear_color: Res<Appearance>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    clear_color.0 = app_clear_color
        .color
        .darker(app_clear_color.darkening_factor);

    ambient_light.brightness = app_clear_color.ambient_light_brightness;
}

#[derive(Component, Debug)]
pub struct PrimaryCamera;

pub fn spawn_camera(
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut q_stars_camera: Query<(Entity, &mut Transform), With<StarsCamera>>,
) {
    let clear_color = Color::srgba(0., 0., 0., 0.);

    // we know we have one because we spawn the stars camera prior to this system
    // we're going to change its transform to zero and attach it to this as a child
    // so it always goes wherever we go
    let (stars_camera_entity, mut stars_camera_transform) =
        q_stars_camera.get_single_mut().unwrap();
    stars_camera_transform.translation = Vec3::ZERO;

    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    order: GAME_CAMERA_ORDER,
                    clear_color: ClearColorConfig::Custom(clear_color),
                    ..default()
                },

                transform: Transform::from_xyz(0.0, 0.0, boundary.transform.scale.z * 2.)
                    .looking_at(Vec3::ZERO, Vec3::Y),

                ..default()
            },
            // if you want to add a skybox on a level, you can do it here
            // Skybox {
            //     image: scene_assets.cubemap.image_handle.clone(),
            //     brightness: 1000.0,
            // },
        ))
        .insert(RenderLayers::layer(GAME_LAYER))
        .insert(InputManagerBundle::with_map(
            CameraMovement::camera_input_map(),
        ))
        .add_child(stars_camera_entity)
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
