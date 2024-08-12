use bevy::prelude::*;
use primary_camera::PrimaryCameraPlugin;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;

pub mod primary_camera;
pub use primary_camera::PrimaryCamera; // make this name available to inspector for ease of use

mod star_twinkling;
pub mod stars;
pub use stars::StarsCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PrimaryCameraPlugin)
            .add_plugins(StarsPlugin)
            .add_plugins(StarTwinklingPlugin);
    }
}
