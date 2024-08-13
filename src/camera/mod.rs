use bevy::{
    prelude::*,
    render::view::Layer,
};
use primary_camera::PrimaryCameraPlugin;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;

pub mod primary_camera;
pub use primary_camera::PrimaryCamera; /* make this name available to inspector for ease of
                                        * use */
pub use stars::{
    StarConfig,
    StarsCamera,
};

mod star_twinkling;
pub mod stars;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PrimaryCameraPlugin)
            .add_plugins(StarsPlugin)
            .add_plugins(StarTwinklingPlugin);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraOrder {
    Game,
    Stars,
}

impl CameraOrder {
    pub const fn order(self) -> isize {
        match self {
            CameraOrder::Game => 1,
            CameraOrder::Stars => 0,
        }
    }
}

// todo: how can i get PBRs to actually render on RenderLayer 1 so i could
// choose to have some       affected by bloom and some not...
// weird - if i put game on render layer 1 and stars on render layer 0,
// to line up with the camera order, the PBRs on render layer 1 are still
// showing on render layer 0 even though i don't think i asked for that

// used for both camera order and render layer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderLayer {
    Both,
    Game,
    Stars,
}

// returning the array rather than just one in case we have more complex
// situations in the future that require overlapping layers
impl RenderLayer {
    pub const fn layers(self) -> &'static [Layer] {
        match self {
            RenderLayer::Both => &[0, 1],
            RenderLayer::Game => &[0],
            RenderLayer::Stars => &[1],
        }
    }
}
