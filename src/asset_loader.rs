/// let's use just load assets once, amigos
use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct SceneAssets {
    pub missiles: Handle<Scene>,
    pub nateroid: Handle<Scene>,
    pub spaceship: Handle<Scene>,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    //noinspection Annotator
    //noinspection Annotator
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneAssets>()
            .add_systems(Startup, load_assets);
    }
}

//noinspection Annotator
fn load_assets(mut scene_assets: ResMut<SceneAssets>, asset_server: Res<AssetServer>) {
    *scene_assets = SceneAssets {
        missiles: asset_server.load("Bullets Pickup.glb#Scene0"),
        nateroid: asset_server.load("Planet.glb#Scene0"),
        spaceship: asset_server.load("Spaceship.glb#Scene0"),
    }
}