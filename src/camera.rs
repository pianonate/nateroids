use crate::{boundary::Boundary, input::CameraMovement, schedule::InGameSet};
use bevy::asset::LoadState;
use bevy::core_pipeline::Skybox;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};
use bevy::render::texture::CompressedImageFormats;
use bevy::{
    color::palettes::css,
    prelude::{Color::Srgba, *},
};
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
        app.insert_resource(AppClearColor {
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
        .add_systems(PreStartup, spawn_camera)
        .add_systems(Update, zoom_camera.in_set(InGameSet::UserInput))
        .add_systems(
            Update,
            (update_clear_color, asset_loaded).in_set(InGameSet::EntityUpdates),
        );
    }
}

// this allows us to use Inspector reflection to manually update ClearColor to different values
// while the game is running from the ui_for_resources provided by bevy_inspector_egui
fn update_clear_color(app_clear_color: Res<AppClearColor>, mut clear_color: ResMut<ClearColor>) {
    clear_color.0 = app_clear_color
        .color
        .darker(app_clear_color.darkening_factor);
}

#[derive(Component, Debug)]
pub struct PrimaryCamera;

const CUBEMAPS: &[(&str, CompressedImageFormats)] =
    &[("textures/cubemap.png", CompressedImageFormats::NONE)];

fn spawn_camera(mut commands: Commands, boundary: Res<Boundary>, asset_server: Res<AssetServer>) {
    let skybox_handle = asset_server.load(CUBEMAPS[0].0);
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, boundary.transform.scale.z * 2.)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            Skybox {
                image: skybox_handle.clone(),
                brightness: 50.0,
            },
        ))
        .insert(InputManagerBundle::with_map(
            CameraMovement::camera_input_map(),
        ))
        .insert(PrimaryCamera);

    commands.insert_resource(Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
    });
}

#[derive(Resource)]
struct Cubemap {
    is_loaded: bool,
    index: usize,
    image_handle: Handle<Image>,
}

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle) == LoadState::Loaded {
        info!("Swapping to {}...", CUBEMAPS[cubemap.index].0);
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(image.height() / image.width());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.image = cubemap.image_handle.clone();
        }

        cubemap.is_loaded = true;
    }
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
