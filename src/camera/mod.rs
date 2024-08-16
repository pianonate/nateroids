use crate::{
    input::GlobalAction,
    utils::toggle_active,
};
use bevy::{
    color::palettes::tailwind,
    prelude::*,
    render::view::Layer,
};
use bevy_inspector_egui::{
    inspector_options::{
        std_options::NumberDisplay,
        ReflectInspectorOptions,
    },
    quick::ResourceInspectorPlugin,
    InspectorOptions,
};

use lights::DirectionalLightsPlugin;
pub use lights::LightConfig;
pub use primary_camera::PrimaryCamera;
use primary_camera::PrimaryCameraPlugin;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;
pub use stars::{
    StarConfig,
    StarsCamera,
};

mod lights;
mod primary_camera;
mod star_twinkling;
mod stars;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DirectionalLightsPlugin)
            .add_plugins(PrimaryCameraPlugin)
            .add_plugins(StarsPlugin)
            .add_plugins(StarTwinklingPlugin)
            .add_plugins(
                ResourceInspectorPlugin::<CameraConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::CameraInspector)),
            )
            .init_resource::<CameraConfig>();
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

// todo: #bevyquestion - how can i get PBRs to actually render on RenderLayer 1
// so i could choose to have some affected by bloom and some not...
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

// #todo - #bevyquestion #bug - why doesn't the slider show when it works on all
//                              the other ResourceInspectorPlugin instances?
#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct CameraConfig {
    pub clear_color:           Color,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub darkening_factor:      f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    bloom_intensity:           f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    bloom_low_frequency_boost: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    bloom_high_pass_frequency: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    rotation_speed:            f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            clear_color:               Color::from(tailwind::SLATE_900),
            darkening_factor:          0.002,
            bloom_intensity:           0.9,
            bloom_low_frequency_boost: 0.5,
            bloom_high_pass_frequency: 0.5,
            rotation_speed:            0.01,
        }
    }
}
