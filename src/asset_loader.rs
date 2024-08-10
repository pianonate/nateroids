/// let's use just load assets once, amigos
use bevy::prelude::*;

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneAssets>()
            // make sure this loads before the camera uses it - right now that is
            // handled by running this PreStartup and camera in Startup - if you need
            // to change that ordering, you may get a panic unless you make this run .before()
            .add_systems(PreStartup, load_assets);
    }
}

// all the models are loaded via SceneBundle - the models
// can have multiple elements and scene makes all that possible
#[derive(Resource, Clone, Debug, Default)]
pub struct SceneAssets {
    pub missiles:  Handle<Scene>,
    pub nateroid:  Handle<Scene>,
    pub spaceship: Handle<Scene>, // pub sphere: Handle<Scene>,
}

pub fn load_assets(
    //    mut commands: Commands,
    mut scene_assets: ResMut<SceneAssets>,
    asset_server: Res<AssetServer>,
) {
    *scene_assets = SceneAssets {
        missiles:  asset_server.load("models/Bullets Pickup.glb#Scene0"),
        nateroid:  asset_server.load("models/donut.glb#Scene0"),
        spaceship: asset_server.load("models/Spaceship.glb#Scene0"), /*    sphere:
                                                                      * asset_server.load("
                                                                      * models/sphere.glb#
                                                                      * Scene0"), */
    };
}
