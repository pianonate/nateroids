/// let's use just load assets once, amigos
use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneAssets>()
            // make sure this loads before the camera uses it - right now that is
            // handled by running this PreStartup and camera in Startup - if you need
            // to change that ordering, you may get a panic unless you make this run .before()
            .add_systems(PreStartup, load_assets)
            .add_systems(Update, cubemap_loaded);
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct SkyboxCubemap {
    is_loaded: bool,
    pub image_handle: Handle<Image>,
}

// impl SkyboxCubemap {
//     pub fn handle(&self) -> Handle<Image> {
//         self.image_handle.clone()
//     }
// }

// all the models are loaded via SceneBundle - the models
// can have multiple elements and scene makes all that possible
#[derive(Resource, Clone, Debug, Default)]
pub struct SceneAssets {
    pub cubemap: SkyboxCubemap,
    pub missiles: Handle<Scene>,
    pub nateroid: Handle<Scene>,
    pub spaceship: Handle<Scene>,
}

fn load_assets(
    //    mut commands: Commands,
    mut scene_assets: ResMut<SceneAssets>,
    asset_server: Res<AssetServer>,
) {
    *scene_assets = SceneAssets {
        cubemap: SkyboxCubemap {
            is_loaded: false,
            image_handle: asset_server.load("textures/cubemap.png"),
        },
        missiles: asset_server.load("models/Bullets Pickup.glb#Scene0"),
        nateroid: asset_server.load("models/Planet.glb#Scene0"),
        spaceship: asset_server.load("models/Spaceship.glb#Scene0"),
    };
}

// borrowed from bevy/examples/3d/skybox.rs
fn cubemap_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut scene_assets: ResMut<SceneAssets>,
    mut skyboxes: Query<&mut Skybox>,
) {
    let cubemap = &mut scene_assets.cubemap;
    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle) == LoadState::Loaded {
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
