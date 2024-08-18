use crate::{
    camera::lights::LightPosition,
    input::GlobalAction,
    utils::toggle_active,
};
use bevy::{
    color::{
        palettes::tailwind,
        Color,
    },
    prelude::*,
};
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};

pub struct CameraConfigPlugin;

impl Plugin for CameraConfigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<CameraConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::CameraInspector)),
            )
            .init_resource::<CameraConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<LightConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::LightsInspector)),
            )
            .init_resource::<LightConfig>()
            .register_type::<LightConfig>();
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct CameraConfig {
    pub clear_color:               Color,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub darkening_factor:          f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_intensity:           f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_low_frequency_boost: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_high_pass_frequency: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub orbit_speed:               f32,
    #[inspector(min = 10.0, max = 200.0, display = NumberDisplay::Slider)]
    pub zoom_sensitivity_pinch:    f32,
    #[inspector(min = 1.0, max = 20.0, display = NumberDisplay::Slider)]
    pub zoom_sensitivity_mouse:    f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            clear_color:               Color::from(tailwind::SLATE_900),
            darkening_factor:          0.002,
            bloom_intensity:           0.9,
            bloom_low_frequency_boost: 0.5,
            bloom_high_pass_frequency: 0.5,
            orbit_speed:               0.01,
            zoom_sensitivity_pinch:    100.,
            zoom_sensitivity_mouse:    5.,
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct LightSettings {
    pub color:           Color,
    pub enabled:         bool,
    #[inspector(min = 0.0, max = 10_000.0, display = NumberDisplay::Slider)]
    pub illuminance:     f32,
    pub shadows_enabled: bool,
}

impl Default for LightSettings {
    fn default() -> Self {
        Self {
            color:           Color::from(tailwind::AMBER_400),
            enabled:         false,
            illuminance:     3000.0,
            shadows_enabled: false,
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone)]
#[reflect(Resource, InspectorOptions)]
pub struct LightConfig {
    #[inspector(min = 0.0, max = 1_000.0, display = NumberDisplay::Slider)]
    pub ambient_light_brightness: f32,
    pub ambient_light_color:      Color,
    pub front:                    LightSettings,
    pub back:                     LightSettings,
    pub top:                      LightSettings,
    pub bottom:                   LightSettings,
    pub left:                     LightSettings,
    pub right:                    LightSettings,
}

impl Default for LightConfig {
    fn default() -> Self {
        Self {
            ambient_light_brightness: 100.0,
            ambient_light_color:      Color::WHITE,
            front:                    LightSettings {
                enabled: true,
                ..Default::default()
            },
            back:                     LightSettings {
                enabled: true,
                ..Default::default()
            },
            top:                      LightSettings::default(),
            bottom:                   LightSettings::default(),
            left:                     LightSettings::default(),
            right:                    LightSettings::default(),
        }
    }
}

impl LightConfig {
    pub fn get_light_settings(&self, position: LightPosition) -> &LightSettings {
        match position {
            LightPosition::Front => &self.front,
            LightPosition::Back => &self.back,
            LightPosition::Top => &self.top,
            LightPosition::Bottom => &self.bottom,
            LightPosition::Left => &self.left,
            LightPosition::Right => &self.right,
        }
    }
}
